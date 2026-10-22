//! `PygmyLogic`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use crate::logic::apocalypse_locust_closet_xenu::closet_temple_closet_door_logic;
use crate::logic::island_core::bon_fire_logic;
use crate::logic::island_core::sprite_logic;
use crate::logic::island_core::sprite_logic::SpriteLogic;
use crate::logic_table;
use crate::pygmy_extras::pygmy_bottom_logic;

/// `mTimeTemplePygmyType` of an ordinary pygmy.
const PYGMY_TYPE_NORMAL: i32 = 0;

/// `SpriteLogic::mApocalypseDiseaseType` of the plague.
const DISEASE_PLAGUE: i32 = 1;

/// The animation channel a carried plague body is bound to.
const PLAGUE_BODY_CHANNEL: usize = 6;

/// `mPygmyLogicListener`.
fn listener_of(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener
}

/// `[mPygmyLogicListener pygmyLogicArray:self]` (empty for a nil listener).
fn pygmy_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    match listener_of(world, this) {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    }
}

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

/// `[mDisplayObject setXScale:]`.
fn set_x_scale(world: &mut World, this: Entity, x_scale: f32) {
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).scale.x = x_scale;
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

/// `mSacrificeId = @"…"`.
fn set_sacrifice_id(world: &mut World, this: Entity, sacrifice_id: &str) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some(sacrifice_id.to_string());
}

/// `[[IdolManager instance] triggerEpisodeEvent:]`.
fn trigger_episode(world: &mut World, event: &str) {
    let idol_manager = crate::managers::idol_manager::instance(world);

    if let Some(idol_manager) = idol_manager {
        crate::managers::idol_manager::trigger_episode_event(world, idol_manager, event);
    }
}

/// `[marker insertAfter:mDisplayObject]` (nothing for a nil marker), then
/// `[self updateLabelLayer]`.
fn move_after_marker(world: &mut World, this: Entity, marker: Option<Entity>) {
    let display = pg_engine::logic::display_object(world, this);

    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display));
    }

    update_label_layer(world, this);
}

/// The guard every check of this part shares: the busy set FREE (`docs/re/pygmy.md` §1.3) plus
/// `mbSharkOnHead` and a non-zero tilt state, without `mbFalling` and the pygmy type, which the
/// callers test themselves.
fn is_busy(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let sprite = world.obj::<SpriteLogic>(this);

    core.touch_began_inside
        || core.remove_from_story_mode
        || core.tilt_state != 0
        || core.cut_in_half_by_charlie
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
}

/// The guard of the locust kills and the dance balloon: [`is_busy`] clear, standing, and an
/// ordinary pygmy.
fn is_free_standing(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let standing = !core.falling && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL;

    standing && !is_busy(world, this)
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

/// The stop list S31 of the infection (`docs/re/pygmy.md` §1.3).
fn stop_activities_for_infection(world: &mut World, this: Entity) {
    stop_sleeping(world, this);
    stop_being_eaten(world, this);
    stop_being_eaten_by_piranha(world, this);
    stop_holding_chum(world, this);
    stop_being_high_on_nitrous(world, this);
    stop_eating(world, this);
    stop_carrying(world, this);
    stop_carrying_plague_body(world, this);
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
    stop_being_eaten_by_locust_queen(world, this);
    stop_being_thrown_by_locusts(world, this);
    stop_warming_by_bon_fire(world, this);
    stop_staring_at_rainbow(world, this);
    stop_staring_at_kidnapping(world, this);
    stop_fishing(world, this);
    stop_waiting_to_crap(world, this);
    stop_playing_catch_with_monkey(world, this);
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);
}

