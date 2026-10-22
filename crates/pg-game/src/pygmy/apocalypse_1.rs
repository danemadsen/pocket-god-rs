//! `PygmyLogic`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use crate::logic::apocalypse_locust_closet_xenu as closet_xenu;
use crate::logic::island_core::sprite_logic;
use crate::logic::island_core::sprite_logic::SpriteLogic;

/// `PygmyLogic` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct PygmyApocalypse {
    /// `mbUseLocustCatch1`: `B`
    pub use_locust_catch1: bool,

    /// `mThrowingLocustLogic`: `@"LocustLogic"`
    pub throwing_locust_logic: Option<Entity>,

    /// `mMutantLocustIdleLoopCount`: `i`
    pub mutant_locust_idle_loop_count: i32,

    /// `mDiseaseIncubateIdleCycles`: `i`
    pub disease_incubate_idle_cycles: i32,

    /// `mbPlagueBodyInPile`: `B`
    pub plague_body_in_pile: bool,

    /// `mbPlagueBodyPickedUp`: `B`
    pub plague_body_picked_up: bool,

    /// `mbPlagueBodyBeingCarried`: `B`
    pub plague_body_being_carried: bool,

    /// `mPlagueBodyCarryingPygmyLogic`: `@"PygmyLogic"`
    pub plague_body_carrying_pygmy_logic: Option<Entity>,

    /// `mCarryingPlagueBodyLogic`: `@"PygmyLogic"`
    pub carrying_plague_body_logic: Option<Entity>,

    /// `mbWaitingForPossessionByThetan`: `B`
    pub waiting_for_possession_by_thetan: bool,

    /// `mThetanLogic`: `@"ThetanLogic"`
    pub thetan_logic: Option<Entity>,

    /// `mThetanBrawlCoolDown`: `f`
    pub thetan_brawl_cool_down: f32,

    /// `mThetanAttackerPygmyLogic`: `@"PygmyLogic"`
    pub thetan_attacker_pygmy_logic: Option<Entity>,

    /// `mThetanDefenderPygmyLogic`: `@"PygmyLogic"`
    pub thetan_defender_pygmy_logic: Option<Entity>,

    /// `mClosetTempleThetanMeterLogic`: `@"ClosetTempleThetanMeterLogic"`
    pub closet_temple_thetan_meter_logic: Option<Entity>,

    /// `mClosetTempleCouchLogic`: `@"ClosetTempleCouchLogic"`
    pub closet_temple_couch_logic: Option<Entity>,

    /// `mClosetTempleClosetDoorLogic`: `@"ClosetTempleClosetDoorLogic"`
    pub closet_temple_closet_door_logic: Option<Entity>,

    /// `mXenuLogic`: `@"XenuLogic"`
    pub xenu_logic: Option<Entity>,

    /// `mbStartTomBabySacrifice`: `B`
    pub start_tom_baby_sacrifice: bool,

    /// `mbSacrificedToXenu`: `B`
    pub sacrificed_to_xenu: bool,

    /// `mTomPygmyLogic`: `@"PygmyLogic"`
    pub tom_pygmy_logic: Option<Entity>,

    /// `mBabyPygmyLogic`: `@"PygmyLogic"`
    pub baby_pygmy_logic: Option<Entity>,

    /// `mbTempleOldPygmyUpright`: `B`
    pub temple_old_pygmy_upright: bool,
}

/// The display marker the closet temple's pygmies are drawn after.
const CLOSET_DISPLAY_MARKER: &str = "ApocalypseTempleClosetDisplayMarker";

/// `mPersonaPersonality` of Tom.
const PERSONALITY_TOM: i32 = 2;

/// `mTimeTemplePygmyType` values.
const PYGMY_TYPE_NORMAL: i32 = 0;
const PYGMY_TYPE_OLD: i32 = 1;
const PYGMY_TYPE_BABY: i32 = 2;

/// `islandLocationState` of the closet temple and of the Xenu temple.
const CLOSET_TEMPLE: i32 = 16;
const XENU_TEMPLE: i32 = 17;

/// `SpriteLogic::mApocalypseDiseaseType` of the plague.
const DISEASE_PLAGUE: i32 = 1;

/// The animation channels the thetan brawl's attacker and Tom's baby are bound to.
const THETAN_ATTACKER_CHANNEL: usize = 16;
const TOM_BABY_CHANNEL: usize = 15;

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

/// `[mDisplayObject setZRot:0]`.
fn clear_z_rot(world: &mut World, this: Entity) {
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).rot.z = 0.0;
}

/// `mPosVel.x = mPosVel.y = 0`.
fn stop_moving(world: &mut World, this: Entity) {
    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 0.0;
    logic.pos_vel.y = 0.0;
}

/// `[[IdolManager instance] triggerEpisodeEvent:]`.
fn trigger_episode(world: &mut World, event: &str) {
    let idol_manager = crate::managers::idol_manager::instance(world);

    if let Some(idol_manager) = idol_manager {
        crate::managers::idol_manager::trigger_episode_event(world, idol_manager, event);
    }
}

/// `[[mPygmyLogicListener pygmyDisplayMarker] insertAfter:mDisplayObject]`, then
/// `[self updateLabelLayer]`.
fn move_after_pygmy_display_marker(world: &mut World, this: Entity) {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let display = pg_engine::logic::display_object(world, this);

    let marker = listener.and_then(|listener| crate::controller::pygmy_display_marker(world, listener));
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display));
    }

    update_label_layer(world, this);
}

/// `[[[BCLibrary instance] getDisplayMarkerById:@"ApocalypseTempleClosetDisplayMarker"]
/// insertAfter:mDisplayObject]`, then `[self updateLabelLayer]`.
fn move_after_closet_display_marker(world: &mut World, this: Entity) {
    let display = pg_engine::logic::display_object(world, this);

    let marker = pg_engine::library::display_marker(world, CLOSET_DISPLAY_MARKER);
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display));
    }

    update_label_layer(world, this);
}

/// The drop shared by the closet temple and Tom's arms ejects: scale (1, 1), the island floor
/// back at 86, tossed at (-200, 500) under gravity onto the floor, selectable again, a
/// drowning sacrifice, and drawn after the pygmy display marker.
fn eject_and_drop(world: &mut World, this: Entity) {
    let display = pg_engine::logic::display_object(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).island_y_pos = 86.0;

    let mut object = pg_engine::display::object_mut(world, display);
    object.scale.x = 1.0;
    object.scale.y = 1.0;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = -200.0;
    logic.pos_vel.y = 500.0;

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = true;

    set_selectable(world, this, true);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("DroppedDrown".to_string());

    move_after_pygmy_display_marker(world, this);
}

/// The stop list S13 (`docs/re/pygmy.md` §1.3).
fn stop_activities(world: &mut World, this: Entity) {
    stop_being_high_on_nitrous(world, this);
    stop_eating(world, this);
    stop_carrying(world, this);
    stop_carrying_plague_body(world, this);
    stop_fishing(world, this);
    stop_sleeping(world, this);
    stop_warming_by_bon_fire(world, this);
    stop_staring_at_rainbow(world, this);
    stop_staring_at_kidnapping(world, this);
    stop_waiting_to_crap(world, this);
    stop_throwing_spear(world, this);
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);
}

/// The stop list S14: S13 with `stopBeingEaten` after `stopSleeping`.
fn stop_activities_and_being_eaten(world: &mut World, this: Entity) {
    stop_being_high_on_nitrous(world, this);
    stop_eating(world, this);
    stop_carrying(world, this);
    stop_carrying_plague_body(world, this);
    stop_fishing(world, this);
    stop_sleeping(world, this);
    stop_being_eaten(world, this);
    stop_warming_by_bon_fire(world, this);
    stop_staring_at_rainbow(world, this);
    stop_staring_at_kidnapping(world, this);
    stop_waiting_to_crap(world, this);
    stop_throwing_spear(world, this);
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);
}

/// The first frame of the timed walks (`templePygmyWalk:::`, `possessedByThetanWalk:::`): a
/// random duration in hundredths between the two times, side collision and gravity on, then
/// the walk direction rule (`docs/re/pygmy.md` §1.2) at `speed`.
fn start_timed_walk(world: &mut World, this: Entity, min_time: f32, max_time: f32, speed: f32) {
    let random = world.resource_mut::<Random>().arc4random();
    let stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);

    pg_engine::logic::logic_mut(world, this).stall_time = stall_time;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_side_collision = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let coin = world.resource_mut::<Random>().arc4random();
    let x = display_pos(world, this).x;

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let min_x = listener.map_or(0.0, |listener| crate::controller::island_min_x_pos(world, listener));
    let max_x = listener.map_or(0.0, |listener| crate::controller::island_max_x_pos(world, listener));

    let go_left = x == max_x || (coin & 1 == 0 && x != min_x);
    let (x_scale, x_vel) = if go_left { (-1.0, -speed) } else { (1.0, speed) };

    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).scale.x = x_scale;
    pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;
}

/// Every frame of the timed walks: done (and stopped) once the pygmy hits the island side or
/// its time is up; the timer then runs down by the frame's dt.
fn run_timed_walk(world: &mut World, this: Entity, delta_time: f32) -> ItemResult {
    let side_collision = world.obj::<PygmyPygmyCoreFrame>(this).island_side_collision;
    let stall_time = pg_engine::logic::logic(world, this).stall_time;

    let result = if side_collision || stall_time <= 0.0 {
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    pg_engine::logic::logic_mut(world, this).stall_time = stall_time - delta_time;
    result
}

/// The timer-only idles (`templePygmyIdle:::`, `possessedByThetanIdle:::`): a random duration
/// in hundredths between `minTime` and `maxTime`.
fn run_timed_idle(world: &mut World, this: Entity, call: &SequenceCall) -> ItemResult {
    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");

        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);

        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;
    }

    let stall_time = pg_engine::logic::logic(world, this).stall_time;
    let result = if stall_time <= 0.0 { ItemResult::Done } else { ItemResult::Pending };

    pg_engine::logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    result
}

/// `-[PygmyLogic (Apocalypse) stopBeingPossessedByThetan]` @ 0x10003a58c
pub fn stop_being_possessed_by_thetan(world: &mut World, this: Entity) {
    let thetan = world.obj::<PygmyApocalypse>(this).thetan_logic;
    let Some(thetan) = thetan else {
        return;
    };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).possessed_by_thetan = false;
    world.obj_mut::<PygmyApocalypse>(this).waiting_for_possession_by_thetan = false;

    closet_xenu::thetan_logic::stop_possessing_pygmy(world, thetan, this);

    world.obj_mut::<PygmyApocalypse>(this).thetan_logic = None;
}

