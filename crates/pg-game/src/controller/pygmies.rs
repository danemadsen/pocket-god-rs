//! `PocketGodViewController`: methods in this part of the class.
//!
//! The pygmies' lifecycle: creating and placing a new pygmy (`createPygmyLogic`), the tribe size
//! limit of `addPygmy` (6, one more with Charlie, one more while Tom lives), baby and Tom pygmies,
//! and the bookkeeping behind the pygmy listener callbacks (drag, tumble, explosion, sleeping,
//! death) plus the `canTouchPygmy` gate.

use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::sequence;
use pg_engine::view::BcView;

use crate::managers::challenge_manager;
use crate::persona::tribe_persona;
use crate::prelude::*;

use super::*;

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerPygmies {
    /// `mPygmyLogicArray`: `@"NSMutableArray"`
    pub pygmy_logic_array: Vec<Entity>,

    /// `mTomPygmyLogic`: `@"PygmyLogic"`
    pub tom_pygmy_logic: Option<Entity>,

    /// `mPygmyCount`: `i`
    pub pygmy_count: i32,

    /// `mLivePygmyCount`: `i`
    pub live_pygmy_count: i32,

    /// `mSleepingCount`: `i`
    pub sleeping_count: i32,

    /// `mDraggedGhostPygmyLogic`: `@"PygmyLogic"`
    pub dragged_ghost_pygmy_logic: Option<Entity>,

    /// `mDraggedPygmyLogic`: `@"PygmyLogic"`
    pub dragged_pygmy_logic: Option<Entity>,
}

/// Challenge game 9, the Ice Monster battle: its score counts the pygmies lost.
const KILL_ICE_MONSTER_GAME: i32 = 9;

/// `personaPersonality` of Tom's persona.
const TOM_PERSONALITY: i32 = 2;

/// `timeTemplePygmyType` of a baby pygmy.
const BABY_TIME_TEMPLE_PYGMY_TYPE: i32 = 2;

/// `-[PocketGodViewController isTomPygmyActive]` @ 0x100087e18
///
/// Whether a pygmy with Tom's personality is out of the closet.
pub fn is_tom_pygmy_active(world: &mut World, this: Entity) -> bool {
    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        let personality = crate::pygmy::persona_personality(world, pygmy);
        if personality == TOM_PERSONALITY && !crate::pygmy::trapped_in_closet(world, pygmy) {
            return true;
        }
    }
    false
}

/// `-[PocketGodViewController isBabyPygmyActive]` @ 0x100087f60
pub fn is_baby_pygmy_active(world: &mut World, this: Entity) -> bool {
    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        if crate::pygmy::time_temple_pygmy_type(world, pygmy) == BABY_TIME_TEMPLE_PYGMY_TYPE {
            return true;
        }
    }
    false
}

/// `-[PocketGodViewController (Game) isTomPygmyAlive]` @ 0x10015b9a8
pub fn is_tom_pygmy_alive(world: &mut World, this: Entity) -> bool {
    world.obj::<ControllerPygmies>(this).tom_pygmy_logic.is_some()
}

/// `-[PocketGodViewController (Game) createTomPygmy]` @ 0x10015b9d8
///
/// A new pygmy that takes Tom's guest persona, which is active only while the tribe hands it out.
pub fn create_tom_pygmy(world: &mut World, this: Entity) -> Option<Entity> {
    let (tribe_persona, tom_pygmy_persona) = tribe_and_tom_personas(world, this);
    if let Some(tribe_persona) = tribe_persona
        && let Some(tom_pygmy_persona) = tom_pygmy_persona
    {
        tribe_persona::activate_guest_pygmy_persona(world, tribe_persona, tom_pygmy_persona);
    }

    let tom_pygmy_logic = create_pygmy_logic(world, this);
    world.obj_mut::<ControllerPygmies>(this).tom_pygmy_logic = tom_pygmy_logic;
    resurrect_pygmy_logic(world, this, tom_pygmy_logic);

    let (tribe_persona, tom_pygmy_persona) = tribe_and_tom_personas(world, this);
    if let Some(tribe_persona) = tribe_persona
        && let Some(tom_pygmy_persona) = tom_pygmy_persona
    {
        tribe_persona::deactivate_guest_pygmy_persona(world, tribe_persona, tom_pygmy_persona);
    }

    world.obj::<ControllerPygmies>(this).tom_pygmy_logic
}

