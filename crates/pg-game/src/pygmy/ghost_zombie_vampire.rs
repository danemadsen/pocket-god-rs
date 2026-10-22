//! `PygmyLogic`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use crate::logic::island_core::sprite_logic::SpriteLogic;
use pg_engine::display;
use pg_engine::touch::BcTouch;

/// `PygmyLogic` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct PygmyGhostZombieVampire {
    /// `mbGhostFollowingPath`: `B`
    pub ghost_following_path: bool,

    /// `mbGhostFlipping`: `B`
    pub ghost_flipping: bool,

    /// `mGhostTouchXOffset`: `f`
    pub ghost_touch_x_offset: f32,

    /// `mGhostTouchYOffset`: `f`
    pub ghost_touch_y_offset: f32,

    /// `mGhostTargetZRot`: `f`
    pub ghost_target_z_rot: f32,

    /// `mGhostTargetYPos`: `f`
    pub ghost_target_y_pos: f32,

    /// `mpGhostPathVector`: `^{vector<VECTOR4, std::__1::allocator<VECTOR4> >=^{?}^{?}{__compressed_pair<VECTOR4 *, std::__1::allocator<VECTOR4> >=^{?}}}`
    pub ghost_path_vector: Vec<Vec4>,

    /// `mGhostLastPathPos`: `{?="x"f"y"f"z"f"w"f}`
    pub ghost_last_path_pos: Vec4,

    /// `mGhostLastTouchPos`: `{?="x"f"y"f"z"f"w"f}`
    pub ghost_last_touch_pos: Vec4,

    /// `mGhostTouchPygmyLogic`: `@"PygmyLogic"`
    pub ghost_touch_pygmy_logic: Option<Entity>,

    /// `mGhostTouchType`: `i`
    pub ghost_touch_type: i32,

    /// `mGhostEmergeXPos`: `f`
    pub ghost_emerge_x_pos: f32,

    /// `mbGhostBansheeScream`: `B`
    pub ghost_banshee_scream: bool,

    /// `mbGhostBansheeScreamStarted`: `B`
    pub ghost_banshee_scream_started: bool,

    /// `mTheLightDisplayObject`: `@"BCDisplayObject"`
    pub the_light_display_object: Option<Entity>,

    /// `mTheLightLogic`: `@"MultiModelLogic"`
    pub the_light_logic: Option<Entity>,

    /// `mbGoIntoTheLight`: `B`
    pub go_into_the_light: bool,

    /// `mbGhostActionTriggered`: `B`
    pub ghost_action_triggered: bool,

    /// `mGhostPygmyLogic`: `@"PygmyLogic"`
    pub ghost_pygmy_logic: Option<Entity>,

    /// `mEatingBrainsPygmyLogic`: `@"PygmyLogic"`
    pub eating_brains_pygmy_logic: Option<Entity>,

    /// `mbZombieChooseDirection`: `B`
    pub zombie_choose_direction: bool,

    /// `mbBitByBat`: `B`
    pub bit_by_bat: bool,
}

/// Tweenable channel 1: the ghost on its victim's animation, the carried victim on the ghost's.
const GHOST_CHANNEL: usize = 1;

/// How far `ghostFollowPath:::` moves the ghost along its path per call (frame-rate dependent).
const GHOST_PATH_STEP: f32 = 16.666666;

/// The stall time of the vampire items that wait without a time limit.
const ENDLESS_WAIT: f32 = -1000.0;

/// `mGhostTouchType` values: how the drawn path entered the victim's box.
const GHOST_TOUCH_PULL_UNDER: i32 = 0;
const GHOST_TOUCH_CARRY_AWAY: i32 = 1;
const GHOST_TOUCH_POSSESS: i32 = 2;
const GHOST_TOUCH_SCARE: i32 = 3;

/// `mTimeTemplePygmyType` of a ginger pygmy.
const GINGER_PYGMY: i32 = 3;

/// `mPygmyLogicListener`: always the view controller (docs/re/pygmy.md §23).
fn listener_of(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener
}

/// `[mPygmyLogicListener pygmyLogicArray:self]` (a nil listener answers no pygmies).
fn pygmy_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let listener = listener_of(world, this);
    match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    }
}

/// `[[mPygmyLogicListener pygmyDisplayMarker] insertBefore:mDisplayObject]`: back into the
/// pygmies' layer, just below the marker.
fn insert_before_pygmy_display_marker(world: &mut World, this: Entity) {
    let listener = listener_of(world, this);
    let marker = match listener {
        Some(listener) => crate::controller::pygmy_display_marker(world, listener),
        None => None,
    };

    if let Some(marker) = marker {
        let display_object = pg_engine::logic::display_object(world, this);
        display::insert_before(world, marker, Some(display_object));
    }
}

/// `[[[pygmy displayObject] displayGroup] viewToDisplayObject:point]`; outside any group the
/// message goes to nil and answers all zero.
fn view_to_display_group(world: &World, display_object: Entity, point: Vec4) -> Vec4 {
    let group = display::object(world, display_object).display_group;

    match group {
        Some(group) => display::view_to_display_object(world, group, point),
        None => Vec4::ZERO,
    }
}

/// `mPosVel.x = mPosVel.y = 0` (z and w keep their values).
fn stop_moving(world: &mut World, this: Entity) {
    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 0.0;
    logic.pos_vel.y = 0.0;
}

/// `[mDisplayObject setZRot:0]`.
fn reset_z_rot(world: &mut World, this: Entity) {
    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
}

/// `[mDisplayObject setXScale:x_scale]`.
fn set_x_scale(world: &mut World, this: Entity, x_scale: f32) {
    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).scale.x = x_scale;
}

/// `[mDisplayObject setIsSelectable:selectable]`.
fn set_selectable(world: &mut World, this: Entity, selectable: bool) {
    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = selectable;
}

/// `mSacrificeId = sacrifice_id`: what the coming death will be recorded as.
fn set_sacrifice_id(world: &mut World, this: Entity, sacrifice_id: &str) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some(sacrifice_id.to_string());
}

/// `[[pygmy displayObject] xPos]` (0 for a nil pygmy).
fn x_pos_of(world: &World, pygmy: Option<Entity>) -> f32 {
    match pygmy {
        Some(pygmy) => {
            let display_object = pg_engine::logic::display_object(world, pygmy);
            display::object(world, display_object).pos.x
        }
        None => 0.0,
    }
}

/// `[mAnimation setTweenable:[pygmy displayObject] channelId:1]`.
fn set_ghost_channel(world: &mut World, this: Entity, pygmy: Option<Entity>) {
    let tweenable = pygmy.map(|pygmy| pg_engine::logic::display_object(world, pygmy));
    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::set_tweenable(world, animation, tweenable, GHOST_CHANNEL);
}

/// `[mAnimation clearTweenableChannelId:1]`.
fn clear_ghost_channel(world: &mut World, this: Entity) {
    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, GHOST_CHANNEL);
}

/// The ghost's victims turn toward it: `xScale = 1`, or -1 when the ghost is to the left.
fn face_ghost(world: &mut World, this: Entity, ghost: Entity) {
    let x_pos = x_pos_of(world, Some(this));
    let ghost_x_pos = x_pos_of(world, Some(ghost));

    let x_scale = if ghost_x_pos < x_pos { -1.0 } else { 1.0 };
    set_x_scale(world, this, x_scale);
}

