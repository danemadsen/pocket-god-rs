//! `AntHillLogic` (superclass `SpriteLogic`).
//!
//! The ant hill: taps stress it; once frenzied, a finger dragged out of it directs a column of
//! five ants, which then look for a pygmy to swarm.

use crate::prelude::*;

use crate::logic::island_anthill::ant_logic;
use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::logic;
use pg_engine::touch::BcTouch;

/// `AntHillLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct AntHillLogic {
    /// `mAntHillLogicListener`: `@"<AntHillLogicListener>"`
    pub ant_hill_logic_listener: Option<Entity>,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mStressLevel`: `f`
    pub stress_level: f32,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mbSwarmPygmy`: `B`
    pub swarm_pygmy: bool,

    /// `mAntHillState`: `i`
    pub ant_hill_state: i32,

    /// `mAntLogicArray`: `@"NSMutableArray"`
    pub ant_logic_array: Vec<Entity>,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbDirecting`: `B`
    pub directing: bool,

    /// `mbActiveAnts`: `B`
    pub active_ants: bool,

    /// `mbActive`: `B`
    pub active: bool,
}

pub const ANT_HILL_LOGIC: Class = Class {
    name: "AntHillLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    touch_began_inside,
    touch_began_outside,
    touch_moved_entered,
    touch_moved_exited,
    touch_moved_inside,
    touch_moved_outside,
    touch_ended_inside,
    touch_ended_outside,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// How many ants `directAnts:::` sends out.
const ANT_COUNT: usize = 5;

/// The ants' origin x (`setOriginXPos:`).
const ANT_ORIGIN_X_POS: f32 = -5.0;

/// How close (in x) an ant must come to a pygmy to swarm it.
const SWARM_RANGE: f32 = 10.0;

/// What one tap adds to the stress level.
const TAP_STRESS: f32 = 0.7;

/// What the stress level loses per frame while no ants are out.
const STRESS_DECAY: f32 = 0.01;

/// Above this stress level the hill is lightly stirred (state 1).
const LIGHT_STRESS_LEVEL: f32 = 0.5;

/// Above this stress level the hill is frenzied (state 2), and a touch can direct the ants.
const FRENZIED_STRESS_LEVEL: f32 = 1.5;

/// `mAntHillState` of a frenzied hill.
const FRENZIED_STATE: i32 = 2;

/// Sequence-item handlers implemented by `AntHillLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "removeAnts" => seq_remove_ants(world, this, call),
        "directAnts" => seq_direct_ants(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[AntHillLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&ANT_HILL_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[AntHillLogic initWithDisplayObject:]` @ 0x10013d7e0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(AntHillLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut ant_hill = world.obj_mut::<AntHillLogic>(this);
        ant_hill.ant_logic_array = Vec::with_capacity(ANT_COUNT);
        ant_hill.ant_hill_logic_listener = None;
        ant_hill.active_touch = None;
        ant_hill.stress_level = 0.0;
        ant_hill.ant_hill_state = 0;
        ant_hill.swarm_pygmy = false;
        ant_hill.touch_began_inside = false;
        ant_hill.directing = false;
        ant_hill.active_ants = false;
        ant_hill.touch_screen_pos = Vec4::new(0.0, 0.0, 0.0, 1.0);
    }

    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
}

/// `-[AntHillLogic setAntHillLogicListener:]` @ 0x10013d904
pub fn set_ant_hill_logic_listener(world: &mut World, this: Entity, ant_hill_logic_listener: Entity) {
    world.obj_mut::<AntHillLogic>(this).ant_hill_logic_listener = Some(ant_hill_logic_listener);
}

/// `-[AntHillLogic onAntDie:]` @ 0x10013d960
///
/// Frees a dead ant; the hill calms down when the last one is gone.
pub fn on_ant_die(world: &mut World, this: Entity, ant_logic: Entity) {
    let ant_display_object = logic::display_object(world, ant_logic);
    display::remove_from_display_group(world, ant_display_object);

    world.obj_mut::<AntHillLogic>(this).ant_logic_array.retain(|ant| *ant != ant_logic);
    logic::shutdown(world, ant_logic);

    // The array held the ant's last owning reference besides the frame-listener list and its
    // animation player, both of which `shutdown` drops: the ant is deallocated here.
    ant_logic::dealloc(world, ant_logic);

    let mut ant_hill = world.obj_mut::<AntHillLogic>(this);
    if ant_hill.ant_logic_array.is_empty() {
        ant_hill.active_ants = false;
        ant_hill.swarm_pygmy = false;
    }
}

/// `-[AntHillLogic antLogicArray]` @ 0x10013da10
pub fn ant_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<AntHillLogic>(this).ant_logic_array.clone()
}

/// `-[AntHillLogic removeAntLogics]` @ 0x10013da20
///
/// Sends every ant home to die (each reports back through `onAntDie:`).
pub fn remove_ant_logics(world: &mut World, this: Entity) {
    {
        let mut ant_hill = world.obj_mut::<AntHillLogic>(this);
        ant_hill.swarm_pygmy = false;
        ant_hill.active_ants = false;
    }

    let ants = world.obj::<AntHillLogic>(this).ant_logic_array.clone();
    for ant in ants {
        ant_logic::die(world, ant);
    }
}

/// `-[AntHillLogic enterIsland:]` @ 0x10013db44
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    if world.obj::<AntHillLogic>(this).active {
        return;
    }

    {
        let mut ant_hill = world.obj_mut::<AntHillLogic>(this);
        ant_hill.active = true;
        ant_hill.stress_level = 0.0;
        ant_hill.ant_hill_state = 0;
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;

    let marker = pg_engine::library::display_marker(world, "AntHillMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
    logic::set_behavior(world, this, Some("AntHillEnterIsland"));
}

/// `-[AntHillLogic exitIsland]` @ 0x10013dc1c
pub fn exit_island(world: &mut World, this: Entity) {
    if !world.obj::<AntHillLogic>(this).active {
        return;
    }

    {
        let mut ant_hill = world.obj_mut::<AntHillLogic>(this);
        ant_hill.active = false;
        ant_hill.stress_level = 0.0;
        ant_hill.ant_hill_state = 0;
    }
    remove_ant_logics(world, this);

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;

    if world.obj::<AntHillLogic>(this).directing {
        world.obj_mut::<AntHillLogic>(this).directing = false;
        let listener = world.obj::<AntHillLogic>(this).ant_hill_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_ant_hill_direct_complete(world, listener, this);
        }
    }
    logic::set_behavior(world, this, Some("AntHillExitIsland"));
}

/// `-[AntHillLogic exitIslandNow]` @ 0x10013dcf4
pub fn exit_island_now(world: &mut World, this: Entity) {
    {
        let mut ant_hill = world.obj_mut::<AntHillLogic>(this);
        ant_hill.active = false;
        ant_hill.stress_level = 0.0;
        ant_hill.ant_hill_state = 0;
    }
    remove_ant_logics(world, this);

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;

    if world.obj::<AntHillLogic>(this).directing {
        world.obj_mut::<AntHillLogic>(this).directing = false;
        let listener = world.obj::<AntHillLogic>(this).ant_hill_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_ant_hill_direct_complete(world, listener, this);
        }
    }
    logic::set_behavior(world, this, Some("AntHillDie"));
}

/// `-[AntHillLogic swarmPygmy:]` @ 0x10013de40
pub fn swarm_pygmy(world: &mut World, this: Entity, pygmy_logic: Entity) {
    crate::pygmy::swarmed_by_ant_hill(world, pygmy_logic, this);

    let ants = world.obj::<AntHillLogic>(this).ant_logic_array.clone();
    for ant in ants {
        ant_logic::swarm_pygmy(world, ant, pygmy_logic);
    }
}

/// `-[AntHillLogic checkPygmyCollision:]` @ 0x10013df78
///
/// While the ants roam on their own, the first ant that comes near a pygmy that can be
/// swarmed leads the whole column onto it.
pub fn check_pygmy_collision(world: &mut World, this: Entity, _dt: f32) {
    let (active_ants, swarm_pygmy_active, directing, listener) = {
        let ant_hill = world.obj::<AntHillLogic>(this);
        (ant_hill.active_ants, ant_hill.swarm_pygmy, ant_hill.directing, ant_hill.ant_hill_logic_listener)
    };
    if !active_ants || swarm_pygmy_active || directing {
        return;
    }

    let pygmies = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmies {
        if !crate::pygmy::can_be_swarmed_by_ant_hill(world, pygmy, this) {
            continue;
        }

        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_x_pos = display::object(world, pygmy_display_object).pos.x;

        let ants = world.obj::<AntHillLogic>(this).ant_logic_array.clone();
        for ant in ants {
            let ant_display_object = logic::display_object(world, ant);
            let ant_x_pos = display::object(world, ant_display_object).pos.x;
            let distance = (ant_x_pos - pygmy_x_pos).abs();
            if distance < SWARM_RANGE {
                world.obj_mut::<AntHillLogic>(this).swarm_pygmy = true;
                ant_logic::set_lead_ant_logic(world, ant);
                swarm_pygmy(world, this, pygmy);
                return;
            }
        }
    }
}

/// `-[AntHillLogic updateStressLevel]` @ 0x10013e258
///
/// Plays the hill's mood (`AntHillNoActivity`, `AntHillLight`, `AntHillFenzied`) when its
/// stress state changes.
pub fn update_stress_level(world: &mut World, this: Entity) {
    let (stress_level, ant_hill_state) = {
        let ant_hill = world.obj::<AntHillLogic>(this);
        (ant_hill.stress_level, ant_hill.ant_hill_state)
    };

    let mut new_state = 0;
    if stress_level > LIGHT_STRESS_LEVEL {
        new_state = 1;
    }
    if stress_level > FRENZIED_STRESS_LEVEL {
        new_state = FRENZIED_STATE;
    }
    if new_state == ant_hill_state {
        return;
    }

    world.obj_mut::<AntHillLogic>(this).ant_hill_state = new_state;
    if new_state == FRENZIED_STATE {
        logic::set_behavior(world, this, Some("AntHillFenzied"));
    } else if new_state == 1 {
        logic::set_behavior(world, this, Some("AntHillLight"));
    } else if new_state == 0 {
        logic::set_behavior(world, this, Some("AntHillNoActivity"));
    }
}

/// `-[AntHillLogic onGameFrame:]` @ 0x10013e2f8
///
/// The hill calms down a little every frame while no ants are out.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    check_pygmy_collision(world, this, dt);
    if world.obj::<AntHillLogic>(this).active_ants {
        return;
    }

    {
        let mut ant_hill = world.obj_mut::<AntHillLogic>(this);
        let decayed_stress_level = ant_hill.stress_level - STRESS_DECAY;
        ant_hill.stress_level = if decayed_stress_level < 0.0 { 0.0 } else { decayed_stress_level };
    }
    update_stress_level(world, this);
}

/// `-[AntHillLogic removeAnts:::]` @ 0x10013e394
pub fn seq_remove_ants(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    remove_ant_logics(world, this);
    ItemResult::Done
}

/// `-[AntHillLogic directAnts:::]` @ 0x10013e3b8
///
/// Sends five ants out on the first call; then, every frame, spreads them evenly from the hill
/// towards the finger (held over the island) and tells each the column's ends.
pub fn seq_direct_ants(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    let active_ants = world.obj::<AntHillLogic>(this).active_ants;
    if call.first_time && !active_ants {
        world.obj_mut::<AntHillLogic>(this).active_ants = true;

        // Each ant goes right after the hill, in front of the ones before it.
        let display_group = display::object(world, display_object).display_group;
        let hill_index = match display_group {
            Some(display_group) => display::group::index_of_display_object(world, display_group, display_object),
            None => 0,
        };
        let ant_index = hill_index + 1;

        for _ in 0..ANT_COUNT {
            let ant_model = display::multi_model::spawn(world, Some("Ant"));
            display::move_to_display_group_at_index(world, ant_model, display_group, ant_index);

            let hill_pos = display::object(world, display_object).pos;
            {
                let mut ant_object = display::object_mut(world, ant_model);
                ant_object.pos.x = hill_pos.x;
                ant_object.pos.y = hill_pos.y;
                ant_object.pos.z = hill_pos.z;
                ant_object.origin_pos.x = ANT_ORIGIN_X_POS;
            }

            let ant = ant_logic::spawn(world, ant_model);
            ant_logic::set_ant_logic_listener(world, ant, this);
            world.obj_mut::<AntHillLogic>(this).ant_logic_array.push(ant);
        }
    }

    let display_group = display::object(world, display_object).display_group;
    let touch_screen_pos = world.obj::<AntHillLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };
    let hill_x_pos = display::object(world, display_object).pos.x;

    let listener = world.obj::<AntHillLogic>(this).ant_hill_logic_listener;
    let mut island_min_x_pos = 0.0;
    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
        island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
    }

    let mut end_x_pos = touch_pos.x;
    if end_x_pos < island_min_x_pos {
        end_x_pos = island_min_x_pos;
    }
    if end_x_pos > island_max_x_pos {
        end_x_pos = island_max_x_pos;
    }

    let pos_x = display::object(world, display_object).pos.x;
    let ants = world.obj::<AntHillLogic>(this).ant_logic_array.clone();
    let ant_spacing = (end_x_pos - pos_x) / ants.len() as f32;
    let mut ant_x_pos = hill_x_pos;
    for ant in ants {
        let ant_display_object = logic::display_object(world, ant);
        display::object_mut(world, ant_display_object).pos.x = ant_x_pos;
        ant_logic::set_start_x_pos(world, ant, hill_x_pos, end_x_pos);
        ant_x_pos = ant_spacing + ant_x_pos;
    }
    ItemResult::Pending
}

/// `-[AntHillLogic die:::]` @ 0x10013e7f0
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);

    let mut ant_hill = world.obj_mut::<AntHillLogic>(this);
    ant_hill.directing = false;
    ant_hill.active_touch = None;
    ItemResult::Done
}

/// `-[AntHillLogic touchBeganInside:allTouches:]` @ 0x10013e848
///
/// A tap stirs the hill up; a touch on a frenzied hill can go on to direct the ants.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<AntHillLogic>(this).ant_hill_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };
    let (active_touch, active) = {
        let ant_hill = world.obj::<AntHillLogic>(this);
        (ant_hill.active_touch, ant_hill.active)
    };
    if story_active || active_touch.is_some() || !active {
        return;
    }

    world.obj_mut::<AntHillLogic>(this).stress_level += TAP_STRESS;
    update_stress_level(world, this);

    let mut ant_hill = world.obj_mut::<AntHillLogic>(this);
    if ant_hill.ant_hill_state == FRENZIED_STATE {
        ant_hill.touch_began_inside = true;
        ant_hill.active_touch = Some(touch);
    }
}

/// `-[AntHillLogic touchBeganOutside:allTouches:]` @ 0x10013e908
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    if world.obj::<AntHillLogic>(this).active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let mut ant_hill = world.obj_mut::<AntHillLogic>(this);
    ant_hill.touch_screen_pos = touch_pos;
    ant_hill.touch_began_inside = false;
}

/// `-[AntHillLogic touchMovedEntered:allTouches:]` @ 0x10013e964
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<AntHillLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<AntHillLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[AntHillLogic touchMovedExited:allTouches:]` @ 0x10013e9bc
///
/// A touch that began on the frenzied hill and leaves it starts directing the ants.
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<AntHillLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<AntHillLogic>(this).touch_screen_pos = touch_pos;

    let (touch_began_inside, active_ants, active) = {
        let ant_hill = world.obj::<AntHillLogic>(this);
        (ant_hill.touch_began_inside, ant_hill.active_ants, ant_hill.active)
    };
    if !touch_began_inside || active_ants || !active {
        return;
    }

    world.obj_mut::<AntHillLogic>(this).directing = true;
    let listener = world.obj::<AntHillLogic>(this).ant_hill_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_ant_hill_direct(world, listener, this);
    }
    logic::set_behavior(world, this, Some("AntHillDirect"));
}

/// `-[AntHillLogic touchMovedInside:allTouches:]` @ 0x10013ea98
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[AntHillLogic touchMovedOutside:allTouches:]` @ 0x10013eaa8
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<AntHillLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<AntHillLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[AntHillLogic touchEndedInside:allTouches:]` @ 0x10013eb00
///
/// Letting go of a directed column sends its ants looking for pygmies.
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<AntHillLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    if world.obj::<AntHillLogic>(this).directing {
        world.obj_mut::<AntHillLogic>(this).directing = false;

        let ants = world.obj::<AntHillLogic>(this).ant_logic_array.clone();
        for ant in ants {
            ant_logic::look_for_pygmy(world, ant);
        }

        let listener = world.obj::<AntHillLogic>(this).ant_hill_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_ant_hill_direct_complete(world, listener, this);
        }
        if world.obj::<AntHillLogic>(this).active {
            logic::set_behavior(world, this, Some("AntHillIdle"));
        }
    }

    let mut ant_hill = world.obj_mut::<AntHillLogic>(this);
    ant_hill.touch_began_inside = false;
    ant_hill.active_touch = None;
}

/// `-[AntHillLogic touchEndedOutside:allTouches:]` @ 0x10013eca0
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
