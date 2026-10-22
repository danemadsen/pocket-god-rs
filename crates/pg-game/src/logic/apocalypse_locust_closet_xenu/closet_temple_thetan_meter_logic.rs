//! `ClosetTempleThetanMeterLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::logic;

use crate::logic::apocalypse_locust_closet_xenu::thetan_logic;
use crate::logic::island_core::sprite_logic;

/// `ClosetTempleThetanMeterLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ClosetTempleThetanMeterLogic {
    /// `mClosetTempleThetanMeterLogicListener`: `@"<ClosetTempleThetanMeterLogicListener>"`
    pub closet_temple_thetan_meter_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,

    /// `mThetanLevel`: `i`
    pub thetan_level: i32,

    /// `mThetanLogicArray`: `@"NSMutableArray"`
    pub thetan_logic_array: Vec<Entity>,
}

pub const CLOSET_TEMPLE_THETAN_METER_LOGIC: Class = Class {
    name: "ClosetTempleThetanMeterLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
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

/// How many thetans one `spawnThetans:` creates.
const THETAN_SPAWN_COUNT: usize = 8;

/// Sequence-item handlers implemented by `ClosetTempleThetanMeterLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "thetanCreationComplete" => seq_thetan_creation_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `ClosetTempleThetanMeterLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animSpawnThetans" => anim_spawn_thetans(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[ClosetTempleThetanMeterLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&CLOSET_TEMPLE_THETAN_METER_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `mClosetTempleThetanMeterLogicListener` (the view controller).
fn meter_listener(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<ClosetTempleThetanMeterLogic>(this).closet_temple_thetan_meter_logic_listener
}

/// `mThetanLogicArray`, copied for an enumeration.
fn thetans(world: &World, this: Entity) -> Vec<Entity> {
    world.obj::<ClosetTempleThetanMeterLogic>(this).thetan_logic_array.clone()
}

/// `-[ClosetTempleThetanMeterLogic initWithDisplayObject:]` @ 0x10019df28
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ClosetTempleThetanMeterLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().closet_temple_thetan_meter_logic = Some(this);
    logic::stop_game_frame(world, this);
    world.obj_mut::<ClosetTempleThetanMeterLogic>(this).thetan_logic_array = Vec::with_capacity(5);
}

/// `-[ClosetTempleThetanMeterLogic setClosetTempleThetanMeterLogicListener:]` @ 0x10019dfcc
pub fn set_closet_temple_thetan_meter_logic_listener(world: &mut World, this: Entity, closet_temple_thetan_meter_logic_listener: Entity) {
    world.obj_mut::<ClosetTempleThetanMeterLogic>(this).closet_temple_thetan_meter_logic_listener = Some(closet_temple_thetan_meter_logic_listener);
}

/// `-[ClosetTempleThetanMeterLogic isAvailable]` @ 0x10019e028
pub fn is_available(world: &mut World, this: Entity) -> bool {
    world.obj::<ClosetTempleThetanMeterLogic>(this).pygmy_logic.is_none()
}

/// `-[ClosetTempleThetanMeterLogic fallOntoClosetTempleThetanMeter:]` @ 0x10019e040
pub fn fall_onto_closet_temple_thetan_meter(world: &mut World, this: Entity, fall_onto_closet_temple_thetan_meter: Entity) {
    {
        let mut meter = world.obj_mut::<ClosetTempleThetanMeterLogic>(this);
        meter.thetan_level = 0;
        meter.pygmy_logic = Some(fall_onto_closet_temple_thetan_meter);
    }

    let animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, fall_onto_closet_temple_thetan_meter);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), 1);

    logic::set_behavior(world, this, Some("ClosetTempleThetanMeterPygmyFall"));
}

/// `-[ClosetTempleThetanMeterLogic ejectFromClosetTempleThetanMeter:]` @ 0x10019e0cc
pub fn eject_from_closet_temple_thetan_meter(world: &mut World, this: Entity, eject_from_closet_temple_thetan_meter: Entity) {
    let pygmy = world.obj::<ClosetTempleThetanMeterLogic>(this).pygmy_logic;
    if pygmy != Some(eject_from_closet_temple_thetan_meter) {
        return;
    }

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, 1);

    logic::set_behavior(world, this, Some("ClosetTempleThetanMeterEjectPygmy"));
    world.obj_mut::<ClosetTempleThetanMeterLogic>(this).pygmy_logic = None;
}

/// `-[ClosetTempleThetanMeterLogic enterIsland:]` @ 0x10019e140
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    if world.obj::<ClosetTempleThetanMeterLogic>(this).active {
        return;
    }
    world.obj_mut::<ClosetTempleThetanMeterLogic>(this).active = true;
    logic::set_behavior(world, this, Some("ClosetTempleThetanMeterEnterIsland"));
}

/// `-[ClosetTempleThetanMeterLogic locustDisplayMarker]` @ 0x10019e174
pub fn locust_display_marker(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = meter_listener(world, this)?;
    crate::controller::locust_display_marker(world, listener)
}