/// The random duration `[minTime, maxTime]` of the timed items (one `arc4random()`, in
/// hundredths of a second).
fn random_item_time(world: &mut World, call: &SequenceCall) -> f32 {
    let min_time = call.float("minTime");
    let max_time = call.float("maxTime");
    let random = world.resource_mut::<Random>().arc4random();
    pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random)
}

/// The busy set FREE of `docs/re/pygmy.md` §1.3 without its pygmy-type term: true when the
/// pygmy is in none of the other 82 states. Pure reads, so the order of the original's checks
/// does not matter.
fn is_free_of_any_type(world: &World, this: Entity) -> bool {
    let sprite = world.obj::<SpriteLogic>(this);
    if sprite.blown_by_hurricane || sprite.picked_up_by_hurricane {
        return false;
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let busy = core.touch_began_inside
        || core.remove_from_story_mode
        || core.cut_in_half_by_charlie
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

    !busy
}

/// The whole busy set FREE (`docs/re/pygmy.md` §1.3): a normal pygmy in none of the 82 states.
fn is_free(world: &World, this: Entity) -> bool {
    let time_temple_pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    time_temple_pygmy_type == 0 && is_free_of_any_type(world, this)
}

/// FREE +{mbSharkOnHead} with `mTiltState == 0`: the guard of the ghost touch and the zombie
/// bite.
fn is_free_upright_without_shark_on_head(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let tilt_state = core.tilt_state;
    let shark_on_head = core.shark_on_head;

    tilt_state == 0 && !shark_on_head && is_free(world, this)
}

/// The stop list S13 (`docs/re/pygmy.md` §1.3), in the original's order.
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

/// `[listener isNightActive] || [listener isEclipseActive]`: the dark the vampires hunt in.
fn is_vampire_night(world: &mut World, listener: Option<Entity>) -> bool {
    let Some(listener) = listener else {
        return false;
    };

    crate::controller::is_night_active(world, listener) || crate::controller::is_eclipse_active(world, listener)
}

/// `[listener isSunsetActive]` (NO for a nil listener).
fn is_sunset_active(world: &mut World, listener: Option<Entity>) -> bool {
    match listener {
        Some(listener) => crate::controller::is_sunset_active(world, listener),
        None => false,
    }
}

/// The daylight rule of the vampire items: at sunset `PygmyVampireSunset`, otherwise the
/// vampire burns (`mSacrificeId = VampireInSunlight`, `PygmyVampireSun`).
fn vampire_daylight_behavior(world: &mut World, this: Entity, listener: Option<Entity>) -> &'static str {
    if is_sunset_active(world, listener) {
        return "PygmyVampireSunset";
    }

    set_sacrifice_id(world, this, "VampireInSunlight");
    "PygmyVampireSun"
}

/// Whether any pygmy of the island `canBeHitByVampire` (the first one ends the search).
fn any_pygmy_can_be_hit_by_vampire(world: &mut World, this: Entity) -> bool {
    let pygmies = pygmy_logic_array(world, this);
    pygmies.into_iter().any(|pygmy| can_be_hit_by_vampire(world, pygmy))
}

/// `-[PygmyLogic animChooseDirection]` @ 0x10012e920
pub fn anim_choose_direction(world: &mut World, this: Entity) {
    let zombie_choose_direction = world.obj::<PygmyGhostZombieVampire>(this).zombie_choose_direction;
    if !zombie_choose_direction {
        return;
    }

    let x_pos = x_pos_of(world, Some(this));
    let coin = world.resource_mut::<Random>().arc4random();

    let listener = listener_of(world, this);
    let (island_min_x_pos, island_max_x_pos) = match listener {
        Some(listener) => {
            let island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
            let island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
            (island_min_x_pos, island_max_x_pos)
        }
        None => (0.0, 0.0),
    };

    let goes_left = x_pos == island_max_x_pos || (coin & 1 == 0 && x_pos != island_min_x_pos);
    let x_scale = if goes_left { -1.0 } else { 1.0 };
    set_x_scale(world, this, x_scale);
}

/// `-[PygmyLogic animFlipComplete]` @ 0x10012ea0c
pub fn anim_flip_complete(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_flipping = false;
}

/// `-[PygmyLogic cancelFlip]` @ 0x10012ea1c
pub fn cancel_flip(world: &mut World, this: Entity) {
    let ghost_flipping = world.obj::<PygmyGhostZombieVampire>(this).ghost_flipping;
    if !ghost_flipping {
        return;
    }

    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_flipping = false;
    anim_ghost_x_flip(world, this);
}

/// `-[PygmyLogic (Island) canBeHitByVampire]` @ 0x1000a1094
pub fn can_be_hit_by_vampire(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let normal_charlie = core.persona_personality == 1 && core.time_temple_pygmy_type == 0;
    let tilt_state = core.tilt_state;
    let bit_by_bat = world.obj::<PygmyGhostZombieVampire>(this).bit_by_bat;

    !normal_charlie && tilt_state == 0 && !bit_by_bat && is_free_of_any_type(world, this)
}

/// `-[PygmyLogic (Island) isVampire]` @ 0x1000a1760
pub fn is_vampire(world: &mut World, this: Entity) -> bool {
    let bit_by_bat = world.obj::<PygmyGhostZombieVampire>(this).bit_by_bat;
    let time_temple_pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    bit_by_bat && time_temple_pygmy_type == 0
}

/// `-[PygmyLogic (Island) hitByVampire:xVel:]` @ 0x1000a1798
pub fn hit_by_vampire(world: &mut World, this: Entity, _vampire: Entity, x_vel: f32) {
    // S14 with `stopBeingEaten` after `stopThrowingSpear`.
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
    stop_being_eaten(world, this);
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.falling = true;
    core.tilt_state = 0;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = x_vel;
    logic.pos_vel.y = 500.0;

    reset_z_rot(world, this);
    set_sacrifice_id(world, this, "Vampire");

    let time_temple_pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    let behavior = if time_temple_pygmy_type == 0 {
        "PygmyHitByVampire"
    } else {
        "TemplePygmyHitByVampire"
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Island) canBeBitByBat]` @ 0x1000a1988
pub fn can_be_bit_by_bat(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyPygmyCoreFrame>(this).sleeping
}

/// `-[PygmyLogic (Island) bitByBat:]` @ 0x1000a199c
pub fn bit_by_bat(world: &mut World, this: Entity, _bat: Entity) {
    stop_activities(world, this);

    world.obj_mut::<PygmyGhostZombieVampire>(this).bit_by_bat = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    stop_moving(world, this);
    reset_z_rot(world, this);

    pg_engine::logic::set_behavior(world, this, Some("PygmyBitByBat"));
}

/// `-[PygmyLogic (Island) canBeTouchedByGhost:]` @ 0x1000b1cac
pub fn can_be_touched_by_ghost(world: &mut World, this: Entity, _ghost: Entity) -> bool {
    is_free_upright_without_shark_on_head(world, this)
}

/// `-[PygmyLogic (Island) touchedByGhost:]` @ 0x1000b2360
pub fn touched_by_ghost(world: &mut World, this: Entity, ghost: Entity) {
    stop_activities_and_being_eaten(world, this);

    stop_moving(world, this);
    reset_z_rot(world, this);
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    set_selectable(world, this, false);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).touched_by_ghost = true;
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic = Some(ghost);

    // Original bug, kept: `stopSleeping` above has already cleared `mbSleeping`, so the
    // sleeping variant never plays.
    let sleeping = world.obj::<PygmyPygmyCoreFrame>(this).sleeping;
    let behavior = if sleeping {
        "PygmyWaitForGhostTouchSleep"
    } else {
        "PygmyWaitForGhostTouch"
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Island) possessedByGhost:]` @ 0x1000b2568
pub fn possessed_by_ghost(world: &mut World, this: Entity, ghost: Entity) {
    face_ghost(world, this, ghost);

    let ghost_pygmy_logic = world.obj::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic;
    set_ghost_channel(world, this, ghost_pygmy_logic);

    pg_engine::logic::set_behavior(world, this, Some("PygmyPossessedByGhost"));
}