/// `-[PygmyLogic (Apocalypse) checkMutantLocustFlyOff:::]` @ 0x100042e44
pub fn seq_check_mutant_locust_fly_off(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut apocalypse = world.obj_mut::<PygmyApocalypse>(this);
    let loop_count = apocalypse.mutant_locust_idle_loop_count.wrapping_add(1);
    apocalypse.mutant_locust_idle_loop_count = loop_count;

    if loop_count == 3 {
        let display = pg_engine::logic::display_object(world, this);
        let x_scale = pg_engine::display::object(world, display).scale.x;

        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = x_scale * 100.0;
        logic.pos_vel.y = 100.0;

        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.enable_island_floor_collision = false;
        core.enable_island_side_collision = false;

        pg_engine::logic::set_behavior(world, this, Some("PygmyMutatedByLocustsFlyOff"));
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Apocalypse) waitForMutantLocustFlyOffComplete:::]` @ 0x100042f20
pub fn seq_wait_for_mutant_locust_fly_off_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display = pg_engine::logic::display_object(world, this);
    let min_pos = pg_engine::display::min_pos(world, display);
    let max_pos = pg_engine::display::max_pos(world, display);

    let off_screen = max_pos.x < 0.0 || 480.0 < min_pos.x || 580.0 < min_pos.y;

    if off_screen { ItemResult::Done } else { ItemResult::Pending }
}

/// `-[PygmyLogic (Apocalypse) mouthFilledByLocustsComplete:::]` @ 0x100042fbc
pub fn seq_mouth_filled_by_locusts_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    set_selectable(world, this, true);

    ItemResult::Done
}

/// `-[PygmyLogic (Apocalypse) cutInAirByLocusts:]` @ 0x100042ff4
pub fn cut_in_air_by_locusts(world: &mut World, this: Entity, _locust: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let flying = core.falling && core.time_temple_pygmy_type == PYGMY_TYPE_NORMAL;

    if !flying || is_busy(world, this) {
        return false;
    }

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 100.0;
    logic.pos_vel.y = 700.0;

    let bottom = pg_engine::library::logic(world, logic_table::PYGMY_BOTTOM);
    pygmy_bottom_logic::toss_bottom(world, bottom, this);

    world.obj_mut::<PygmyGhostZombieVampire>(this).bit_by_bat = false;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.falling = true;
    core.enable_island_floor_collision = false;
    core.enable_island_side_collision = false;

    set_selectable(world, this, false);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.invalid_sacrifice = false;
    core.killed_by_locusts = true;

    set_sacrifice_id(world, this, "SlicedByLocust");
    pg_engine::logic::set_behavior(world, this, Some("PygmyCutInAirByLocusts"));

    let listener = listener_of(world, this);
    let marker = listener.and_then(|listener| crate::controller::pygmy_drag_display_marker(world, listener));
    move_after_marker(world, this, marker);

    true
}

/// `-[PygmyLogic (Apocalypse) stripFleshByLocusts:]` @ 0x100043814
pub fn strip_flesh_by_locusts(world: &mut World, this: Entity, _locust: Entity) -> bool {
    if !is_free_standing(world, this) {
        return false;
    }

    stop_activities(world, this);
    stop_moving(world, this);

    world.obj_mut::<PygmyGhostZombieVampire>(this).bit_by_bat = false;

    set_selectable(world, this, false);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).killed_by_locusts = true;
    set_sacrifice_id(world, this, "LocustFood");

    pg_engine::logic::set_behavior(world, this, Some("PygmyStripFleshByLocusts"));

    true
}

/// `-[PygmyLogic (Apocalypse) animSpawnSkullAndBone]` @ 0x100044068
///
/// Takes the first free skull and bone of the three pairs; when all three are in use, the
/// first pair is spawned again.
pub fn anim_spawn_skull_and_bone(world: &mut World, this: Entity) {
    let mut skull = pg_engine::library::logic(world, logic_table::LOCUST_PYGMY_SKULL1);
    let mut bone = pg_engine::library::logic(world, logic_table::LOCUST_PYGMY_BONE1);

    if pygmy_bottom_logic::is_active(world, skull) {
        skull = pg_engine::library::logic(world, logic_table::LOCUST_PYGMY_SKULL2);
        bone = pg_engine::library::logic(world, logic_table::LOCUST_PYGMY_BONE2);
    }

    if pygmy_bottom_logic::is_active(world, skull) {
        skull = pg_engine::library::logic(world, logic_table::LOCUST_PYGMY_SKULL3);
        bone = pg_engine::library::logic(world, logic_table::LOCUST_PYGMY_BONE3);
    }

    if pygmy_bottom_logic::is_active(world, skull) {
        skull = pg_engine::library::logic(world, logic_table::LOCUST_PYGMY_SKULL1);
        bone = pg_engine::library::logic(world, logic_table::LOCUST_PYGMY_BONE1);
    }

    pygmy_bottom_logic::spawn_locust_skull(world, skull, this);
    pygmy_bottom_logic::spawn_locust_bone(world, bone, this);
}

/// `-[PygmyLogic (Apocalypse) mouthFilledByLocusts:]` @ 0x100044200
pub fn mouth_filled_by_locusts(world: &mut World, this: Entity, _locust: Entity) -> bool {
    if !is_free_standing(world, this) {
        return false;
    }

    stop_activities(world, this);
    stop_moving(world, this);
    set_selectable(world, this, false);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).killed_by_locusts = true;
    world.obj_mut::<PygmyPygmyBehaviourItems>(this).mouth_filled_by_locusts = true;
    set_sacrifice_id(world, this, "PopLocustBalloon");

    pg_engine::logic::set_behavior(world, this, Some("PygmyMouthFilledByLocusts"));

    true
}

/// `-[PygmyLogic (Apocalypse) mutatedByLocusts:]` @ 0x100044a50
pub fn mutated_by_locusts(world: &mut World, this: Entity, _locust: Entity) -> bool {
    if !is_free_standing(world, this) {
        return false;
    }

    stop_activities(world, this);
    stop_moving(world, this);

    world.obj_mut::<PygmyApocalypse>(this).mutant_locust_idle_loop_count = 0;

    set_selectable(world, this, false);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.killed_by_locusts = true;
    core.mutated_by_locusts = true;

    set_sacrifice_id(world, this, "TurnedIntoMutantLocust");
    pg_engine::logic::set_behavior(world, this, Some("PygmyMutatedByLocusts"));

    true
}

/// `-[PygmyLogic (Apocalypse) inhaleDanceBalloon:]` @ 0x1000452b4
pub fn inhale_dance_balloon(world: &mut World, this: Entity, _dance_balloon: Entity) -> bool {
    if !is_free_standing(world, this) {
        return false;
    }

    stop_activities(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).high_on_nitrous = true;
    stop_moving(world, this);

    pg_engine::logic::set_behavior(world, this, Some("PygmyInhaleDanceBalloon"));

    true
}

/// `-[PygmyLogic (Apocalypse) canBeInfected:]` @ 0x100045b54
pub fn can_be_infected(world: &mut World, this: Entity, _source: Entity) -> bool {
    let falling = world.obj::<PygmyPygmyCoreFrame>(this).falling;

    !falling && !is_busy(world, this)
}

/// `-[PygmyLogic (Apocalypse) infectWithDisease:recoilFromPygmyLogic:]` @ 0x1000461f4
///
/// `source` is the sick pygmy the disease comes from.
pub fn infect_with_disease(world: &mut World, this: Entity, disease: i32, source: Entity) {
    let own_disease = world.obj::<SpriteLogic>(this).apocalypse_disease_type;

    if disease != 0 && own_disease != disease {
        // The original inlines the body of `recoilFromPygmyDisease:` here.
        recoil_from_pygmy_disease(world, this, source);

        let mut apocalypse = world.obj_mut::<PygmyApocalypse>(this);
        if apocalypse.disease_incubate_idle_cycles < 5 {
            apocalypse.disease_incubate_idle_cycles = 5;
        }
    }

    sprite_logic::infect_with_disease(world, this, disease);
}

/// `-[PygmyLogic (Apocalypse) recoilFromPygmyDisease:]` @ 0x10004658c
///
/// Turns away from `source`, the sick pygmy.
pub fn recoil_from_pygmy_disease(world: &mut World, this: Entity, source: Entity) {
    stop_activities_for_infection(world, this);

    let x = display_pos(world, this).x;
    let source_x = display_pos(world, source).x;
    let x_scale = if source_x < x { -1.0 } else { 1.0 };

    set_x_scale(world, this, x_scale);
    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;

    pg_engine::logic::set_behavior(world, this, Some("PygmyRecoilFromDisease"));
}

/// `-[PygmyLogic (Apocalypse) animStartPygmyRecoil]` @ 0x1000468c8
pub fn anim_start_pygmy_recoil(world: &mut World, this: Entity) {
    let pygmies = pygmy_logic_array(world, this);

    for other in pygmies {
        if other == this {
            continue;
        }

        let infectable = can_be_infected(world, other, this);
        if !infectable {
            continue;
        }

        let x = display_pos(world, this).x;
        let other_x = display_pos(world, other).x;
        let near = (x - other_x).abs() < 100.0;

        if near {
            recoil_from_pygmy_disease(world, other, this);
        }
    }
}

/// `-[PygmyLogic (Apocalypse) animInfectPygmies]` @ 0x100046aac
pub fn anim_infect_pygmies(world: &mut World, this: Entity) {
    let pygmies = pygmy_logic_array(world, this);

    for other in pygmies {
        if other == this {
            continue;
        }

        let infectable = can_be_infected(world, other, this);
        if !infectable {
            continue;
        }

        let display = pg_engine::logic::display_object(world, this);
        let object = pg_engine::display::object(world, display);
        let breath_x = object.pos.x + object.scale.x * 40.0;
        let other_x = display_pos(world, other).x;
        let near = (breath_x - other_x).abs() < 60.0;

        if !near {
            continue;
        }

        let disease = world.obj::<SpriteLogic>(this).apocalypse_disease_type;
        let event = if disease == DISEASE_PLAGUE {
            "infectPygmyWithGerms"
        } else {
            "infectPygmyWithPureHell"
        };

        trigger_episode(world, event);

        // The disease is read again after the episode event.
        let disease = world.obj::<SpriteLogic>(this).apocalypse_disease_type;
        infect_with_disease(world, other, disease, this);
    }
}

/// `-[PygmyLogic (Apocalypse) explodeFromThetanBirth]` @ 0x100046d60
pub fn explode_from_thetan_birth(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).disable_labels = true;
    set_sacrifice_id(world, this, "ExplodeFromThetanBirth");

    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Apocalypse) closetIdle:::]` @ 0x100047290
