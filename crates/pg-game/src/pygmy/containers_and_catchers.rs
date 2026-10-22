//! `PygmyLogic`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use crate::logic::apocalypse_dance::dance_dance_execution_logic;
use crate::logic::apocalypse_dance::go_go_cage_logic;
use crate::logic::apocalypse_locust_closet_xenu::locust_queen_logic;
use crate::logic::apocalypse_temples::apocalypse_temple_door_logic;
use crate::logic::apocalypse_temples::apocalypse_temple_fountain_logic;
use crate::logic::apocalypse_temples::apocalypse_time_temple_platform_logic;
use crate::logic::ape_ritual::ape_altar_logic;
use crate::logic::ape_ritual::ape_bongo_logic;
use crate::logic::ape_ritual::ape_wench_logic;
use crate::logic::battle::idol_pedestal_logic;
use crate::logic::battle::temple_door_logic;
use crate::logic::ice_island::ice_hole_logic;
use crate::logic::ice_island::ice_monster_logic;
use crate::logic::ice_island::igloo_logic;
use crate::logic::island_core::sprite_logic;
use crate::logic::island_core::sprite_logic::SpriteLogic;
use crate::logic::konkeydong::kong_statue_logic;
use crate::logic::monsters::zombie_grave_logic;

/// `PygmyLogic` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct PygmyContainersAndCatchers {
    /// `mZombieGraveLogic`: `@"ZombieGraveLogic"`
    pub zombie_grave_logic: Option<Entity>,

    /// `mIdolPedestalLogic`: `@"IdolPedestalLogic"`
    pub idol_pedestal_logic: Option<Entity>,

    /// `mTempleDoorLogic`: `@"TempleDoorLogic"`
    pub temple_door_logic: Option<Entity>,

    /// `mApocalypseTempleDoorLogic`: `@"ApocalypseTempleDoorLogic"`
    pub apocalypse_temple_door_logic: Option<Entity>,

    /// `mApocalypseTimeTemplePlatformLogic`: `@"ApocalypseTimeTemplePlatformLogic"`
    pub apocalypse_time_temple_platform_logic: Option<Entity>,

    /// `mLocustQueenLogic`: `@"LocustQueenLogic"`
    pub locust_queen_logic: Option<Entity>,

    /// `mApocalypseTempleFountainLogic`: `@"ApocalypseTempleFountainLogic"`
    pub apocalypse_temple_fountain_logic: Option<Entity>,

    /// `mIglooLogic`: `@"IglooLogic"`
    pub igloo_logic: Option<Entity>,

    /// `mDanceDanceExecutionLogic`: `@"DanceDanceExecutionLogic"`
    pub dance_dance_execution_logic: Option<Entity>,

    /// `mGoGoCageLogic`: `@"GoGoCageLogic"`
    pub go_go_cage_logic: Option<Entity>,
}

/// `mTimeTemplePygmyType` of an ordinary pygmy (the temple types are 1 to 4).
const PYGMY_TYPE_NORMAL: i32 = 0;

/// The pygmy's own animation channels that carry the Battle of the Gods objects.
const BATTLE_OBJECT_CHANNELS: std::ops::RangeInclusive<usize> = 7..=13;

/// A listener query `[listener isXEnabled]`.
type EnabledQuery = fn(&mut World, Entity) -> bool;

/// A listener getter `[listener xLogicArray:self]`.
type LogicArrayQuery = fn(&mut World, Entity, Entity) -> Vec<Entity>;

/// The temple rooms' doors in the order `checkForApocalypseTempleDoorDuringFall` tries them
/// after the island's temple door and portal: whether the room's doors are enabled, and the
/// doors.
const TEMPLE_ROOM_DOORS: [(EnabledQuery, LogicArrayQuery); 7] = [
    (crate::controller::is_time_temple_door_enabled, crate::controller::time_temple_door_logic_array),
    (crate::controller::is_locust_temple_door_enabled, crate::controller::locust_temple_door_logic_array),
    (crate::controller::is_dance_temple_door_enabled, crate::controller::dance_temple_door_logic_array),
    (crate::controller::is_plague_temple_door_enabled, crate::controller::plague_temple_door_logic_array),
    (crate::controller::is_purification_temple_door_enabled, crate::controller::purification_temple_door_logic_array),
    (crate::controller::is_closet_temple_door_enabled, crate::controller::closet_temple_door_logic_array),
    (crate::controller::is_xenu_temple_door_enabled, crate::controller::xenu_temple_door_logic_array),
];

/// A temple fountain slot: `[fountain isSlotNAvailable]`, the slot's x offset from the
/// fountain, and `[fountain fallIntoApocalypseTempleFountainSlotN:self]`.
type FountainSlot = (fn(&mut World, Entity) -> bool, f32, fn(&mut World, Entity, Entity));

/// The fountain's three slots, in the order they are tried.
const FOUNTAIN_SLOTS: [FountainSlot; 3] = [
    (
        apocalypse_temple_fountain_logic::is_slot1_available,
        -51.0,
        apocalypse_temple_fountain_logic::fall_into_apocalypse_temple_fountain_slot1,
    ),
    (
        apocalypse_temple_fountain_logic::is_slot2_available,
        -3.0,
        apocalypse_temple_fountain_logic::fall_into_apocalypse_temple_fountain_slot2,
    ),
    (
        apocalypse_temple_fountain_logic::is_slot3_available,
        47.0,
        apocalypse_temple_fountain_logic::fall_into_apocalypse_temple_fountain_slot3,
    ),
];

/// The height of every fountain slot above the fountain.
const FOUNTAIN_SLOT_Y_OFFSET: f32 = 40.0;

/// `[[logic displayObject] pos]`.
fn display_pos(world: &World, logic: Entity) -> Vec4 {
    let display = pg_engine::logic::display_object(world, logic);
    pg_engine::display::object(world, display).pos
}

/// `[mDisplayObject setIsSelectable:]`.
fn set_selectable(world: &mut World, this: Entity, selectable: bool) {
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).is_selectable = selectable;
}

/// `mSacrificeId = @"…"`.
fn set_sacrifice_id(world: &mut World, this: Entity, sacrifice_id: &str) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some(sacrifice_id.to_owned());
}

/// `mPosVel.x` and `mPosVel.y`; z is left alone.
fn set_xy_vel(world: &mut World, this: Entity, x_vel: f32, y_vel: f32) {
    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = x_vel;
    logic.pos_vel.y = y_vel;
}

/// `*[mDisplayObject scale] = (1, 1)`: x and y only.
fn reset_scale(world: &mut World, this: Entity) {
    let display = pg_engine::logic::display_object(world, this);
    let mut object = pg_engine::display::object_mut(world, display);
    object.scale.x = 1.0;
    object.scale.y = 1.0;
}

/// `mbFalling = YES` under gravity, with the island floor collision back on.
fn start_falling(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = true;
}

/// The start of a catch: the pygmy stops (`mPosVel.x = mPosVel.y = 0`) and is held without
/// gravity or island floor collision.
fn hold_in_place(world: &mut World, this: Entity) {
    set_xy_vel(world, this, 0.0, 0.0);
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = false;
}

/// `[[mPygmyLogicListener pygmyDisplayMarker] insertAfter:mDisplayObject]`, then
/// `[self updateLabelLayer]`.
fn move_after_pygmy_display_marker(world: &mut World, this: Entity) {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let display = pg_engine::logic::display_object(world, this);

    let marker = match listener {
        Some(listener) => crate::controller::pygmy_display_marker(world, listener),
        None => None,
    };

    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display));
    }

    update_label_layer(world, this);
}