/// `-[PygmyLogic (Island) possessedByGhostComplete:]` @ 0x1000b2674
pub fn possessed_by_ghost_complete(world: &mut World, this: Entity, _victim: Entity) {
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_touch_pygmy_logic = None;
    set_selectable(world, this, true);
    pg_engine::logic::set_behavior(world, this, Some("PygmyGhostIdle"));
}

/// `-[PygmyLogic (Island) cancelGhostTouch:]` @ 0x1000b26dc
pub fn cancel_ghost_touch(world: &mut World, this: Entity, ghost: Entity) {
    let ghost_pygmy_logic = world.obj::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic;
    if ghost_pygmy_logic != Some(ghost) {
        return;
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).touched_by_ghost = false;
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic = None;
    set_selectable(world, this, true);

    pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
}

/// `-[PygmyLogic (Island) cancelGhostHold:]` @ 0x1000b277c
pub fn cancel_ghost_hold(world: &mut World, this: Entity, _victim: Entity) {
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_touch_pygmy_logic = None;
    set_selectable(world, this, true);

    let remove_from_story_mode = world.obj::<PygmyPygmyCoreFrame>(this).remove_from_story_mode;
    if remove_from_story_mode {
        return;
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyGhostIdle"));
}

/// `-[PygmyLogic (Island) cancelGhostTouchDead:]` @ 0x1000b2804
pub fn cancel_ghost_touch_dead(world: &mut World, this: Entity, _ghost: Entity) {
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic = None;
}

/// `-[PygmyLogic (Island) scaredByGhost:]` @ 0x1000b2818
pub fn scared_by_ghost(world: &mut World, this: Entity, ghost: Entity) {
    set_sacrifice_id(world, this, "ScaredByGhost");
    face_ghost(world, this, ghost);

    let ghost_pygmy_logic = world.obj::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic;
    set_ghost_channel(world, this, ghost_pygmy_logic);

    pg_engine::logic::set_behavior(world, this, Some("PygmyScaredByGhost"));
}

/// `-[PygmyLogic (Island) scaredByGhostComplete:]` @ 0x1000b293c
pub fn scared_by_ghost_complete(world: &mut World, this: Entity, _victim: Entity) {
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_touch_pygmy_logic = None;
    set_selectable(world, this, true);
    pg_engine::logic::set_behavior(world, this, Some("PygmyGhostIdle"));
}

/// `-[PygmyLogic (Island) animPygmyScaredComplete]` @ 0x1000b29a4
pub fn anim_pygmy_scared_complete(world: &mut World, this: Entity) {
    stop_moving(world, this);
    reset_z_rot(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).touched_by_ghost = false;

    let ghost = world.obj::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic;
    if let Some(ghost) = ghost {
        scared_by_ghost_complete(world, ghost, this);
    }
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic = None;

    clear_ghost_channel(world, this);
}

/// `-[PygmyLogic (Island) pulledUnderByGhost:]` @ 0x1000b2a50
pub fn pulled_under_by_ghost(world: &mut World, this: Entity, ghost: Entity) {
    set_sacrifice_id(world, this, "PulledIntoGround");
    face_ghost(world, this, ghost);

    let ghost_pygmy_logic = world.obj::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic;
    set_ghost_channel(world, this, ghost_pygmy_logic);

    pg_engine::logic::set_behavior(world, this, Some("PygmyPulledUnderByGhost"));
}

/// `-[PygmyLogic (Island) pulledUnderByGhostComplete:]` @ 0x1000b2b74
pub fn pulled_under_by_ghost_complete(world: &mut World, this: Entity, _victim: Entity) {
    let ghost_emerge_x_pos = world.obj::<PygmyGhostZombieVampire>(this).ghost_emerge_x_pos;

    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.pos.x = ghost_emerge_x_pos;
    object.pos.y = 96.0;

    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_touch_pygmy_logic = None;
    pg_engine::logic::set_behavior(world, this, Some("PygmyGhostPullPygmyUnderComplete"));
}

/// `-[PygmyLogic (Island) carriedAwayByGhost:]` @ 0x1000b2bf4
pub fn carried_away_by_ghost(world: &mut World, this: Entity, _ghost: Entity) {
    set_sacrifice_id(world, this, "CarriedAwayGhost");

    let x_pos = x_pos_of(world, Some(this));
    let x_scale = if x_pos > 240.0 { -1.0 } else { 1.0 };
    set_x_scale(world, this, x_scale);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).carried_away_by_ghost = true;

    let ghost_pygmy_logic = world.obj::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic;
    set_ghost_channel(world, this, ghost_pygmy_logic);

    pg_engine::logic::set_behavior(world, this, Some("PygmyCarriedAwayByGhost"));
}

/// `-[PygmyLogic (Island) initCarriedAwayByGhostComplete:]` @ 0x1000b2d04
pub fn init_carried_away_by_ghost_complete(world: &mut World, this: Entity, _victim: Entity) {
    let display_object = pg_engine::logic::display_object(world, this);
    let x_scale = display::object(world, display_object).scale.x;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = x_scale * 100.0;
    logic.pos_vel.y = 100.0;

    let victim = world.obj::<PygmyGhostZombieVampire>(this).ghost_touch_pygmy_logic;
    set_ghost_channel(world, this, victim);

    pg_engine::logic::set_behavior(world, this, Some("PygmyGhostCarryAwayPygmy"));
}

