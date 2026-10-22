//! `PygmyLogic`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;

use crate::logic::ice_island::ice_hole_logic;
use crate::logic::island_core::sprite_logic::SpriteLogic;
use crate::logic::monsters::zombie_grave_logic;
use crate::logic::moronpests_chase::moron_snake_logic;
use crate::logic::sky_powers::hurricane_logic;

/// `[listener islandLocationState]` on the ice island.
const ICE_LOCATION: i32 = 4;

/// The x range around the ice hole that the sun walk and the sun burn aim for.
const ICE_HOLE_MIN_X_POS: f32 = 290.0;
const ICE_HOLE_MAX_X_POS: f32 = 350.0;

/// The monster a pygmy keeps away from (`checkForTRex:::`, `checkForSpider:::` and the items
/// of the sequences they start). The T-Rex and spider versions of the original are identical
/// apart from these values.
struct Monster {
    /// `[listener isTRexActive]` / `[listener isSpiderActive]`.
    is_active: fn(&mut World, Entity) -> bool,

    /// `[listener tRexLogic:self]` / `[listener spiderLogic:self]`.
    logic: fn(&mut World, Entity, Entity) -> Option<Entity>,

    /// `(modulus, offset)` of the draws `(arc4random() % modulus + offset) / 100` for
    /// `mSafeIslandMinXPos`, `mSafeIslandMaxXPos`, `mSafeRunIslandMinXPos` and
    /// `mSafeRunIslandMaxXPos`, in that order.
    zones: [(u32, u32); 4],

    stay_away_behavior: &'static str,
    run_under_behavior: &'static str,
    step_away_behavior: &'static str,
}

const T_REX: Monster = Monster {
    is_active: crate::controller::is_t_rex_active,
    logic: crate::controller::t_rex_logic,
    zones: [(3642, 6819), (3641, 35940), (9101, 10460), (9101, 26840)],
    stay_away_behavior: "PygmyStayAwayFromTRex",
    run_under_behavior: "PygmyRunUnderTRex",
    step_away_behavior: "PygmyStepAwayFromTRex",
};

const SPIDER: Monster = Monster {
    is_active: crate::controller::is_spider_active,
    logic: crate::controller::spider_logic,
    zones: [(4161, 5280), (4161, 38560), (10401, 9440), (10402, 28159)],
    stay_away_behavior: "PygmyStayAwayFromSpider",
    run_under_behavior: "PygmyRunUnderSpider",
    step_away_behavior: "PygmyStepAwayFromSpider",
};

/// `mPygmyLogicListener`: always the view controller (`docs/re/pygmy.md` §23).
fn listener_of(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener
}

/// `[listener pygmyDisplayMarker]` (nil for a nil listener).
fn pygmy_display_marker_of(world: &mut World, listener: Option<Entity>) -> Option<Entity> {
    match listener {
        Some(listener) => crate::controller::pygmy_display_marker(world, listener),
        None => None,
    }
}

/// `[listener moronSnakeLogic:self]` (nil for a nil listener).
fn moron_snake_logic_of(world: &mut World, listener: Option<Entity>, this: Entity) -> Option<Entity> {
    match listener {
        Some(listener) => crate::controller::moron_snake_logic(world, listener, this),
        None => None,
    }
}

/// `[listener isInsideApocalypseTemple]` (NO for a nil listener).
fn is_inside_apocalypse_temple_of(world: &mut World, listener: Option<Entity>) -> bool {
    match listener {
        Some(listener) => crate::controller::is_inside_apocalypse_temple(world, listener),
        None => false,
    }
}

/// `[listener sunState]`: 0 at night (or for a nil listener), 1 burning, 2 freezing.
fn sun_state_of(world: &mut World, listener: Option<Entity>) -> i32 {
    match listener {
        Some(listener) => crate::controller::sun_state(world, listener),
        None => 0,
    }
}

/// `[listener islandCenterXPos]` (0 for a nil listener).
fn island_center_x_pos_of(world: &mut World, listener: Option<Entity>) -> f32 {
    match listener {
        Some(listener) => crate::controller::island_center_x_pos(world, listener),
        None => 0.0,
    }
}

/// The busy set FREE of `docs/re/pygmy.md` §1.3 without the members that some guard of this
/// part leaves out (the pygmy type, the two hurricane flags and the six of
/// [`is_cut_trapped_zombie_or_dead`]), plus `mbSharkOnHead` and a tilted pygmy, which every
/// guard here adds. Pure reads, so the order of the original's checks does not matter.
fn is_busy_common(core: &PygmyPygmyCoreFrame) -> bool {
    core.touch_began_inside
        || core.remove_from_story_mode
        || core.tilt_state != 0
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
        || core.drowning
}

/// The members of FREE that `pickedUpByHurricane:` leaves out besides the pygmy type and
/// `mbBlownByHurricane`: cut in half, trapped in a zombie grave or an idol pedestal, a zombie,
/// cooked, or a plague body.
fn is_cut_trapped_zombie_or_dead(core: &PygmyPygmyCoreFrame) -> bool {
    core.cut_in_half_by_charlie
        || core.trapped_in_zombie_grave
        || core.trapped_in_idol_pedestal
        || core.is_zombie
        || core.cooked
        || core.killed_by_plague
}

/// FREE +{mbSharkOnHead}, tilted: the guard of `canFallIntoCrackHole:` and `canMeltIceHole`.
fn is_busy(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let sprite = world.obj::<SpriteLogic>(this);

    is_busy_common(core)
        || core.time_temple_pygmy_type != 0
        || sprite.blown_by_hurricane
        || sprite.picked_up_by_hurricane
        || is_cut_trapped_zombie_or_dead(core)
}

/// FREE +{mbSharkOnHead, mbSleeping}, tilted: the guard of `canBeCaughtByBird:` and
/// `caughtByBird:`.
fn is_busy_for_bird(world: &World, this: Entity) -> bool {
    let sleeping = world.obj::<PygmyPygmyCoreFrame>(this).sleeping;
    is_busy(world, this) || sleeping
}