/// `[containerDisplay insertAfter:mDisplayObject]`, then `[self updateLabelLayer]`: the pygmy
/// is drawn right above the container that caught it.
fn move_after_container(world: &mut World, this: Entity, container_display: Entity) {
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::insert_after(world, container_display, Some(display));

    update_label_layer(world, this);
}

/// The end of a toss out of a container: selectable again, drawn after the pygmy display
/// marker, a drowning sacrifice and `PygmyFall`.
fn finish_toss(world: &mut World, this: Entity) {
    set_selectable(world, this, true);
    move_after_pygmy_display_marker(world, this);

    set_sacrifice_id(world, this, "DroppedDrown");
    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// The body the temple ejects share (`docs/re/pygmy.md` §11.3): the island floor back at 86,
/// unscaled, tossed at (-200, 500) under gravity onto the floor, selectable again, with a
/// drowning sacrifice.
fn eject_onto_island(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).island_y_pos = 86.0;
    reset_scale(world, this);
    set_xy_vel(world, this, -200.0, 500.0);
    start_falling(world, this);

    set_selectable(world, this, true);
    set_sacrifice_id(world, this, "DroppedDrown");
}

/// `PygmyFall`, or `TemplePygmyFall` for the temple pygmy types.
fn fall_behavior(world: &World, this: Entity) -> &'static str {
    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;

    if pygmy_type == PYGMY_TYPE_NORMAL {
        "PygmyFall"
    } else {
        "TemplePygmyFall"
    }
}

/// `-[PygmyLogic (Apocalypse) stopBeingInsideApocalypseTempleDoor]` @ 0x10003a3f8
pub fn stop_being_inside_apocalypse_temple_door(world: &mut World, this: Entity) {
    let door = world.obj::<PygmyContainersAndCatchers>(this).apocalypse_temple_door_logic;
    let Some(door) = door else {
        return;
    };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).inside_apocalypse_temple_door = false;
    apocalypse_temple_door_logic::eject_from_apocalypse_temple_door(world, door, this);

    world.obj_mut::<PygmyContainersAndCatchers>(this).apocalypse_temple_door_logic = None;
}

/// `-[PygmyLogic (Apocalypse) stopBeingOnTopApocalypseTimeTemplePlatform]` @ 0x10003a454
pub fn stop_being_on_top_apocalypse_time_temple_platform(world: &mut World, this: Entity) {
    let platform = world.obj::<PygmyContainersAndCatchers>(this).apocalypse_time_temple_platform_logic;
    let Some(platform) = platform else {
        return;
    };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).on_top_apocalypse_time_temple_platform = false;
    apocalypse_time_temple_platform_logic::eject_from_apocalypse_time_temple_platform(world, platform, this);

    world.obj_mut::<PygmyContainersAndCatchers>(this).apocalypse_time_temple_platform_logic = None;
}

/// `-[PygmyLogic (Apocalypse) stopBeingEatenByLocustQueen]` @ 0x10003a4b0
pub fn stop_being_eaten_by_locust_queen(world: &mut World, this: Entity) {
    let locust_queen = world.obj::<PygmyContainersAndCatchers>(this).locust_queen_logic;
    let Some(locust_queen) = locust_queen else {
        return;
    };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).eaten_by_locust_queen = false;
    set_selectable(world, this, true);
    locust_queen_logic::rescued_from_locust_queen(world, locust_queen, this);

    world.obj_mut::<PygmyContainersAndCatchers>(this).locust_queen_logic = None;
}

/// `-[PygmyLogic (Apocalypse) stopBeingInApocalypseTempleFountain]` @ 0x10003a530
pub fn stop_being_in_apocalypse_temple_fountain(world: &mut World, this: Entity) {
    let fountain = world.obj::<PygmyContainersAndCatchers>(this).apocalypse_temple_fountain_logic;
    let Some(fountain) = fountain else {
        return;
    };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).in_apocalypse_temple_fountain = false;
    apocalypse_temple_fountain_logic::eject_from_apocalypse_temple_fountain(world, fountain, this);

    world.obj_mut::<PygmyContainersAndCatchers>(this).apocalypse_temple_fountain_logic = None;
}

/// `-[PygmyLogic (Apocalypse) tossFromGoGoCage]` @ 0x10003aad0
pub fn toss_from_go_go_cage(world: &mut World, this: Entity) {
    let in_go_go_cage = world.obj::<PygmyPygmyCoreFrame>(this).in_go_go_cage;
    if !in_go_go_cage {
        return;
    }

    start_falling(world, this);
    reset_scale(world, this);
    set_xy_vel(world, this, -200.0, 500.0);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).in_go_go_cage = false;
    world.obj_mut::<PygmyContainersAndCatchers>(this).go_go_cage_logic = None;

    finish_toss(world, this);
}

/// `-[PygmyLogic (Apocalypse) tossFromDanceDanceExecution]` @ 0x10003ac38
pub fn toss_from_dance_dance_execution(world: &mut World, this: Entity) {
    let in_dance_dance_execution = world.obj::<PygmyPygmyCoreFrame>(this).in_dance_dance_execution;
    if !in_dance_dance_execution {
        return;
    }

    start_falling(world, this);
    reset_scale(world, this);
    set_xy_vel(world, this, -200.0, 500.0);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).in_dance_dance_execution = false;
    world.obj_mut::<PygmyContainersAndCatchers>(this).dance_dance_execution_logic = None;

    finish_toss(world, this);
}

/// `-[PygmyLogic (Apocalypse) ejectFromApocalypseTempleDoor]` @ 0x10003ada0
pub fn eject_from_apocalypse_temple_door(world: &mut World, this: Entity) {
    let door = world.obj::<PygmyContainersAndCatchers>(this).apocalypse_temple_door_logic;
    if door.is_none() {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).inside_apocalypse_temple_door = false;
    world.obj_mut::<PygmyContainersAndCatchers>(this).apocalypse_temple_door_logic = None;

    eject_onto_island(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Apocalypse) ejectFromApocalypseTimeTemplePlatform]` @ 0x10003aec8
pub fn eject_from_apocalypse_time_temple_platform(world: &mut World, this: Entity) {
    let platform = world.obj::<PygmyContainersAndCatchers>(this).apocalypse_time_temple_platform_logic;
    if platform.is_none() {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).on_top_apocalypse_time_temple_platform = false;
    world.obj_mut::<PygmyContainersAndCatchers>(this).apocalypse_time_temple_platform_logic = None;

    eject_onto_island(world, this);

    let behavior = fall_behavior(world, this);
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Apocalypse) rescuedFromLocustQueen]` @ 0x10003b010
pub fn rescued_from_locust_queen(world: &mut World, this: Entity) {
    let locust_queen = world.obj::<PygmyContainersAndCatchers>(this).locust_queen_logic;
    if locust_queen.is_none() {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).eaten_by_locust_queen = false;
    world.obj_mut::<PygmyContainersAndCatchers>(this).locust_queen_logic = None;

    eject_onto_island(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Apocalypse) ejectFromApocalypseTempleFountain]` @ 0x10003b138
pub fn eject_from_apocalypse_temple_fountain(world: &mut World, this: Entity) {
    let fountain = world.obj::<PygmyContainersAndCatchers>(this).apocalypse_temple_fountain_logic;
    if fountain.is_none() {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).in_apocalypse_temple_fountain = false;
    world.obj_mut::<PygmyContainersAndCatchers>(this).apocalypse_temple_fountain_logic = None;

    eject_onto_island(world, this);
    move_after_pygmy_display_marker(world, this);

    let behavior = fall_behavior(world, this);
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Apocalypse) checkForApocalypseTimeTemplePlatformDuringFall]` @ 0x10003c844
pub fn check_for_apocalypse_time_temple_platform_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let had_heart_attack = world.obj::<PygmyPygmyBehaviourItems>(this).had_heart_attack;
    let can_be_caught = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && !core.invalid_sacrifice
        && !core.doodle_jumping
        && !core.dropped_by_hurricane
        && !had_heart_attack
        && !core.thrown_by_locusts;
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_apocalypse_time_temple_platform_enabled(world, listener) {
        return false;
    }

    let platform = crate::controller::apocalypse_time_temple_platform_logic(world, listener, this);
    let Some(platform) = platform else {
        return false;
    };

    if !apocalypse_time_temple_platform_logic::is_available(world, platform) {
        return false;
    }

    let platform_pos = display_pos(world, platform);
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let x_distance = (pos.x - platform_pos.x).abs();
    let y_distance = (pos.y - (platform_pos.y + 50.0)).abs();
    let lands_on_platform = vel_y < 0.0 && x_distance <= 30.0 && y_distance <= 30.0;

    if !lands_on_platform {
        return false;
    }

    hold_in_place(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.tilt_state = 0;
    core.on_top_apocalypse_time_temple_platform = true;

    world.obj_mut::<PygmyContainersAndCatchers>(this).apocalypse_time_temple_platform_logic = Some(platform);
    apocalypse_time_temple_platform_logic::fall_onto_apocalypse_time_temple_platform(world, platform, this);

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallOntoApocalypseTimeTemplePlatform"));
    true
}