pub fn seq_closet_idle(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");

        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);

        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;
    }

    let mut logic = pg_engine::logic::logic_mut(world, this);
    if logic.stall_time <= 0.0 {
        return ItemResult::Done;
    }

    logic.stall_time -= call.delta_time;

    ItemResult::Pending
}

/// `-[PygmyLogic (Apocalypse) closetWalk:::]` @ 0x100047394
pub fn seq_closet_walk(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display = pg_engine::logic::display_object(world, this);
    let listener = listener_of(world, this);
    let door = listener.and_then(|listener| crate::controller::closet_temple_closet_door_logic(world, listener, this));

    let min_closet_x = door.map_or(0.0, |door| closet_temple_closet_door_logic::min_closet_x_pos(world, door));
    let max_closet_x = door.map_or(0.0, |door| closet_temple_closet_door_logic::max_closet_x_pos(world, door));

    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");

        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);

        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;

        let coin = world.resource_mut::<Random>().arc4random();
        let x = pg_engine::display::object(world, display).pos.x;

        let go_left = (min_closet_x < x && coin & 1 == 0) || max_closet_x <= x;
        let (x_scale, x_vel) = if go_left { (-1.0, -50.0) } else { (1.0, 50.0) };

        pg_engine::display::object_mut(world, display).scale.x = x_scale;
        pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;
    }

    let mut result = ItemResult::Pending;
    let mut x = pg_engine::display::object(world, display).pos.x;

    if x < min_closet_x {
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        pg_engine::display::object_mut(world, display).pos.x = min_closet_x;

        result = ItemResult::Done;
        x = min_closet_x;
    }

    if max_closet_x < x {
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        pg_engine::display::object_mut(world, display).pos.x = max_closet_x;

        result = ItemResult::Done;
    }

    let stall_time = pg_engine::logic::logic(world, this).stall_time;
    if stall_time <= 0.0 {
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        result = ItemResult::Done;
    }

    pg_engine::logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;

    result
}