/// `-[PygmyLogic (Apocalypse) stopThetanBrawl]` @ 0x10003a5f8
pub fn stop_thetan_brawl(world: &mut World, this: Entity) {
    set_selectable(world, this, true);

    let attacker = world.obj::<PygmyApocalypse>(this).thetan_attacker_pygmy_logic;
    if let Some(attacker) = attacker {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).thetan_brawl = false;

        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, THETAN_ATTACKER_CHANNEL);

        stop_thetan_brawl_by_pygmy(world, attacker, this);
        world.obj_mut::<PygmyApocalypse>(this).thetan_attacker_pygmy_logic = None;
    }

    let defender = world.obj::<PygmyApocalypse>(this).thetan_defender_pygmy_logic;
    if let Some(defender) = defender {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).thetan_brawl = false;

        stop_thetan_brawl_by_pygmy(world, defender, this);
        world.obj_mut::<PygmyApocalypse>(this).thetan_defender_pygmy_logic = None;
    }
}

/// `-[PygmyLogic (Apocalypse) stopBeingOnClosetTempleThetanMeter]` @ 0x10003a6dc
pub fn stop_being_on_closet_temple_thetan_meter(world: &mut World, this: Entity) {
    let meter = world.obj::<PygmyApocalypse>(this).closet_temple_thetan_meter_logic;
    let Some(meter) = meter else {
        return;
    };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).on_closet_temple_thetan_meter = false;

    closet_xenu::closet_temple_thetan_meter_logic::eject_from_closet_temple_thetan_meter(world, meter, this);

    world.obj_mut::<PygmyApocalypse>(this).closet_temple_thetan_meter_logic = None;
}

/// `-[PygmyLogic (Apocalypse) stopBeingOnClosetTempleCouch]` @ 0x10003a738
pub fn stop_being_on_closet_temple_couch(world: &mut World, this: Entity) {
    let couch = world.obj::<PygmyApocalypse>(this).closet_temple_couch_logic;
    let Some(couch) = couch else {
        return;
    };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).on_closet_temple_couch = false;

    closet_xenu::closet_temple_couch_logic::eject_from_closet_temple_couch(world, couch, this);

    world.obj_mut::<PygmyApocalypse>(this).closet_temple_couch_logic = None;
}

/// `-[PygmyLogic (Apocalypse) stopBeingInClosetTempleClosetDoor]` @ 0x10003a794
pub fn stop_being_in_closet_temple_closet_door(world: &mut World, this: Entity) {
    let door = world.obj::<PygmyApocalypse>(this).closet_temple_closet_door_logic;
    let Some(door) = door else {
        return;
    };

    let personality = world.obj::<PygmyPygmyCoreFrame>(this).persona_personality;
    if personality == PERSONALITY_TOM {
        trigger_episode(world, "RemoveTomFromCloset");
    }

    move_after_pygmy_display_marker(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).in_closet_temple_closet_door = false;

    closet_xenu::closet_temple_closet_door_logic::eject_from_closet_temple_closet_door(world, door, this);

    world.obj_mut::<PygmyApocalypse>(this).closet_temple_closet_door_logic = None;
}

/// `-[PygmyLogic (Apocalypse) stopSlammingClosetTempleClosetDoor]` @ 0x10003a88c
pub fn stop_slamming_closet_temple_closet_door(world: &mut World, this: Entity) {
    let door = world.obj::<PygmyApocalypse>(this).closet_temple_closet_door_logic;
    let Some(door) = door else {
        return;
    };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).slamming_closet_temple_closet_door = false;

    closet_xenu::closet_temple_closet_door_logic::eject_from_closet_temple_closet_door(world, door, this);

    world.obj_mut::<PygmyApocalypse>(this).closet_temple_closet_door_logic = None;
}

/// `-[PygmyLogic (Apocalypse) stopBeingHeldByXenu]` @ 0x10003a8e8
///
/// Leaves `mbHeldByXenu` set.
pub fn stop_being_held_by_xenu(world: &mut World, this: Entity) {
    let xenu = world.obj::<PygmyApocalypse>(this).xenu_logic;
    let Some(xenu) = xenu else {
        return;
    };

    closet_xenu::xenu_logic::eject_from_xenu(world, xenu, this);

    world.obj_mut::<PygmyApocalypse>(this).xenu_logic = None;
}

/// `-[PygmyLogic (Apocalypse) stopBeingHeldInTomsArms]` @ 0x10003a930
pub fn stop_being_held_in_toms_arms(world: &mut World, this: Entity) {
    let tom = world.obj::<PygmyApocalypse>(this).tom_pygmy_logic;
    let Some(tom) = tom else {
        return;
    };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).held_in_toms_arms = false;

    eject_from_toms_arms_with_arg(world, tom, this);

    world.obj_mut::<PygmyApocalypse>(this).tom_pygmy_logic = None;
}

/// `-[PygmyLogic (Apocalypse) stopBeingThrownByLocusts]` @ 0x10003a98c
pub fn stop_being_thrown_by_locusts(world: &mut World, this: Entity) {
    let locust = world.obj::<PygmyApocalypse>(this).throwing_locust_logic;
    let Some(locust) = locust else {
        return;
    };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_floor_collision = true;
    core.thrown_by_locusts = false;

    set_selectable(world, this, true);
    move_after_pygmy_display_marker(world, this);

    closet_xenu::locust_logic::stop_being_thrown_by_locusts(world, locust, this);

    world.obj_mut::<PygmyApocalypse>(this).throwing_locust_logic = None;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("DroppedDrown".to_string());

    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Apocalypse) thetanStopPossessingPygmy:]` @ 0x10003b2c8
pub fn thetan_stop_possessing_pygmy(world: &mut World, this: Entity, _thetan: Entity) {
    let thetan = world.obj::<PygmyApocalypse>(this).thetan_logic;
    if thetan.is_none() {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).possessed_by_thetan = false;

    let mut apocalypse = world.obj_mut::<PygmyApocalypse>(this);
    apocalypse.waiting_for_possession_by_thetan = false;
    apocalypse.thetan_logic = None;

    pg_engine::logic::set_behavior(world, this, Some("PygmyUnpossessedByThetan"));
}

/// `-[PygmyLogic (Apocalypse) stopThetanBrawlByPygmy:]` @ 0x10003b31c
pub fn stop_thetan_brawl_by_pygmy(world: &mut World, this: Entity, _pygmy: Entity) {
    set_selectable(world, this, true);

    let attacker = world.obj::<PygmyApocalypse>(this).thetan_attacker_pygmy_logic;
    if attacker.is_some() {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).thetan_brawl = false;

        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, THETAN_ATTACKER_CHANNEL);

        world.obj_mut::<PygmyApocalypse>(this).thetan_attacker_pygmy_logic = None;
        pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
    }

    let defender = world.obj::<PygmyApocalypse>(this).thetan_defender_pygmy_logic;
    if defender.is_some() {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).thetan_brawl = false;
        world.obj_mut::<PygmyApocalypse>(this).thetan_defender_pygmy_logic = None;

        pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
    }

    let died_from_thetan_brawl = world.obj::<PygmyPygmyCoreFrame>(this).died_from_thetan_brawl;
    if died_from_thetan_brawl {
        pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
    }
}

/// `-[PygmyLogic (Apocalypse) ejectFromClosetTempleThetanMeter]` @ 0x10003b440
pub fn eject_from_closet_temple_thetan_meter(world: &mut World, this: Entity) {
    let meter = world.obj::<PygmyApocalypse>(this).closet_temple_thetan_meter_logic;
    if meter.is_none() {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).on_closet_temple_thetan_meter = false;
    world.obj_mut::<PygmyApocalypse>(this).closet_temple_thetan_meter_logic = None;

    eject_and_drop(world, this);

    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Apocalypse) ejectFromClosetTempleCouch]` @ 0x10003b5b0
pub fn eject_from_closet_temple_couch(world: &mut World, this: Entity) {
    let couch = world.obj::<PygmyApocalypse>(this).closet_temple_couch_logic;
    if couch.is_none() {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).on_closet_temple_couch = false;
    world.obj_mut::<PygmyApocalypse>(this).closet_temple_couch_logic = None;

    eject_and_drop(world, this);

    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Apocalypse) ejectFromClosetTempleClosetDoor]` @ 0x10003b720
pub fn eject_from_closet_temple_closet_door(world: &mut World, this: Entity) {
    let door = world.obj::<PygmyApocalypse>(this).closet_temple_closet_door_logic;
    if door.is_none() {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).in_closet_temple_closet_door = false;
    world.obj_mut::<PygmyApocalypse>(this).closet_temple_closet_door_logic = None;

    eject_and_drop(world, this);

    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Apocalypse) ejectFromXenu]` @ 0x10003b890
pub fn eject_from_xenu(world: &mut World, this: Entity) {
    let xenu = world.obj::<PygmyApocalypse>(this).xenu_logic;
    if xenu.is_none() {
        return;
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));

    world.obj_mut::<PygmyApocalypse>(this).xenu_logic = None;
}