/// `-[PygmyLogic (Apocalypse) checkForTempleDoorJump:::]` @ 0x10003cf08
pub fn seq_check_for_temple_door_jump(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_jump = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL
        && !core.doodle_jumping
        && !core.invalid_sacrifice
        && !core.dropped_by_hurricane;
    let listener = core.pygmy_logic_listener;

    if !can_jump {
        return ItemResult::Done;
    }

    let Some(listener) = listener else {
        return ItemResult::Done;
    };

    if !crate::controller::is_temple_door_enabled(world, listener) {
        return ItemResult::Done;
    }

    let temple_door = crate::controller::temple_door_logic(world, listener, this);
    let Some(temple_door) = temple_door else {
        return ItemResult::Done;
    };

    if !temple_door_logic::is_available(world, temple_door) {
        return ItemResult::Done;
    }

    let door_pos = display_pos(world, temple_door);
    let display = pg_engine::logic::display_object(world, this);
    let pos = pg_engine::display::object(world, display).pos;

    let x_distance = (pos.x - (door_pos.x + 90.0)).abs();
    let is_within_jump = x_distance <= 80.0;

    if !is_within_jump {
        return ItemResult::Done;
    }

    set_xy_vel(world, this, 0.0, 0.0);
    pg_engine::display::object_mut(world, display).scale.x = 1.0;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_floor_collision = false;
    core.inside_temple_door = true;

    world.obj_mut::<PygmyContainersAndCatchers>(this).temple_door_logic = Some(temple_door);
    temple_door_logic::jump_into_temple_door(world, temple_door, this);
    set_selectable(world, this, false);

    pg_engine::logic::set_behavior(world, this, Some("PygmyJumpIntoTempleDoor"));
    ItemResult::Replaced
}

/// `-[PygmyLogic (Apocalypse) jumpIntoTempleDoorComplete:::]` @ 0x10003d14c
pub fn seq_jump_into_temple_door_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let temple_door = world.obj::<PygmyContainersAndCatchers>(this).temple_door_logic;

    if let Some(temple_door) = temple_door {
        temple_door_logic::jump_into_temple_door_complete(world, temple_door, this);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Apocalypse) checkForLocustQueenDuringFall]` @ 0x10003d188
pub fn check_for_locust_queen_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_be_caught = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && !core.invalid_sacrifice
        && !core.doodle_jumping
        && !core.dropped_by_hurricane
        && !core.thrown_by_locusts
        && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL;
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_locust_queen_enabled(world, listener) {
        return false;
    }

    let locust_queen = crate::controller::locust_queen_logic(world, listener, this);
    let Some(locust_queen) = locust_queen else {
        return false;
    };

    if !locust_queen_logic::is_available(world, locust_queen) {
        return false;
    }

    let queen_pos = display_pos(world, locust_queen);
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let x_distance = (pos.x - (queen_pos.x + 40.0)).abs();
    let y_distance = (pos.y - (queen_pos.y + 130.0)).abs();
    let falls_into_arms = vel_y < 0.0 && x_distance <= 30.0 && y_distance <= 30.0;

    if !falls_into_arms {
        return false;
    }

    hold_in_place(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.tilt_state = 0;
    core.eaten_by_locust_queen = true;

    set_selectable(world, this, false);

    world.obj_mut::<PygmyContainersAndCatchers>(this).locust_queen_logic = Some(locust_queen);
    locust_queen_logic::fall_into_locust_queen_arms(world, locust_queen, this);

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoLocustQueenArms"));
    true
}

/// `-[PygmyLogic (Apocalypse) checkForApocalypseTempleFountainDuringFall:xOffset:yOffset:]` @ 0x10003d404
pub fn check_for_apocalypse_temple_fountain_during_fall_x_offset_y_offset(world: &mut World, this: Entity, fountain: Entity, x_offset: f32, y_offset: f32) -> bool {
    let fountain_pos = display_pos(world, fountain);
    let display = pg_engine::logic::display_object(world, this);
    let pos = pg_engine::display::object(world, display).pos;
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let x_distance = (pos.x - (fountain_pos.x + x_offset)).abs();
    let y_distance = (pos.y - (fountain_pos.y + y_offset)).abs();
    let falls_into_slot = vel_y < 0.0 && x_distance <= 30.0 && y_distance <= 10.0;

    if !falls_into_slot {
        return false;
    }

    hold_in_place(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.tilt_state = 0;
    core.in_apocalypse_temple_fountain = true;

    world.obj_mut::<PygmyContainersAndCatchers>(this).apocalypse_temple_fountain_logic = Some(fountain);

    let fountain_disease = apocalypse_temple_fountain_logic::apocalypse_disease_type(world, fountain);
    let own_disease = world.obj::<SpriteLogic>(this).apocalypse_disease_type;

    if own_disease != fountain_disease {
        world.obj_mut::<PygmyApocalypse>(this).disease_incubate_idle_cycles = 5;
    }

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let marker = match listener {
        Some(listener) => crate::controller::pygmy_display_marker(world, listener),
        None => None,
    };

    if let Some(marker) = marker {
        pg_engine::display::insert_before(world, marker, Some(display));
    }

    update_label_layer(world, this);
    sprite_logic::infect_with_disease(world, this, fountain_disease);

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoApocalypseTempleFountain"));
    true
}

/// `-[PygmyLogic (Apocalypse) checkForApocalypseTempleFountainDuringFall:]` @ 0x10003d5fc
pub fn check_for_apocalypse_temple_fountain_during_fall_with_arg(world: &mut World, this: Entity, fountain: Entity) -> bool {
    for (is_slot_available, slot_x_offset, fall_into_slot) in FOUNTAIN_SLOTS {
        if !is_slot_available(world, fountain) {
            continue;
        }

        let caught = check_for_apocalypse_temple_fountain_during_fall_x_offset_y_offset(world, this, fountain, slot_x_offset, FOUNTAIN_SLOT_Y_OFFSET);
        if !caught {
            continue;
        }

        let caught_fountain = world.obj::<PygmyContainersAndCatchers>(this).apocalypse_temple_fountain_logic;
        if let Some(caught_fountain) = caught_fountain {
            fall_into_slot(world, caught_fountain, this);
        }

        return true;
    }

    false
}

/// `-[PygmyLogic (Apocalypse) checkForApocalypseTempleFountainDuringFall]` @ 0x10003d74c
pub fn check_for_apocalypse_temple_fountain_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let had_heart_attack = world.obj::<PygmyPygmyBehaviourItems>(this).had_heart_attack;
    let can_be_caught = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && !core.invalid_sacrifice
        && !core.doodle_jumping
        && !core.dropped_by_hurricane
        && !had_heart_attack
        && !core.thrown_by_locusts;
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    let fountain = if crate::controller::is_apocalypse_plague_temple_fountain_enabled(world, listener) {
        crate::controller::apocalypse_plague_temple_fountain_logic(world, listener, this)
    } else if crate::controller::is_apocalypse_purification_temple_fountain_enabled(world, listener) {
        crate::controller::apocalypse_purification_temple_fountain_logic(world, listener, this)
    } else {
        return false;
    };

    // A nil fountain has no available slot.
    let Some(fountain) = fountain else {
        return false;
    };

    check_for_apocalypse_temple_fountain_during_fall_with_arg(world, this, fountain)
}

/// `-[PygmyLogic (Apocalypse) eatenByLocustQueenComplete:]` @ 0x100041fe0
pub fn eaten_by_locust_queen_complete(world: &mut World, this: Entity, _locust_queen: Entity) {
    let eaten_by_locust_queen = world.obj::<PygmyPygmyCoreFrame>(this).eaten_by_locust_queen;
    if !eaten_by_locust_queen {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).disable_labels = true;
    set_sacrifice_id(world, this, "EatenByLocustQueen");

    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Apocalypse) danceDanceExecuted:]` @ 0x100045ac4