/// `-[PygmyLogic (Apocalypse) respawnPlagueBody:::]` @ 0x1000475f8
pub fn seq_respawn_plague_body(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let min_x = listener.map_or(0.0, |listener| crate::controller::island_min_x_pos(world, listener));
    let max_x = listener.map_or(0.0, |listener| crate::controller::island_max_x_pos(world, listener));

    let random = world.resource_mut::<Random>().arc4random();
    let x = pg_engine::logic::random_scaled_time(min_x + 15.0, max_x + -15.0, 10.0, random);

    let display = pg_engine::logic::display_object(world, this);
    let mut object = pg_engine::display::object_mut(world, display);
    object.pos.x = x;
    object.pos.y = 340.0;
    object.pos.z = -277.12;
    object.scale.x = 1.0;
    object.scale.y = 1.0;

    stop_moving(world, this);
    clear_z_rot(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.tilt_state = 0;
    core.drowning = false;

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_side_collision = false;
    core.enable_island_floor_collision = true;

    let dropped_by_hurricane = core.dropped_by_hurricane;
    if dropped_by_hurricane {
        core.dropped_by_hurricane = false;

        set_selectable(world, this, true);

        let marker = listener.and_then(|listener| crate::controller::pygmy_display_marker(world, listener));
        move_after_marker(world, this, marker);
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).invalid_sacrifice = true;

    ItemResult::Done
}

/// `-[PygmyLogic (Apocalypse) animStartFloat]` @ 0x100047810
pub fn anim_start_float(world: &mut World, this: Entity) {
    pg_engine::logic::logic_mut(world, this).pos_vel.x = 20.0;
}

/// `-[PygmyLogic (Apocalypse) plagueBodyFloat:::]` @ 0x100047828
///
/// Wraps the floating body around the screen edges; never completes.
pub fn seq_plague_body_float(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.enable_island_side_collision = false;
        core.enable_island_floor_collision = false;
    }

    let display = pg_engine::logic::display_object(world, this);
    let min_pos = pg_engine::display::min_pos(world, display);
    let max_pos = pg_engine::display::max_pos(world, display);

    if max_pos.x < 0.0 {
        let x = pg_engine::display::object(world, display).pos.x;
        pg_engine::display::object_mut(world, display).pos.x = (x - min_pos.x) + 480.0;
    }

    if 480.0 < min_pos.x {
        let x = pg_engine::display::object(world, display).pos.x;
        pg_engine::display::object_mut(world, display).pos.x = 0.0 - (max_pos.x - x);
    }

    ItemResult::Pending
}