/// `-[PygmyLogic (Apocalypse) ejectFromTomsArms]` @ 0x10003b8e0
pub fn eject_from_toms_arms(world: &mut World, this: Entity) {
    let tom = world.obj::<PygmyApocalypse>(this).tom_pygmy_logic;
    if tom.is_none() {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).held_in_toms_arms = false;
    world.obj_mut::<PygmyApocalypse>(this).tom_pygmy_logic = None;

    eject_and_drop(world, this);

    let sacrificed_to_xenu = world.obj::<PygmyApocalypse>(this).sacrificed_to_xenu;
    let behavior = if sacrificed_to_xenu { "PygmyDie" } else { "TemplePygmyFall" };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Apocalypse) incubateDisease:infectChallengeEvent:infectedByChallengeEvent:]` @ 0x10003ba70
///
/// `incubate_disease` is the object the pygmy touches; the two events are the episode events
/// of the pygmy infecting it and of it infecting the pygmy.
pub fn incubate_disease(world: &mut World, this: Entity, incubate_disease: Option<Entity>, infect_challenge_event: &str, infected_by_challenge_event: &str) {
    let source = incubate_disease;
    let source_disease = crate::send::apocalypse_disease_type(world, source);
    let own_disease = world.obj::<SpriteLogic>(this).apocalypse_disease_type;

    if own_disease == source_disease {
        return;
    }

    if own_disease == 0 {
        let mut apocalypse = world.obj_mut::<PygmyApocalypse>(this);
        if apocalypse.disease_incubate_idle_cycles < 5 {
            apocalypse.disease_incubate_idle_cycles = 5;
        }

        trigger_episode(world, infected_by_challenge_event);
        sprite_logic::infect_with_disease(world, this, source_disease);
        return;
    }

    if source_disease == 0 {
        trigger_episode(world, infect_challenge_event);

        if let Some(source) = source {
            sprite_logic::infect_with_disease(world, source, own_disease);
        }
        return;
    }

    let roll = world.resource_mut::<Random>().arc4random_mod(3);
    match roll {
        2 => {
            // Both are cured.
            sprite_logic::infect_with_disease(world, this, 0);

            if let Some(source) = source {
                sprite_logic::infect_with_disease(world, source, 0);
            }
        }
        1 => {
            if let Some(source) = source {
                sprite_logic::infect_with_disease(world, source, own_disease);
            }
        }
        _ => {
            let mut apocalypse = world.obj_mut::<PygmyApocalypse>(this);
            if apocalypse.disease_incubate_idle_cycles < 5 {
                apocalypse.disease_incubate_idle_cycles = 5;
            }

            sprite_logic::infect_with_disease(world, this, source_disease);
        }
    }
}

/// `-[PygmyLogic (Apocalypse) moveToApocalypseTempleDisplayGroup]` @ 0x10003bc2c
pub fn move_to_apocalypse_temple_display_group(world: &mut World, this: Entity) {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let tom_in_closet = core.persona_personality == PERSONALITY_TOM && core.in_closet_temple_closet_door;

    if tom_in_closet {
        return;
    }

    move_after_pygmy_display_marker(world, this);
}

/// `-[PygmyLogic (Apocalypse) moveFromApocalypseTempleDisplayGroup]` @ 0x10003bcd0
pub fn move_from_apocalypse_temple_display_group(world: &mut World, this: Entity) {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let tom_in_closet = core.persona_personality == PERSONALITY_TOM && core.in_closet_temple_closet_door;

    if tom_in_closet {
        return;
    }

    move_after_pygmy_display_marker(world, this);
}

/// `-[PygmyLogic (Apocalypse) dropIntoApocalypseTempleRoom]` @ 0x10003bd74
pub fn drop_into_apocalypse_temple_room(world: &mut World, this: Entity) {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let personality = core.persona_personality;
    let tom_in_closet = personality == PERSONALITY_TOM && core.in_closet_temple_closet_door;
    let held_by_xenu = core.held_by_xenu;
    let died_from_thetan_brawl = core.died_from_thetan_brawl;
    let listener = core.pygmy_logic_listener;

    if tom_in_closet {
        return;
    }

    if held_by_xenu {
        stop_being_held_by_xenu(world, this);
        pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
        return;
    }

    let sacrificed_to_xenu = world.obj::<PygmyApocalypse>(this).sacrificed_to_xenu;
    if sacrificed_to_xenu {
        pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
        return;
    }

    if died_from_thetan_brawl {
        stop_thetan_brawl(world, this);
        pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
        return;
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let busy = core.remove_from_story_mode
        || core.dropped_by_hurricane
        || core.hit_by_lightning
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_frozen
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.slamming_closet_temple_closet_door
        || core.in_kong_statue
        || core.in_ape_wench
        || core.in_ape_bongo
        || core.in_ape_altar
        || core.in_go_go_cage
        || core.in_dance_dance_execution
        || core.resting_in_igloo
        || core.trapped_in_spider_web
        || core.eaten_by_locust_queen
        || core.killed_by_locusts
        || core.killed_by_purification
        || core.dance_dance_executed
        || core.zombie_explode
        || core.is_zombie
        || core.is_ghost
        || core.touched_by_ghost
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.catch_shark
        || core.shark_on_head
        || core.swarmed
        || core.caught_by_bird
        || core.burnt_by_magnifying_glass
        || core.tumble
        || core.in_crack_hole
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.beheaded_by_squid
        || core.cooked;
    let can_drop = !busy && (!core.drowning || core.killed_by_plague) && !core.underwater;

    if !can_drop {
        if personality == PERSONALITY_TOM {
            let location = listener.map_or(0, |listener| crate::controller::island_location_state(world, listener));

            if location == CLOSET_TEMPLE {
                world.obj_mut::<PygmyPygmyBehaviourItems>(this).tell_closet_to_grab_new_tom = true;
            }
        }

        pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
        return;
    }

    stop_being_high_on_nitrous(world, this);
    stop_eating(world, this);
    stop_carrying(world, this);
    stop_carrying_plague_body(world, this);
    stop_fishing(world, this);
    stop_sleeping(world, this);
    stop_warming_by_bon_fire(world, this);
    stop_staring_at_rainbow(world, this);
    stop_staring_at_kidnapping(world, this);
    stop_waiting_to_crap(world, this);
    stop_throwing_spear(world, this);
    stop_being_trapped_in_zombie_grave(world, this);
    stop_being_trapped_in_idol_pedestal(world, this);
    stop_being_inside_temple_door(world, this);
    stop_being_inside_apocalypse_temple_door(world, this);
    stop_being_on_top_apocalypse_time_temple_platform(world, this);
    stop_being_in_apocalypse_temple_fountain(world, this);
    stop_thetan_brawl(world, this);
    stop_being_on_closet_temple_thetan_meter(world, this);
    stop_being_on_closet_temple_couch(world, this);
    stop_being_in_closet_temple_closet_door(world, this);
    stop_being_held_in_toms_arms(world, this);
    stop_holding_baby_pygmy(world, this);
    stop_being_thrown_by_locusts(world, this);
    stop_playing_catch_with_monkey(world, this);
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);

    let hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane_logic;
    if let Some(hurricane) = hurricane {
        crate::logic::sky_powers::hurricane_logic::drop_sprite_logic(world, hurricane, this);

        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_hurricane_logic = None;
        sprite.picked_up_by_hurricane = false;
    }

    let dragging = world.obj::<PygmyPygmyCoreFrame>(this).dragging;
    if dragging {
        world.obj_mut::<PygmyPygmyTouchDrag>(this).over_fire = false;

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.dragging = false;
        core.apply_spring = false;
        core.drag_label_x_offset = 0.0;
        core.drag_label_y_offset = 0.0;

        if let Some(listener) = listener {
            crate::controller::on_pygmy_drag_complete(world, listener, this);
        }
    }

    let doodle_jumping = world.obj::<PygmyPygmyCoreFrame>(this).doodle_jumping;
    if doodle_jumping {
        world.obj_mut::<PygmyMinigames>(this).doodle_on_ice = false;

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.doodle_jumping = false;
        core.doodle_knocked_down = false;

        if let Some(listener) = listener {
            crate::controller::on_pygmy_stop_doodle_jump(world, listener, this);
        }
    }

    let min_x = listener.map_or(0.0, |listener| crate::controller::island_min_x_pos(world, listener));
    let max_x = listener.map_or(0.0, |listener| crate::controller::island_max_x_pos(world, listener));

    let random = world.resource_mut::<Random>().arc4random();
    let x = pg_engine::logic::random_scaled_time(min_x + 15.0, max_x - 15.0, 10.0, random);

    let display = pg_engine::logic::display_object(world, this);
    let mut object = pg_engine::display::object_mut(world, display);
    object.pos.x = x;
    object.pos.y = 340.0;
    object.pos.z = -277.12;
    object.scale.x = 1.0;
    object.scale.y = 1.0;

    stop_moving(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.tilt_state = 0;
    core.enable_island_side_collision = false;
    core.enable_island_floor_collision = true;
    core.inside_apocalypse_temple_door = false;

    world.obj_mut::<PygmyContainersAndCatchers>(this).apocalypse_temple_door_logic = None;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    set_selectable(world, this, true);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.invalid_sacrifice = true;
    core.sacrifice_id = Some("DroppedDrown".to_string());

    let behavior = if core.killed_by_plague {
        if core.killed_by_plague_disappear {
            "PygmyDie"
        } else {
            "PygmyPlagueBodyFallOffWorld"
        }
    } else if core.cut_in_half_by_charlie {
        "HalfPygmyFallInsideApocalypseTempleRoom"
    } else if core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL {
        "PygmyFallInsideApocalypseTempleRoom"
    } else {
        "TemplePygmyFallInsideApocalypseTempleRoom"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Apocalypse) checkTempleOldPygmyFallDownFromSlide:::]` @ 0x10003cae0
pub fn seq_check_temple_old_pygmy_fall_down_from_slide(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let upright = world.obj::<PygmyApocalypse>(this).temple_old_pygmy_upright;
    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;

    if !upright || pygmy_type != PYGMY_TYPE_OLD {
        return ItemResult::Done;
    }

    world.obj_mut::<PygmyApocalypse>(this).temple_old_pygmy_upright = false;
    pg_engine::logic::set_behavior(world, this, Some("TempleOldPygmyFallDownFromSlide"));

    ItemResult::Replaced
}

/// `-[PygmyLogic (Apocalypse) templeOldPygmyFallDown:::]` @ 0x10003cb44
pub fn seq_temple_old_pygmy_fall_down(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyApocalypse>(this).temple_old_pygmy_upright = false;

    ItemResult::Done
}

/// `-[PygmyLogic (Apocalypse) templePygmyWalk:::]` @ 0x10003cb5c
pub fn seq_temple_pygmy_walk(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let speed = call.float("xPosVel");

        start_timed_walk(world, this, min_time, max_time, speed);
    }

    let found_food = check_for_coconut(world, this)
        || check_for_fish(world, this)
        || check_for_bird(world, this)
        || check_for_banana(world, this);

    if found_food {
        return ItemResult::Replaced;
    }

    run_timed_walk(world, this, call.delta_time)
}

/// `-[PygmyLogic (Apocalypse) templePygmyIdle:::]` @ 0x10003ce08
pub fn seq_temple_pygmy_idle(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    run_timed_idle(world, this, call)
}