pub fn dance_dance_executed(world: &mut World, this: Entity, _dance_dance_execution: Entity) {
    let in_dance_dance_execution = world.obj::<PygmyPygmyCoreFrame>(this).in_dance_dance_execution;
    if !in_dance_dance_execution {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).dance_dance_executed = true;
    set_selectable(world, this, false);
}

/// `-[PygmyLogic (Apocalypse) danceDanceExecutedComplete:]` @ 0x100045b14
pub fn dance_dance_executed_complete(world: &mut World, this: Entity, _dance_dance_execution: Entity) {
    world.obj_mut::<PygmyContainersAndCatchers>(this).dance_dance_execution_logic = None;
    set_sacrifice_id(world, this, "DanceDanceExecuted");

    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Apocalypse) checkForGoGoCageDuringFall]` @ 0x100046da4
pub fn check_for_go_go_cage_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_be_caught = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && !core.invalid_sacrifice
        && !core.doodle_jumping
        && !core.dropped_by_hurricane
        && !core.thrown_by_locusts;
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_go_go_cage_enabled(world, listener) {
        return false;
    }

    let go_go_cage = crate::controller::go_go_cage_logic(world, listener, this);
    let Some(go_go_cage) = go_go_cage else {
        return false;
    };

    if !go_go_cage_logic::is_available(world, go_go_cage) {
        return false;
    }

    let cage_display = pg_engine::logic::display_object(world, go_go_cage);
    let cage_pos = pg_engine::display::object(world, cage_display).pos;
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let x_distance = (pos.x - cage_pos.x).abs();
    let y_offset = pos.y - cage_pos.y;
    let falls_into_cage = vel_y < 0.0 && x_distance <= 20.0 && y_offset <= 0.0 && y_offset >= -15.0;

    if !falls_into_cage {
        return false;
    }

    hold_in_place(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).in_go_go_cage = true;
    world.obj_mut::<PygmyContainersAndCatchers>(this).go_go_cage_logic = Some(go_go_cage);

    go_go_cage_logic::pygmy_fall_into_go_go_cage(world, go_go_cage, this);
    move_after_container(world, this, cage_display);

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoGoGoCage"));
    true
}

/// `-[PygmyLogic (Apocalypse) checkForDanceDanceExecutionDuringFall]` @ 0x10004700c
pub fn check_for_dance_dance_execution_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_be_caught = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL
        && !core.invalid_sacrifice
        && !core.doodle_jumping
        && !core.dropped_by_hurricane
        && !core.thrown_by_locusts;
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_dance_dance_execution_enabled(world, listener) {
        return false;
    }

    let dance_dance_execution = crate::controller::dance_dance_execution_logic(world, listener, this);
    let Some(dance_dance_execution) = dance_dance_execution else {
        return false;
    };

    if !dance_dance_execution_logic::is_available(world, dance_dance_execution) {
        return false;
    }

    let execution_display = pg_engine::logic::display_object(world, dance_dance_execution);
    let execution_pos = pg_engine::display::object(world, execution_display).pos;
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let x_distance = (pos.x - execution_pos.x).abs();
    let y_offset = pos.y - execution_pos.y;
    let falls_into_execution = vel_y < 0.0 && x_distance <= 20.0 && y_offset <= 0.0 && y_offset >= -15.0;

    if !falls_into_execution {
        return false;
    }

    hold_in_place(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).in_dance_dance_execution = true;
    world.obj_mut::<PygmyContainersAndCatchers>(this).dance_dance_execution_logic = Some(dance_dance_execution);

    dance_dance_execution_logic::pygmy_fall_into_dance_dance_execution(world, dance_dance_execution, this);
    move_after_container(world, this, execution_display);

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoDanceDanceExecution"));
    true
}

/// `-[PygmyLogic (Island) stopBeingTrappedInZombieGrave]` @ 0x1000953a8
pub fn stop_being_trapped_in_zombie_grave(world: &mut World, this: Entity) {
    let zombie_grave = world.obj::<PygmyContainersAndCatchers>(this).zombie_grave_logic;
    let Some(zombie_grave) = zombie_grave else {
        return;
    };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).trapped_in_zombie_grave = false;
    zombie_grave_logic::eject_from_grave(world, zombie_grave, this);

    world.obj_mut::<PygmyContainersAndCatchers>(this).zombie_grave_logic = None;
}

/// `-[PygmyLogic (Island) stopBeingTrappedInIdolPedestal]` @ 0x100095404
pub fn stop_being_trapped_in_idol_pedestal(world: &mut World, this: Entity) {
    let idol_pedestal = world.obj::<PygmyContainersAndCatchers>(this).idol_pedestal_logic;
    let Some(idol_pedestal) = idol_pedestal else {
        return;
    };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).trapped_in_idol_pedestal = false;
    idol_pedestal_logic::eject_from_idol_pedestal(world, idol_pedestal, this);

    world.obj_mut::<PygmyContainersAndCatchers>(this).idol_pedestal_logic = None;
}

/// `-[PygmyLogic (Island) stopBeingInsideTempleDoor]` @ 0x100095460
pub fn stop_being_inside_temple_door(world: &mut World, this: Entity) {
    let temple_door = world.obj::<PygmyContainersAndCatchers>(this).temple_door_logic;
    let Some(temple_door) = temple_door else {
        return;
    };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).inside_temple_door = false;
    temple_door_logic::eject_from_temple_door(world, temple_door, this);

    world.obj_mut::<PygmyContainersAndCatchers>(this).temple_door_logic = None;
}

/// `-[PygmyLogic (Island) tossFromKongStatue]` @ 0x100095960
pub fn toss_from_kong_statue(world: &mut World, this: Entity) {
    let in_kong_statue = world.obj::<PygmyPygmyCoreFrame>(this).in_kong_statue;
    if !in_kong_statue {
        return;
    }

    start_falling(world, this);
    reset_scale(world, this);
    set_xy_vel(world, this, 200.0, 500.0);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.in_kong_statue = false;
    core.kidnapped_by_konkey_dong = false;
    core.kong_statue_logic = None;

    finish_toss(world, this);
}

/// `-[PygmyLogic (Island) tossFromApeWench]` @ 0x100095ad8
pub fn toss_from_ape_wench(world: &mut World, this: Entity) {
    let in_ape_wench = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_wench;
    if !in_ape_wench {
        return;
    }

    start_falling(world, this);
    reset_scale(world, this);
    set_xy_vel(world, this, 200.0, 500.0);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.in_ape_wench = false;
    core.ape_wench_logic = None;

    finish_toss(world, this);
}

/// `-[PygmyLogic (Island) tossFromApeBongo]` @ 0x100095c40
pub fn toss_from_ape_bongo(world: &mut World, this: Entity) {
    let in_ape_bongo = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_bongo;
    if !in_ape_bongo {
        return;
    }

    start_falling(world, this);
    reset_scale(world, this);
    set_xy_vel(world, this, -200.0, 500.0);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.in_ape_bongo = false;
    core.ape_bongo_logic = None;

    finish_toss(world, this);
}

/// `-[PygmyLogic (Island) tossFromApeAltar]` @ 0x100095da8
pub fn toss_from_ape_altar(world: &mut World, this: Entity) {
    let in_ape_altar = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_altar;
    if !in_ape_altar {
        return;
    }

    start_falling(world, this);
    reset_scale(world, this);
    set_xy_vel(world, this, 200.0, 500.0);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.in_ape_altar = false;
    core.ape_altar_logic = None;

    finish_toss(world, this);
}

/// `-[PygmyLogic (Island) tornApartFromApeAltar]` @ 0x100095f10
///
/// Unlike the tosses, `mbInApeAltar` stays set and the island floor collision is turned off.
pub fn torn_apart_from_ape_altar(world: &mut World, this: Entity) {
    let in_ape_altar = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_altar;
    if !in_ape_altar {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = false;
    reset_scale(world, this);

    // `(float)(int)(arc4random() % 4001 - 2000) / 10`: unsigned wrap, then signed.
    let roll = world.resource_mut::<Random>().arc4random_mod(4001);
    let spread = roll.wrapping_sub(2000) as i32;
    let x_vel = spread as f32 / 10.0;
    set_xy_vel(world, this, x_vel, 500.0);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).ape_altar_logic = None;
    set_selectable(world, this, true);

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let display = pg_engine::logic::display_object(world, this);

    let marker = match listener {
        Some(listener) => crate::controller::pygmy_drag_display_marker(world, listener),
        None => None,
    };

    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display));
    }

    update_label_layer(world, this);

    set_sacrifice_id(world, this, "ApeAltar");
    world.obj_mut::<PygmyPygmyBehaviourItems>(this).torn_apart_by_ape_altar = true;

    pg_engine::logic::set_behavior(world, this, Some("PygmyTornApartFromApeAltar"));
}

/// `-[PygmyLogic (Island) grabbedFromApeAltar]` @ 0x1000960bc
pub fn grabbed_from_ape_altar(world: &mut World, this: Entity) {
    let in_ape_altar = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_altar;
    if !in_ape_altar {
        return;
    }

    set_sacrifice_id(world, this, "ApeEaten");
    set_selectable(world, this, false);

    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Island) apeSmashOnApeWench]` @ 0x10009614c