/// `-[PygmyLogic (Apocalypse) animThrowPlagueBody]` @ 0x100047918
pub fn anim_throw_plague_body(world: &mut World, this: Entity) {
    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, PLAGUE_BODY_CHANNEL);

    let body = world.obj::<PygmyApocalypse>(this).carrying_plague_body_logic;
    if let Some(body) = body {
        throw_plague_body_on_pile(world, body);
    }

    world.obj_mut::<PygmyApocalypse>(this).carrying_plague_body_logic = None;
}

/// `-[PygmyLogic (Apocalypse) pickupPlagueBody]` @ 0x10004797c
pub fn pickup_plague_body(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyApocalypse>(this).plague_body_picked_up = true;
}

/// `-[PygmyLogic (Apocalypse) isPlagueBodyPickedUp]` @ 0x100047994
pub fn is_plague_body_picked_up(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyApocalypse>(this).plague_body_picked_up
}

/// `-[PygmyLogic (Apocalypse) isKilledByPlague]` @ 0x1000479a8
pub fn is_killed_by_plague(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyPygmyCoreFrame>(this).killed_by_plague
}

/// `-[PygmyLogic (Apocalypse) plagueBodyCarryingPygmyLogic]` @ 0x1000479bc
pub fn plague_body_carrying_pygmy_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<PygmyApocalypse>(this).plague_body_carrying_pygmy_logic
}

/// `-[PygmyLogic (Apocalypse) animPickupPlagueBody]` @ 0x1000479d0
pub fn anim_pickup_plague_body(world: &mut World, this: Entity) {
    let body = world.obj::<PygmyApocalypse>(this).carrying_plague_body_logic;

    if let Some(body) = body {
        pickup_plague_body(world, body);
    }
}