/// `-[ClosetTempleThetanMeterLogic pygmyLogicArray:]` @ 0x10019e190
///
/// Returns the array itself, as the view controller's `pygmyLogicArray:` does.
pub fn pygmy_logic_array(world: &mut World, this: Entity, pygmy_logic_array: Entity) -> Vec<Entity> {
    match meter_listener(world, this) {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, pygmy_logic_array),
        None => Vec::new(),
    }
}

/// `-[ClosetTempleThetanMeterLogic isStoryActive]` @ 0x10019e1ac
pub fn is_story_active(world: &mut World, this: Entity) -> bool {
    match meter_listener(world, this) {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    }
}

/// `-[ClosetTempleThetanMeterLogic isApocalypseActive]` @ 0x10019e1c8
pub fn is_apocalypse_active(world: &mut World, this: Entity) -> bool {
    match meter_listener(world, this) {
        Some(listener) => crate::controller::is_apocalypse_active(world, listener),
        None => false,
    }
}

/// `-[ClosetTempleThetanMeterLogic onThetanDrag:]` @ 0x10019e1e4
pub fn on_thetan_drag(world: &mut World, this: Entity, thetan_drag: Entity) {
    if let Some(listener) = meter_listener(world, this) {
        crate::controller::on_thetan_drag(world, listener, thetan_drag);
    }
}

/// `-[ClosetTempleThetanMeterLogic onThetanDragComplete:]` @ 0x10019e200
pub fn on_thetan_drag_complete(world: &mut World, this: Entity, thetan_drag_complete: Entity) {
    if let Some(listener) = meter_listener(world, this) {
        crate::controller::on_thetan_drag_complete(world, listener, thetan_drag_complete);
    }
}

/// `-[ClosetTempleThetanMeterLogic areThetansActive]` @ 0x10019e21c
pub fn are_thetans_active(world: &mut World, this: Entity) -> bool {
    !world.obj::<ClosetTempleThetanMeterLogic>(this).thetan_logic_array.is_empty()
}

/// `-[ClosetTempleThetanMeterLogic holdClosetDoor]` @ 0x10019e250
///
/// Up to five free thetans line up in the doorway, 20 apart from y = 155.
pub fn hold_closet_door(world: &mut World, this: Entity) -> bool {
    let mut holding_count: i32 = 0;
    for thetan in thetans(world, this) {
        if !thetan_logic::can_hold_closet(world, thetan) {
            continue;
        }

        let hold_y_pos = holding_count as f32 * 20.0 + 155.0;
        thetan_logic::hold_closet_open_at_x_pos(world, thetan, 430.0, hold_y_pos);

        let previous_count = holding_count;
        holding_count = holding_count + 1;
        if previous_count > 3 {
            break;
        }
    }
    holding_count != 0
}

/// `-[ClosetTempleThetanMeterLogic stopHoldingClosetDoor]` @ 0x10019e3e0
pub fn stop_holding_closet_door(world: &mut World, this: Entity) {
    for thetan in thetans(world, this) {
        thetan_logic::stop_holding_closet_door(world, thetan);
    }
}

/// `-[ClosetTempleThetanMeterLogic exitIsland:]` @ 0x10019e4ec
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    if !world.obj::<ClosetTempleThetanMeterLogic>(this).active {
        return;
    }
    world.obj_mut::<ClosetTempleThetanMeterLogic>(this).active = false;

    let pygmy = world.obj::<ClosetTempleThetanMeterLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, 1);

        crate::pygmy::eject_from_closet_temple_thetan_meter(world, pygmy);
        world.obj_mut::<ClosetTempleThetanMeterLogic>(this).pygmy_logic = None;
    }

    // Outside the apocalypse the thetans vanish with the temple (`arrayWithArray:`: each one
    // leaves the array while it is enumerated).
    let apocalypse_active = match meter_listener(world, this) {
        Some(listener) => crate::controller::is_apocalypse_active(world, listener),
        None => false,
    };
    if !apocalypse_active {
        for thetan in thetans(world, this) {
            thetan_logic::exit_screen_now(world, thetan);
        }
    }
    logic::set_behavior(world, this, Some("ClosetTempleThetanMeterExitIsland"));
}

/// `-[ClosetTempleThetanMeterLogic dropApocalypseThetansIntoApoclaypseTempleRoom]` @ 0x10019e6b8
pub fn drop_apocalypse_thetans_into_apoclaypse_temple_room(world: &mut World, this: Entity) {
    for thetan in thetans(world, this) {
        thetan_logic::refresh_display_marker(world, thetan);
    }
}

/// `-[ClosetTempleThetanMeterLogic exitIslandNow]` @ 0x10019e7c4
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[ClosetTempleThetanMeterLogic onGameFrame:]` @ 0x10019e854
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[ClosetTempleThetanMeterLogic thetanCreationComplete:::]` @ 0x10019e858
pub fn seq_thetan_creation_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let pygmy = world.obj::<ClosetTempleThetanMeterLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, 1);

        crate::pygmy::explode_from_thetan_birth(world, pygmy);
        world.obj_mut::<ClosetTempleThetanMeterLogic>(this).pygmy_logic = None;
    }
    ItemResult::Done
}