pub fn ape_smash_on_ape_wench(world: &mut World, this: Entity) {
    let in_ape_wench = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_wench;
    if !in_ape_wench {
        return;
    }

    set_sacrifice_id(world, this, "ApeWenchSmash");
    set_selectable(world, this, false);

    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Island) apeSmashOnApeBongo]` @ 0x1000961dc
pub fn ape_smash_on_ape_bongo(world: &mut World, this: Entity) {
    let in_ape_bongo = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_bongo;
    if !in_ape_bongo {
        return;
    }

    set_sacrifice_id(world, this, "ApeBongoSmash");
    set_selectable(world, this, false);

    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Island) apeOwnsPygmy]` @ 0x10009626c
pub fn ape_owns_pygmy(world: &mut World, this: Entity) {
    set_selectable(world, this, false);
}

/// `-[PygmyLogic (Island) tossFromIgloo]` @ 0x100096290
pub fn toss_from_igloo(world: &mut World, this: Entity) {
    let resting_in_igloo = world.obj::<PygmyPygmyCoreFrame>(this).resting_in_igloo;
    if !resting_in_igloo {
        return;
    }

    start_falling(world, this);
    reset_scale(world, this);
    set_xy_vel(world, this, 200.0, 500.0);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).resting_in_igloo = false;
    world.obj_mut::<PygmyContainersAndCatchers>(this).igloo_logic = None;

    set_selectable(world, this, true);
    set_sacrifice_id(world, this, "DroppedDrown");

    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Island) leaveIglooDoorExplode]` @ 0x1000963b0
pub fn leave_igloo_door_explode(world: &mut World, this: Entity) {
    let resting_in_igloo = world.obj::<PygmyPygmyCoreFrame>(this).resting_in_igloo;
    if !resting_in_igloo {
        return;
    }

    start_falling(world, this);
    reset_scale(world, this);
    pg_engine::logic::logic_mut(world, this).pos_vel.x = 400.0;

    let island_y_pos = world.obj::<PygmyPygmyCoreFrame>(this).island_y_pos;
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).pos.y = island_y_pos;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.last_pos.y = island_y_pos + 2.0;
    core.igloo_roll = true;
    core.resting_in_igloo = false;

    world.obj_mut::<PygmyContainersAndCatchers>(this).igloo_logic = None;

    set_selectable(world, this, true);
    set_sacrifice_id(world, this, "ShotOutIgloo");

    pg_engine::logic::set_behavior(world, this, Some("PygmyLeaveIglooDoorExplode"));
}

/// `-[PygmyLogic (Island) leaveIglooTopExplode]` @ 0x10009651c
///
/// Shot out of the igloo's top towards the ice monster when it is awake.
pub fn leave_igloo_top_explode(world: &mut World, this: Entity) {
    let resting_in_igloo = world.obj::<PygmyPygmyCoreFrame>(this).resting_in_igloo;
    if !resting_in_igloo {
        return;
    }

    start_falling(world, this);
    reset_scale(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).resting_in_igloo = false;
    world.obj_mut::<PygmyContainersAndCatchers>(this).igloo_logic = None;

    pg_engine::logic::logic_mut(world, this).pos_vel.x = 50.0;

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    if let Some(listener) = listener
        && crate::controller::is_ice_monster_enabled(world, listener)
    {
        let ice_monster = crate::controller::ice_monster_logic(world, listener, this);

        if let Some(ice_monster) = ice_monster
            && !ice_monster_logic::is_dormant(world, ice_monster)
        {
            let ice_monster_pos = display_pos(world, ice_monster);
            let pos = display_pos(world, this);
            pg_engine::logic::logic_mut(world, this).pos_vel.x = ice_monster_pos.x - pos.x;
        }
    }

    world.obj_mut::<PygmyPygmyBehaviourItems>(this).igloo_cannon = true;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.y = 1500.0;
    logic.pos_vel.z = -300.0;

    let pos = display_pos(world, this);
    world.obj_mut::<PygmyPygmyBehaviourItems>(this).tumble_pos = pos;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.falling = true;
    core.drowning = false;

    set_sacrifice_id(world, this, "ShotOutIgloo");

    pg_engine::logic::set_behavior(world, this, Some("PygmyLeaveIglooTopExplode"));
}

/// `-[PygmyLogic (Island) leaveIglooDoorCrawl]` @ 0x100096720
pub fn leave_igloo_door_crawl(world: &mut World, this: Entity) {
    let resting_in_igloo = world.obj::<PygmyPygmyCoreFrame>(this).resting_in_igloo;
    if !resting_in_igloo {
        return;
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = true;

    let island_y_pos = world.obj::<PygmyPygmyCoreFrame>(this).island_y_pos;
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).pos.y = island_y_pos;

    world.obj_mut::<PygmyPygmyCoreFrame>(this).last_pos.y = island_y_pos + 2.0;
    world.obj_mut::<PygmyIslandLife>(this).crawled_out_of_igloo = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).resting_in_igloo = false;
    world.obj_mut::<PygmyContainersAndCatchers>(this).igloo_logic = None;

    set_selectable(world, this, true);

    pg_engine::logic::set_behavior(world, this, Some("PygmyCrawlOutOfIgloo"));
}

/// `-[PygmyLogic (Island) ejectFromZombieGrave]` @ 0x100096834
pub fn eject_from_zombie_grave(world: &mut World, this: Entity) {
    let zombie_grave = world.obj::<PygmyContainersAndCatchers>(this).zombie_grave_logic;
    if zombie_grave.is_none() {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).trapped_in_zombie_grave = false;
    world.obj_mut::<PygmyContainersAndCatchers>(this).zombie_grave_logic = None;

    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).pos.y += 20.0;

    reset_scale(world, this);
    set_xy_vel(world, this, -200.0, 500.0);
    start_falling(world, this);

    set_selectable(world, this, true);
    set_sacrifice_id(world, this, "DroppedDrown");

    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Island) ejectFromIdolPedestal]` @ 0x10009697c