/// `-[PygmyLogic (Apocalypse) checkForClosetTempleThetanMeterDuringFall]` @ 0x10003d88c
pub fn check_for_closet_temple_thetan_meter_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let busy = core.cooked
        || core.killed_by_plague
        || core.cut_in_half_by_charlie
        || core.invalid_sacrifice
        || core.doodle_jumping
        || core.dropped_by_hurricane
        || core.thrown_by_locusts
        || core.time_temple_pygmy_type != PYGMY_TYPE_NORMAL;
    let listener = core.pygmy_logic_listener;

    if busy {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    let meter_enabled = crate::controller::is_closet_temple_thetan_meter_enabled(world, listener);
    if !meter_enabled {
        return false;
    }

    let meter = crate::controller::closet_temple_thetan_meter_logic(world, listener, this);
    let Some(meter) = meter else {
        return false;
    };

    let available = closet_xenu::closet_temple_thetan_meter_logic::is_available(world, meter);
    if !available {
        return false;
    }

    let meter_pos = display_pos(world, meter);
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    if 0.0 <= vel_y {
        return false;
    }

    let distance_x = (pos.x - meter_pos.x).abs();
    let distance_y = (pos.y - meter_pos.y).abs();
    let inside = distance_x <= 20.0 && distance_y <= 20.0;

    if !inside {
        return false;
    }

    stop_moving(world, this);
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_floor_collision = false;
    core.tilt_state = 0;
    core.on_closet_temple_thetan_meter = true;

    world.obj_mut::<PygmyApocalypse>(this).closet_temple_thetan_meter_logic = Some(meter);

    closet_xenu::closet_temple_thetan_meter_logic::fall_onto_closet_temple_thetan_meter(world, meter, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyFallOntoClosetTempleThetanMeter"));

    true
}

/// `-[PygmyLogic (Apocalypse) checkForClosetTempleCouchDuringFall]` @ 0x10003dadc
pub fn check_for_closet_temple_couch_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let busy = core.cooked
        || core.killed_by_plague
        || core.cut_in_half_by_charlie
        || core.invalid_sacrifice
        || core.doodle_jumping
        || core.dropped_by_hurricane
        || core.thrown_by_locusts
        || core.time_temple_pygmy_type != PYGMY_TYPE_NORMAL;
    let listener = core.pygmy_logic_listener;

    if busy {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    let couch_enabled = crate::controller::is_closet_temple_couch_enabled(world, listener);
    if !couch_enabled {
        return false;
    }

    let door_enabled = crate::controller::is_closet_temple_closet_door_enabled(world, listener);
    if !door_enabled {
        return false;
    }

    let door = crate::controller::closet_temple_closet_door_logic(world, listener, this);
    let couch = crate::controller::closet_temple_couch_logic(world, listener, this);

    let Some(door) = door else {
        return false;
    };

    let door_available = closet_xenu::closet_temple_closet_door_logic::is_available(world, door);
    if !door_available {
        return false;
    }

    let Some(couch) = couch else {
        return false;
    };

    let couch_available = closet_xenu::closet_temple_couch_logic::is_available(world, couch);
    if !couch_available {
        return false;
    }

    let couch_pos = display_pos(world, couch);
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    if 0.0 <= vel_y {
        return false;
    }

    let distance_x = (pos.x - (couch_pos.x + 50.0)).abs();
    let distance_y = (pos.y - (couch_pos.y + -25.0)).abs();
    let inside = distance_x <= 20.0 && distance_y <= 20.0;

    if !inside {
        return false;
    }

    stop_moving(world, this);
    clear_z_rot(world, this);

    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).pos.y = couch_pos.y + -35.0;

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_floor_collision = false;
    core.tilt_state = 0;

    move_after_closet_display_marker(world, this);

    let in_closet = world.obj::<PygmyPygmyCoreFrame>(this).in_closet_temple_closet_door;
    if !in_closet {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).in_closet_temple_closet_door = true;
        world.obj_mut::<PygmyApocalypse>(this).closet_temple_closet_door_logic = Some(door);

        closet_xenu::closet_temple_closet_door_logic::fall_into_closet_temple_closet_door(world, door, this);
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).on_closet_temple_couch = true;
    world.obj_mut::<PygmyApocalypse>(this).closet_temple_couch_logic = Some(couch);

    closet_xenu::closet_temple_couch_logic::fall_onto_closet_temple_couch(world, couch, this);

    let personality = world.obj::<PygmyPygmyCoreFrame>(this).persona_personality;
    let behavior = if personality == PERSONALITY_TOM {
        "PygmyTomFallOntoClosetTempleCouch"
    } else {
        "PygmyFallOntoClosetTempleCouch"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));

    true
}

/// `-[PygmyLogic (Apocalypse) runSequenceIfInXenuRoom:::]` @ 0x10003de70
pub fn seq_run_sequence_if_in_xenu_room(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;

    let location = listener.map_or(0, |listener| crate::controller::island_location_state(world, listener));
    if location != XENU_TEMPLE {
        return ItemResult::Done;
    }

    let apocalypse_active = listener.is_some_and(|listener| crate::controller::is_apocalypse_active(world, listener));
    if apocalypse_active {
        return ItemResult::Done;
    }

    let Some(id) = call.param("id") else {
        return ItemResult::Done;
    };

    pg_engine::logic::set_behavior(world, this, Some(id));

    ItemResult::Replaced
}

/// `-[PygmyLogic (Apocalypse) checkTomJumpOnCouch:::]` @ 0x10003df04
pub fn seq_check_tom_jump_on_couch(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let tom_in_closet = core.in_closet_temple_closet_door && core.persona_personality == PERSONALITY_TOM;
    let listener = core.pygmy_logic_listener;

    if !tom_in_closet {
        return ItemResult::Done;
    }

    let Some(listener) = listener else {
        return ItemResult::Done;
    };

    let couch_enabled = crate::controller::is_closet_temple_couch_enabled(world, listener);
    if !couch_enabled {
        return ItemResult::Done;
    }

    let couch = crate::controller::closet_temple_couch_logic(world, listener, this);
    let Some(couch) = couch else {
        return ItemResult::Done;
    };

    let available = closet_xenu::closet_temple_couch_logic::is_available(world, couch);
    if !available {
        return ItemResult::Done;
    }

    let couch_x = display_pos(world, couch).x;
    let display = pg_engine::logic::display_object(world, this);
    let object = pg_engine::display::object(world, display);
    let x = object.pos.x;
    let x_scale = object.scale.x;

    let couch_start_x = couch_x + 10.0;
    let couch_end_x = couch_x + 90.0;
    let couch_center_x = (couch_start_x + couch_end_x) * 0.5;
    let facing_couch = x_scale * (couch_center_x - x);

    let jump = 0.0 < facing_couch && couch_start_x <= x && x <= couch_end_x;
    if !jump {
        return ItemResult::Done;
    }

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_floor_collision = false;
    core.tilt_state = 0;
    core.on_closet_temple_couch = true;

    world.obj_mut::<PygmyApocalypse>(this).closet_temple_couch_logic = Some(couch);

    closet_xenu::closet_temple_couch_logic::jump_onto_closet_temple_couch(world, couch, this);

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = x_scale * 100.0;
    logic.pos_vel.y = 600.0;

    pg_engine::logic::set_behavior(world, this, Some("PygmyTomJumpOnCouch"));

    ItemResult::Replaced
}

/// `-[PygmyLogic (Apocalypse) waitApexTomJumpOnCouch:::]` @ 0x10003e114
pub fn seq_wait_apex_tom_jump_on_couch(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    if vel_y < 0.0 { ItemResult::Done } else { ItemResult::Pending }
}

/// `-[PygmyLogic (Apocalypse) waitLandTomJumpOnCouch:::]` @ 0x10003e138
pub fn seq_wait_land_tom_jump_on_couch(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let couch = listener.and_then(|listener| crate::controller::closet_temple_couch_logic(world, listener, this));

    // The original reads the position of a nil couch's display object (a crash); the port waits.
    let Some(couch) = couch else {
        return ItemResult::Pending;
    };

    let couch_y = display_pos(world, couch).y;
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let seat_y = couch_y + -35.0;
    let landed = vel_y < 0.0 && (pos.y - seat_y).abs() <= 20.0;

    if !landed {
        return ItemResult::Pending;
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    stop_moving(world, this);

    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).pos.y = seat_y;

    ItemResult::Done
}

/// `-[PygmyLogic (Apocalypse) tomJumpOffCouch:::]` @ 0x10003e238
pub fn seq_tom_jump_off_couch(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    stop_being_on_closet_temple_couch(world, this);

    let display = pg_engine::logic::display_object(world, this);
    let x_scale = pg_engine::display::object(world, display).scale.x;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = x_scale * 100.0;
    logic.pos_vel.y = 600.0;

    ItemResult::Done
}

/// `-[PygmyLogic (Apocalypse) waitLandTomJumpOffCouch:::]` @ 0x10003e2c4
pub fn seq_wait_land_tom_jump_off_couch(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let door = listener.and_then(|listener| crate::controller::closet_temple_closet_door_logic(world, listener, this));

    let pos = display_pos(world, this);
    let closet_y = door.map_or(0.0, |door| closet_xenu::closet_temple_closet_door_logic::closet_y_pos(world, door));
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let landed = vel_y < 0.0 && (pos.y - closet_y).abs() <= 20.0;
    if !landed {
        return ItemResult::Pending;
    }

    stop_moving(world, this);

    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).pos.y = closet_y;

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    ItemResult::Done
}

/// `-[PygmyLogic (Apocalypse) checkSlamDoorBouncingOnCouch:::]` @ 0x10003e3a4
pub fn seq_check_slam_door_bouncing_on_couch(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let on_couch = world.obj::<PygmyPygmyCoreFrame>(this).on_closet_temple_couch;

    if !on_couch {
        return ItemResult::Done;
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyTomJumpOffCouchAndSlamDoor"));

    ItemResult::Replaced
}

/// `-[PygmyLogic (Apocalypse) canBePossessedByThetan]` @ 0x10003e3ec
pub fn can_be_possessed_by_thetan(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let sprite = world.obj::<SpriteLogic>(this);

    let busy = core.touch_began_inside
        || core.remove_from_story_mode
        || core.tilt_state != 0
        || core.cut_in_half_by_charlie
        || core.time_temple_pygmy_type != PYGMY_TYPE_NORMAL
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || sprite.blown_by_hurricane
        || sprite.picked_up_by_hurricane
        || core.dropped_by_hurricane
        || core.catch_shark
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_frozen
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.in_kong_statue
        || core.in_ape_wench
        || core.in_ape_bongo
        || core.in_ape_altar
        || core.in_go_go_cage
        || core.in_dance_dance_execution
        || core.resting_in_igloo
        || core.crapping
        || core.trapped_in_spider_web
        || core.trapped_in_zombie_grave
        || core.trapped_in_idol_pedestal
        || core.inside_temple_door
        || core.inside_apocalypse_temple_door
        || core.on_top_apocalypse_time_temple_platform
        || core.eaten_by_locust_queen
        || core.in_apocalypse_temple_fountain
        || core.thetan_brawl
        || core.on_closet_temple_thetan_meter
        || core.on_closet_temple_couch
        || core.in_closet_temple_closet_door
        || core.slamming_closet_temple_closet_door
        || core.held_by_xenu
        || core.held_in_toms_arms
        || core.holding_baby_pygmy
        || core.thrown_by_locusts
        || core.killed_by_locusts
        || core.killed_by_purification
        || core.dance_dance_executed
        || core.battle_of_the_gods
        || core.zombie_explode
        || core.is_zombie
        || core.is_ghost
        || core.touched_by_ghost
        || core.possessed_by_thetan
        || core.caught_by_monkey
        || core.monkey_on_back
        || core.eaten_by_moron_snake
        || core.wearing_water_wings
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.skewered
        || core.underwater
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.dragging
        || core.shark_on_head
        || core.crushed
        || core.caught_by_bird
        || core.falling
        || core.in_crack_hole
        || core.doodle_jumping
        || core.swarmed
        || core.burnt_by_magnifying_glass
        || core.tumble
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.cooked
        || core.killed_by_plague
        || core.sleeping
        || core.drowning;

    !busy
}

/// `-[PygmyLogic (Apocalypse) startPossessedByThetan:]` @ 0x10003eab4
pub fn start_possessed_by_thetan(world: &mut World, this: Entity, possessed_by_thetan: Entity) {
    let thetan = possessed_by_thetan;

    stop_activities_and_being_eaten(world, this);

    stop_moving(world, this);
    clear_z_rot(world, this);

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;

    set_selectable(world, this, false);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).possessed_by_thetan = true;

    let mut apocalypse = world.obj_mut::<PygmyApocalypse>(this);
    apocalypse.waiting_for_possession_by_thetan = true;
    apocalypse.thetan_logic = Some(thetan);

    pg_engine::logic::set_behavior(world, this, Some("PygmyStartPossessedByThetan"));
}