/// `-[ClosetTempleThetanMeterLogic spawnThetans:]` @ 0x10019e8c4
///
/// The old thetans leave; eight new ones appear on the meter, or (apocalypse) at a random x
/// above the screen.
pub fn spawn_thetans(world: &mut World, this: Entity, spawn_thetans: bool) {
    let display_object = logic::display_object(world, this);

    for thetan in thetans(world, this) {
        thetan_logic::exit_screen(world, thetan);
    }

    let listener = meter_listener(world, this);
    for _ in 0..THETAN_SPAWN_COUNT {
        let thetan_model = pg_engine::display::multi_model::spawn(world, Some("Thetan"));
        let marker = match listener {
            Some(listener) => crate::controller::locust_display_marker(world, listener),
            None => None,
        };
        if let Some(marker) = marker {
            display::insert_after(world, marker, Some(thetan_model));
        }

        let thetan = thetan_logic::spawn(world, thetan_model);
        thetan_logic::set_thetan_logic_listener(world, thetan, this);
        let apocalypse_active = match listener {
            Some(listener) => crate::controller::is_apocalypse_active(world, listener),
            None => false,
        };
        thetan_logic::set_apocalypse_mode(world, thetan, apocalypse_active);

        if spawn_thetans {
            let random = world.resource_mut::<Random>().arc4random_mod(4801);
            let mut object = display::object_mut(world, thetan_model);
            object.pos.x = random as f32 / 10.0;
            object.pos.y = 370.0;
            object.pos.z = SPRITE_Z;
        } else {
            let meter_pos = display::object(world, display_object).pos;
            let mut object = display::object_mut(world, thetan_model);
            object.pos = meter_pos;
            object.pos.x = object.pos.x + 0.0;
            object.pos.y = object.pos.y + 24.0;
        }

        world.obj_mut::<ClosetTempleThetanMeterLogic>(this).thetan_logic_array.push(thetan);
    }
}

/// `-[ClosetTempleThetanMeterLogic spawnApocalypseThetans]` @ 0x10019ebe8
pub fn spawn_apocalypse_thetans(world: &mut World, this: Entity) {
    spawn_thetans(world, this, true);
}

/// `-[ClosetTempleThetanMeterLogic removeApocalypseThetans]` @ 0x10019ebfc
pub fn remove_apocalypse_thetans(world: &mut World, this: Entity) {
    for thetan in thetans(world, this) {
        thetan_logic::set_apocalypse_mode(world, thetan, false);
    }
}

/// `-[ClosetTempleThetanMeterLogic animSpawnThetans]` @ 0x10019ed0c
pub fn anim_spawn_thetans(world: &mut World, this: Entity) {
    spawn_thetans(world, this, false);
}

/// `-[ClosetTempleThetanMeterLogic onThetanDie:]` @ 0x10019ed20
///
/// The array and this call held the thetan's last references: it is deallocated here.
pub fn on_thetan_die(world: &mut World, this: Entity, thetan_die: Entity) {
    world.obj_mut::<ClosetTempleThetanMeterLogic>(this).thetan_logic_array.retain(|&thetan| thetan != thetan_die);

    let thetan_display_object = logic::display_object(world, thetan_die);
    display::remove_from_display_group(world, thetan_display_object);
    logic::shutdown(world, thetan_die);
    logic::dealloc(world, thetan_die);
}

/// `-[ClosetTempleThetanMeterLogic touchBeganInside:allTouches:]` @ 0x10019eda0
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    if world.obj::<ClosetTempleThetanMeterLogic>(this).pygmy_logic.is_none() {
        return;
    }

    let previous_level = {
        let mut meter = world.obj_mut::<ClosetTempleThetanMeterLogic>(this);
        let previous_level = meter.thetan_level;
        meter.thetan_level = previous_level + 1;
        previous_level
    };
    match previous_level {
        0 => logic::set_behavior(world, this, Some("ClosetTempleThetanMeterLevel1")),
        1 => logic::set_behavior(world, this, Some("ClosetTempleThetanMeterLevel2")),
        _ => {}
    }
}

/// `-[ClosetTempleThetanMeterLogic touchBeganOutside:allTouches:]` @ 0x10019ee04
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ClosetTempleThetanMeterLogic touchMovedEntered:allTouches:]` @ 0x10019ee08
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ClosetTempleThetanMeterLogic touchMovedExited:allTouches:]` @ 0x10019ee0c
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ClosetTempleThetanMeterLogic touchMovedInside:allTouches:]` @ 0x10019ee10
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[ClosetTempleThetanMeterLogic touchMovedOutside:allTouches:]` @ 0x10019ee20
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ClosetTempleThetanMeterLogic touchEndedInside:allTouches:]` @ 0x10019ee24
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ClosetTempleThetanMeterLogic touchEndedOutside:allTouches:]` @ 0x10019ee28
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}

/// `+[ClosetTempleThetanMeterLogic instance]` @ 0x10019df1c
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().closet_temple_thetan_meter_logic
}