/// `-[PygmyLogic (Island) carriedAwayByGhostComplete:]` @ 0x1000b2dd0
pub fn carried_away_by_ghost_complete(world: &mut World, this: Entity, _ghost: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Island) stopBansheeScream]` @ 0x1000b2de8
pub fn stop_banshee_scream(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_banshee_scream = false;

    let started = world.obj::<PygmyGhostZombieVampire>(this).ghost_banshee_scream_started;
    if !started {
        pg_engine::logic::set_behavior(world, this, Some("PygmyGhostWaitForPath"));
        return;
    }

    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_banshee_scream_started = false;

    let pygmies = pygmy_logic_array(world, this);
    for pygmy in pygmies {
        stop_hold_ears_from_banshee_scream(world, pygmy, this);
    }

    pg_engine::sound::stop_sound(world, "BansheeCryLoop");
    pg_engine::logic::set_behavior(world, this, Some("PygmyGhostStopScream"));
}

/// `-[PygmyLogic (Island) holdEarsFromBansheeScream:]` @ 0x1000b2fe4
pub fn hold_ears_from_banshee_scream(world: &mut World, this: Entity, _ghost: Entity) {
    if !is_free(world, this) {
        return;
    }

    stop_activities(world, this);

    stop_moving(world, this);
    reset_z_rot(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    set_selectable(world, this, false);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).hold_ears_from_banshee_scream = true;

    pg_engine::logic::set_behavior(world, this, Some("PygmyHoldEarsFromBansheeScream"));
}

/// `-[PygmyLogic (Island) explodeFromBansheeScream:]` @ 0x1000b3814
pub fn explode_from_banshee_scream(world: &mut World, this: Entity, _ghost: Entity) {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    if !core.hold_ears_from_banshee_scream || core.is_ghost {
        return;
    }

    set_sacrifice_id(world, this, "FromBansheeScream");
    world.obj_mut::<PygmyPygmyCoreFrame>(this).explode_from_banshee_scream = true;
    pg_engine::logic::set_behavior(world, this, Some("PygmyExplodeFromBansheeScream"));
}

/// `-[PygmyLogic (Island) stopHoldEarsFromBansheeScream:]` @ 0x1000b3884
pub fn stop_hold_ears_from_banshee_scream(world: &mut World, this: Entity, _ghost: Entity) {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    if !core.hold_ears_from_banshee_scream || core.explode_from_banshee_scream {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).hold_ears_from_banshee_scream = false;
    set_selectable(world, this, true);
    pg_engine::logic::set_behavior(world, this, Some("PygmyStopHoldEarsFromBansheeScream"));
}

/// `-[PygmyLogic (Island) canHaveBrainsEaten:]` @ 0x1000b3914
pub fn can_have_brains_eaten(world: &mut World, this: Entity, _zombie: Entity) -> bool {
    is_free_upright_without_shark_on_head(world, this)
}

/// `-[PygmyLogic (Island) animSleepBrainsEatenXFlip]` @ 0x1000b3fc8
pub fn anim_sleep_brains_eaten_x_flip(world: &mut World, this: Entity) {
    let zombie = world.obj::<PygmyGhostZombieVampire>(this).eating_brains_pygmy_logic;
    let zombie_x_pos = x_pos_of(world, zombie);
    let x_pos = x_pos_of(world, Some(this));

    let x_scale = if zombie_x_pos > x_pos { -1.0 } else { 1.0 };
    set_x_scale(world, this, x_scale);

    world.obj_mut::<PygmyGhostZombieVampire>(this).eating_brains_pygmy_logic = None;
}

/// `-[PygmyLogic (Island) haveBrainsEaten:]` @ 0x1000b4080
pub fn have_brains_eaten(world: &mut World, this: Entity, zombie: Entity) {
    let sleeping = world.obj::<PygmyPygmyCoreFrame>(this).sleeping;

    stop_activities(world, this);

    world.obj_mut::<PygmyGhostZombieVampire>(this).bit_by_bat = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).is_zombie = true;
    world.obj_mut::<PygmyPygmyBehaviourItems>(this).brains_eaten = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    stop_moving(world, this);
    reset_z_rot(world, this);
    insert_before_pygmy_display_marker(world, this);

    if sleeping {
        world.obj_mut::<PygmyGhostZombieVampire>(this).eating_brains_pygmy_logic = Some(zombie);
        pg_engine::logic::set_behavior(world, this, Some("PygmyZombieHaveBrainsEatenSleep"));
        return;
    }

    let zombie_x_pos = x_pos_of(world, Some(zombie));
    let x_pos = x_pos_of(world, Some(this));
    let x_scale = if zombie_x_pos > x_pos { -1.0 } else { 1.0 };
    set_x_scale(world, this, x_scale);

    pg_engine::logic::set_behavior(world, this, Some("PygmyZombieHaveBrainsEaten"));
}

/// `-[PygmyLogic (Island) isZombie]` @ 0x1000b431c
pub fn is_zombie(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyPygmyCoreFrame>(this).is_zombie
}

/// `-[PygmyLogic (Island) zombieExplode]` @ 0x1000b664c
pub fn zombie_explode(world: &mut World, this: Entity) {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    if !core.is_zombie || core.zombie_explode {
        return;
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).zombie_explode = true;
    pg_engine::logic::set_behavior(world, this, Some("PygmyZombieExplode"));
}

/// `-[PygmyLogic (Island) startZombieIdle]` @ 0x1000b66a8
pub fn start_zombie_idle(world: &mut World, this: Entity) {
    let trapped_in_zombie_grave = world.obj::<PygmyPygmyCoreFrame>(this).trapped_in_zombie_grave;
    if !trapped_in_zombie_grave {
        return;
    }

    world.obj_mut::<PygmyContainersAndCatchers>(this).zombie_grave_logic = None;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).trapped_in_zombie_grave = false;
    world.obj_mut::<PygmyGhostZombieVampire>(this).bit_by_bat = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).is_zombie = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_side_collision = true;
    core.enable_island_floor_collision = true;
    let island_y_pos = core.island_y_pos;

    stop_moving(world, this);
    reset_z_rot(world, this);
    set_x_scale(world, this, 1.0);

    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).pos.y = island_y_pos;
    set_selectable(world, this, true);

    insert_before_pygmy_display_marker(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyZombieIdle"));
}

/// `-[PygmyLogic (Island) checkForFreshBrains]` @ 0x1000b6824
pub fn check_for_fresh_brains(world: &mut World, this: Entity) -> bool {
    let display_object = pg_engine::logic::display_object(world, this);
    let pygmies = pygmy_logic_array(world, this);

    for pygmy in pygmies {
        if pygmy == this || !can_have_brains_eaten(world, pygmy, this) {
            continue;
        }

        let object = display::object(world, display_object);
        let mouth_x_pos = object.pos.x + object.scale.x * 18.0;
        let x_pos = object.pos.x;
        let pygmy_x_pos = x_pos_of(world, Some(pygmy));

        let distance = (mouth_x_pos - pygmy_x_pos).abs();
        if distance < 5.0 {
            let x_scale = if pygmy_x_pos < x_pos { -1.0 } else { 1.0 };
            set_x_scale(world, this, x_scale);

            pg_engine::logic::set_behavior(world, this, Some("PygmyZombieEatBrains"));
            have_brains_eaten(world, pygmy, this);
            return true;
        }
    }

    false
}

/// `-[PygmyLogic (Island) possessedByGhostComplete:::]` @ 0x1000b6fac
pub fn seq_possessed_by_ghost_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    stop_moving(world, this);
    reset_z_rot(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    set_selectable(world, this, true);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).touched_by_ghost = false;

    let ghost = world.obj::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic;
    if let Some(ghost) = ghost {
        possessed_by_ghost_complete(world, ghost, this);
    }
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic = None;

    clear_ghost_channel(world, this);
    ItemResult::Done
}

/// `-[PygmyLogic (Island) penetrateGroundComplete:::]` @ 0x1000b709c
pub fn seq_penetrate_ground_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let victim = world.obj::<PygmyGhostZombieVampire>(this).ghost_touch_pygmy_logic;
    if let Some(victim) = victim {
        pulled_under_by_ghost(world, victim, this);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) pulledUnderComplete:::]` @ 0x1000b70d8