/// `-[PygmyLogic (Apocalypse) animReleaseThetan]` @ 0x10003ecac
pub fn anim_release_thetan(world: &mut World, this: Entity) {
    stop_being_possessed_by_thetan(world, this);
}

/// `-[PygmyLogic (Apocalypse) possessedByThetan:]` @ 0x10003ecbc
pub fn possessed_by_thetan(world: &mut World, this: Entity, _thetan: Entity) {
    world.obj_mut::<PygmyApocalypse>(this).waiting_for_possession_by_thetan = false;

    set_selectable(world, this, true);

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let apocalypse_active = listener.is_some_and(|listener| crate::controller::is_apocalypse_active(world, listener));

    let behavior = if apocalypse_active {
        "PygmyPossessedByApocalypseThetan"
    } else {
        "PygmyPossessedByThetan"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Apocalypse) checkForPossessedByThetan:::]` @ 0x10003ed54
pub fn seq_check_for_possessed_by_thetan(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let possessed = world.obj::<PygmyPygmyCoreFrame>(this).possessed_by_thetan;

    if !possessed {
        return ItemResult::Done;
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyPossessedByThetanIdle"));

    ItemResult::Replaced
}

/// `-[PygmyLogic (Apocalypse) isPossessedByThetan]` @ 0x10003ed9c
pub fn is_possessed_by_thetan(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyPygmyCoreFrame>(this).possessed_by_thetan
}

/// `-[PygmyLogic (Apocalypse) canDoThetanBrawl:]` @ 0x10003edb0
pub fn can_do_thetan_brawl(world: &mut World, this: Entity, _attacker: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let sprite = world.obj::<SpriteLogic>(this);
    let apocalypse = world.obj::<PygmyApocalypse>(this);

    let busy = core.touch_began_inside
        || core.remove_from_story_mode
        || core.tilt_state != 0
        || core.cut_in_half_by_charlie
        || core.time_temple_pygmy_type != PYGMY_TYPE_NORMAL
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || sprite.blown_by_hurricane
        || sprite.picked_up_by_hurricane
        || core.dropped_by_hurricane
        || core.catch_shark
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_frozen
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.in_kong_statue
        || core.in_ape_wench
        || core.in_ape_bongo
        || core.in_ape_altar
        || core.in_go_go_cage
        || core.in_dance_dance_execution
        || core.resting_in_igloo
        || core.crapping
        || core.trapped_in_spider_web
        || core.trapped_in_zombie_grave
        || core.trapped_in_idol_pedestal
        || core.inside_temple_door
        || core.inside_apocalypse_temple_door
        || core.on_top_apocalypse_time_temple_platform
        || core.eaten_by_locust_queen
        || core.in_apocalypse_temple_fountain
        || apocalypse.waiting_for_possession_by_thetan
        || core.thetan_brawl
        || core.on_closet_temple_thetan_meter
        || core.on_closet_temple_couch
        || core.in_closet_temple_closet_door
        || core.slamming_closet_temple_closet_door
        || core.held_by_xenu
        || core.held_in_toms_arms
        || core.holding_baby_pygmy
        || core.thrown_by_locusts
        || core.killed_by_locusts
        || core.killed_by_purification
        || core.dance_dance_executed
        || core.battle_of_the_gods
        || core.zombie_explode
        || core.is_zombie
        || core.is_ghost
        || core.touched_by_ghost
        || core.caught_by_monkey
        || core.monkey_on_back
        || core.eaten_by_moron_snake
        || core.wearing_water_wings
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.skewered
        || core.underwater
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.dragging
        || core.shark_on_head
        || core.crushed
        || core.caught_by_bird
        || core.falling
        || core.in_crack_hole
        || core.doodle_jumping
        || core.swarmed
        || core.burnt_by_magnifying_glass
        || core.tumble
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.cooked
        || core.killed_by_plague
        || core.sleeping
        || core.drowning;

    !busy
}

/// `-[PygmyLogic (Apocalypse) defendAgainstThetanBrawl:]` @ 0x10003f478
pub fn defend_against_thetan_brawl(world: &mut World, this: Entity, defend_against_thetan_brawl: Entity) {
    let attacker = defend_against_thetan_brawl;

    stop_activities_and_being_eaten(world, this);

    stop_moving(world, this);
    clear_z_rot(world, this);

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.tilt_state = 0;
    core.thetan_brawl = true;

    world.obj_mut::<PygmyApocalypse>(this).thetan_attacker_pygmy_logic = Some(attacker);

    let possessed = world.obj::<PygmyPygmyCoreFrame>(this).possessed_by_thetan;
    let behavior = if possessed {
        "PygmyThetanBrawlDefend"
    } else {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).died_from_thetan_brawl = true;
        set_selectable(world, this, false);

        let animation = pg_engine::logic::animation(world, this);
        let attacker_display = pg_engine::logic::display_object(world, attacker);
        pg_engine::animation::set_tweenable(world, animation, Some(attacker_display), THETAN_ATTACKER_CHANNEL);

        world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("BittenToDeath".to_string());
        "PygmyThetanBrawlGetJumped"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Apocalypse) checkForThetanBrawl]` @ 0x10003f6f4
pub fn check_for_thetan_brawl(world: &mut World, this: Entity) -> bool {
    let display = pg_engine::logic::display_object(world, this);
    let object = pg_engine::display::object(world, display);
    let x = object.pos.x;
    let x_scale = object.scale.x;

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let pygmies = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };

    for other in pygmies {
        if other == this {
            continue;
        }

        let can_brawl = can_do_thetan_brawl(world, other, this);
        if !can_brawl {
            continue;
        }

        let other_display = pg_engine::logic::display_object(world, other);
        let other_x = pg_engine::display::object(world, other_display).pos.x;
        let distance = x - other_x;
        let other_possessed = is_possessed_by_thetan(world, other);
        let abs_distance = distance.abs();

        let facing_other = (0.0 < distance && x_scale < 0.0) || (distance < 0.0 && 0.0 < x_scale);

        if !other_possessed {
            let pounce = 60.0 < abs_distance && abs_distance < 70.0 && facing_other;
            if !pounce {
                continue;
            }

            stop_moving(world, this);
            clear_z_rot(world, this);

            pg_engine::display::object_mut(world, other_display).scale.x = x_scale;

            world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
            world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;

            set_selectable(world, this, false);

            world.obj_mut::<PygmyPygmyCoreFrame>(this).thetan_brawl = true;

            let mut apocalypse = world.obj_mut::<PygmyApocalypse>(this);
            apocalypse.thetan_defender_pygmy_logic = Some(other);
            apocalypse.thetan_brawl_cool_down = 2.0;

            defend_against_thetan_brawl(world, other, this);

            let coin = world.resource_mut::<Random>().arc4random();
            let behavior = if coin & 1 == 0 { "PygmyThetanBrawlPummel" } else { "PygmyThetanBrawlEatFace" };

            pg_engine::logic::set_behavior(world, this, Some(behavior));
            return true;
        }

        let fight = 25.0 < abs_distance && abs_distance < 35.0 && facing_other;
        if !fight {
            continue;
        }

        pg_engine::display::object_mut(world, other_display).scale.x = -x_scale;

        stop_moving(world, this);
        clear_z_rot(world, this);

        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.tilt_state = 0;
        core.thetan_brawl = true;

        let mut apocalypse = world.obj_mut::<PygmyApocalypse>(this);
        apocalypse.thetan_defender_pygmy_logic = Some(other);
        apocalypse.thetan_brawl_cool_down = 2.0;

        defend_against_thetan_brawl(world, other, this);
        pg_engine::logic::set_behavior(world, this, Some("PygmyThetanBrawlAttack"));
        return true;
    }

    false
}

/// `-[PygmyLogic (Apocalypse) thetanPounceComplete]` @ 0x10003fb84
pub fn thetan_pounce_complete(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).thetan_brawl = false;
    world.obj_mut::<PygmyApocalypse>(this).thetan_defender_pygmy_logic = None;

    set_selectable(world, this, true);
    clear_z_rot(world, this);

    pg_engine::logic::set_behavior(world, this, Some("PygmyThetanBrawlJumpOff"));
}

/// `-[PygmyLogic (Apocalypse) thetanBrawlGetPummeled]` @ 0x10003fc14
pub fn thetan_brawl_get_pummeled(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("ThetanPummeled".to_string());

    pg_engine::logic::set_behavior(world, this, Some("PygmyThetanBrawlGetPummeled"));
}

/// `-[PygmyLogic (Apocalypse) thetanBrawlGetFaceEaten]` @ 0x10003fc44
pub fn thetan_brawl_get_face_eaten(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("BittenToDeath".to_string());

    pg_engine::logic::set_behavior(world, this, Some("PygmyThetanBrawlGetFaceEaten"));
}

/// `-[PygmyLogic (Apocalypse) animPounceComplete]` @ 0x10003fc74
pub fn anim_pounce_complete(world: &mut World, this: Entity) {
    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, THETAN_ATTACKER_CHANNEL);

    let attacker = world.obj::<PygmyApocalypse>(this).thetan_attacker_pygmy_logic;
    if let Some(attacker) = attacker {
        thetan_pounce_complete(world, attacker);
    }

    world.obj_mut::<PygmyApocalypse>(this).thetan_attacker_pygmy_logic = None;
}

/// `-[PygmyLogic (Apocalypse) thetanPummelPygmy:::]` @ 0x10003fcd8
pub fn seq_thetan_pummel_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let defender = world.obj::<PygmyApocalypse>(this).thetan_defender_pygmy_logic;

    if let Some(defender) = defender {
        thetan_brawl_get_pummeled(world, defender);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Apocalypse) thetanEatPygmyFace:::]` @ 0x10003fd0c
pub fn seq_thetan_eat_pygmy_face(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let defender = world.obj::<PygmyApocalypse>(this).thetan_defender_pygmy_logic;

    if let Some(defender) = defender {
        thetan_brawl_get_face_eaten(world, defender);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Apocalypse) thetanBrawlLandPunch]` @ 0x10003fd40
pub fn thetan_brawl_land_punch(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("PygmyThetanBrawlGetHit"));
}