/// `-[PygmyLogic (Apocalypse) dropPlagueBodyWhileCarrying]` @ 0x1000479f0
pub fn drop_plague_body_while_carrying(world: &mut World, this: Entity) {
    let mut apocalypse = world.obj_mut::<PygmyApocalypse>(this);
    apocalypse.plague_body_picked_up = false;
    apocalypse.plague_body_being_carried = false;
    apocalypse.plague_body_carrying_pygmy_logic = None;

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    pg_engine::logic::logic_mut(world, this).pos_vel.y = 400.0;

    clear_z_rot(world, this);
    set_selectable(world, this, true);

    pg_engine::logic::set_behavior(world, this, Some("PygmyPlagueBodyDropWhileCarrying"));
}

/// `-[PygmyLogic (Apocalypse) startCarryingPlagueBody:]` @ 0x100047ab8
///
/// Sent to the body; `carrier` is the pygmy that picks it up.
pub fn start_carrying_plague_body(world: &mut World, this: Entity, carrier: Entity) {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = false;

    let mut apocalypse = world.obj_mut::<PygmyApocalypse>(this);
    apocalypse.plague_body_picked_up = false;
    apocalypse.plague_body_being_carried = true;
    apocalypse.plague_body_carrying_pygmy_logic = Some(carrier);

    set_selectable(world, this, false);
}

/// `-[PygmyLogic (Apocalypse) throwPlagueBodyOnPile]` @ 0x100047b34
pub fn throw_plague_body_on_pile(world: &mut World, this: Entity) {
    let mut apocalypse = world.obj_mut::<PygmyApocalypse>(this);
    apocalypse.plague_body_picked_up = false;
    apocalypse.plague_body_being_carried = false;
    apocalypse.plague_body_carrying_pygmy_logic = None;

    let marker = pg_engine::library::display_marker(world, "SmallObjectsMarker");
    move_after_marker(world, this, marker);

    pg_engine::logic::logic_mut(world, this).pos_vel.y = 100.0;

    pg_engine::logic::set_behavior(world, this, Some("PygmyPlagueBodyThrowOnPile"));
}

/// `-[PygmyLogic (Apocalypse) isAvailablePlagueBodyToBeCarried]` @ 0x100047c14
pub fn is_available_plague_body_to_be_carried(world: &mut World, this: Entity) -> bool {
    let apocalypse = world.obj::<PygmyApocalypse>(this);
    let core = world.obj::<PygmyPygmyCoreFrame>(this);

    !apocalypse.plague_body_in_pile
        && core.on_island
        && !apocalypse.plague_body_being_carried
        && core.killed_by_plague
}

/// `-[PygmyLogic (Apocalypse) throwPlagueBodyOnPile:::]` @ 0x100047c74
///
/// The thrown body flies towards the bonfire and burns on it when it is lit.
pub fn seq_throw_plague_body_on_pile(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let island_floor_collision = world.obj::<PygmyPygmyBehaviourItems>(this).island_floor_collision;
        if !island_floor_collision {
            world.obj_mut::<PygmyPygmyCoreFrame>(this).on_island = false;
        }

        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;

        clear_z_rot(world, this);
    }

    let listener = listener_of(world, this);
    let fire = listener.and_then(|listener| crate::controller::bon_fire_logic(world, listener, this));

    // PORT: the original reads the position of a nil bonfire's display object (a crash); the
    // port waits instead.
    let Some(fire) = fire else {
        return ItemResult::Pending;
    };

    let fire_pos = display_pos(world, fire);

    let display = pg_engine::logic::display_object(world, this);
    let mut object = pg_engine::display::object_mut(world, display);
    let x = object.pos.x;
    object.pos.x = x + (fire_pos.x - x) * 0.5;
    let y = object.pos.y;

    let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;
    let landed = vel_y < 0.0 && y < fire_pos.y;

    if !landed {
        return ItemResult::Pending;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    stop_moving(world, this);

    let bon_fire_enabled = listener.is_some_and(|listener| crate::controller::is_bon_fire_enabled(world, listener));
    if bon_fire_enabled {
        let fire = listener.and_then(|listener| crate::controller::bon_fire_logic(world, listener, this));
        let on_fire = fire.is_some_and(|fire| bon_fire_logic::is_on_fire(world, fire));

        if on_fire {
            set_sacrifice_id(world, this, "PlagueBodyBurntOnFire");
            pg_engine::logic::set_behavior(world, this, Some("PygmyPlagueBodyBurnOnFire"));

            return ItemResult::Replaced;
        }
    }

    let marker = listener.and_then(|listener| crate::controller::pygmy_island_display_marker(world, listener));
    move_after_marker(world, this, marker);

    set_selectable(world, this, true);

    pg_engine::logic::set_behavior(world, this, Some("PygmyPlagueBodyFall"));

    ItemResult::Replaced
}