pub fn seq_pulled_under_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    stop_moving(world, this);
    reset_z_rot(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    set_selectable(world, this, true);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).touched_by_ghost = false;

    let ghost = world.obj::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic;
    if let Some(ghost) = ghost {
        pulled_under_by_ghost_complete(world, ghost, this);
    }
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic = None;

    clear_ghost_channel(world, this);
    ItemResult::Done
}

/// `-[PygmyLogic (Island) initCarriedAwayByGhostComplete:::]` @ 0x1000b71c8
pub fn seq_init_carried_away_by_ghost_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    stop_moving(world, this);
    reset_z_rot(world, this);
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = false;

    let ghost = world.obj::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic;
    if let Some(ghost) = ghost {
        init_carried_away_by_ghost_complete(world, ghost, this);
    }

    clear_ghost_channel(world, this);
    ItemResult::Done
}

/// `-[PygmyLogic (Island) carryPygmyOffScreen:::]` @ 0x1000b728c
pub fn seq_carry_pygmy_off_screen(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let x_pos = x_pos_of(world, Some(this));
    let off_screen = x_pos < -50.0 || x_pos > 530.0;
    if !off_screen {
        return ItemResult::Pending;
    }

    stop_moving(world, this);

    let victim = world.obj::<PygmyGhostZombieVampire>(this).ghost_touch_pygmy_logic;
    if let Some(victim) = victim {
        carried_away_by_ghost_complete(world, victim, this);
    }

    clear_ghost_channel(world, this);
    ItemResult::Done
}

/// `-[PygmyLogic (Island) startBansheeScream:::]` @ 0x1000b7354
pub fn seq_start_banshee_scream(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_banshee_scream_started = true;

    let pygmies = pygmy_logic_array(world, this);
    for pygmy in pygmies {
        hold_ears_from_banshee_scream(world, pygmy, this);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) burstPygmyEarDrums:::]` @ 0x1000b74a0
pub fn seq_burst_pygmy_ear_drums(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let pygmies = pygmy_logic_array(world, this);
    for pygmy in pygmies {
        explode_from_banshee_scream(world, pygmy, this);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) explodeFromBansheeScream:::]` @ 0x1000b75d8
pub fn seq_explode_from_banshee_scream(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    stop_moving(world, this);
    reset_z_rot(world, this);
    set_sacrifice_id(world, this, "FromBansheeScream");

    seq_explode(world, this, call)
}

/// `-[PygmyLogic (Island) ghostFlyUp:::]` @ 0x1000b7674
pub fn seq_ghost_fly_up(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        set_selectable(world, this, true);

        let random = world.resource_mut::<Random>().arc4random();
        let hundredths = (random % 7001 + 13000) as i32;
        world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_target_y_pos = hundredths as f32 / 100.0;

        pg_engine::logic::logic_mut(world, this).pos_vel.y = 100.0;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    let ghost_target_y_pos = world.obj::<PygmyGhostZombieVampire>(this).ghost_target_y_pos;
    if y_pos >= ghost_target_y_pos {
        stop_moving(world, this);
        return ItemResult::Done;
    }

    ItemResult::Pending
}

/// `-[PygmyLogic (Island) ghostFlyIdle:::]` @ 0x1000b7778
pub fn seq_ghost_fly_idle(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);

    if call.first_time {
        let stall_time = random_item_time(world, call);
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;

        let x_coin = world.resource_mut::<Random>().arc4random();
        let x_pos = display::object(world, display_object).pos.x;
        let goes_left = (x_pos > 20.0 && x_coin & 1 == 0) || x_pos >= 440.0;
        let (x_scale, x_vel) = if goes_left { (-1.0, -50.0) } else { (1.0, 50.0) };
        set_x_scale(world, this, x_scale);
        pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;

        let y_coin = world.resource_mut::<Random>().arc4random();
        let y_pos = display::object(world, display_object).pos.y;
        let goes_down = (y_pos > 40.0 && y_coin & 1 == 0) || y_pos >= 280.0;
        let y_vel = if goes_down { -20.0 } else { 20.0 };
        pg_engine::logic::logic_mut(world, this).pos_vel.y = y_vel;
    }

    let pos = display::object(world, display_object).pos;
    let logic = pg_engine::logic::logic(world, this);
    let pos_vel = logic.pos_vel;
    let stall_time = logic.stall_time;

    let leaves_bottom = pos.y <= 40.0 && pos_vel.y < 0.0;
    let leaves_top = pos.y >= 280.0 && pos_vel.y > 0.0;
    let leaves_left = pos.x <= 20.0 && pos_vel.x < 0.0;
    let leaves_right = pos.x >= 440.0 && pos_vel.x > 0.0;

    let mut result = ItemResult::Pending;
    if leaves_bottom || leaves_top || leaves_left || leaves_right || stall_time <= 0.0 {
        stop_moving(world, this);
        result = ItemResult::Done;
    }

    pg_engine::logic::logic_mut(world, this).stall_time -= call.delta_time;
    result
}

/// `-[PygmyLogic (Island) animGhostXFlip]` @ 0x1000b7a5c
pub fn anim_ghost_x_flip(world: &mut World, this: Entity) {
    let ghost_target_z_rot = world.obj::<PygmyGhostZombieVampire>(this).ghost_target_z_rot;
    let x_scale = ghost_x_scale_for_heading(ghost_target_z_rot);
    set_x_scale(world, this, x_scale);
}

/// The facing of a ghost heading `z_rot` degrees: right (1) within [-90, 90] or from 270 on,
/// left (-1) otherwise.
fn ghost_x_scale_for_heading(z_rot: f32) -> f32 {
    let faces_left = (z_rot > 90.0 && z_rot < 270.0) || z_rot < -90.0;
    if faces_left { -1.0 } else { 1.0 }
}

/// `-[PygmyLogic (Island) ghostFollowPath:::]` @ 0x1000b7ad4
pub fn seq_ghost_follow_path(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    let mut step = GHOST_PATH_STEP;

    loop {
        let path = &world.obj::<PygmyGhostZombieVampire>(this).ghost_path_vector;
        if path.len() < 2 {
            break;
        }

        let next_point = path[1];
        let pos = display::object(world, display_object).pos;
        let gap_x = pos.x - next_point.x;
        let gap_y = pos.y - next_point.y;
        let distance = (gap_x * gap_x + gap_y * gap_y).sqrt();

        if distance > step {
            let delta_x = next_point.x - pos.x;
            let delta_y = next_point.y - pos.y;

            let mut object = display::object_mut(world, display_object);
            object.pos.x = pos.x + step * delta_x / distance;
            object.pos.y = step * delta_y / distance + pos.y;

            return follow_ghost_path_heading(world, this, display_object, delta_x, delta_y);
        }

        let mut object = display::object_mut(world, display_object);
        object.pos.x = next_point.x;
        object.pos.y = next_point.y;

        world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_path_vector.remove(0);
        step -= distance;
    }

    stop_moving(world, this);
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_target_z_rot = 0.0;
    cancel_flip(world, this);

    let ghost = world.obj::<PygmyGhostZombieVampire>(this);
    let go_into_the_light = ghost.go_into_the_light;
    let ghost_action_triggered = ghost.ghost_action_triggered;
    let the_light_logic = ghost.the_light_logic;
    let victim = ghost.ghost_touch_pygmy_logic;

    if go_into_the_light {
        set_selectable(world, this, false);

        if let Some(the_light_logic) = the_light_logic {
            pg_engine::logic::set_behavior(world, the_light_logic, Some("TheLightEnter"));
        }

        pg_engine::logic::set_behavior(world, this, Some("PygmyGhostEnterTheLight"));
        return ItemResult::Replaced;
    }

    if !ghost_action_triggered {
        if let Some(victim) = victim {
            cancel_ghost_touch(world, victim, this);
            world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_touch_pygmy_logic = None;
        }
    }

    ItemResult::Done
}