/// `-[PygmyLogic (Apocalypse) animLandThetanPunch]` @ 0x10003fd58
pub fn anim_land_thetan_punch(world: &mut World, this: Entity) {
    let defender = world.obj::<PygmyApocalypse>(this).thetan_defender_pygmy_logic;
    if let Some(defender) = defender {
        thetan_brawl_land_punch(world, defender);
    }

    let attacker = world.obj::<PygmyApocalypse>(this).thetan_attacker_pygmy_logic;
    if let Some(attacker) = attacker {
        thetan_brawl_land_punch(world, attacker);
    }
}

/// `-[PygmyLogic (Apocalypse) animStopThetanPossession]` @ 0x10003fdc4
pub fn anim_stop_thetan_possession(world: &mut World, this: Entity) {
    stop_being_possessed_by_thetan(world, this);
}

/// `-[PygmyLogic (Apocalypse) thetanBrawlThrowPunch]` @ 0x10003fdd4
pub fn thetan_brawl_throw_punch(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("PygmyThetanBrawlThrowPunch"));
}

/// `-[PygmyLogic (Apocalypse) thetanBrawlStartSlapFight]` @ 0x10003fdec
pub fn thetan_brawl_start_slap_fight(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).thetan_brawl = false;
    world.obj_mut::<PygmyApocalypse>(this).thetan_attacker_pygmy_logic = None;

    pg_engine::logic::set_behavior(world, this, Some("PygmyThetanBrawlStartSlapFight"));
}

/// `-[PygmyLogic (Apocalypse) chooseBrawlAttack:::]` @ 0x10003fe24
pub fn seq_choose_brawl_attack(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let attack = world.resource_mut::<Random>().arc4random_mod(3);

    match attack {
        2 => {
            world.obj_mut::<PygmyPygmyCoreFrame>(this).thetan_brawl = false;
            pg_engine::logic::set_behavior(world, this, Some("PygmyThetanBrawlStartSlapFight"));

            let defender = world.obj::<PygmyApocalypse>(this).thetan_defender_pygmy_logic;
            if let Some(defender) = defender {
                thetan_brawl_start_slap_fight(world, defender);
            }

            world.obj_mut::<PygmyApocalypse>(this).thetan_defender_pygmy_logic = None;
            ItemResult::Replaced
        }
        1 => {
            let defender = world.obj::<PygmyApocalypse>(this).thetan_defender_pygmy_logic;
            if let Some(defender) = defender {
                thetan_brawl_throw_punch(world, defender);
            }

            ItemResult::Done
        }
        _ => {
            pg_engine::logic::set_behavior(world, this, Some("PygmyThetanBrawlThrowPunch"));
            ItemResult::Replaced
        }
    }
}

/// `-[PygmyLogic (Apocalypse) brawlAttackComplete:::]` @ 0x10003ff14
pub fn seq_brawl_attack_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let defender = world.obj::<PygmyApocalypse>(this).thetan_defender_pygmy_logic;

    if defender.is_none() {
        return ItemResult::Done;
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyThetanBrawlAttackLoop"));

    ItemResult::Replaced
}

/// `-[PygmyLogic (Apocalypse) possessedByThetanWalk:::]` @ 0x10003ff5c
pub fn seq_possessed_by_thetan_walk(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");

        start_timed_walk(world, this, min_time, max_time, 50.0);
    }

    let mut apocalypse = world.obj_mut::<PygmyApocalypse>(this);
    apocalypse.thetan_brawl_cool_down -= call.delta_time;
    let cool_down = apocalypse.thetan_brawl_cool_down;

    if !(0.0 <= cool_down) {
        let brawl = check_for_thetan_brawl(world, this);
        if brawl {
            return ItemResult::Replaced;
        }
    }

    run_timed_walk(world, this, call.delta_time)
}

/// `-[PygmyLogic (Apocalypse) possessedByThetanIdle:::]` @ 0x1000401dc
pub fn seq_possessed_by_thetan_idle(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    run_timed_idle(world, this, call)
}

/// `-[PygmyLogic (Apocalypse) startInClosetTempleClosetDoor]` @ 0x1000402dc
pub fn start_in_closet_temple_closet_door(world: &mut World, this: Entity) {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let door = listener.and_then(|listener| crate::controller::closet_temple_closet_door_logic(world, listener, this));

    let min_closet_x = door.map_or(0.0, |door| closet_xenu::closet_temple_closet_door_logic::min_closet_x_pos(world, door));
    let max_closet_x = door.map_or(0.0, |door| closet_xenu::closet_temple_closet_door_logic::max_closet_x_pos(world, door));
    let closet_y = door.map_or(0.0, |door| closet_xenu::closet_temple_closet_door_logic::closet_y_pos(world, door));

    let display = pg_engine::logic::display_object(world, this);
    let mut object = pg_engine::display::object_mut(world, display);
    object.pos.x = (min_closet_x + max_closet_x) * 0.5;
    object.pos.y = closet_y;

    clear_z_rot(world, this);
    stop_moving(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.falling = false;
    core.invalid_sacrifice = false;

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_floor_collision = false;
    core.tilt_state = 0;

    move_after_closet_display_marker(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).in_closet_temple_closet_door = true;
    world.obj_mut::<PygmyApocalypse>(this).closet_temple_closet_door_logic = door;

    if let Some(door) = door {
        closet_xenu::closet_temple_closet_door_logic::fall_into_closet_temple_closet_door(world, door, this);
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoClosetTempleClosetDoor"));
}

/// `-[PygmyLogic (Apocalypse) checkForClosetTempleClosetDoorDuringFall]` @ 0x1000404d0
pub fn check_for_closet_temple_closet_door_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let busy = core.cooked
        || core.killed_by_plague
        || core.cut_in_half_by_charlie
        || core.invalid_sacrifice
        || core.doodle_jumping
        || core.dropped_by_hurricane
        || core.thrown_by_locusts
        || core.time_temple_pygmy_type != PYGMY_TYPE_NORMAL;
    let listener = core.pygmy_logic_listener;

    if busy {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    let door_enabled = crate::controller::is_closet_temple_closet_door_enabled(world, listener);
    if !door_enabled {
        return false;
    }

    let door = crate::controller::closet_temple_closet_door_logic(world, listener, this);
    let Some(door) = door else {
        return false;
    };

    let available = closet_xenu::closet_temple_closet_door_logic::is_available(world, door);
    if !available {
        return false;
    }

    let pos = display_pos(world, this);
    let min_closet_x = closet_xenu::closet_temple_closet_door_logic::min_closet_x_pos(world, door);
    let max_closet_x = closet_xenu::closet_temple_closet_door_logic::max_closet_x_pos(world, door);
    let closet_y = closet_xenu::closet_temple_closet_door_logic::closet_y_pos(world, door);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    let inside = vel_y < 0.0
        && min_closet_x < pos.x
        && pos.x < max_closet_x
        && (pos.y - closet_y).abs() <= 20.0;

    if !inside {
        return false;
    }

    stop_moving(world, this);

    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).pos.y = closet_y;

    clear_z_rot(world, this);

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_floor_collision = false;
    core.tilt_state = 0;

    move_after_closet_display_marker(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).in_closet_temple_closet_door = true;
    world.obj_mut::<PygmyApocalypse>(this).closet_temple_closet_door_logic = Some(door);

    closet_xenu::closet_temple_closet_door_logic::fall_into_closet_temple_closet_door(world, door, this);

    let personality = world.obj::<PygmyPygmyCoreFrame>(this).persona_personality;
    if personality != PERSONALITY_TOM {
        trigger_episode(world, "PlacePygmyInCloset");
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoClosetTempleClosetDoor"));

    true
}

/// `-[PygmyLogic (Apocalypse) checkForTomPygmyDuringFall]` @ 0x1000407f8
pub fn check_for_tom_pygmy_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let busy = core.cooked
        || core.killed_by_plague
        || core.cut_in_half_by_charlie
        || core.invalid_sacrifice
        || core.doodle_jumping
        || core.dropped_by_hurricane
        || core.thrown_by_locusts
        || core.persona_personality == PERSONALITY_TOM
        || core.time_temple_pygmy_type != PYGMY_TYPE_BABY;
    let listener = core.pygmy_logic_listener;

    if busy {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    let xenu_temple_enabled = crate::controller::is_xenu_temple_enabled(world, listener);
    if !xenu_temple_enabled {
        return false;
    }

    let xenu_enabled = crate::controller::is_xenu_enabled(world, listener);
    if xenu_enabled {
        return false;
    }

    let tom_alive = crate::controller::is_tom_pygmy_alive(world, listener);
    if !tom_alive {
        return false;
    }

    let tom = crate::controller::tom_pygmy_logic(world, listener, this);
    let Some(tom) = tom else {
        return false;
    };

    let holding_available = is_baby_holding_available(world, tom);
    if !holding_available {
        return false;
    }

    let tom_pos = display_pos(world, tom);
    let pos = display_pos(world, this);
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    if 0.0 <= vel_y {
        return false;
    }

    let distance_x = (pos.x - tom_pos.x).abs();
    let distance_y = (pos.y - (tom_pos.y + 20.0)).abs();
    let inside = distance_x <= 20.0 && distance_y <= 20.0;

    if !inside {
        return false;
    }

    stop_moving(world, this);
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_floor_collision = false;
    core.tilt_state = 0;
    core.held_in_toms_arms = true;

    world.obj_mut::<PygmyApocalypse>(this).tom_pygmy_logic = Some(tom);

    fall_into_toms_arms(world, tom, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoTomsArms"));

    true
}

/// `-[PygmyLogic (Apocalypse) startTomBabySacrifice]` @ 0x100040a9c
pub fn start_tom_baby_sacrifice(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyApocalypse>(this).sacrificed_to_xenu = true;

    set_selectable(world, this, false);
}

/// `-[PygmyLogic (Apocalypse) babySacrificeHideLabel]` @ 0x100040ad4
pub fn baby_sacrifice_hide_label(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).disable_labels = true;
}

/// `-[PygmyLogic (Apocalypse) animTomBabySacrificeComplete]` @ 0x100040aec
pub fn anim_tom_baby_sacrifice_complete(world: &mut World, this: Entity) {
    let baby = world.obj::<PygmyApocalypse>(this).baby_pygmy_logic;

    if let Some(baby) = baby {
        baby_sacrifice_hide_label(world, baby);
    }
}

/// `-[PygmyLogic (Apocalypse) animTomShowXenu]` @ 0x100040b14
pub fn anim_tom_show_xenu(world: &mut World, this: Entity) {
    let baby = world.obj::<PygmyApocalypse>(this).baby_pygmy_logic;

    if let Some(baby) = baby {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).holding_baby_pygmy = false;

        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, TOM_BABY_CHANNEL);

        baby_sacrifice_complete(world, baby);
        world.obj_mut::<PygmyApocalypse>(this).baby_pygmy_logic = None;
    }

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let xenu = listener.and_then(|listener| crate::controller::xenu_logic(world, listener, this));
    if let Some(xenu) = xenu {
        closet_xenu::xenu_logic::spawn(world, xenu);
    }

    world.obj_mut::<PygmyApocalypse>(this).start_tom_baby_sacrifice = false;

    set_selectable(world, this, true);
}

