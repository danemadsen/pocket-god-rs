//! `PygmyLogic`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use crate::logic::island_core::sprite_logic::SpriteLogic;
use pg_engine::display;

/// Tweenable channel of the spear a pygmy holds (`catchSpear:`).
const SPEAR_CHANNEL: usize = 5;

/// `mPygmyLogicListener`: always the view controller (docs/re/pygmy.md §23).
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

/// The part every guard of this file shares: the busy set FREE of `docs/re/pygmy.md` §1.3
/// without its pygmy-type term and its three underwater terms (`mbEatenByPiranha`,
/// `mbSuckedIntoDrain`, `mbPainDrain`), plus `mTiltState == 0`. True when the pygmy is in none
/// of these 80 states. Pure reads, so the order of the original's checks does not matter.
fn is_free_base(world: &World, this: Entity) -> bool {
    let sprite = world.obj::<SpriteLogic>(this);
    if sprite.blown_by_hurricane || sprite.picked_up_by_hurricane {
        return false;
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let busy = core.touch_began_inside
        || core.remove_from_story_mode
        || core.tilt_state != 0
        || core.cut_in_half_by_charlie
        || core.dropped_by_hurricane
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || core.catch_shark
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

/// The three underwater terms of FREE: `mbEatenByPiranha`, `mbSuckedIntoDrain` and
/// `mbPainDrain` all clear.
fn is_free_underwater(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    !core.eaten_by_piranha && !core.sucked_into_drain && !core.pain_drain
}

/// FREE with `mTiltState == 0`: the guard of the hits that throw the pygmy into the sea.
fn is_free(world: &World, this: Entity) -> bool {
    let time_temple_pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    is_free_base(world, this) && time_temple_pygmy_type == 0 && is_free_underwater(world, this)
}

/// FREE +{mbSharkOnHead} with `mTiltState == 0`: the guard of the crushes, the bites, the burn
/// and the ants.
fn is_free_without_shark_on_head(world: &World, this: Entity) -> bool {
    let shark_on_head = world.obj::<PygmyPygmyCoreFrame>(this).shark_on_head;
    is_free(world, this) && !shark_on_head
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

/// `mPosVel.x = mPosVel.y = 0` (z and w keep their values).
fn stop_moving(world: &mut World, this: Entity) {
    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 0.0;
    logic.pos_vel.y = 0.0;
}

/// `-[PygmyLogic (Island) hitByBoulder:xVel:]` @ 0x10009ef00
pub fn hit_by_boulder(world: &mut World, this: Entity, _boulder_logic: Entity, x_vel: f32) -> bool {
    if !is_free(world, this) {
        return false;
    }

    stop_activities_and_being_eaten(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.tilt_state = 0;
    core.falling = true;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = x_vel;
    logic.pos_vel.y = 500.0;

    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;

    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("BoulderDrown".to_string());
    pg_engine::logic::set_behavior(world, this, Some("PygmyHitByBoulder"));
    true
}

/// `-[PygmyLogic (Island) canBeHitByPygmyIglooRoll:]` @ 0x10009f764
pub fn can_be_hit_by_pygmy_igloo_roll(world: &mut World, this: Entity, _rolling_pygmy_logic: Entity) -> bool {
    is_free(world, this)
}

/// `-[PygmyLogic (Island) hitByPygmyIglooRoll:]` @ 0x10009fe04
pub fn hit_by_pygmy_igloo_roll(world: &mut World, this: Entity, _rolling_pygmy_logic: Entity) {
    stop_activities_and_being_eaten(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.tilt_state = 0;
    core.falling = true;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 400.0;
    logic.pos_vel.y = 500.0;

    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;

    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("IglooPygmy".to_string());
    pg_engine::logic::set_behavior(world, this, Some("PygmyHitByPygmyIglooRoll"));
}

/// `-[PygmyLogic (Island) hitByTRexEgg:xVel:]` @ 0x10009ffcc
pub fn hit_by_t_rex_egg(world: &mut World, this: Entity, _t_rex_egg_logic: Entity, x_vel: f32) -> bool {
    if !is_free(world, this) {
        return false;
    }

    stop_activities_and_being_eaten(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.tilt_state = 0;
    core.falling = true;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = x_vel;
    logic.pos_vel.y = 500.0;

    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;

    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("TRexEggDrown".to_string());
    pg_engine::logic::set_behavior(world, this, Some("PygmyHitByTRexEgg"));
    true
}

/// `-[PygmyLogic (Island) hitByShark:xVel:]` @ 0x1000a0830
pub fn hit_by_shark(world: &mut World, this: Entity, _shark_logic: Entity, x_vel: f32) -> bool {
    if !is_free(world, this) {
        return false;
    }

    stop_activities_and_being_eaten(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.tilt_state = 0;
    core.falling = true;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = x_vel;
    logic.pos_vel.y = 500.0;

    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;

    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("SharkDrown".to_string());
    pg_engine::logic::set_behavior(world, this, Some("PygmyHitByShark"));
    true
}

/// `-[PygmyLogic (Island) crushedByBoulder:]` @ 0x1000a1b2c
pub fn crushed_by_boulder(world: &mut World, this: Entity, _boulder_logic: Entity) -> bool {
    if !is_free_without_shark_on_head(world, this) {
        return false;
    }

    stop_activities(world, this);

    world.obj_mut::<PygmyIslandHazards>(this).crushed_by_boulder = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.crushed = true;
    core.tilt_state = 0;

    stop_moving(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.rot.z = 0.0;
    object.is_selectable = false;

    pg_engine::logic::set_behavior(world, this, Some("PygmyCrushedByBoulder"));

    // The pygmy is drawn right after the boulder it lies under.
    let display_group = display::object(world, display_object).display_group;
    let boulder_display_object = match display_group {
        Some(display_group) => display::group::get_display_object(world, display_group, "Boulder"),
        None => None,
    };
    if let Some(boulder_display_object) = boulder_display_object {
        display::insert_after(world, boulder_display_object, Some(display_object));
    }

    update_label_layer(world, this);
    true
}

/// `-[PygmyLogic (Island) uncrushedByBoulder:dontWait:]` @ 0x1000a23dc
pub fn uncrushed_by_boulder(world: &mut World, this: Entity, _boulder_logic: Entity, dont_wait: bool) {
    let crushed = world.obj::<PygmyPygmyCoreFrame>(this).crushed;
    if !crushed {
        return;
    }

    if dont_wait {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).crushed = false;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;

    let listener = listener_of(world, this);
    let pygmy_marker = pygmy_display_marker_of(world, listener);
    if let Some(pygmy_marker) = pygmy_marker {
        display::insert_after(world, pygmy_marker, Some(display_object));
    }

    update_label_layer(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyUncrushedByBoulder"));
}

/// `-[PygmyLogic (Island) crushedByTRexEgg:]` @ 0x1000a24a4
pub fn crushed_by_t_rex_egg(world: &mut World, this: Entity, _t_rex_egg_logic: Entity) -> bool {
    if !is_free_without_shark_on_head(world, this) {
        return false;
    }

    stop_activities(world, this);

    world.obj_mut::<PygmyIslandHazards>(this).crushed_by_boulder = false;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.crushed = true;
    core.tilt_state = 0;

    stop_moving(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.rot.z = 0.0;
    object.is_selectable = false;

    pg_engine::logic::set_behavior(world, this, Some("PygmyCrushedByTRexEgg"));

    // The pygmy is drawn right after the egg it lies under.
    let display_group = display::object(world, display_object).display_group;
    let egg_display_object = match display_group {
        Some(display_group) => display::group::get_display_object(world, display_group, "TRexEgg"),
        None => None,
    };
    if let Some(egg_display_object) = egg_display_object {
        display::insert_after(world, egg_display_object, Some(display_object));
    }

    update_label_layer(world, this);
    true
}

/// `-[PygmyLogic (Island) uncrushedByTRexEgg:dontWait:]` @ 0x1000a2d54
pub fn uncrushed_by_t_rex_egg(world: &mut World, this: Entity, _t_rex_egg_logic: Entity, dont_wait: bool) {
    let crushed = world.obj::<PygmyPygmyCoreFrame>(this).crushed;
    if !crushed {
        return;
    }

    if dont_wait {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).crushed = false;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;

    let listener = listener_of(world, this);
    let pygmy_marker = pygmy_display_marker_of(world, listener);
    if let Some(pygmy_marker) = pygmy_marker {
        display::insert_after(world, pygmy_marker, Some(display_object));
    }

    update_label_layer(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyUncrushedByTRexEgg"));
}

/// `-[PygmyLogic (Island) swallowedByTRex:]` @ 0x1000a2e1c
pub fn swallowed_by_t_rex(world: &mut World, this: Entity, _t_rex_logic: Entity) {
    let eaten_by_t_rex = world.obj::<PygmyPygmyCoreFrame>(this).eaten_by_t_rex;
    if !eaten_by_t_rex {
        return;
    }

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.disable_labels = true;

    let sacrifice_id = if core.time_temple_pygmy_type == 2 {
        "BabyPygmyEatenByTrex"
    } else {
        "EatenByTRex"
    };
    core.sacrifice_id = Some(sacrifice_id.to_string());

    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Island) eatenByTRex:]` @ 0x1000a2e98
pub fn eaten_by_t_rex(world: &mut World, this: Entity, _t_rex_logic: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let is_regular_or_baby = matches!(core.time_temple_pygmy_type, 0 | 2);
    let shark_on_head = core.shark_on_head;

    let can_be_eaten = is_free_base(world, this)
        && is_regular_or_baby
        && is_free_underwater(world, this)
        && !shark_on_head;
    if !can_be_eaten {
        return false;
    }

    stop_activities(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).eaten_by_t_rex = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;

    stop_moving(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.rot.z = 0.0;
    object.is_selectable = false;

    let time_temple_pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    let behavior = if time_temple_pygmy_type == 2 {
        "TempleBabyPygmyEatenByTRex"
    } else {
        "PygmyEatenByTRex"
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));

    update_label_layer(world, this);
    true
}

/// `-[PygmyLogic (Island) uneatenByTRex:]` @ 0x1000a372c
pub fn uneaten_by_t_rex(world: &mut World, this: Entity, _t_rex_logic: Entity) {
    let eaten_by_t_rex = world.obj::<PygmyPygmyCoreFrame>(this).eaten_by_t_rex;
    if !eaten_by_t_rex {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).eaten_by_t_rex = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.rot.z = 0.0;
    object.is_selectable = true;

    let listener = listener_of(world, this);
    let pygmy_marker = pygmy_display_marker_of(world, listener);
    if let Some(pygmy_marker) = pygmy_marker {
        display::insert_before(world, pygmy_marker, Some(display_object));
    }

    update_label_layer(world, this);

    let time_temple_pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    let behavior = if time_temple_pygmy_type == 2 {
        "TempleBabyPygmyUneatenByTRex"
    } else {
        "PygmyUneatenByTRex"
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Island) swallowedBySpider:]` @ 0x1000a383c
pub fn swallowed_by_spider(world: &mut World, this: Entity, _spider_logic: Entity) {
    let eaten_by_spider = world.obj::<PygmyPygmyCoreFrame>(this).eaten_by_spider;
    if !eaten_by_spider {
        return;
    }

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.disable_labels = true;
    core.sacrifice_id = Some("EatenBySpider".to_string());

    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Island) eatenBySpider:]` @ 0x1000a3898
pub fn eaten_by_spider(world: &mut World, this: Entity, _spider_logic: Entity) -> bool {
    // Only a normal pygmy (`mTimeTemplePygmyType == 0`): unlike `eatenByTRex:`, the spider
    // never takes a baby pygmy.
    if !is_free_without_shark_on_head(world, this) {
        return false;
    }

    stop_activities(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).eaten_by_spider = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;

    stop_moving(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.rot.z = 0.0;
    object.is_selectable = false;

    pg_engine::logic::set_behavior(world, this, Some("PygmyEatenBySpider"));
    update_label_layer(world, this);
    true
}

/// `-[PygmyLogic (Island) ejectFromSpiderTree]` @ 0x1000a4108
pub fn eject_from_spider_tree(world: &mut World, this: Entity) {
    let trapped_in_spider_web = world.obj::<PygmyPygmyCoreFrame>(this).trapped_in_spider_web;
    if !trapped_in_spider_web {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).trapped_in_spider_web = false;

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

    display::object_mut(world, display_object).is_selectable = true;

    let listener = listener_of(world, this);
    let pygmy_marker = pygmy_display_marker_of(world, listener);
    if let Some(pygmy_marker) = pygmy_marker {
        display::insert_before(world, pygmy_marker, Some(display_object));
    }

    update_label_layer(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Island) eatenByLittleSpiders:]` @ 0x1000a4270
pub fn eaten_by_little_spiders(world: &mut World, this: Entity, _spider_tree_logic: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("EatenByLittleSpiders".to_string());
    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Island) uneatenBySpider:]` @ 0x1000a42a0
pub fn uneaten_by_spider(world: &mut World, this: Entity, _spider_logic: Entity) {
    let eaten_by_spider = world.obj::<PygmyPygmyCoreFrame>(this).eaten_by_spider;
    if !eaten_by_spider {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).eaten_by_spider = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.rot.z = 0.0;
    object.is_selectable = true;

    let listener = listener_of(world, this);
    let pygmy_marker = pygmy_display_marker_of(world, listener);
    if let Some(pygmy_marker) = pygmy_marker {
        display::insert_before(world, pygmy_marker, Some(display_object));
    }

    update_label_layer(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyUneatenBySpider"));
}

/// `-[PygmyLogic (Island) canBeBurntByMagnifyingGlass:]` @ 0x1000a438c
pub fn can_be_burnt_by_magnifying_glass(world: &mut World, this: Entity, _magnifying_glass_logic: Entity) -> bool {
    is_free_without_shark_on_head(world, this)
}

/// `-[PygmyLogic (Island) burntByMagnifyingGlass:]` @ 0x1000a4a40
pub fn burnt_by_magnifying_glass(world: &mut World, this: Entity, magnifying_glass_logic: Entity) {
    if !is_free_without_shark_on_head(world, this) {
        return;
    }

    stop_activities(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).burnt_by_magnifying_glass = true;
    world.obj_mut::<PygmyPygmyBehaviourItems>(this).burning_magnifying_glass_logic = Some(magnifying_glass_logic);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;

    stop_moving(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.rot.z = 0.0;
    object.is_selectable = false;

    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("MagnifyingGlass".to_string());
    pg_engine::logic::set_behavior(world, this, Some("PygmyBurntByMagnifyingGlass"));
}

/// `-[PygmyLogic (Island) unburntByMagnifyingGlass:]` @ 0x1000a52bc
pub fn unburnt_by_magnifying_glass(world: &mut World, this: Entity, _magnifying_glass_logic: Entity) {
    let burnt_by_magnifying_glass = world.obj::<PygmyPygmyCoreFrame>(this).burnt_by_magnifying_glass;
    if !burnt_by_magnifying_glass {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).burnt_by_magnifying_glass = false;
    world.obj_mut::<PygmyPygmyBehaviourItems>(this).burning_magnifying_glass_logic = None;

    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;

    pg_engine::logic::set_behavior(world, this, Some("PygmyUnburntByMagnifyingGlass"));
}

/// `-[PygmyLogic (Island) canCatchSpear]` @ 0x1000a5348
pub fn can_catch_spear(world: &mut World, this: Entity) -> bool {
    let throwing_spear = world.obj::<PygmyIslandLife>(this).throwing_spear;
    is_free_without_shark_on_head(world, this) && !throwing_spear
}

/// `-[PygmyLogic (Island) catchSpear:]` @ 0x1000a5a10
pub fn catch_spear(world: &mut World, this: Entity, spear_logic: Entity) {
    let throwing_spear = world.obj::<PygmyIslandLife>(this).throwing_spear;
    let can_catch = is_free_without_shark_on_head(world, this) && !throwing_spear;
    if !can_catch {
        return;
    }

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

    let mut island_life = world.obj_mut::<PygmyIslandLife>(this);
    island_life.throwing_spear = true;
    island_life.throwing_spear_logic = Some(spear_logic);

    let animation = pg_engine::logic::animation(world, this);
    let spear_display_object = pg_engine::logic::display_object(world, spear_logic);
    pg_engine::animation::set_tweenable(world, animation, Some(spear_display_object), SPEAR_CHANNEL);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    stop_moving(world, this);

    // Face the active T-Rex, else the spider (a nil monster reads as x = 0).
    let listener = listener_of(world, this);
    let monster_logic = match listener {
        Some(listener) => {
            let t_rex_active = crate::controller::is_t_rex_active(world, listener);
            if t_rex_active {
                crate::controller::t_rex_logic(world, listener, this)
            } else {
                crate::controller::spider_logic(world, listener, this)
            }
        }
        None => None,
    };
    let monster_x_pos = match monster_logic {
        Some(monster_logic) => {
            let monster_display_object = pg_engine::logic::display_object(world, monster_logic);
            display::object(world, monster_display_object).pos.x
        }
        None => 0.0,
    };

    let display_object = pg_engine::logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;
    let x_scale = if monster_x_pos > x_pos { 1.0 } else { -1.0 };

    let mut object = display::object_mut(world, display_object);
    object.scale.x = x_scale;
    object.rot.z = 0.0;

    pg_engine::logic::set_behavior(world, this, Some("PygmyCatchSpear"));
}

/// `-[PygmyLogic (Island) canBeSwarmedByAntHill:]` @ 0x1000a635c
pub fn can_be_swarmed_by_ant_hill(world: &mut World, this: Entity, _ant_hill_logic: Entity) -> bool {
    is_free_without_shark_on_head(world, this)
}

/// `-[PygmyLogic (Island) swarmedByAntHill:]` @ 0x1000a6a10
pub fn swarmed_by_ant_hill(world: &mut World, this: Entity, _ant_hill_logic: Entity) {
    // Unlike `canBeSwarmedByAntHill:`, this guard leaves out the three underwater terms.
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let time_temple_pygmy_type = core.time_temple_pygmy_type;
    let shark_on_head = core.shark_on_head;

    let can_be_swarmed = is_free_base(world, this) && time_temple_pygmy_type == 0 && !shark_on_head;
    if !can_be_swarmed {
        return;
    }

    stop_activities(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).swarmed = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;

    stop_moving(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.rot.z = 0.0;
    object.is_selectable = false;

    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("FireAnts".to_string());
    pg_engine::logic::set_behavior(world, this, Some("PygmySwarmedByAntHill"));
}