/// The rest of `ghostFollowPath:::` while the next path point is still ahead: the heading
/// toward it, then the ghost's action on its victim once it gets there, or the flip toward the
/// heading.
fn follow_ghost_path_heading(world: &mut World, this: Entity, display_object: Entity, delta_x: f32, delta_y: f32) -> ItemResult {
    let heading = delta_y.atan2(delta_x);
    let ghost_target_z_rot = heading * 180.0 / std::f32::consts::PI - 90.0;
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_target_z_rot = ghost_target_z_rot;

    let ghost = world.obj::<PygmyGhostZombieVampire>(this);
    let victim = ghost.ghost_touch_pygmy_logic;
    let ghost_touch_type = ghost.ghost_touch_type;

    if let Some(victim) = victim {
        if ghost_touch_type != GHOST_TOUCH_PULL_UNDER {
            if is_inside_victim(world, display_object, victim) {
                return start_ghost_action(world, this, ghost_touch_type);
            }
        } else {
            let y_pos = display::object(world, display_object).pos.y;
            if y_pos < 96.0 {
                return start_pulling_victim_under(world, this, display_object);
            }
        }
    }

    let ghost_flipping = world.obj::<PygmyGhostZombieVampire>(this).ghost_flipping;
    if ghost_flipping {
        return ItemResult::Pending;
    }

    let target_x_scale = ghost_x_scale_for_heading(ghost_target_z_rot);
    let x_scale = display::object(world, display_object).scale.x;
    if target_x_scale == x_scale {
        return ItemResult::Pending;
    }

    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_flipping = true;
    pg_engine::logic::set_behavior(world, this, Some("PygmyGhostFollowPathFlip"));
    ItemResult::Replaced
}

/// Whether the ghost is strictly inside the victim's box.
fn is_inside_victim(world: &mut World, display_object: Entity, victim: Entity) -> bool {
    let victim_display_object = pg_engine::logic::display_object(world, victim);
    let min_pos = display::min_pos(world, victim_display_object);
    let max_pos = display::max_pos(world, victim_display_object);
    let pos = display::object(world, display_object).pos;

    pos.x > min_pos.x && pos.x < max_pos.x && pos.y > min_pos.y && pos.y < max_pos.y
}

/// The ghost reached its victim: it stops, drops the rest of its path and scares, possesses
/// or carries away the victim.
fn start_ghost_action(world: &mut World, this: Entity, ghost_touch_type: i32) -> ItemResult {
    stop_moving(world, this);

    let mut ghost = world.obj_mut::<PygmyGhostZombieVampire>(this);
    ghost.ghost_target_z_rot = 0.0;
    ghost.ghost_action_triggered = true;

    cancel_flip(world, this);
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_path_vector.clear();
    set_selectable(world, this, false);

    let (behavior, act_on_victim): (&str, fn(&mut World, Entity, Entity)) = match ghost_touch_type {
        GHOST_TOUCH_SCARE => ("PygmyGhostScarePygmy", scared_by_ghost),
        GHOST_TOUCH_POSSESS => ("PygmyGhostPossessPygmy", possessed_by_ghost),
        GHOST_TOUCH_CARRY_AWAY => ("PygmyGhostStartCarryAwayPygmy", carried_away_by_ghost),
        _ => return ItemResult::Replaced,
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));

    let victim = world.obj::<PygmyGhostZombieVampire>(this).ghost_touch_pygmy_logic;
    if let Some(victim) = victim {
        act_on_victim(world, victim, this);
    }

    ItemResult::Replaced
}

/// The ghost dives into the ground at its victim's feet to pull it under.
fn start_pulling_victim_under(world: &mut World, this: Entity, display_object: Entity) -> ItemResult {
    stop_moving(world, this);

    let x_pos = display::object(world, display_object).pos.x;
    let mut ghost = world.obj_mut::<PygmyGhostZombieVampire>(this);
    ghost.ghost_target_z_rot = 0.0;
    ghost.ghost_emerge_x_pos = x_pos;

    reset_z_rot(world, this);
    cancel_flip(world, this);
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_path_vector.clear();
    set_selectable(world, this, false);
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_action_triggered = true;

    pg_engine::logic::set_behavior(world, this, Some("PygmyGhostPullPygmyUnder"));
    ItemResult::Replaced
}

/// `-[PygmyLogic (Island) eatBrainsComplete:::]` @ 0x1000b80a8
pub fn seq_eat_brains_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.scale.x = -object.scale.x;

    ItemResult::Done
}

/// `-[PygmyLogic (Island) zombieWalk:::]` @ 0x1000b81e0
pub fn seq_zombie_walk(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let stall_time = random_item_time(world, call);
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;

        let zombie_choose_direction = call.param("noFlip").is_none();
        world.obj_mut::<PygmyGhostZombieVampire>(this).zombie_choose_direction = zombie_choose_direction;
    }

    if check_for_fresh_brains(world, this) {
        return ItemResult::Replaced;
    }

    let island_side_collision = world.obj::<PygmyPygmyCoreFrame>(this).island_side_collision;
    let mut result = if island_side_collision {
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    let stall_time = pg_engine::logic::logic(world, this).stall_time;
    if stall_time <= 0.0 {
        result = ItemResult::Done;
    }

    pg_engine::logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    result
}

/// `-[PygmyLogic (Island) zombieIdle:::]` @ 0x1000b833c
pub fn seq_zombie_idle(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let stall_time = random_item_time(world, call);
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;
    }

    if check_for_fresh_brains(world, this) {
        return ItemResult::Replaced;
    }

    let stall_time = pg_engine::logic::logic(world, this).stall_time;
    let result = if stall_time <= 0.0 {
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    pg_engine::logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    result
}

/// `-[PygmyLogic (Island) explodeVampireSun:::]` @ 0x1000ba8d0
pub fn seq_explode_vampire_sun(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).explode_from_vampire_sun = true;
    seq_explode(world, this, call)
}

/// `-[PygmyLogic (Island) explodeVampireGinger:::]` @ 0x1000ba8f4
pub fn seq_explode_vampire_ginger(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).explode_from_vampire_ginger = true;
    seq_explode(world, this, call)
}