///
/// The pygmy stayed selectable in the pedestal, so unlike the grave this does not touch it.
pub fn eject_from_idol_pedestal(world: &mut World, this: Entity) {
    let idol_pedestal = world.obj::<PygmyContainersAndCatchers>(this).idol_pedestal_logic;
    if idol_pedestal.is_none() {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).trapped_in_idol_pedestal = false;
    world.obj_mut::<PygmyContainersAndCatchers>(this).idol_pedestal_logic = None;

    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).pos.y += 20.0;

    reset_scale(world, this);
    set_xy_vel(world, this, -200.0, 500.0);
    start_falling(world, this);

    set_sacrifice_id(world, this, "DroppedDrown");

    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Island) ejectFromTempleDoor]` @ 0x100096aac
///
/// Back from the Battle of the Gods: dropped in front of the temple door, with the battle
/// objects released from the pygmy's animation.
pub fn eject_from_temple_door(world: &mut World, this: Entity) {
    let temple_door = world.obj::<PygmyContainersAndCatchers>(this).temple_door_logic;
    if temple_door.is_none() {
        return;
    }

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.battle_of_the_gods = false;
    core.inside_temple_door = false;

    world.obj_mut::<PygmyContainersAndCatchers>(this).temple_door_logic = None;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).island_y_pos = 86.0;

    let display = pg_engine::logic::display_object(world, this);
    let mut object = pg_engine::display::object_mut(world, display);
    object.pos.x = 440.0;
    object.pos.y = 100.0;

    reset_scale(world, this);
    set_xy_vel(world, this, -200.0, 500.0);

    let animation = pg_engine::logic::animation(world, this);
    for channel in BATTLE_OBJECT_CHANNELS {
        pg_engine::animation::clear_tweenable_channel_id(world, animation, channel);
    }

    start_falling(world, this);

    set_selectable(world, this, true);
    set_sacrifice_id(world, this, "DroppedDrown");

    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Island) checkForZombieGrave]` @ 0x1000b4330
///
/// A walking pygmy stumbles into the open grave.
pub fn check_for_zombie_grave(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_stumble = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL;
    let listener = core.pygmy_logic_listener;

    if !can_stumble {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_zombie_grave_enabled(world, listener) {
        return false;
    }

    let zombie_grave = crate::controller::zombie_grave_logic(world, listener, this);
    let Some(zombie_grave) = zombie_grave else {
        return false;
    };

    if !zombie_grave_logic::is_open_and_unoccupied(world, zombie_grave) {
        return false;
    }

    let grave_display = pg_engine::logic::display_object(world, zombie_grave);
    let display = pg_engine::logic::display_object(world, this);
    let grave_x = pg_engine::display::object(world, grave_display).pos.x;
    let x = pg_engine::display::object(world, display).pos.x;

    let is_at_grave = (x - grave_x).abs() <= 30.0;
    if !is_at_grave {
        return false;
    }

    hold_in_place(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).trapped_in_zombie_grave = true;
    world.obj_mut::<PygmyContainersAndCatchers>(this).zombie_grave_logic = Some(zombie_grave);

    zombie_grave_logic::lose_balance_by_grave(world, zombie_grave, this);
    set_selectable(world, this, false);

    // The original reads both positions again through the `pos` pointers it kept, after
    // `loseBalanceByGrave:` has bound the pygmy to the grave's animation.
    let object = pg_engine::display::object(world, display);
    let x_scale = object.scale.x;
    let x = object.pos.x;
    let grave_x = pg_engine::display::object(world, grave_display).pos.x;

    let faces_away_from_grave = (grave_x < x && x_scale > 0.0) || (x < grave_x && x_scale < 0.0);
    let behavior = if faces_away_from_grave {
        "PygmyPushedIntoGraveBackward"
    } else {
        "PygmyPushedIntoGraveForward"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
    true
}

/// `-[PygmyLogic (Island) checkForZombieGraveDuringFall]` @ 0x1000b4568
pub fn check_for_zombie_grave_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_be_caught = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL
        && !core.doodle_jumping
        && !core.dropped_by_hurricane
        && !core.thrown_by_locusts;
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_zombie_grave_enabled(world, listener) {
        return false;
    }

    let zombie_grave = crate::controller::zombie_grave_logic(world, listener, this);
    let Some(zombie_grave) = zombie_grave else {
        return false;
    };

    if !zombie_grave_logic::is_open_and_unoccupied(world, zombie_grave) {
        return false;
    }

    let grave_pos = display_pos(world, zombie_grave);
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let x_distance = (pos.x - grave_pos.x).abs();
    let y_distance = (pos.y - (grave_pos.y + 30.0)).abs();
    let falls_into_grave = vel_y < 0.0 && x_distance <= 30.0 && y_distance <= 30.0;

    if !falls_into_grave {
        return false;
    }

    hold_in_place(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).trapped_in_zombie_grave = true;
    world.obj_mut::<PygmyContainersAndCatchers>(this).zombie_grave_logic = Some(zombie_grave);

    zombie_grave_logic::fall_into_grave(world, zombie_grave, this);
    set_selectable(world, this, false);

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoGrave"));
    true
}

/// `-[PygmyLogic (Island) checkForIdolPedestalDuringFall]` @ 0x1000b47a8
pub fn check_for_idol_pedestal_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_be_caught = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL
        && !core.doodle_jumping
        && !core.invalid_sacrifice
        && !core.dropped_by_hurricane
        && !core.thrown_by_locusts;
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_idol_pedestal_enabled(world, listener) {
        return false;
    }

    let idol_pedestals = crate::controller::idol_pedestal_logic_array(world, listener, this);

    for idol_pedestal in idol_pedestals {
        if !idol_pedestal_logic::is_available(world, idol_pedestal) {
            continue;
        }

        let pedestal_pos = display_pos(world, idol_pedestal);
        let pos = display_pos(world, this);
        let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

        let x_distance = (pos.x - pedestal_pos.x).abs();
        let y_distance = (pos.y - (pedestal_pos.y + 43.0)).abs();
        let falls_into_pedestal = vel_y < 0.0 && x_distance <= 20.0 && y_distance <= 20.0;

        if !falls_into_pedestal {
            continue;
        }

        hold_in_place(world, this);

        world.obj_mut::<PygmyPygmyCoreFrame>(this).trapped_in_idol_pedestal = true;
        world.obj_mut::<PygmyContainersAndCatchers>(this).idol_pedestal_logic = Some(idol_pedestal);

        idol_pedestal_logic::fall_into_idol_pedestal(world, idol_pedestal, this);

        pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoIdolPedestal"));
        return true;
    }

    false
}