/// FREE −{T} +{mbSharkOnHead, mbSleeping}, tilted, and only a normal or a baby pygmy
/// (`(T | 2) == 2`): the guard of `eatenByMoronSnake:` and `wearWaterWings:`.
fn is_busy_for_snake_or_wings(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let sprite = world.obj::<SpriteLogic>(this);
    let pygmy_type = core.time_temple_pygmy_type as u32;

    is_busy_common(core)
        || (pygmy_type | 2) != 2
        || sprite.blown_by_hurricane
        || sprite.picked_up_by_hurricane
        || is_cut_trapped_zombie_or_dead(core)
        || core.sleeping
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

/// The stop list S12′: S13 without `stopThrowingSpear`, so a pygmy that panics at the T-Rex or
/// the spider keeps a spear it holds.
fn stop_activities_keep_spear(world: &mut World, this: Entity) {
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
}

/// `vel.xy = 0`: only x and y of `mPosVel` are cleared.
fn clear_xy_vel(world: &mut World, this: Entity) {
    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 0.0;
    logic.pos_vel.y = 0.0;
}

/// `[mDisplayObject setZRot:0]`.
fn clear_z_rot(world: &mut World, this: Entity) {
    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
}

/// `[mDisplayObject setIsSelectable:]`.
fn set_selectable(world: &mut World, this: Entity, selectable: bool) {
    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = selectable;
}

/// `mSacrificeId = @"…"`.
fn set_sacrifice_id(world: &mut World, this: Entity, sacrifice_id: &str) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some(sacrifice_id.to_owned());
}

/// `arc4random() % 3 + 2`: the sun burn / freeze cycles before the pygmy is burnt or frozen.
fn random_sun_cycle_count(world: &mut World) -> i32 {
    let draw = world.resource_mut::<Random>().arc4random_mod(3);
    (draw + 2) as i32
}

/// `(float)(int)(arc4random() % modulus + offset) / 100`: one bound of a T-Rex or spider zone.
fn random_zone_bound(world: &mut World, (modulus, offset): (u32, u32)) -> f32 {
    let draw = world.resource_mut::<Random>().arc4random_mod(modulus);
    let hundredths = (draw + offset) as i32;
    hundredths as f32 / 100.0
}

/// `(float)(int)(arc4random() % 121 + 80) / 10`: the speed of the squid's strength cycle.
fn random_tentacle_strength_rot_vel(world: &mut World) -> f32 {
    let draw = world.resource_mut::<Random>().arc4random_mod(121);
    let tenths = (draw + 80) as i32;
    tenths as f32 / 10.0
}

/// `[[[listener tRexLogic:self] displayObject] xPos]` (or the spider's): 0 when there is none.
fn monster_x_pos_of(world: &mut World, listener: Option<Entity>, this: Entity, monster: &Monster) -> f32 {
    let Some(listener) = listener else {
        return 0.0;
    };
    let Some(monster_logic) = (monster.logic)(world, listener, this) else {
        return 0.0;
    };

    let monster_display_object = pg_engine::logic::display_object(world, monster_logic);
    display::object(world, monster_display_object).pos.x
}

/// `[listener isIceHoleEnabled] && [hole isFrozen] && [hole shouldPygmyWalkTowardIceHole:self]`
/// with `hole = [listener iceHoleLogic:self]`, asked in that order.
fn is_ice_hole_calling(world: &mut World, this: Entity, listener: Option<Entity>) -> bool {
    let Some(listener) = listener else {
        return false;
    };

    let ice_hole = crate::controller::ice_hole_logic(world, listener, this);
    if !crate::controller::is_ice_hole_enabled(world, listener) {
        return false;
    }

    let Some(ice_hole) = ice_hole else {
        return false;
    };
    if !ice_hole_logic::is_frozen(world, ice_hole) {
        return false;
    }

    ice_hole_logic::should_pygmy_walk_toward_ice_hole(world, ice_hole, this)
}

/// `[hole tentaclePos]`: the position of the ice hole's tentacle display object.
fn ice_hole_tentacle_pos(world: &mut World, ice_hole: Option<Entity>) -> Vec4 {
    match ice_hole {
        Some(ice_hole) => ice_hole_logic::tentacle_pos(world, ice_hole),
        // PORT: the original reads through the NULL pointer `[nil tentaclePos]` returns; the hole
        // exists whenever the squid holds a pygmy.
        None => Vec4::ZERO,
    }
}

/// `checkStayAwayFromTRex:::` / `checkStayAwayFromSpider:::`.
fn check_stay_away(world: &mut World, this: Entity, monster: &Monster) -> ItemResult {
    let listener = listener_of(world, this);
    let is_active = match listener {
        Some(listener) => (monster.is_active)(world, listener),
        None => false,
    };

    if !is_active {
        pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
        return ItemResult::Replaced;
    }

    let monster_x_pos = monster_x_pos_of(world, listener, this, monster);
    let display_object = pg_engine::logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;
    let center_x_pos = island_center_x_pos_of(world, listener);

    let both_left_of_center = monster_x_pos < center_x_pos && x_pos < center_x_pos;
    let both_right_of_center = monster_x_pos > center_x_pos && x_pos > center_x_pos;

    if both_left_of_center || both_right_of_center {
        // Same side as the monster: face the far side and run under it.
        let x_scale = if monster_x_pos < center_x_pos { 1.0 } else { -1.0 };
        display::object_mut(world, display_object).scale.x = x_scale;

        pg_engine::logic::set_behavior(world, this, Some(monster.run_under_behavior));
        return ItemResult::Replaced;
    }

    let hazards = world.obj::<PygmyIslandHazards>(this);
    let near_monster_on_left = monster_x_pos < center_x_pos && x_pos < hazards.safe_island_max_x_pos;
    let near_monster_on_right = monster_x_pos > center_x_pos && x_pos > hazards.safe_island_min_x_pos;

    if !near_monster_on_left && !near_monster_on_right {
        return ItemResult::Done;
    }

    pg_engine::logic::set_behavior(world, this, Some(monster.step_away_behavior));
    ItemResult::Replaced
}