/// `-[PygmyLogic (Apocalypse) babySacrificeComplete]` @ 0x100040bf4
pub fn baby_sacrifice_complete(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyApocalypse>(this).tom_pygmy_logic = None;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("BecameXenu".to_string());

    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Apocalypse) walkToXenu:::]` @ 0x100040c34
pub fn seq_walk_to_xenu(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let xenu = listener.and_then(|listener| crate::controller::xenu_logic(world, listener, this));
    let xenu_x = xenu.map_or(0.0, |xenu| display_pos(world, xenu).x);

    let display = pg_engine::logic::display_object(world, this);

    if call.first_time {
        let x = pg_engine::display::object(world, display).pos.x;
        let (x_scale, x_vel) = if x <= xenu_x { (1.0, 50.0) } else { (-1.0, -50.0) };

        pg_engine::display::object_mut(world, display).scale.x = x_scale;
        pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;
    }

    let object = pg_engine::display::object(world, display);
    let x = object.pos.x;
    let x_scale = object.scale.x;

    let arrived = (0.0 < x_scale && xenu_x + -5.0 < x) || (x_scale < 0.0 && x < xenu_x + 5.0);
    if !arrived {
        return ItemResult::Pending;
    }

    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
    pg_engine::display::object_mut(world, display).pos.x = xenu_x;

    let xenu_temple = listener.and_then(|listener| crate::controller::xenu_temple_logic(world, listener, this));
    if let Some(xenu_temple) = xenu_temple {
        closet_xenu::xenu_temple_logic::tear_down_wall(world, xenu_temple);
    }

    world.obj_mut::<PygmyApocalypse>(this).start_tom_baby_sacrifice = true;

    set_selectable(world, this, false);

    let baby = world.obj::<PygmyApocalypse>(this).baby_pygmy_logic;
    if let Some(baby) = baby {
        start_tom_baby_sacrifice(world, baby);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Apocalypse) isBabyHoldingAvailable]` @ 0x100040dfc
pub fn is_baby_holding_available(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let apocalypse = world.obj::<PygmyApocalypse>(this);

    let busy = core.touch_began_inside
        || core.remove_from_story_mode
        || core.cut_in_half_by_charlie
        || core.time_temple_pygmy_type != PYGMY_TYPE_NORMAL
        || core.dropped_by_hurricane
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || core.catch_shark
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_frozen
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.resting_in_igloo
        || core.crapping
        || core.trapped_in_spider_web
        || core.trapped_in_zombie_grave
        || core.trapped_in_idol_pedestal
        || core.inside_temple_door
        || core.inside_apocalypse_temple_door
        || core.on_top_apocalypse_time_temple_platform
        || core.eaten_by_locust_queen
        || core.in_apocalypse_temple_fountain
        || core.thetan_brawl
        || core.on_closet_temple_thetan_meter
        || core.on_closet_temple_couch
        || core.in_closet_temple_closet_door
        || core.slamming_closet_temple_closet_door
        || core.held_by_xenu
        || core.held_in_toms_arms
        || core.holding_baby_pygmy
        || core.thrown_by_locusts
        || core.killed_by_locusts
        || core.killed_by_purification
        || core.dance_dance_executed
        || core.battle_of_the_gods
        || core.zombie_explode
        || core.is_ghost
        || core.touched_by_ghost
        || core.possessed_by_thetan
        || core.caught_by_monkey
        || core.monkey_on_back
        || core.eaten_by_moron_snake
        || core.wearing_water_wings
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.skewered
        || core.underwater
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.dragging
        || core.crushed
        || core.caught_by_bird
        || core.falling
        || core.in_crack_hole
        || core.doodle_jumping
        || core.swarmed
        || core.burnt_by_magnifying_glass
        || core.tumble
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.cooked
        || core.killed_by_plague
        || core.drowning
        || core.persona_personality != PERSONALITY_TOM
        || apocalypse.baby_pygmy_logic.is_some();

    !busy
}

/// `-[PygmyLogic (Apocalypse) fallIntoTomsArms:]` @ 0x100041400
pub fn fall_into_toms_arms(world: &mut World, this: Entity, fall_into_toms_arms: Entity) {
    let baby = fall_into_toms_arms;

    stop_moving(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.tilt_state = 0;
    core.holding_baby_pygmy = true;

    world.obj_mut::<PygmyApocalypse>(this).baby_pygmy_logic = Some(baby);

    let animation = pg_engine::logic::animation(world, this);
    let baby_display = pg_engine::logic::display_object(world, baby);
    pg_engine::animation::set_tweenable(world, animation, Some(baby_display), TOM_BABY_CHANNEL);

    let display = pg_engine::logic::display_object(world, this);
    let mut object = pg_engine::display::object_mut(world, display);

    let behavior = if 240.0 <= object.pos.x {
        object.scale.x = -1.0;
        "PygmyTomCatchBabyPygmyRight"
    } else {
        object.scale.x = 1.0;
        "PygmyTomCatchBabyPygmyLeft"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// Undoes Tom's baby sacrifice once it has started (`mbStartTomBabySacrifice`): the
/// apocalypse starts again and Xenu spawns all the same.
fn interrupt_tom_baby_sacrifice(world: &mut World, this: Entity) {
    let started = world.obj::<PygmyApocalypse>(this).start_tom_baby_sacrifice;
    if !started {
        return;
    }

    world.obj_mut::<PygmyApocalypse>(this).start_tom_baby_sacrifice = false;
    set_selectable(world, this, true);

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    if let Some(listener) = listener {
        crate::controller::start_apocalypse(world, listener);
    }

    let xenu = listener.and_then(|listener| crate::controller::xenu_logic(world, listener, this));
    if let Some(xenu) = xenu {
        closet_xenu::xenu_logic::spawn(world, xenu);
    }
}

/// `-[PygmyLogic (Apocalypse) ejectFromTomsArms:]` @ 0x10004153c
pub fn eject_from_toms_arms_with_arg(world: &mut World, this: Entity, eject_from_toms_arms: Entity) {
    let ejected_baby = eject_from_toms_arms;

    interrupt_tom_baby_sacrifice(world, this);

    let baby = world.obj::<PygmyApocalypse>(this).baby_pygmy_logic;
    if baby != Some(ejected_baby) {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).holding_baby_pygmy = false;

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, TOM_BABY_CHANNEL);

    pg_engine::logic::set_behavior(world, this, Some("PygmyTomEjectBabyPygmy"));

    world.obj_mut::<PygmyApocalypse>(this).baby_pygmy_logic = None;
}

/// `-[PygmyLogic (Apocalypse) stopHoldingBabyPygmy]` @ 0x100041654
pub fn stop_holding_baby_pygmy(world: &mut World, this: Entity) {
    interrupt_tom_baby_sacrifice(world, this);

    let baby = world.obj::<PygmyApocalypse>(this).baby_pygmy_logic;
    let Some(baby) = baby else {
        return;
    };

    trigger_episode(world, "HaveTomDropBaby");

    world.obj_mut::<PygmyPygmyCoreFrame>(this).holding_baby_pygmy = false;

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, TOM_BABY_CHANNEL);

    eject_from_toms_arms(world, baby);

    world.obj_mut::<PygmyApocalypse>(this).baby_pygmy_logic = None;
}

/// Catches a falling pygmy in one of Xenu's two slots (the tail of `checkForXenuDuringFall`).
fn fall_into_xenu(world: &mut World, this: Entity, xenu: Entity, slot: i32) {
    stop_moving(world, this);
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_floor_collision = false;
    core.tilt_state = 0;

    set_selectable(world, this, false);

    let cut_in_half = world.obj::<PygmyPygmyCoreFrame>(this).cut_in_half_by_charlie;
    let sacrifice_id = if cut_in_half {
        "XenuPuppetShow"
    } else if slot == 1 {
        "EatenByXenu"
    } else {
        "XenuStripped"
    };

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.sacrifice_id = Some(sacrifice_id.to_string());
    core.held_by_xenu = true;

    world.obj_mut::<PygmyApocalypse>(this).xenu_logic = Some(xenu);

    if slot == 1 {
        closet_xenu::xenu_logic::fall_into_xenu_slot1(world, xenu, this);
    } else {
        closet_xenu::xenu_logic::fall_into_xenu_slot2(world, xenu, this);
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoXenu"));
}

/// `-[PygmyLogic (Apocalypse) checkForXenuDuringFall]` @ 0x100041784
pub fn check_for_xenu_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let busy = core.cooked
        || core.killed_by_plague
        || core.invalid_sacrifice
        || core.doodle_jumping
        || core.dropped_by_hurricane
        || core.thrown_by_locusts
        || core.time_temple_pygmy_type != PYGMY_TYPE_NORMAL;
    let listener = core.pygmy_logic_listener;

    if busy {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };

    let xenu_enabled = crate::controller::is_xenu_enabled(world, listener);
    if !xenu_enabled {
        return false;
    }

    // The original reads the position of Xenu's display object unchecked (a nil Xenu crashes).
    let xenu = crate::controller::xenu_logic(world, listener, this);
    let Some(xenu) = xenu else {
        return false;
    };

    let xenu_pos = display_pos(world, xenu);
    let pos = display_pos(world, this);

    let slot1_available = closet_xenu::xenu_logic::is_slot1_available(world, xenu, this);
    if slot1_available {
        let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

        if vel_y < 0.0 {
            let distance_x = (pos.x - (xenu_pos.x + 74.0)).abs();
            let distance_y = (pos.y - (xenu_pos.y + 2.0)).abs();
            let inside = distance_x <= 20.0 && distance_y <= 20.0;

            if inside {
                fall_into_xenu(world, this, xenu, 1);
                return true;
            }
        }
    }

    let slot2_available = closet_xenu::xenu_logic::is_slot2_available(world, xenu, this);
    if !slot2_available {
        return false;
    }

    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;
    if 0.0 <= vel_y {
        return false;
    }

    let distance_x = (pos.x - (xenu_pos.x + -68.0)).abs();
    let distance_y = (pos.y - (xenu_pos.y + 2.0)).abs();
    let inside = distance_x <= 20.0 && distance_y <= 20.0;

    if !inside {
        return false;
    }

    fall_into_xenu(world, this, xenu, 2);

    true
}

/// `-[PygmyLogic (Apocalypse) canSlamClosetDoor]` @ 0x100041b50
pub fn can_slam_closet_door(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);

    !core.slamming_closet_temple_closet_door
        && core.in_closet_temple_closet_door
        && core.persona_personality == PERSONALITY_TOM
        && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL
}

/// `-[PygmyLogic (Apocalypse) trappedInCloset]` @ 0x100041bb4
pub fn trapped_in_closet(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyPygmyCoreFrame>(this).in_closet_temple_closet_door
}