/// `-[PocketGodViewController (Game) createPygmyLogic]` @ 0x10015ba88
///
/// A new pygmy dropped from the sky at a random x (0.1 steps) between the island edges, inset
/// by 15, thrown down at 1200 unless the device is tilted past ±90°; underwater it starts at a
/// random depth instead. It is always returned.
pub fn create_pygmy_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let model = display::multi_model::spawn(world, Some("NewPygmy"));

    let (island_min_x_pos, island_max_x_pos) = {
        let locations = world.obj::<ControllerIslandLocations>(this);
        (locations.island_min_x_pos, locations.island_max_x_pos)
    };
    let min_x_tenths = ((island_min_x_pos + 15.0) * 10.0) as i32;
    let max_x_tenths = ((island_max_x_pos - 15.0) * 10.0) as i32;
    let x_tenths_range = max_x_tenths.wrapping_add(1).wrapping_sub(min_x_tenths) as u32;
    let random_x_tenths = world.resource_mut::<Random>().arc4random_mod(x_tenths_range);
    let x_tenths = random_x_tenths.wrapping_add(min_x_tenths as u32) as i32;
    let x_pos = x_tenths as f32 / 10.0;

    {
        let mut model_object = display::object_mut(world, model);
        model_object.is_selectable = true;
        model_object.origin_pos.x = -32.0;
        model_object.pos.x = x_pos;
        model_object.pos.y = 340.0;
        model_object.pos.z = SPRITE_Z;
        model_object.col_x_min = 0.0;
        model_object.col_x_max = 64.0;
        model_object.col_y_min = 0.0;
        model_object.col_y_max = 64.0;
    }

    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let pygmy_display_marker_id = match island_location_state {
        11 => "TimeTemplePygmyDisplayMarker",
        12 => "LocustTemplePygmyDisplayMarker",
        13 => "DanceTemplePygmyDisplayMarker",
        14 => "PlagueTemplePygmyDisplayMarker",
        15 => "PurificationTemplePygmyDisplayMarker",
        16 => "ClosetTemplePygmyDisplayMarker",
        17 => "XenuTemplePygmyDisplayMarker",
        _ => "PygmyDisplayMarker",
    };
    let pygmy_display_marker = library::display_marker(world, pygmy_display_marker_id);
    if let Some(pygmy_display_marker) = pygmy_display_marker {
        display::insert_after(world, pygmy_display_marker, Some(model));
    }

    let pygmy = crate::pygmy::spawn(world, model);
    crate::pygmy::set_pygmy_logic_listener(world, pygmy, this);

    let raw_accel_z_rot = world.resource::<BcView>().raw_accel_rot_degrees.z;
    if -90.0 < raw_accel_z_rot && raw_accel_z_rot < 90.0 {
        logic::logic_mut(world, pygmy).pos_vel.y = -1200.0;
    }

    let (island_location_state, island_location_transition) = {
        let locations = world.obj::<ControllerIslandLocations>(this);
        (locations.island_location_state, locations.island_location_transition)
    };
    let underwater = island_location_state == 2 || island_location_state == 9;
    if underwater && !island_location_transition {
        let random_y_tenths = world.resource_mut::<Random>().arc4random_mod(1401);
        let y_tenths = (random_y_tenths + 2000) as i32;
        display::object_mut(world, model).pos.y = y_tenths as f32 / 10.0;

        let random_flip = world.resource_mut::<Random>().arc4random();
        if random_flip & 1 == 0 {
            display::object_mut(world, model).scale.x = -1.0;
        }
        crate::pygmy::added_underwater(world, pygmy);
    }

    let bait_master = world.obj::<ControllerWorldObjectEvents>(this).bait_master;
    if bait_master {
        crate::pygmy::start_bait_master(world, pygmy);
    }

    world.obj_mut::<ControllerWorldRegistry>(this).logic_array.push(pygmy);
    {
        let mut pygmies = world.obj_mut::<ControllerPygmies>(this);
        pygmies.pygmy_logic_array.push(pygmy);
        pygmies.pygmy_count += 1;
        pygmies.live_pygmy_count += 1;
    }
    Some(pygmy)
}