/// `-[PygmyLogic (Island) checkKillPygmies:::]` @ 0x1000bd798
pub fn seq_check_kill_pygmies(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let mut stall_time = ENDLESS_WAIT;
        if call.has("minTime") {
            stall_time = random_item_time(world, call);
        }
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;

        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let listener = listener_of(world, this);
    if !is_vampire_night(world, listener) {
        let behavior = vampire_daylight_behavior(world, this, listener);
        pg_engine::logic::set_behavior(world, this, Some(behavior));
        return ItemResult::Replaced;
    }

    if any_pygmy_can_be_hit_by_vampire(world, this) {
        pg_engine::logic::set_behavior(world, this, Some("PygmyKillPygmies"));
        return ItemResult::Replaced;
    }

    let stall_time = pg_engine::logic::logic(world, this).stall_time;
    if stall_time == ENDLESS_WAIT {
        return ItemResult::Pending;
    }
    if stall_time <= 0.0 {
        return ItemResult::Done;
    }

    pg_engine::logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    ItemResult::Pending
}

/// `-[PygmyLogic (Island) vampireRun:::]` @ 0x1000bda9c
pub fn seq_vampire_run(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let x_pos = x_pos_of(world, Some(this));
    let listener = listener_of(world, this);

    if call.first_time {
        let stall_time = random_item_time(world, call);
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;

        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_side_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

        let island_center_x_pos = match listener {
            Some(listener) => crate::controller::island_center_x_pos(world, listener),
            None => 0.0,
        };
        let (x_scale, x_vel) = if x_pos > island_center_x_pos {
            (-1.0, -100.0)
        } else {
            (1.0, 100.0)
        };
        set_x_scale(world, this, x_scale);
        pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;
    }

    if !is_vampire_night(world, listener) {
        let behavior = vampire_daylight_behavior(world, this, listener);
        pg_engine::logic::set_behavior(world, this, Some(behavior));
        return ItemResult::Replaced;
    }

    if any_pygmy_can_be_hit_by_vampire(world, this) {
        pg_engine::logic::set_behavior(world, this, Some("PygmyKillPygmies"));
        return ItemResult::Replaced;
    }

    let island_side_collision = world.obj::<PygmyPygmyCoreFrame>(this).island_side_collision;
    let mut result = if island_side_collision {
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    let stall_time = pg_engine::logic::logic(world, this).stall_time;
    if stall_time <= 0.0 {
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        result = ItemResult::Done;
    }

    pg_engine::logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    result
}

/// `-[PygmyLogic (Island) checkForVampire:::]` @ 0x1000bde44
pub fn seq_check_for_vampire(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let bit_by_bat = world.obj::<PygmyGhostZombieVampire>(this).bit_by_bat;
    if !bit_by_bat {
        return ItemResult::Done;
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyKillPygmies"));
    ItemResult::Replaced
}

/// `-[PygmyLogic (Island) waitForNight:::]` @ 0x1000bde8c
pub fn seq_wait_for_night(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let mut stall_time = ENDLESS_WAIT;
        if call.has("minTime") {
            stall_time = random_item_time(world, call);
        }
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;
    }

    let listener = listener_of(world, this);
    if is_vampire_night(world, listener) {
        pg_engine::logic::set_behavior(world, this, Some("PygmyKillPygmies"));
        return ItemResult::Replaced;
    }

    // A timed wait runs out its timer in daylight and leaves at sunset; an endless one waits
    // through the sunset and burns once it is day.
    let sunset = is_sunset_active(world, listener);
    let stall_time = pg_engine::logic::logic(world, this).stall_time;
    let endless = stall_time == ENDLESS_WAIT;

    if sunset && !endless {
        pg_engine::logic::set_behavior(world, this, Some("PygmyVampireSunset"));
        return ItemResult::Replaced;
    }
    if !sunset && endless {
        set_sacrifice_id(world, this, "VampireInSunlight");
        pg_engine::logic::set_behavior(world, this, Some("PygmyVampireSun"));
        return ItemResult::Replaced;
    }

    if endless {
        return ItemResult::Pending;
    }

    let result = if stall_time <= 0.0 {
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    pg_engine::logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    result
}

/// `-[PygmyLogic (Island) killPygmies:::]` @ 0x1000be098
pub fn seq_kill_pygmies(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);

    if call.first_time {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_side_collision = true;
    }

    let listener = listener_of(world, this);
    if !is_vampire_night(world, listener) {
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;

        let behavior = vampire_daylight_behavior(world, this, listener);
        pg_engine::logic::set_behavior(world, this, Some(behavior));
        return ItemResult::Replaced;
    }

    let mut victim = None;
    let mut victim_distance = 1000.0;

    let pygmies = pygmy_logic_array(world, this);
    for pygmy in pygmies {
        if !can_be_hit_by_vampire(world, pygmy) {
            continue;
        }

        let x_pos = display::object(world, display_object).pos.x;
        let pygmy_x_pos = x_pos_of(world, Some(pygmy));
        let distance = (x_pos - pygmy_x_pos).abs();
        if distance < victim_distance {
            victim = Some(pygmy);
            victim_distance = distance;
        }
    }

    let Some(victim) = victim else {
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        pg_engine::logic::set_behavior(world, this, Some("PygmyVampireIdle"));
        return ItemResult::Done;
    };

    let victim_type = time_temple_pygmy_type(world, victim);
    let reach = if victim_type == GINGER_PYGMY { 40.0 } else { 5.0 };

    let x_pos = display::object(world, display_object).pos.x;
    let victim_x_pos = x_pos_of(world, Some(victim));
    let in_reach = x_pos > victim_x_pos - reach && x_pos < reach + victim_x_pos;

    if in_reach {
        if victim_type == GINGER_PYGMY {
            set_sacrifice_id(world, this, "GingerPygmyKillVampire");
            pg_engine::logic::set_behavior(world, this, Some("PygmyVampireAttackGinger"));
            return ItemResult::Replaced;
        }

        let x_vel = pg_engine::logic::logic(world, this).pos_vel.x;
        hit_by_vampire(world, victim, this, x_vel);
    }

    let x_pos = display::object(world, display_object).pos.x;
    let victim_x_pos = x_pos_of(world, Some(victim));
    let (x_scale, x_vel) = if x_pos > victim_x_pos {
        (-1.0, -200.0)
    } else {
        (1.0, 200.0)
    };
    set_x_scale(world, this, x_scale);
    pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;

    ItemResult::Pending
}

/// `-[PygmyLogic (Island) touchBeganInsideGhost:allTouches:]` @ 0x1000bec98
pub fn touch_began_inside_ghost(world: &mut World, this: Entity, touch: Entity, _all_touches: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).active_touch = Some(touch);

    let display_object = pg_engine::logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let point = view_to_display_group(world, display_object, touch_pos);

    stop_moving(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).touch_began_inside = true;

    let mut ghost = world.obj_mut::<PygmyGhostZombieVampire>(this);
    ghost.ghost_path_vector.clear();
    ghost.ghost_path_vector.push(pos);
    ghost.ghost_touch_x_offset = point.x - pos.x;
    ghost.ghost_touch_y_offset = point.y - pos.y;

    let listener = listener_of(world, this);
    if let Some(listener) = listener {
        crate::controller::on_pygmy_ghost_drag(world, listener, this);
    }

    let the_light_logic = world.obj::<PygmyGhostZombieVampire>(this).the_light_logic;
    if the_light_logic.is_none() {
        create_the_light(world, this);
    }

    let ghost = world.obj::<PygmyGhostZombieVampire>(this);
    let the_light_display_object = ghost.the_light_display_object;
    let the_light_logic = ghost.the_light_logic;
    display::insert_before(world, display_object, the_light_display_object);
    if let Some(the_light_logic) = the_light_logic {
        pg_engine::logic::set_behavior(world, the_light_logic, Some("TheLightInit"));
    }

    let mut ghost = world.obj_mut::<PygmyGhostZombieVampire>(this);
    ghost.ghost_action_triggered = false;
    ghost.ghost_banshee_scream = true;

    pg_engine::logic::set_behavior(world, this, Some("PygmyGhostStartScream"));
}