/// `-[PygmyLogic (Island) checkForTempleDoorDuringFall]` @ 0x1000b4ae4
pub fn check_for_temple_door_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_be_caught = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL
        && !core.doodle_jumping
        && !core.invalid_sacrifice
        && !core.dropped_by_hurricane
        && !core.thrown_by_locusts;
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_temple_door_enabled(world, listener) {
        return false;
    }

    let temple_door = crate::controller::temple_door_logic(world, listener, this);
    let Some(temple_door) = temple_door else {
        return false;
    };

    if !temple_door_logic::is_available(world, temple_door) {
        return false;
    }

    let door_pos = display_pos(world, temple_door);
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let x_distance = (pos.x - (door_pos.x + 90.0)).abs();
    let y_distance = (pos.y - (door_pos.y + 10.0)).abs();
    let falls_into_door = vel_y < 0.0 && x_distance <= 30.0 && y_distance <= 30.0;

    if !falls_into_door {
        return false;
    }

    hold_in_place(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.tilt_state = 0;
    core.inside_temple_door = true;

    world.obj_mut::<PygmyContainersAndCatchers>(this).temple_door_logic = Some(temple_door);

    temple_door_logic::fall_into_temple_door(world, temple_door, this);
    set_selectable(world, this, false);

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoTempleDoor"));
    true
}