/// `-[PocketGodViewController (Game) addPygmy]` @ 0x10015bfa4
///
/// A new pygmy for the tribe, unless 6 are alive already (7 with Charlie on the island, one more
/// while Tom lives).
pub fn add_pygmy(world: &mut World, this: Entity) {
    let option_preferences = world.obj::<ControllerBootAndFrame>(this).option_preferences;
    let charlie_state = match option_preferences {
        Some(option_preferences) => pg_engine::prefs::get_int_preference(world, option_preferences, "CharlieState"),
        None => 0,
    };
    let tribe_limit = if charlie_state == 1 { 7 } else { 6 };

    let tom_pygmy_alive = is_tom_pygmy_alive(world, this);
    let live_pygmy_limit = i32::from(tom_pygmy_alive) + tribe_limit;
    let live_pygmy_count = world.obj::<ControllerPygmies>(this).live_pygmy_count;
    if live_pygmy_count < live_pygmy_limit {
        let pygmy = create_pygmy_logic(world, this);
        resurrect_pygmy_logic(world, this, pygmy);
    }
}

/// `-[PocketGodViewController (Game) addPygmy:::]` @ 0x10015c064
pub fn seq_add_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    add_pygmy(world, this);
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) addBabyPygmy:::]` @ 0x10015c088
pub fn seq_add_baby_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let pygmy = create_pygmy_logic(world, this);
    resurrect_pygmy_logic(world, this, pygmy);
    if let Some(pygmy) = pygmy {
        crate::pygmy::set_time_temple_pygmy_type(world, pygmy, BABY_TIME_TEMPLE_PYGMY_TYPE);
    }
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) addTomPygmy:::]` @ 0x10015c0f8
pub fn seq_add_tom_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    create_tom_pygmy(world, this);
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) isDraggingPygmy]` @ 0x100163bfc
pub fn is_dragging_pygmy(world: &mut World, this: Entity) -> bool {
    let pygmies = world.obj::<ControllerPygmies>(this);
    pygmies.dragged_pygmy_logic.is_some() || pygmies.dragged_ghost_pygmy_logic.is_some()
}

/// `-[PocketGodViewController (Game) canTouchPygmy]` @ 0x100163c34
///
/// NO while a help, debug, challenge or stats screen is open, the volcano erupts, an earthquake
/// runs, or the finger drags one of the 22 blocking objects (or directs the ants).
pub fn can_touch_pygmy(world: &mut World, this: Entity) -> bool {
    let menus = world.obj::<ControllerMenusAndScreens>(this);
    let bridge = world.obj::<ControllerManagersBridge>(this);
    let screen_open =
        menus.help_screen_open || menus.debug_screen_open || bridge.challenge_screen_open || menus.stats_screen_open;
    if screen_open {
        return false;
    }

    let god_powers = world.obj::<ControllerGodPowers>(this);
    if god_powers.eruption {
        return false;
    }

    let events = world.obj::<ControllerWorldObjectEvents>(this);
    let blocking_objects = [
        events.dragged_shark_logic,
        events.dragged_bird_logic,
        events.dragged_anchor_logic,
        events.dragged_tile_plug_logic,
        events.dragged_boulder_logic,
        events.dragged_t_rex_egg_logic,
        events.directed_ant_hill_logic,
        events.dragged_magnifying_glass_logic,
        events.dragged_coconut_logic,
        events.dragged_wood_logic,
        events.dragged_spear_logic,
        events.dragged_water_wings_logic,
        events.dragged_fishing_pole_logic,
        events.dragged_sword_fish_logic,
        events.dragged_fish_logic,
        events.dragged_banana_logic,
        events.dragged_dance_balloon_logic,
        events.dragged_monkey_logic,
        events.dragged_moron_snake_logic,
        events.dragged_jellyfish_logic,
        events.dragged_god_idol_logic,
        events.dragged_closet_temple_closet_door_logic,
        events.dragged_thetan_logic,
    ];
    if blocking_objects.iter().any(Option::is_some) {
        return false;
    }

    god_powers.earthquake_sequence.is_none()
}

/// `-[PocketGodViewController (Game) onPygmySleepingStart:]` @ 0x100164474
///
/// The first pygmy to fall asleep starts the snoring (`AllPygmiesSleepingStart`).
pub fn on_pygmy_sleeping_start(world: &mut World, this: Entity, _pygmy_logic: Entity) {
    let mut sleeping_count = world.obj::<ControllerPygmies>(this).sleeping_count;
    if sleeping_count == 0 {
        set_main_sequence_def(world, this, "AllPygmiesSleepingStart");
        sleeping_count = world.obj::<ControllerPygmies>(this).sleeping_count;
    }
    world.obj_mut::<ControllerPygmies>(this).sleeping_count = sleeping_count + 1;
}

/// `-[PocketGodViewController (Game) onPygmySleepingStop:]` @ 0x100164514
///
/// The last pygmy to wake up stops the snoring (`AllPygmiesSleepingStop`).
pub fn on_pygmy_sleeping_stop(world: &mut World, this: Entity, _pygmy_logic: Entity) {
    let sleeping_count = {
        let mut pygmies = world.obj_mut::<ControllerPygmies>(this);
        pygmies.sleeping_count -= 1;
        pygmies.sleeping_count
    };
    if sleeping_count != 0 {
        return;
    }
    set_main_sequence_def(world, this, "AllPygmiesSleepingStop");
}

/// `-[PocketGodViewController (Game) onPygmyGhostDrag:]` @ 0x1001645a8
pub fn on_pygmy_ghost_drag(world: &mut World, this: Entity, pygmy_logic: Entity) {
    world.obj_mut::<ControllerPygmies>(this).dragged_ghost_pygmy_logic = Some(pygmy_logic);
}

/// `-[PocketGodViewController (Game) onPygmyGhostDragComplete:]` @ 0x1001645bc
pub fn on_pygmy_ghost_drag_complete(world: &mut World, this: Entity, _pygmy_logic: Entity) {
    world.obj_mut::<ControllerPygmies>(this).dragged_ghost_pygmy_logic = None;
}

/// `-[PocketGodViewController (Game) onPygmyDrag:]` @ 0x1001645d0
pub fn on_pygmy_drag(world: &mut World, this: Entity, pygmy_logic: Entity) {
    world.obj_mut::<ControllerPygmies>(this).dragged_pygmy_logic = Some(pygmy_logic);
}

/// `-[PocketGodViewController (Game) onPygmyDragComplete:]` @ 0x1001645e4
pub fn on_pygmy_drag_complete(world: &mut World, this: Entity, _pygmy_logic: Entity) {
    world.obj_mut::<ControllerPygmies>(this).dragged_pygmy_logic = None;
    check_count_shark_snaps(world, this);
}

/// `-[PocketGodViewController (Game) onPygmyTumble:]` @ 0x100164604
///
/// The tumbling pygmy moves to the front of `mPygmyLogicArray`.
pub fn on_pygmy_tumble(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let mut pygmies = world.obj_mut::<ControllerPygmies>(this);
    pygmies.pygmy_logic_array.retain(|pygmy| *pygmy != pygmy_logic);
    pygmies.pygmy_logic_array.insert(0, pygmy_logic);
}

/// `-[PocketGodViewController (Game) onPygmyTumbleComplete:]` @ 0x100164664
pub fn on_pygmy_tumble_complete(_world: &mut World, _this: Entity, _pygmy_logic: Entity) {}

/// `-[PocketGodViewController (Game) onPygmyExplode:]` @ 0x100164714
///
/// Every pygmy, the exploding one included, is thrown away from the explosion.
pub fn on_pygmy_explode(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let exploding_display_object = logic::display_object(world, pygmy_logic);
    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        // The original passes a pointer to the exploding pygmy's live `pos`, so each pygmy sees
        // the position as it is at its own call.
        let explosion_pos = display::object(world, exploding_display_object).pos;
        crate::pygmy::throw_from_explosion(world, pygmy, explosion_pos);
    }
}

/// `-[PocketGodViewController (Game) onPygmyDie:sacrificeId:]` @ 0x10016485c
///
/// The sacrifice is recorded, then the pygmy leaves the island for good: out of the display
/// tree, shut down, out of both arrays, and forgotten as Tom or as the pygmy of a mini-game
/// (which ends that mini-game's camera). A nil `sacrifice_id` is an invalid sacrifice.
pub fn on_pygmy_die(world: &mut World, this: Entity, pygmy_logic: Entity, sacrifice_id: Option<&str>) {
    on_trigger_sacrifice(world, this, pygmy_logic, sacrifice_id);
    world.obj_mut::<ControllerPygmies>(this).live_pygmy_count -= 1;

    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    display::remove_from_display_group(world, pygmy_display_object);

    let dragged_pygmy_logic = world.obj::<ControllerPygmies>(this).dragged_pygmy_logic;
    if dragged_pygmy_logic == Some(pygmy_logic) {
        world.obj_mut::<ControllerPygmies>(this).dragged_pygmy_logic = None;
        check_count_shark_snaps(world, this);
    }

    logic::shutdown(world, pygmy_logic);

    let in_logic_array = {
        let mut registry = world.obj_mut::<ControllerWorldRegistry>(this);
        let count_before = registry.logic_array.len();
        registry.logic_array.retain(|logic| *logic != pygmy_logic);
        registry.logic_array.len() != count_before
    };
    let in_pygmy_logic_array = {
        let mut pygmies = world.obj_mut::<ControllerPygmies>(this);
        let count_before = pygmies.pygmy_logic_array.len();
        pygmies.pygmy_logic_array.retain(|pygmy| *pygmy != pygmy_logic);
        pygmies.pygmy_logic_array.len() != count_before
    };

    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    if let Some(challenge_manager) = challenge_manager
        && challenge_manager::is_game_active(world, challenge_manager, KILL_ICE_MONSTER_GAME)
    {
        challenge_manager::increment_game_score(world, challenge_manager, KILL_ICE_MONSTER_GAME);
    }

    {
        let mut pygmies = world.obj_mut::<ControllerPygmies>(this);
        if pygmies.tom_pygmy_logic == Some(pygmy_logic) {
            pygmies.tom_pygmy_logic = None;
        }
    }

    {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        if minigames.doodle_jump_pygmy_logic == Some(pygmy_logic) {
            minigames.doodle_jump_pygmy_logic = None;
        }
        if minigames.pain_drain_pygmy_logic == Some(pygmy_logic) {
            minigames.pain_drain_pygmy_logic = None;
        }
        if minigames.the_runs_pygmy_logic == Some(pygmy_logic) {
            minigames.the_runs = false;
            minigames.the_runs_move_camera = false;
            minigames.the_runs_pygmy_logic = None;
        }
        if minigames.konkey_dong_pygmy_logic == Some(pygmy_logic) {
            minigames.konkey_dong = false;
            minigames.konkey_dong_move_camera = false;
            minigames.konkey_dong_pygmy_logic = None;
        }
        if minigames.moron_pests_pygmy_logic == Some(pygmy_logic) {
            minigames.moron_pests = false;
            minigames.moron_pests_move_camera = false;
            minigames.moron_pests_pygmy_logic = None;
        }
        if minigames.no_pee_pygmy_logic == Some(pygmy_logic) {
            minigames.no_pee = false;
            minigames.no_pee_move_camera = false;
            minigames.no_pee_pygmy_logic = None;
        }
    }

    // The arrays held the pygmy's last references: its persona let go of it in
    // `sacrifice:pygmyLogic:`, and `shutdown` dropped the frame-listener and animation ones. (The
    // original frees it slightly later, when this frame's listener snapshot releases it.)
    if in_logic_array || in_pygmy_logic_array {
        crate::pygmy::dealloc(world, pygmy_logic);
    }
}

/// `mTribePersona` and `mTomPygmyPersona`.
fn tribe_and_tom_personas(world: &World, this: Entity) -> (Option<Entity>, Option<Entity>) {
    let stats = world.obj::<ControllerStatsAndAchievements>(this);
    (stats.tribe_persona, stats.tom_pygmy_persona)
}

/// `[mTribePersona resurrectPygmyLogic:pygmy]`: the pygmy takes the first dead persona.
fn resurrect_pygmy_logic(world: &mut World, this: Entity, pygmy: Option<Entity>) {
    let tribe_persona = world.obj::<ControllerStatsAndAchievements>(this).tribe_persona;
    if let Some(tribe_persona) = tribe_persona
        && let Some(pygmy) = pygmy
    {
        tribe_persona::resurrect_pygmy_logic(world, tribe_persona, pygmy);
    }
}

/// `[mSequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:id]]` (nothing while
/// `mSequence` is nil).
fn set_main_sequence_def(world: &mut World, this: Entity, id: &str) {
    let main_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
    let Some(main_sequence) = main_sequence else {
        return;
    };
    let def = library::sequence_def(world, id);
    sequence::set_sequence_def(world, main_sequence, def);
}