/// The light a ghost can be dragged into: a `TheLight` multi-model with its own logic, which
/// reports its events to the ghost.
fn create_the_light(world: &mut World, this: Entity) {
    let the_light_display_object = display::multi_model::spawn(world, Some("TheLight"));
    world.obj_mut::<PygmyGhostZombieVampire>(this).the_light_display_object = Some(the_light_display_object);

    let the_light_logic = pg_engine::logic::multi_model::spawn(world, the_light_display_object);
    world.obj_mut::<PygmyGhostZombieVampire>(this).the_light_logic = Some(the_light_logic);
    pg_engine::logic::multi_model::set_multi_model_logic_listener(world, the_light_logic, Some(this));

    let mut object = display::object_mut(world, the_light_display_object);
    object.pos.x = 50.0;
    object.pos.y = 250.0;
    object.pos.z = display::model::SPRITE_Z;
}

/// `-[PygmyLogic (Island) touchMovedGhost:allTouches:]` @ 0x1000bef70
pub fn touch_moved_ghost(world: &mut World, this: Entity, touch: Entity, _all_touches: Entity) {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let active_touch = core.active_touch;
    let touch_began_inside = core.touch_began_inside;

    let ghost = world.obj::<PygmyGhostZombieVampire>(this);
    let go_into_the_light = ghost.go_into_the_light;
    let path_point_count = ghost.ghost_path_vector.len();

    let is_own_touch = active_touch.is_none() || active_touch == Some(touch);
    if !is_own_touch || !touch_began_inside || go_into_the_light || path_point_count >= 40 {
        return;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let point = view_to_display_group(world, display_object, touch_pos);

    let victim = world.obj::<PygmyGhostZombieVampire>(this).ghost_touch_pygmy_logic;
    match victim {
        None => {
            if path_point_count > 2 && is_near_the_light(world, this, point) {
                world.obj_mut::<PygmyGhostZombieVampire>(this).go_into_the_light = true;
                return;
            }

            choose_ghost_victim(world, this, display_object, point);
        }
        Some(victim) => {
            let ghost_touch_type = world.obj::<PygmyGhostZombieVampire>(this).ghost_touch_type;
            if ghost_touch_type == GHOST_TOUCH_POSSESS && !is_point_on_victim(world, victim, point) {
                world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_touch_type = GHOST_TOUCH_SCARE;
            }
        }
    }

    add_ghost_path_point(world, this, point);
}

/// Whether the finger is within 20 points of the light. The light always exists while the
/// ghost is held; without one the original would read through a nil pointer.
fn is_near_the_light(world: &World, this: Entity, point: Vec4) -> bool {
    let the_light_display_object = world.obj::<PygmyGhostZombieVampire>(this).the_light_display_object;
    let Some(the_light_display_object) = the_light_display_object else {
        return false;
    };

    let light_pos = display::object(world, the_light_display_object).pos;
    let delta_x = point.x - light_pos.x;
    let delta_y = point.y - light_pos.y;
    let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

    distance < 20.0
}

/// A possessed victim stays possessed while the finger stays on its box (edges included).
fn is_point_on_victim(world: &mut World, victim: Entity, point: Vec4) -> bool {
    let victim_display_object = pg_engine::logic::display_object(world, victim);
    let min_pos = display::min_pos(world, victim_display_object);
    let max_pos = display::max_pos(world, victim_display_object);

    let off_victim = point.x < min_pos.x || point.x > max_pos.x || point.y < min_pos.y || point.y > max_pos.y;
    !off_victim
}

/// The first pygmy that can be touched and whose box the finger is strictly inside becomes the
/// ghost's victim, with a touch type from where the finger entered the box (the last touch
/// point): from below (with the ghost above 100) it is pulled under, from above carried away,
/// from the side possessed. A finger that was already inside the box picks no one.
fn choose_ghost_victim(world: &mut World, this: Entity, display_object: Entity, point: Vec4) {
    let pygmies = pygmy_logic_array(world, this);

    for pygmy in pygmies {
        if !can_be_touched_by_ghost(world, pygmy, this) {
            continue;
        }

        let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
        let min_pos = display::min_pos(world, pygmy_display_object);
        let max_pos = display::max_pos(world, pygmy_display_object);

        let inside = point.x > min_pos.x && point.x < max_pos.x && point.y > min_pos.y && point.y < max_pos.y;
        if !inside {
            continue;
        }

        let last_touch_pos = world.obj::<PygmyGhostZombieVampire>(this).ghost_last_touch_pos;
        let ghost_y_pos = display::object(world, display_object).pos.y;

        let ghost_touch_type = if last_touch_pos.y <= min_pos.y && ghost_y_pos > 100.0 {
            GHOST_TOUCH_PULL_UNDER
        } else if last_touch_pos.y >= max_pos.y {
            GHOST_TOUCH_CARRY_AWAY
        } else if last_touch_pos.x <= min_pos.x || last_touch_pos.x >= max_pos.x {
            GHOST_TOUCH_POSSESS
        } else {
            continue;
        };

        let mut ghost = world.obj_mut::<PygmyGhostZombieVampire>(this);
        ghost.ghost_touch_type = ghost_touch_type;
        ghost.ghost_touch_pygmy_logic = Some(pygmy);

        touched_by_ghost(world, pygmy, this);
        return;
    }
}

/// The tail of `touchMovedGhost:allTouches:`: remembers the touch, then turns it into the
/// ghost's next path point (minus the grab offset, kept on screen). A point within 20 of the
/// last appended one replaces the path's last point instead of adding one.
fn add_ghost_path_point(world: &mut World, this: Entity, point: Vec4) {
    let mut ghost = world.obj_mut::<PygmyGhostZombieVampire>(this);
    ghost.ghost_last_touch_pos = point;

    let mut path_point = point;
    path_point.x = point.x - ghost.ghost_touch_x_offset;
    path_point.y = point.y - ghost.ghost_touch_y_offset;

    if path_point.y < 40.0 {
        path_point.y = 40.0;
    }
    if path_point.y > 280.0 {
        path_point.y = 280.0;
    }

    if path_point.x < 21.0 {
        path_point.x = 21.0;
    } else if path_point.x > 440.0 {
        path_point.x = 440.0;
    }

    let last_path_pos = ghost.ghost_last_path_pos;
    let delta_x = path_point.x - last_path_pos.x;
    let delta_y = path_point.y - last_path_pos.y;
    let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

    if distance > 20.0 {
        ghost.ghost_path_vector.push(path_point);
        ghost.ghost_last_path_pos = path_point;
        return;
    }

    // The path always holds the ghost's start point here; the original would write in front
    // of an empty vector.
    if let Some(last_point) = ghost.ghost_path_vector.last_mut() {
        *last_point = path_point;
    }
}