/// `-[PygmyLogic (Island) checkForApocalypseTempleDoorDuringFall:]` @ 0x1000b4d5c
pub fn check_for_apocalypse_temple_door_during_fall_with_arg(world: &mut World, this: Entity, door: Entity) -> bool {
    if !apocalypse_temple_door_logic::is_available(world, door) {
        return false;
    }

    let door_pos = display_pos(world, door);
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let x_distance = (pos.x - door_pos.x).abs();
    let y_distance = (pos.y - (door_pos.y + 50.0)).abs();
    let falls_into_door = vel_y < 0.0 && x_distance <= 30.0 && y_distance <= 30.0;

    if !falls_into_door {
        return false;
    }

    hold_in_place(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.tilt_state = 0;
    core.inside_apocalypse_temple_door = true;

    world.obj_mut::<PygmyContainersAndCatchers>(this).apocalypse_temple_door_logic = Some(door);

    apocalypse_temple_door_logic::fall_into_apocalypse_temple_door(world, door, this);
    set_selectable(world, this, false);

    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    let behavior = if pygmy_type == PYGMY_TYPE_NORMAL {
        "PygmyFallIntoApocalypseTempleDoor"
    } else {
        "TemplePygmyFallIntoApocalypseTempleDoor"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
    true
}

/// `-[PygmyLogic (Island) checkForApocalypseTempleDoorDuringFall]` @ 0x1000b4f10
///
/// Any pygmy type: the temple pygmies leave the rooms through their doors.
pub fn check_for_apocalypse_temple_door_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let current_door = world.obj::<PygmyContainersAndCatchers>(this).apocalypse_temple_door_logic;
    let can_be_caught = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && !core.doodle_jumping
        && !core.invalid_sacrifice
        && !core.dropped_by_hurricane
        && !core.thrown_by_locusts
        && current_door.is_none();
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if crate::controller::is_apocalypse_temple_door_enabled(world, listener) {
        let temple_door = crate::controller::apocalypse_temple_door_logic(world, listener, this);

        if let Some(temple_door) = temple_door
            && check_for_apocalypse_temple_door_during_fall_with_arg(world, this, temple_door)
        {
            return true;
        }
    }

    if crate::controller::is_apocalypse_portal_temple_door_enabled(world, listener) {
        let portal_door = crate::controller::apocalypse_portal_temple_door_logic(world, listener, this);

        if let Some(portal_door) = portal_door
            && check_for_apocalypse_temple_door_during_fall_with_arg(world, this, portal_door)
        {
            return true;
        }
    }

    for (are_doors_enabled, room_doors) in TEMPLE_ROOM_DOORS {
        if !are_doors_enabled(world, listener) {
            continue;
        }

        let doors = room_doors(world, listener, this);
        for door in doors {
            if check_for_apocalypse_temple_door_during_fall_with_arg(world, this, door) {
                return true;
            }
        }
    }

    false
}

/// `-[PygmyLogic (Island) iceHoleSplashComplete]` @ 0x1000b56ec
pub fn ice_hole_splash_complete(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyIslandHazards>(this).ice_hole_logic = None;

    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Island) checkForIceHoleDuringFall]` @ 0x1000b5714
///
/// Only a cooked (burnt) pygmy falls into the ice hole.
pub fn check_for_ice_hole_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_be_caught = core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL
        && !core.invalid_sacrifice
        && !core.doodle_jumping
        && !core.dropped_by_hurricane
        && !core.thrown_by_locusts;
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_ice_hole_enabled(world, listener) {
        return false;
    }

    let ice_hole = crate::controller::ice_hole_logic(world, listener, this);
    let Some(ice_hole) = ice_hole else {
        return false;
    };

    if !ice_hole_logic::is_available_for_falling(world, ice_hole) {
        return false;
    }

    let hole_pos = display_pos(world, ice_hole);
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let x_distance = (pos.x - hole_pos.x).abs();
    let y_distance = (pos.y - (hole_pos.y + 30.0)).abs();
    let falls_into_hole = vel_y < 0.0 && x_distance <= 30.0 && y_distance <= 10.0;

    if !falls_into_hole {
        return false;
    }

    hold_in_place(world, this);
    world.obj_mut::<PygmyIslandHazards>(this).ice_hole_logic = Some(ice_hole);

    ice_hole_logic::burnt_pygmy_fall_into_ice_hole(world, ice_hole, this);

    pg_engine::logic::set_behavior(world, this, Some("PygmyBurntFallIntoIceHole"));
    true
}

/// `-[PygmyLogic (Island) checkForKongStatueDuringFall]` @ 0x1000b5940
pub fn check_for_kong_statue_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_be_caught = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL
        && !core.invalid_sacrifice
        && !core.doodle_jumping
        && !core.dropped_by_hurricane
        && !core.thrown_by_locusts;
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_kong_statue_enabled(world, listener) {
        return false;
    }

    let kong_statue = crate::controller::kong_statue_logic(world, listener, this);
    let Some(kong_statue) = kong_statue else {
        return false;
    };

    if !kong_statue_logic::is_available(world, kong_statue) {
        return false;
    }

    let statue_display = pg_engine::logic::display_object(world, kong_statue);
    let statue_pos = pg_engine::display::object(world, statue_display).pos;
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let x_distance = (pos.x - statue_pos.x + 2.0).abs();
    let y_distance = (pos.y - (statue_pos.y + 23.0)).abs();
    let falls_into_statue = vel_y < 0.0 && x_distance <= 20.0 && y_distance <= 20.0;

    if !falls_into_statue {
        return false;
    }

    hold_in_place(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.in_kong_statue = true;
    core.kong_statue_logic = Some(kong_statue);

    kong_statue_logic::pygmy_fall_into_kong_statue(world, kong_statue, this);
    move_after_container(world, this, statue_display);

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoKongStatue"));
    true
}

/// `-[PygmyLogic (Island) checkForApeWenchDuringFall]` @ 0x1000b5bbc
pub fn check_for_ape_wench_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_be_caught = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL
        && !core.invalid_sacrifice
        && !core.doodle_jumping
        && !core.dropped_by_hurricane
        && !core.thrown_by_locusts;
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_ape_wench_enabled(world, listener) {
        return false;
    }

    let ape_wench = crate::controller::ape_wench_logic(world, listener, this);
    let Some(ape_wench) = ape_wench else {
        return false;
    };

    if !ape_wench_logic::is_available(world, ape_wench) {
        return false;
    }

    let wench_display = pg_engine::logic::display_object(world, ape_wench);
    let wench_pos = pg_engine::display::object(world, wench_display).pos;
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let x_distance = (pos.x - wench_pos.x).abs();
    let y_offset = pos.y - wench_pos.y;
    let falls_into_wench = vel_y < 0.0 && x_distance <= 20.0 && y_offset <= 0.0 && y_offset >= -15.0;

    if !falls_into_wench {
        return false;
    }

    hold_in_place(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.in_ape_wench = true;
    core.ape_wench_logic = Some(ape_wench);

    ape_wench_logic::pygmy_fall_into_ape_wench(world, ape_wench, this);

    let caught_ape_wench = world.obj::<PygmyPygmyCoreFrame>(this).ape_wench_logic;
    incubate_disease(world, this, caught_ape_wench, "InfectApeWinch", "InfectPygmyFromApeWinch");

    move_after_container(world, this, wench_display);

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoApeWench"));
    true
}

/// `-[PygmyLogic (Island) checkForApeBongoDuringFall]` @ 0x1000b5e68
pub fn check_for_ape_bongo_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_be_caught = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL
        && !core.invalid_sacrifice
        && !core.doodle_jumping
        && !core.dropped_by_hurricane
        && !core.thrown_by_locusts;
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_ape_bongo_enabled(world, listener) {
        return false;
    }

    let ape_bongo = crate::controller::ape_bongo_logic(world, listener, this);
    let Some(ape_bongo) = ape_bongo else {
        return false;
    };

    if !ape_bongo_logic::is_available(world, ape_bongo) {
        return false;
    }

    let bongo_display = pg_engine::logic::display_object(world, ape_bongo);
    let bongo_pos = pg_engine::display::object(world, bongo_display).pos;
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let x_distance = (pos.x - bongo_pos.x).abs();
    let y_offset = pos.y - bongo_pos.y;
    let falls_into_bongo = vel_y < 0.0 && x_distance <= 20.0 && y_offset <= 0.0 && y_offset >= -15.0;

    if !falls_into_bongo {
        return false;
    }

    hold_in_place(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.in_ape_bongo = true;
    core.ape_bongo_logic = Some(ape_bongo);

    ape_bongo_logic::pygmy_fall_into_ape_bongo(world, ape_bongo, this);

    let caught_ape_bongo = world.obj::<PygmyPygmyCoreFrame>(this).ape_bongo_logic;
    incubate_disease(world, this, caught_ape_bongo, "InfectApeBongos", "InfectPygmyFromApeBongos");

    move_after_container(world, this, bongo_display);

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoApeBongo"));
    true
}

/// `-[PygmyLogic (Island) checkForApeAltarDuringFall]` @ 0x1000b6114
pub fn check_for_ape_altar_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_be_caught = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL
        && !core.invalid_sacrifice
        && !core.doodle_jumping
        && !core.dropped_by_hurricane
        && !core.thrown_by_locusts;
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_ape_altar_enabled(world, listener) {
        return false;
    }

    let ape_altar = crate::controller::ape_altar_logic(world, listener, this);
    let Some(ape_altar) = ape_altar else {
        return false;
    };

    if !ape_altar_logic::is_available(world, ape_altar) {
        return false;
    }

    let altar_display = pg_engine::logic::display_object(world, ape_altar);
    let altar_pos = pg_engine::display::object(world, altar_display).pos;
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let x_distance = (pos.x - altar_pos.x).abs();
    let y_offset = pos.y - altar_pos.y;
    let falls_into_altar = vel_y < 0.0 && x_distance <= 20.0 && y_offset <= 0.0 && y_offset >= -15.0;

    if !falls_into_altar {
        return false;
    }

    hold_in_place(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.in_ape_altar = true;
    core.ape_altar_logic = Some(ape_altar);

    ape_altar_logic::pygmy_fall_into_ape_altar(world, ape_altar, this);

    let caught_ape_altar = world.obj::<PygmyPygmyCoreFrame>(this).ape_altar_logic;
    incubate_disease(world, this, caught_ape_altar, "InfectApeAltar", "InfectPygmyFromApeAltar");

    move_after_container(world, this, altar_display);

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoApeAltar"));
    true
}

/// `-[PygmyLogic (Island) checkForIglooDuringFall]` @ 0x1000b63c0
pub fn check_for_igloo_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_be_caught = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL
        && !core.invalid_sacrifice
        && !core.doodle_jumping
        && !core.dropped_by_hurricane
        && !core.thrown_by_locusts;
    let listener = core.pygmy_logic_listener;

    if !can_be_caught {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_igloo_enabled(world, listener) {
        return false;
    }

    let igloo = crate::controller::igloo_logic(world, listener, this);
    let Some(igloo) = igloo else {
        return false;
    };

    if !igloo_logic::is_pygmy_fall_available(world, igloo, this) {
        return false;
    }

    let igloo_pos = display_pos(world, igloo);
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let x_distance = (pos.x - igloo_pos.x).abs();
    let y_distance = (pos.y - (igloo_pos.y + 48.0)).abs();
    let falls_into_igloo = vel_y < 0.0 && x_distance <= 30.0 && y_distance <= 30.0;

    if !falls_into_igloo {
        return false;
    }

    hold_in_place(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).resting_in_igloo = true;
    world.obj_mut::<PygmyContainersAndCatchers>(this).igloo_logic = Some(igloo);

    igloo_logic::pygmy_fall_into_igloo(world, igloo, this);
    set_selectable(world, this, false);

    let caught_igloo = world.obj::<PygmyContainersAndCatchers>(this).igloo_logic;
    incubate_disease(world, this, caught_igloo, "InfectIgloo", "InfectPygmyFromIgloo");

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoIgloo"));
    true
}

/// `-[PygmyLogic (Island) tripIntoZombieGrave:::]` @ 0x1000b80e8
///
/// The grave plays the fall forward, or backward when the pygmy faces away from it.
pub fn seq_trip_into_zombie_grave(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let listed_grave = match listener {
        Some(listener) => crate::controller::zombie_grave_logic(world, listener, this),
        None => None,
    };

    // `[[nil displayObject] xPos]` is 0.
    let grave_x = match listed_grave {
        Some(listed_grave) => display_pos(world, listed_grave).x,
        None => 0.0,
    };

    let display = pg_engine::logic::display_object(world, this);
    let object = pg_engine::display::object(world, display);
    let x_scale = object.scale.x;
    let x = object.pos.x;

    let faces_away_from_grave = (grave_x < x && x_scale > 0.0) || (x < grave_x && x_scale < 0.0);
    let fall_forward = !faces_away_from_grave;

    let zombie_grave = world.obj::<PygmyContainersAndCatchers>(this).zombie_grave_logic;
    if let Some(zombie_grave) = zombie_grave {
        zombie_grave_logic::trip_into_grave(world, zombie_grave, this, fall_forward);
    }

    ItemResult::Done
}