/// `-[PygmyLogic (Apocalypse) slamClosetDoor]` @ 0x100041bc8
pub fn slam_closet_door(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).slamming_closet_temple_closet_door = true;

    pg_engine::logic::set_behavior(world, this, Some("PygmyTomSlamDoor"));
}

/// `-[PygmyLogic (Apocalypse) animSlamClosetDoorComplete]` @ 0x100041bf4
pub fn anim_slam_closet_door_complete(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).slamming_closet_temple_closet_door = false;

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let door = listener.and_then(|listener| crate::controller::closet_temple_closet_door_logic(world, listener, this));

    if let Some(door) = door {
        closet_xenu::closet_temple_closet_door_logic::slam_door(world, door);
    }
}

/// `-[PygmyLogic (Apocalypse) checkForFinishClosetIdle:::]` @ 0x100041c48
pub fn seq_check_for_finish_closet_idle(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let in_closet = world.obj::<PygmyPygmyCoreFrame>(this).in_closet_temple_closet_door;

    if !in_closet {
        return ItemResult::Done;
    }

    pg_engine::logic::set_behavior(world, this, Some("FinishPygmyTomClosetIdle"));

    ItemResult::Replaced
}

/// `-[PygmyLogic (Apocalypse) runToClosetDoor:::]` @ 0x100041c90
pub fn seq_run_to_closet_door(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let door = listener.and_then(|listener| crate::controller::closet_temple_closet_door_logic(world, listener, this));
    let max_closet_x = door.map_or(0.0, |door| closet_xenu::closet_temple_closet_door_logic::max_closet_x_pos(world, door));

    let display = pg_engine::logic::display_object(world, this);
    let mut object = pg_engine::display::object_mut(world, display);
    object.scale.x = 1.0;
    let x = object.pos.x;

    pg_engine::logic::logic_mut(world, this).pos_vel.x = 400.0;

    if x <= max_closet_x {
        return ItemResult::Pending;
    }

    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;

    ItemResult::Done
}

/// `-[PygmyLogic (Apocalypse) checkDiseaseIncubation:::]` @ 0x100041d60
pub fn seq_check_disease_incubation(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut apocalypse = world.obj_mut::<PygmyApocalypse>(this);
    let cycles = apocalypse.disease_incubate_idle_cycles;
    let remaining_cycles = cycles.wrapping_sub(1);
    apocalypse.disease_incubate_idle_cycles = remaining_cycles;

    let incubated = remaining_cycles == 0 || cycles < 1;
    if !incubated {
        return ItemResult::Done;
    }

    let disease = world.obj::<SpriteLogic>(this).apocalypse_disease_type;

    let behavior = if disease == DISEASE_PLAGUE {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).killed_by_plague = true;

        let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
        let pygmies = match listener {
            Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
            None => Vec::new(),
        };

        // In a tribe of six or more, the last pygmy to die of the plague vanishes instead of
        // leaving a body.
        let leaves_body = if pygmies.len() < 6 {
            true
        } else {
            let other_alive = |other: &Entity| *other != this && !is_killed_by_plague(world, *other);
            pygmies.iter().any(other_alive)
        };

        if leaves_body {
            "PygmyKilledByPlague"
        } else {
            world.obj_mut::<PygmyPygmyCoreFrame>(this).killed_by_plague_disappear = true;
            set_selectable(world, this, false);

            "PygmyKilledByPlagueDisappear"
        }
    } else {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).killed_by_purification = true;
        set_selectable(world, this, false);

        world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("KilledByPurification".to_string());
        "PygmyKilledByPurification"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));

    ItemResult::Replaced
}

/// `-[PygmyLogic (Apocalypse) startThrownInAirByLocusts:]` @ 0x10004203c
pub fn start_thrown_in_air_by_locusts(world: &mut World, this: Entity, thrown_in_air_by_locusts: Entity) -> bool {
    let locust = thrown_in_air_by_locusts;

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let sprite = world.obj::<SpriteLogic>(this);

    let busy = core.touch_began_inside
        || core.remove_from_story_mode
        || core.tilt_state != 0
        || core.cut_in_half_by_charlie
        || core.time_temple_pygmy_type != PYGMY_TYPE_NORMAL
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || sprite.blown_by_hurricane
        || sprite.picked_up_by_hurricane
        || core.dropped_by_hurricane
        || core.catch_shark
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_frozen
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.in_kong_statue
        || core.in_ape_wench
        || core.in_ape_bongo
        || core.in_ape_altar
        || core.in_go_go_cage
        || core.in_dance_dance_execution
        || core.resting_in_igloo
        || core.crapping
        || core.trapped_in_spider_web
        || core.trapped_in_zombie_grave
        || core.trapped_in_idol_pedestal
        || core.inside_temple_door
        || core.inside_apocalypse_temple_door
        || core.on_top_apocalypse_time_temple_platform
        || core.eaten_by_locust_queen
        || core.in_apocalypse_temple_fountain
        || core.thetan_brawl
        || core.on_closet_temple_thetan_meter
        || core.on_closet_temple_couch
        || core.in_closet_temple_closet_door
        || core.slamming_closet_temple_closet_door
        || core.held_by_xenu
        || core.held_in_toms_arms
        || core.holding_baby_pygmy
        || core.thrown_by_locusts
        || core.killed_by_locusts
        || core.killed_by_purification
        || core.dance_dance_executed
        || core.battle_of_the_gods
        || core.zombie_explode
        || core.is_zombie
        || core.is_ghost
        || core.touched_by_ghost
        || core.possessed_by_thetan
        || core.caught_by_monkey
        || core.monkey_on_back
        || core.eaten_by_moron_snake
        || core.wearing_water_wings
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.skewered
        || core.underwater
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.dragging
        || core.shark_on_head
        || core.crushed
        || core.caught_by_bird
        || core.falling
        || core.in_crack_hole
        || core.doodle_jumping
        || core.swarmed
        || core.burnt_by_magnifying_glass
        || core.tumble
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.cooked
        || core.killed_by_plague
        || core.drowning;

    if busy {
        return false;
    }

    stop_activities(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.thrown_by_locusts = true;

    world.obj_mut::<PygmyApocalypse>(this).throwing_locust_logic = Some(locust);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.tilt_state = 0;
    core.enable_island_floor_collision = false;
    core.enable_island_side_collision = false;

    stop_moving(world, this);
    clear_z_rot(world, this);
    set_selectable(world, this, false);

    world.obj_mut::<PygmyApocalypse>(this).use_locust_catch1 = true;

    pg_engine::logic::set_behavior(world, this, Some("PygmyThrownByLocustsInitialLift"));

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let display = pg_engine::logic::display_object(world, this);

    let marker = listener.and_then(|listener| crate::controller::pygmy_drag_display_marker(world, listener));
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display));
    }

    update_label_layer(world, this);

    true
}

/// `-[PygmyLogic (Apocalypse) thrownInAirByLocusts:targetPos:targetTime:]` @ 0x100042930
///
/// `target_pos` is the locust's `mThrowTargetPos`, which the original passes by pointer.
pub fn thrown_in_air_by_locusts(world: &mut World, this: Entity, _locust: Entity, target_pos: Vec4, target_time: f32) {
    let thrown = world.obj::<PygmyPygmyCoreFrame>(this).thrown_by_locusts;
    if !thrown {
        return;
    }

    let pos = display_pos(world, this);

    // A ballistic arc that reaches the target after `target_time` under the 1500 gravity.
    let x_vel = (target_pos.x - pos.x) / target_time;
    let y_vel = target_time * 750.0 + (target_pos.y - pos.y) / target_time;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = x_vel;
    logic.pos_vel.y = y_vel;

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let mut apocalypse = world.obj_mut::<PygmyApocalypse>(this);
    let behavior = if apocalypse.use_locust_catch1 {
        apocalypse.use_locust_catch1 = false;
        "PygmyThrownByLocusts1"
    } else {
        apocalypse.use_locust_catch1 = true;
        "PygmyThrownByLocusts2"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Apocalypse) catchInAirByLocusts:]` @ 0x100042a4c
pub fn catch_in_air_by_locusts(world: &mut World, this: Entity, _locust: Entity) {
    let thrown = world.obj::<PygmyPygmyCoreFrame>(this).thrown_by_locusts;
    if !thrown {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    stop_moving(world, this);

    let use_locust_catch1 = world.obj::<PygmyApocalypse>(this).use_locust_catch1;
    let behavior = if use_locust_catch1 {
        "PygmyThrownByLocustsCatch1"
    } else {
        "PygmyThrownByLocustsCatch2"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Apocalypse) droppedInAirByLocusts:]` @ 0x100042ad4
pub fn dropped_in_air_by_locusts(world: &mut World, this: Entity, _locust: Entity) {
    let thrown = world.obj::<PygmyPygmyCoreFrame>(this).thrown_by_locusts;
    if !thrown {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_floor_collision = true;
    core.thrown_by_locusts = false;

    world.obj_mut::<PygmyApocalypse>(this).throwing_locust_logic = None;

    set_selectable(world, this, true);
    move_after_pygmy_display_marker(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.invalid_sacrifice = true;
    core.sacrifice_id = Some("DroppedDrown".to_string());

    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Apocalypse) locustThrowWaitForApex:::]` @ 0x100042c10
pub fn seq_locust_throw_wait_for_apex(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;

    if vel_y < 0.0 { ItemResult::Done } else { ItemResult::Pending }
}

/// `-[PygmyLogic (Apocalypse) checkForLocusts:::]` @ 0x100042c34
pub fn seq_check_for_locusts(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let locust = listener.and_then(|listener| crate::controller::locust_logic(world, listener, this));
    let panic = locust.is_some_and(|locust| closet_xenu::locust_logic::should_pygmy_panic(world, locust));

    if panic {
        // The stop list S13 without `stopThrowingSpear`.
        stop_being_high_on_nitrous(world, this);
        stop_eating(world, this);
        stop_carrying(world, this);
        stop_carrying_plague_body(world, this);
        stop_fishing(world, this);
        stop_sleeping(world, this);
        stop_warming_by_bon_fire(world, this);
        stop_staring_at_rainbow(world, this);
        stop_staring_at_kidnapping(world, this);
        stop_waiting_to_crap(world, this);
        close_word_balloon(world, this);
        stop_charlie_mischief(world, this);

        pg_engine::logic::set_behavior(world, this, Some("PygmyLocustPanicIdle"));
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Apocalypse) checkForLocustsComplete:::]` @ 0x100042d98
pub fn seq_check_for_locusts_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let locust = listener.and_then(|listener| crate::controller::locust_logic(world, listener, this));
    let panic = locust.is_some_and(|locust| closet_xenu::locust_logic::should_pygmy_panic(world, locust));

    if !panic {
        pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Apocalypse) mutatedByLocustsComplete:::]` @ 0x100042e0c
pub fn seq_mutated_by_locusts_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    set_selectable(world, this, true);

    ItemResult::Done
}