/// `-[PygmyLogic (Island) stopCarryingPlagueBody]` @ 0x1000952c0
pub fn stop_carrying_plague_body(world: &mut World, this: Entity) {
    let body = world.obj::<PygmyApocalypse>(this).carrying_plague_body_logic;
    let Some(body) = body else {
        return;
    };

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PLAGUE_BODY_CHANNEL);

    drop_plague_body_while_carrying(world, body);

    world.obj_mut::<PygmyApocalypse>(this).carrying_plague_body_logic = None;
}

/// `-[PygmyLogic (Island) checkForPlagueBody]` @ 0x1000bb25c
///
/// While the bonfire burns, a pygmy walking past a plague body picks it up to throw it on the
/// fire.
pub fn check_for_plague_body(world: &mut World, this: Entity) -> bool {
    let Some(listener) = listener_of(world, this) else {
        return false;
    };

    let bon_fire_enabled = crate::controller::is_bon_fire_enabled(world, listener);
    if !bon_fire_enabled {
        return false;
    }

    let fire = crate::controller::bon_fire_logic(world, listener, this);
    let on_fire = fire.is_some_and(|fire| bon_fire_logic::is_on_fire(world, fire));
    if !on_fire {
        return false;
    }

    let x = display_pos(world, this).x;
    let pygmies = crate::controller::pygmy_logic_array(world, listener, this);

    for body in pygmies {
        let available = is_available_plague_body_to_be_carried(world, body);
        if !available {
            continue;
        }

        let body_x = display_pos(world, body).x;
        let distance = x - body_x;
        let near = distance.abs() <= 16.0;

        if !near {
            continue;
        }

        let x_scale = if distance <= 0.0 { 1.0 } else { -1.0 };
        set_x_scale(world, this, x_scale);
        stop_moving(world, this);

        world.obj_mut::<PygmyApocalypse>(this).carrying_plague_body_logic = Some(body);

        let animation = pg_engine::logic::animation(world, this);
        let body_display = pg_engine::logic::display_object(world, body);
        pg_engine::animation::set_tweenable(world, animation, Some(body_display), PLAGUE_BODY_CHANNEL);

        let carried = world.obj::<PygmyApocalypse>(this).carrying_plague_body_logic;
        if let Some(carried) = carried {
            start_carrying_plague_body(world, carried, this);
        }

        let carried = world.obj::<PygmyApocalypse>(this).carrying_plague_body_logic;
        incubate_disease(world, this, carried, "InfectPlagueBody", "InfectPygmyFromPlagueBody");

        let carried = world.obj::<PygmyApocalypse>(this).carrying_plague_body_logic;
        let pygmy_type = carried.map_or(0, |carried| time_temple_pygmy_type(world, carried));

        let behavior = match pygmy_type {
            0 => Some("PygmyCarryPlagueBody"),
            1 => Some("PygmyCarryOldPlagueBody"),
            2 => Some("PygmyCarryBabyPlagueBody"),
            3 => Some("PygmyCarryGingerPlagueBody"),
            4 => Some("PygmyCarryFastPlagueBody"),
            _ => None,
        };

        if let Some(behavior) = behavior {
            pg_engine::logic::set_behavior(world, this, Some(behavior));
        }

        return true;
    }

    false
}