/// `runUnderTRex:::` / `runUnderSpider:::`: run at 200 to the other side of the island until
/// past the run zone, then stop and face the monster.
fn run_under(world: &mut World, this: Entity, call: &SequenceCall, monster: &Monster) -> ItemResult {
    let listener = listener_of(world, this);
    let monster_x_pos = monster_x_pos_of(world, listener, this, monster);
    let display_object = pg_engine::logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;

    if call.first_time {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_side_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let center_x_pos = island_center_x_pos_of(world, listener);

    // `b.le` sends NaN to the right.
    let (x_scale, x_vel) = if monster_x_pos > center_x_pos { (-1.0, -200.0) } else { (1.0, 200.0) };
    display::object_mut(world, display_object).scale.x = x_scale;
    pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;

    let hazards = world.obj::<PygmyIslandHazards>(this);
    let past_right_zone = monster_x_pos < center_x_pos && x_pos > hazards.safe_run_island_max_x_pos;
    let past_left_zone = monster_x_pos > center_x_pos && x_pos < hazards.safe_run_island_min_x_pos;

    if !past_right_zone && !past_left_zone {
        return ItemResult::Pending;
    }

    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;

    let x_pos = display::object(world, display_object).pos.x;
    let x_scale = if x_pos < monster_x_pos { 1.0 } else { -1.0 };
    display::object_mut(world, display_object).scale.x = x_scale;

    ItemResult::Done
}

/// `checkForTRex:::` / `checkForSpider:::`: while the monster is active, roll the zones, face
/// the monster and keep away from it.
fn check_for_monster(world: &mut World, this: Entity, monster: &Monster) -> ItemResult {
    let listener = listener_of(world, this);
    let is_active = match listener {
        Some(listener) => (monster.is_active)(world, listener),
        None => false,
    };

    if !is_active {
        return ItemResult::Done;
    }

    stop_activities_keep_spear(world, this);

    let [safe_min, safe_max, safe_run_min, safe_run_max] = monster.zones;

    let safe_island_min_x_pos = random_zone_bound(world, safe_min);
    world.obj_mut::<PygmyIslandHazards>(this).safe_island_min_x_pos = safe_island_min_x_pos;

    let safe_island_max_x_pos = random_zone_bound(world, safe_max);
    world.obj_mut::<PygmyIslandHazards>(this).safe_island_max_x_pos = safe_island_max_x_pos;

    let safe_run_island_min_x_pos = random_zone_bound(world, safe_run_min);
    world.obj_mut::<PygmyIslandHazards>(this).safe_run_island_min_x_pos = safe_run_island_min_x_pos;

    let safe_run_island_max_x_pos = random_zone_bound(world, safe_run_max);
    world.obj_mut::<PygmyIslandHazards>(this).safe_run_island_max_x_pos = safe_run_island_max_x_pos;

    let monster_x_pos = monster_x_pos_of(world, listener, this, monster);
    let display_object = pg_engine::logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;

    let x_scale = if x_pos < monster_x_pos { 1.0 } else { -1.0 };
    display::object_mut(world, display_object).scale.x = x_scale;

    pg_engine::logic::set_behavior(world, this, Some(monster.stay_away_behavior));
    ItemResult::Done
}

/// `-[PygmyLogic (Island) canFallIntoCrackHole:]` @ 0x1000a723c
pub fn can_fall_into_crack_hole(world: &mut World, this: Entity, _crack_hole: Entity) -> bool {
    !is_busy(world, this)
}

/// `-[PygmyLogic (Island) fallIntoCrackHole:]` @ 0x1000a78f0
///
/// No guard of its own: the crack hole asks `canFallIntoCrackHole:` first.
pub fn fall_into_crack_hole(world: &mut World, this: Entity, _crack_hole: Entity) {
    stop_activities(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.in_crack_hole = true;
    core.tilt_state = 0;

    clear_xy_vel(world, this);
    clear_z_rot(world, this);
    set_selectable(world, this, false);
    set_sacrifice_id(world, this, "CrackHole");

    let listener = listener_of(world, this);
    let marker = pygmy_display_marker_of(world, listener);
    if let Some(marker) = marker {
        let display_object = pg_engine::logic::display_object(world, this);
        display::insert_before(world, marker, Some(display_object));
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallIntoCrackHole"));
}

/// `-[PygmyLogic (Island) blownByHurricane:]` @ 0x1000a8c50
pub fn blown_by_hurricane(world: &mut World, this: Entity, _hurricane: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let sprite = world.obj::<SpriteLogic>(this);

    // FREE −{mbPickedUpByHurricane} +{mbSharkOnHead}, tilted.
    let busy = is_busy_common(core)
        || core.time_temple_pygmy_type != 0
        || sprite.blown_by_hurricane
        || is_cut_trapped_zombie_or_dead(core);

    if busy {
        return false;
    }

    stop_activities(world, this);

    world.obj_mut::<SpriteLogic>(this).blown_by_hurricane = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;

    clear_xy_vel(world, this);
    clear_z_rot(world, this);
    set_selectable(world, this, false);

    pg_engine::logic::set_behavior(world, this, Some("PygmyBlownByHurricane"));
    true
}

/// `-[PygmyLogic (Island) unblownByHurricane:]` @ 0x1000a9488
pub fn unblown_by_hurricane(world: &mut World, this: Entity, _hurricane: Entity) {
    let blown_by_hurricane = world.obj::<SpriteLogic>(this).blown_by_hurricane;
    let hit_by_lightning = world.obj::<PygmyPygmyCoreFrame>(this).hit_by_lightning;

    if !blown_by_hurricane || hit_by_lightning {
        return;
    }

    world.obj_mut::<SpriteLogic>(this).blown_by_hurricane = false;
    set_selectable(world, this, true);

    pg_engine::logic::set_behavior(world, this, Some("PygmyUnblownByHurricane"));
}

/// `-[PygmyLogic (Island) pickedUpByHurricane:]` @ 0x1000a9518
///
/// A zombie explodes instead. Cut, temple, blown, trapped, cooked and plague pygmies can be
/// picked up; a pygmy trapped in a zombie grave only while the grave is not turning it.
pub fn picked_up_by_hurricane(world: &mut World, this: Entity, hurricane: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    if core.is_zombie {
        zombie_explode(world, this);
        return false;
    }

    let trapped_in_zombie_grave = core.trapped_in_zombie_grave;
    let about_to_become_zombie = if trapped_in_zombie_grave {
        let zombie_grave = world.obj::<PygmyContainersAndCatchers>(this).zombie_grave_logic;
        match zombie_grave {
            Some(zombie_grave) => zombie_grave_logic::is_pygmy_about_to_become_zombie(world, zombie_grave),
            None => false,
        }
    } else {
        false
    };

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let picked_up_by_hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let busy = about_to_become_zombie || is_busy_common(core) || picked_up_by_hurricane;

    if busy {
        return false;
    }

    // S15: S13 with the two trap releases before `closeWordBalloon`.
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
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);

    let mut sprite = world.obj_mut::<SpriteLogic>(this);
    sprite.ignore_gravity = true;
    sprite.blown_by_hurricane = false;
    sprite.picked_up_by_hurricane = true;

    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    clear_xy_vel(world, this);
    world.obj_mut::<SpriteLogic>(this).picked_up_by_hurricane_logic = Some(hurricane);

    clear_z_rot(world, this);
    set_selectable(world, this, false);

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let behavior = if core.cooked {
        "PygmyCookedPickedUpByHurricane"
    } else if core.killed_by_plague {
        "PygmyPlagueBodyPickedUpByHurricane"
    } else if core.cut_in_half_by_charlie {
        "HalfPygmyPickedUpByHurricane"
    } else if core.time_temple_pygmy_type == 0 {
        "PygmyPickedUpByHurricane"
    } else {
        "TemplePygmyPickedUpByHurricane"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
    true
}

/// `-[PygmyLogic (Island) droppedByHurricane:]` @ 0x1000a9e3c
pub fn dropped_by_hurricane(world: &mut World, this: Entity, _hurricane: Entity) {
    let picked_up_by_hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let core = world.obj::<PygmyPygmyCoreFrame>(this);

    if !picked_up_by_hurricane || core.catch_shark || core.hit_by_lightning {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;

    let mut sprite = world.obj_mut::<SpriteLogic>(this);
    sprite.ignore_gravity = false;
    sprite.blown_by_hurricane = false;
    sprite.picked_up_by_hurricane = false;

    world.obj_mut::<PygmyPygmyCoreFrame>(this).dropped_by_hurricane = true;
    world.obj_mut::<SpriteLogic>(this).picked_up_by_hurricane_logic = None;

    set_selectable(world, this, true);

    // Dropped behind the island: draw it behind the island too.
    let display_object = pg_engine::logic::display_object(world, this);
    let z_pos = display::object(world, display_object).pos.z;
    if z_pos < SPRITE_Z {
        let listener = listener_of(world, this);
        let marker = match listener {
            Some(listener) => crate::controller::pygmy_island_display_marker(world, listener),
            None => None,
        };
        if let Some(marker) = marker {
            display::insert_before(world, marker, Some(display_object));
        }

        update_label_layer(world, this);
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let behavior = if core.cooked {
        "PygmyCookedDroppedByHurricane"
    } else if core.killed_by_plague {
        "PygmyPlagueBodyDroppedByHurricane"
    } else if core.cut_in_half_by_charlie {
        "HalfPygmyDroppedByHurricane"
    } else if core.time_temple_pygmy_type == 0 {
        set_sacrifice_id(world, this, "HurricaneDrown");
        "PygmyDroppedByHurricane"
    } else {
        "TemplePygmyDroppedByHurricane"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Island) canBeCaughtByBird:]` @ 0x1000aa9dc
pub fn can_be_caught_by_bird(world: &mut World, this: Entity, _bird: Entity) -> bool {
    !is_busy_for_bird(world, this)
}

/// `-[PygmyLogic (Island) caughtByBird:]` @ 0x1000ab0a4
pub fn caught_by_bird(world: &mut World, this: Entity, bird: Entity) {
    if is_busy_for_bird(world, this) {
        return;
    }

    stop_activities(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).caught_by_bird = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;

    clear_xy_vel(world, this);
    world.obj_mut::<PygmyIslandHazards>(this).carrying_bird_logic = Some(bird);

    clear_z_rot(world, this);
    set_selectable(world, this, false);

    pg_engine::logic::set_behavior(world, this, Some("PygmyCaughtByBird"));
}

/// `-[PygmyLogic (Island) caughtByBirdComplete:]` @ 0x1000ab930
pub fn caught_by_bird_complete(world: &mut World, this: Entity, _bird: Entity) {
    let caught_by_bird = world.obj::<PygmyPygmyCoreFrame>(this).caught_by_bird;
    if !caught_by_bird {
        return;
    }

    world.obj_mut::<PygmyIslandHazards>(this).carrying_bird_logic = None;
    set_sacrifice_id(world, this, "CaughtByBird");

    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Island) droppedByBird:]` @ 0x1000ab988
pub fn dropped_by_bird(world: &mut World, this: Entity, _bird: Entity) {
    let caught_by_bird = world.obj::<PygmyPygmyCoreFrame>(this).caught_by_bird;
    if !caught_by_bird {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).caught_by_bird = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyIslandHazards>(this).carrying_bird_logic = None;

    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.scale.x = 1.0;
    object.scale.y = 1.0;
    object.is_selectable = true;

    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Island) eatenByMoronSnake:]` @ 0x1000b07c4
pub fn eaten_by_moron_snake(world: &mut World, this: Entity, moron_snake: Entity) -> bool {
    if is_busy_for_snake_or_wings(world, this) {
        return false;
    }

    stop_activities(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    clear_xy_vel(world, this);
    clear_z_rot(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).eaten_by_moron_snake = true;
    set_selectable(world, this, false);
    set_sacrifice_id(world, this, "EatenByMoronSnake");

    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    let behavior = if pygmy_type == 2 {
        "TempleBabyPygmyEatenByMoronSnake"
    } else {
        "PygmyEatenByMoronSnake"
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));

    let display_object = pg_engine::logic::display_object(world, this);
    let moron_snake_display_object = pg_engine::logic::display_object(world, moron_snake);
    display::insert_after(world, moron_snake_display_object, Some(display_object));

    update_label_layer(world, this);
    true
}

/// `-[PygmyLogic (Island) ejectFromWaterWings]` @ 0x1000b10a8
pub fn eject_from_water_wings(world: &mut World, this: Entity) {
    let wearing_water_wings = world.obj::<PygmyPygmyCoreFrame>(this).wearing_water_wings;
    if !wearing_water_wings {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).wearing_water_wings = false;

    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.pos.y += 20.0;
    object.scale.x = 1.0;
    object.scale.y = 1.0;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = -200.0;
    logic.pos_vel.y = 500.0;

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = true;

    set_selectable(world, this, true);

    let listener = listener_of(world, this);
    let marker = pygmy_display_marker_of(world, listener);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }

    update_label_layer(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Island) wearWaterWings:]` @ 0x1000b1210
pub fn wear_water_wings(world: &mut World, this: Entity, water_wings: Entity) -> bool {
    if is_busy_for_snake_or_wings(world, this) {
        return false;
    }

    stop_activities(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    clear_xy_vel(world, this);
    clear_z_rot(world, this);

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_side_collision = false;
    core.enable_island_floor_collision = false;
    core.wearing_water_wings = true;

    set_sacrifice_id(world, this, "WaterWingsDrowned");
    set_selectable(world, this, false);
    pg_engine::logic::set_behavior(world, this, Some("PygmyWearWaterWings"));

    let display_object = pg_engine::logic::display_object(world, this);
    let water_wings_display_object = pg_engine::logic::display_object(world, water_wings);
    display::insert_after(world, water_wings_display_object, Some(display_object));

    update_label_layer(world, this);
    true
}

/// `-[PygmyLogic (Island) animBabyThrowSnake]` @ 0x1000b1b08
///
/// A baby pygmy throws the snake off and escapes.
pub fn anim_baby_throw_snake(world: &mut World, this: Entity) {
    let listener = listener_of(world, this);
    let marker = pygmy_display_marker_of(world, listener);
    if let Some(marker) = marker {
        let display_object = pg_engine::logic::display_object(world, this);
        display::insert_after(world, marker, Some(display_object));
    }

    update_label_layer(world, this);
    set_selectable(world, this, true);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).eaten_by_moron_snake = false;

    let moron_snake = moron_snake_logic_of(world, listener, this);
    if let Some(moron_snake) = moron_snake {
        moron_snake_logic::thrown_by_baby(world, moron_snake, this);
    }
}

/// `-[PygmyLogic (Island) eatenByMoronSnakeComplete:::]` @ 0x1000b1bd0
///
/// Returns 2 even when it has replaced the sequence, like the original.
pub fn seq_eaten_by_moron_snake_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let eaten_by_moron_snake = world.obj::<PygmyPygmyCoreFrame>(this).eaten_by_moron_snake;
    if eaten_by_moron_snake {
        set_sacrifice_id(world, this, "EatenByMoronSnake");
        pg_engine::logic::set_behavior(world, this, Some("PygmyEatenByMoronSnakeComplete"));
    }

    let listener = listener_of(world, this);
    let moron_snake = moron_snake_logic_of(world, listener, this);
    if let Some(moron_snake) = moron_snake {
        moron_snake_logic::eat_pygmy_complete(world, moron_snake, this);
    }

    let moron_pests_enabled = match listener {
        Some(listener) => crate::controller::is_moron_pests_enabled(world, listener),
        None => false,
    };
    if moron_pests_enabled {
        if let Some(moron_snake) = moron_snake {
            moron_snake_logic::start_moron_pests(world, moron_snake, this);
        }
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) grabbedByTentacle:]` @ 0x1000b6aac
pub fn grabbed_by_tentacle(world: &mut World, this: Entity, _ice_hole: Entity) {
    let dragging = world.obj::<PygmyPygmyCoreFrame>(this).dragging;
    if !dragging {
        return;
    }

    world.obj_mut::<PygmyIslandHazards>(this).grabbed_by_tentacle = true;
    pg_engine::logic::set_behavior(world, this, Some("PygmyGrabbedByTentacle"));
}

/// `-[PygmyLogic (Island) canBeGrabbedByTentacle:]` @ 0x1000b6af0
pub fn can_be_grabbed_by_tentacle(world: &mut World, this: Entity, _ice_hole: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);

    core.dragging
        && !core.beheaded_by_squid
        && !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == 0
        && !core.sun_frozen
}

/// `-[PygmyLogic (Island) eatenByIceSquid:]` @ 0x1000b6b8c
pub fn eaten_by_ice_squid(world: &mut World, this: Entity, _ice_hole: Entity) {
    set_sacrifice_id(world, this, "SquidBait");
    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Island) tugOfWarWithSquid:::]` @ 0x1000b6bbc
///
/// The dragged pygmy hangs between the finger and the tentacle on a spring; the squid pulls
/// with a strength that cycles every 1000 units. Leaning more than 60° or straying more than
/// 180 from the tentacle costs the pygmy its head.
pub fn seq_tug_of_war_with_squid(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.enable_island_floor_collision = false;
        core.enable_island_side_collision = false;

        let strength_rot_vel = random_tentacle_strength_rot_vel(world);
        world.obj_mut::<PygmyIslandHazards>(this).tentacle_strength_rot_vel = strength_rot_vel;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;
    let touch_screen_pos = world.obj::<PygmyPygmyTouchDrag>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    // The touch point takes the pygmy's own depth, read here.
    let touch_z_pos = display::object(world, display_object).pos.z;

    let listener = listener_of(world, this);
    let ice_hole = match listener {
        Some(listener) => crate::controller::ice_hole_logic(world, listener, this),
        None => None,
    };

    let tentacle_pos = ice_hole_tentacle_pos(world, ice_hole);
    let pos = display::object(world, display_object).pos;
    let to_tentacle_x = tentacle_pos.x - pos.x;
    let to_tentacle_y = tentacle_pos.y - pos.y;

    let angle_radians = to_tentacle_y.atan2(to_tentacle_x);
    let z_rot = pg_engine::math::rad_to_deg(angle_radians) + 90.0;
    display::object_mut(world, display_object).rot.z = z_rot;

    let tentacle_distance = (to_tentacle_x * to_tentacle_x + to_tentacle_y * to_tentacle_y).sqrt();
    if z_rot > 60.0 || z_rot < -60.0 || tentacle_distance > 180.0 {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).beheaded_by_squid = true;

        if let Some(ice_hole) = ice_hole {
            ice_hole_logic::behead_pygmy(world, ice_hole, this);
        }

        pg_engine::logic::set_behavior(world, this, Some("PygmyHeadRippedOffByIceMonster"));
        return ItemResult::Replaced;
    }

    let pos = display::object(world, display_object).pos;
    let from_touch_x = pos.x - touch_pos.x;
    let from_touch_y = pos.y - touch_pos.y;
    let from_touch_z = pos.z - touch_z_pos;
    let from_touch = Vec4::new(from_touch_x, from_touch_y, from_touch_z, pos.w);
    let from_touch_length = pg_engine::math::length3(from_touch);
    if from_touch_length == 0.0 {
        return ItemResult::Pending;
    }

    let pos_vel = pg_engine::logic::logic(world, this).pos_vel;
    let finger_strength = (from_touch_length * -100.0) / from_touch_length;
    let held_x = -8.0 * pos_vel.x + finger_strength * from_touch_x;
    let held_y = -8.0 * pos_vel.y + finger_strength * from_touch_y;
    let held_z = -8.0 * pos_vel.z + finger_strength * from_touch_z;

    let mut hazards = world.obj_mut::<PygmyIslandHazards>(this);
    hazards.tentacle_strength_rot += hazards.tentacle_strength_rot_vel;

    if hazards.tentacle_strength_rot >= 1000.0 {
        hazards.tentacle_strength_rot -= 1000.0;

        let strength_rot_vel = random_tentacle_strength_rot_vel(world);
        world.obj_mut::<PygmyIslandHazards>(this).tentacle_strength_rot_vel = strength_rot_vel;
    }

    let strength_rot = world.obj::<PygmyIslandHazards>(this).tentacle_strength_rot;
    let tentacle_strength = if strength_rot < 700.0 { 25.0 } else { 10.0 };

    let spring_x = held_x + to_tentacle_x * tentacle_strength;
    let spring_y = held_y + to_tentacle_y * tentacle_strength;
    let spring_force = Vec4::new(spring_x, spring_y, held_z, pos.w);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).spring_force = spring_force;

    ItemResult::Pending
}

/// `-[PygmyLogic (Island) onFallIntoCrackHoleComplete:::]` @ 0x1000b6f70
pub fn seq_on_fall_into_crack_hole_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    if let Some(listener) = listener {
        crate::controller::on_pygmy_fall_into_crack_hole_complete(world, listener, this);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) checkStayAwayFromTRex:::]` @ 0x1000b8d18
pub fn seq_check_stay_away_from_t_rex(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    check_stay_away(world, this, &T_REX)
}

/// `-[PygmyLogic (Island) runUnderTRex:::]` @ 0x1000b8eb0
pub fn seq_run_under_t_rex(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    run_under(world, this, call, &T_REX)
}

/// `-[PygmyLogic (Island) checkForTRex:::]` @ 0x1000b9078
pub fn seq_check_for_t_rex(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    check_for_monster(world, this, &T_REX)
}

/// `-[PygmyLogic (Island) checkStayAwayFromSpider:::]` @ 0x1000b9360
pub fn seq_check_stay_away_from_spider(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    check_stay_away(world, this, &SPIDER)
}

/// `-[PygmyLogic (Island) runUnderSpider:::]` @ 0x1000b94f8
pub fn seq_run_under_spider(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    run_under(world, this, call, &SPIDER)
}

/// `-[PygmyLogic (Island) checkForSpider:::]` @ 0x1000b96c0
pub fn seq_check_for_spider(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    check_for_monster(world, this, &SPIDER)
}

/// `-[PygmyLogic (Island) checkForSunState:::]` @ 0x1000b9c28
pub fn seq_check_for_sun_state(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    if is_inside_apocalypse_temple_of(world, listener) {
        return ItemResult::Done;
    }

    let sun_state = sun_state_of(world, listener);
    if sun_state != 0 {
        stop_activities(world, this);

        let cycle_count = random_sun_cycle_count(world);
        world.obj_mut::<PygmyIslandHazards>(this).sun_burn_freeze_cycle_count = cycle_count;

        if sun_state == 2 {
            pg_engine::logic::set_behavior(world, this, Some("PygmySunFreeze"));
        } else if sun_state == 1 {
            pg_engine::logic::set_behavior(world, this, Some("PygmySunBurn"));
        }
    }

    let keep_looping = call.bool("loop");
    if keep_looping {
        ItemResult::Pending
    } else {
        ItemResult::Done
    }
}

/// `-[PygmyLogic (Island) checkForGingerSunBurnt:::]` @ 0x1000b9e38
pub fn seq_check_for_ginger_sun_burnt(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let sheltered = match listener {
        Some(listener) => {
            crate::controller::is_sunset_active(world, listener)
                || crate::controller::is_night_active(world, listener)
                || crate::controller::is_storm_cloud_enabled(world, listener)
                || crate::controller::is_inside_apocalypse_temple(world, listener)
        }
        None => false,
    };

    if !sheltered {
        world.obj_mut::<PygmyIslandHazards>(this).ginger_sun_burn = true;
        set_sacrifice_id(world, this, "GingerPygmySunBurn");
        pg_engine::logic::set_behavior(world, this, Some("TempleGingerPygmySunBurnt"));
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) checkForSunBurnt:::]` @ 0x1000b9f0c
pub fn seq_check_for_sun_burnt(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    if is_inside_apocalypse_temple_of(world, listener) {
        return ItemResult::Done;
    }

    let sun_state = sun_state_of(world, listener);
    let behavior = match sun_state {
        2 => {
            let cycle_count = random_sun_cycle_count(world);
            world.obj_mut::<PygmyIslandHazards>(this).sun_burn_freeze_cycle_count = cycle_count;
            "PygmySunFreeze"
        }
        1 => {
            // While the frozen hole calls the pygmy, it only burns standing on the hole (the
            // original's `hi` and `lt` tests count a NaN position as on it).
            let display_object = pg_engine::logic::display_object(world, this);
            let ice_hole_calling = is_ice_hole_calling(world, this, listener);
            let x_pos = display::object(world, display_object).pos.x;
            let off_ice_hole = x_pos <= ICE_HOLE_MIN_X_POS || x_pos >= ICE_HOLE_MAX_X_POS;
            let can_burn_here = !ice_hole_calling || !off_ice_hole;

            let mut hazards = world.obj_mut::<PygmyIslandHazards>(this);
            let cycle_count = hazards.sun_burn_freeze_cycle_count;
            hazards.sun_burn_freeze_cycle_count = cycle_count.wrapping_sub(1);

            if cycle_count > 1 || !can_burn_here {
                return ItemResult::Done;
            }

            world.obj_mut::<PygmyPygmyCoreFrame>(this).sun_burnt = true;
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
            "PygmySunBurnt"
        }
        0 => "PygmyIdle",
        _ => return ItemResult::Replaced,
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `-[PygmyLogic (Island) checkForSunFrozen:::]` @ 0x1000ba108
pub fn seq_check_for_sun_frozen(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    if is_inside_apocalypse_temple_of(world, listener) {
        return ItemResult::Done;
    }

    let sun_state = sun_state_of(world, listener);
    let behavior = match sun_state {
        2 => {
            let mut hazards = world.obj_mut::<PygmyIslandHazards>(this);
            let cycle_count = hazards.sun_burn_freeze_cycle_count;
            hazards.sun_burn_freeze_cycle_count = cycle_count.wrapping_sub(1);

            if cycle_count > 1 {
                return ItemResult::Done;
            }

            world.obj_mut::<PygmyPygmyCoreFrame>(this).sun_frozen = true;
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
            "PygmySunFrozen"
        }
        1 => {
            let cycle_count = random_sun_cycle_count(world);
            world.obj_mut::<PygmyIslandHazards>(this).sun_burn_freeze_cycle_count = cycle_count;
            "PygmySunBurn"
        }
        0 => "PygmyIdle",
        _ => return ItemResult::Replaced,
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `-[PygmyLogic (Island) iceShatter]` @ 0x1000ba23c
pub fn ice_shatter(world: &mut World, this: Entity) {
    let sun_frozen = world.obj::<PygmyPygmyCoreFrame>(this).sun_frozen;
    let already_shattering = world.obj::<PygmyIslandHazards>(this).ice_shatter;

    if !sun_frozen || already_shattering {
        return;
    }

    clear_xy_vel(world, this);
    world.obj_mut::<PygmyIslandHazards>(this).ice_shatter = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    set_sacrifice_id(world, this, "Shattered");

    pg_engine::logic::set_behavior(world, this, Some("PygmyIceShatter"));
}

/// `-[PygmyLogic (Island) explodeFromLovingPocketGod:::]` @ 0x1000ba66c
pub fn seq_explode_from_loving_pocket_god(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    // S14.
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

    // The hurricane forgets the pygmy without calling back `droppedByHurricane:`.
    let hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane_logic;
    if let Some(hurricane) = hurricane {
        hurricane_logic::drop_sprite_logic(world, hurricane, this);

        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_hurricane_logic = None;
        sprite.picked_up_by_hurricane = false;
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.explode_from_loving_pocket_god = true;
    core.tilt_state = 0;

    clear_xy_vel(world, this);
    clear_z_rot(world, this);
    set_sacrifice_id(world, this, "LovingPocketGod");

    seq_explode(world, this, call)
}

/// `-[PygmyLogic (Island) explodeFromNotCrapping:::]` @ 0x1000ba894
pub fn seq_explode_from_not_crapping(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    set_sacrifice_id(world, this, "NotCrapping");
    world.obj_mut::<PygmyPygmyCoreFrame>(this).explode_from_not_crapping = true;

    seq_explode(world, this, call)
}

/// `-[PygmyLogic (Island) sunBurntComplete:::]` @ 0x1000bc9e0
///
/// Returns 2 even when it has replaced the sequence, like the original.
pub fn seq_sun_burnt_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    set_sacrifice_id(world, this, "GlobalWarming");

    let listener = listener_of(world, this);
    let location = match listener {
        Some(listener) => crate::controller::island_location_state(world, listener),
        None => 0,
    };

    if location != ICE_LOCATION {
        pg_engine::logic::set_behavior(world, this, Some("PygmySunBurntExplode"));
        return ItemResult::Done;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let (ice_hole, ice_hole_enabled) = match listener {
        Some(listener) => {
            let ice_hole = crate::controller::ice_hole_logic(world, listener, this);
            let ice_hole_enabled = crate::controller::is_ice_hole_enabled(world, listener);
            (ice_hole, ice_hole_enabled)
        }
        None => (None, false),
    };

    // A burning pygmy standing on the frozen hole melts it and falls in.
    let ice_hole_frozen = match ice_hole {
        Some(ice_hole) if ice_hole_enabled => ice_hole_logic::is_frozen(world, ice_hole),
        _ => false,
    };
    let x_pos = display::object(world, display_object).pos.x;
    let on_ice_hole = ICE_HOLE_MIN_X_POS <= x_pos && x_pos <= ICE_HOLE_MAX_X_POS;

    match ice_hole {
        Some(ice_hole) if ice_hole_frozen && on_ice_hole => {
            world.obj_mut::<PygmyIslandHazards>(this).ice_hole_logic = Some(ice_hole);
            ice_hole_logic::melt_ice_hole(world, ice_hole, this);
        }
        _ => {
            pg_engine::logic::set_behavior(world, this, Some("PygmySunBurntCreateSoloIceHole"));
        }
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) canMeltIceHole]` @ 0x1000bcb38
///
/// The original answers YES when the pygmy is busy (FREE +{mbSharkOnHead}, tilted) and NO
/// when it is free; `IceHoleLogic` consumes the inverted answer as is.
pub fn can_melt_ice_hole(world: &mut World, this: Entity) -> bool {
    is_busy(world, this)
}

/// `-[PygmyLogic (Island) meltIceHoleComplete:]` @ 0x1000bd1ec
pub fn melt_ice_hole_complete(world: &mut World, this: Entity, _ice_hole: Entity) {
    world.obj_mut::<PygmyIslandHazards>(this).ice_hole_logic = None;
    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Island) sunWalk:::]` @ 0x1000bd214
///
/// A timed walk at 50 under the burning or freezing sun. While the frozen ice hole calls the
/// pygmy (sun burning), it starts toward the hole and stops when it walks out of the hole's
/// range.
pub fn seq_sun_walk(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let sun_state = sun_state_of(world, listener);
    let toward_ice_hole = if sun_state == 1 {
        is_ice_hole_calling(world, this, listener)
    } else {
        false
    };

    let display_object = pg_engine::logic::display_object(world, this);

    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;

        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_side_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

        let coin = world.resource_mut::<Random>().arc4random() & 1;
        let x_pos = display::object(world, display_object).pos.x;

        // NaN positions are left to the coin.
        let goes_right = if !toward_ice_hole {
            coin != 0
        } else if x_pos <= ICE_HOLE_MIN_X_POS {
            true
        } else if x_pos >= ICE_HOLE_MAX_X_POS {
            false
        } else {
            coin != 0
        };

        let (x_scale, x_vel) = if goes_right { (1.0, 50.0) } else { (-1.0, -50.0) };
        display::object_mut(world, display_object).scale.x = x_scale;
        pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;
    }

    let island_side_collision = world.obj::<PygmyPygmyCoreFrame>(this).island_side_collision;
    let mut result = if island_side_collision {
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    if toward_ice_hole {
        let x_pos = display::object(world, display_object).pos.x;
        let x_vel = pg_engine::logic::logic(world, this).pos_vel.x;
        let leaving_left = x_pos <= ICE_HOLE_MIN_X_POS && x_vel < 0.0;
        let leaving_right = x_pos >= ICE_HOLE_MAX_X_POS && x_vel > 0.0;

        if leaving_left || leaving_right {
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
            result = ItemResult::Done;
        }
    }

    let stall_time = pg_engine::logic::logic(world, this).stall_time;
    if stall_time <= 0.0 {
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        result = ItemResult::Done;
    }

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.stall_time -= call.delta_time;
    result
}

/// `-[PygmyLogic (Island) iglooRoll:::]` @ 0x1000be498
///
/// The pygmy shot out of the igloo rolls along the island, knocking over every pygmy within 40,
/// and falls off past the island's end.
pub fn seq_igloo_roll(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);

    if call.first_time {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.enable_island_side_collision = false;
        core.enable_island_floor_collision = false;

        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    }

    let listener = listener_of(world, this);
    let pygmies = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };

    for pygmy in pygmies {
        if pygmy == this || !can_be_hit_by_pygmy_igloo_roll(world, pygmy, this) {
            continue;
        }

        let x_pos = display::object(world, display_object).pos.x;
        let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
        let pygmy_x_pos = display::object(world, pygmy_display_object).pos.x;
        let distance = (x_pos - pygmy_x_pos).abs();

        if distance < 40.0 {
            hit_by_pygmy_igloo_roll(world, pygmy, this);
        }
    }

    let x_pos = display::object(world, display_object).pos.x;
    let island_max_x_pos = match listener {
        Some(listener) => crate::controller::island_max_x_pos(world, listener),
        None => 0.0,
    };

    if x_pos > island_max_x_pos {
        pg_engine::logic::logic_mut(world, this).pos_vel.y = 500.0;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

        pg_engine::logic::set_behavior(world, this, Some("PygmyFallIgnoreIsland"));
        return ItemResult::Replaced;
    }

    ItemResult::Pending
}

/// `-[PygmyLogic (Island) uncrushed:::]` @ 0x1000be7f0
pub fn seq_uncrushed(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).crushed = false;
    ItemResult::Done
}
