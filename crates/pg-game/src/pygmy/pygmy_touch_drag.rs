//! `PygmyLogic`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use crate::logic::island_core::sprite_logic::SpriteLogic;
use pg_engine::touch::BcTouch;

/// `PygmyLogic` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct PygmyPygmyTouchDrag {
    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mOhStall`: `i`
    pub oh_stall: i32,

    /// `mbOverFire`: `B`
    pub over_fire: bool,

    /// `mbTheRuns`: `B`
    pub the_runs: bool,

    /// `mbKonkeyDong`: `B`
    pub konkey_dong: bool,

    /// `mbNoPee`: `B`
    pub no_pee: bool,

    /// `mKonkeyDongLadderTouch`: `@"BCTouch"`
    pub konkey_dong_ladder_touch: Option<Entity>,
}

/// The `allTouches` argument the touch handlers forward to the mini-game and ghost handlers.
/// The port's touch slots carry no touch set and none of those handlers reads it.
const NO_ALL_TOUCHES: Entity = Entity::PLACEHOLDER;

/// `[[display displayGroup] viewToDisplayObject:&point]`; outside any group the message goes
/// to nil and answers all zero.
fn view_to_display_group(world: &World, display: Entity, point: Vec4) -> Vec4 {
    let group = pg_engine::display::object(world, display).display_group;

    match group {
        Some(group) => pg_engine::display::view_to_display_object(world, group, point),
        None => Vec4::ZERO,
    }
}

/// The stop list S14 (`docs/re/pygmy.md` §1.3) that `startDrag` and the three branches of
/// `touchMovedEntered:allTouches:` send.
fn stop_activities(world: &mut World, this: Entity) {
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

/// `[self setBehavior:]` with `PygmyFall`, or `TemplePygmyFall` for the temple pygmy types
/// (`mTimeTemplePygmyType != 0`).
fn set_fall_behavior(world: &mut World, this: Entity) {
    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    let behavior = if pygmy_type == 0 { "PygmyFall" } else { "TemplePygmyFall" };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// The underwater drop shared by both release paths of `touchEndedInside:allTouches:`.
fn drop_underwater(world: &mut World, this: Entity) {
    let bait_master = world.obj::<PygmyUnderwaterNopee>(this).bait_master;

    let behavior = if bait_master {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        "PygmyDropDuringBaitMaster"
    } else {
        "PygmyDropUnderwater"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic drag:::]` @ 0x100132094
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display = pg_engine::logic::display_object(world, this);

    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.enable_island_floor_collision = false;
        core.enable_island_side_collision = false;

        let x_origin = call.float("xOrigin");
        let y_origin = call.float("yOrigin");

        let mut object = pg_engine::display::object_mut(world, display);
        object.origin_pos.x = x_origin;
        object.origin_pos.y = y_origin;

        world.obj_mut::<PygmyPygmyTouchDrag>(this).oh_stall = 0;

        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        let underwater = core.underwater;
        let listener = core.pygmy_logic_listener;

        if !underwater {
            let marker = listener.and_then(|listener| crate::controller::pygmy_drag_display_marker(world, listener));
            if let Some(marker) = marker {
                pg_engine::display::insert_after(world, marker, Some(display));
            }

            update_label_layer(world, this);
        }
    }

    let touch_screen_pos = world.obj::<PygmyPygmyTouchDrag>(this).touch_screen_pos;
    let mut touch_point = view_to_display_group(world, display, touch_screen_pos);

    let underwater = world.obj::<PygmyPygmyCoreFrame>(this).underwater;
    if underwater && touch_point.y < 65.0 {
        touch_point.y = 65.0;
    }

    // The finger is taken at the pygmy's own depth; w keeps the pygmy's.
    let pos = pg_engine::display::object(world, display).pos;
    touch_point.z = pos.z;

    let delta = Vec4::new(pos.x - touch_point.x, pos.y - touch_point.y, pos.z - touch_point.z, pos.w);
    let distance = pg_engine::math::length3(delta);
    if distance == 0.0 {
        return ItemResult::Pending;
    }

    if delta.y != 0.0 {
        let angle = (-delta.y).atan2(-delta.x);
        let z_rot = angle * 180.0 / std::f32::consts::PI - 90.0;

        pg_engine::display::object_mut(world, display).rot.z = z_rot;
    }

    let vel = pg_engine::logic::logic(world, this).pos_vel;
    let spring_strength = (distance * -100.0) / distance;
    let spring_x = -8.0 * vel.x + spring_strength * delta.x;
    let spring_y = -8.0 * vel.y + spring_strength * delta.y;
    let spring_z = -8.0 * vel.z + spring_strength * delta.z;
    let spring_force = Vec4::new(spring_x, spring_y, spring_z, delta.w);

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_say_oh = !core.cooked && !core.killed_by_plague && !core.beheaded_by_squid;
    let pygmy_type = core.time_temple_pygmy_type;

    if can_say_oh {
        let mut touch_drag = world.obj_mut::<PygmyPygmyTouchDrag>(this);
        touch_drag.oh_stall = touch_drag.oh_stall.wrapping_sub(1);
        let oh_stall = touch_drag.oh_stall;

        let speed = (vel.x * vel.x + vel.y * vel.y).sqrt();

        if speed > 500.0 && oh_stall < 0 {
            world.obj_mut::<PygmyPygmyTouchDrag>(this).oh_stall = 60;

            let sound_id = if pygmy_type == 2 {
                "BabySound1"
            } else if pygmy_type == 1 {
                "OldOh"
            } else {
                let roll = world.resource_mut::<Random>().arc4random_mod(3);
                match roll {
                    2 => "Oh3",
                    1 => "Oh2",
                    _ => "Oh1",
                }
            };

            pg_engine::sound::play_sound(world, sound_id);
        }
    }

    world.obj_mut::<PygmyPygmyTouchDrag>(this).over_fire = false;

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let bon_fire_enabled = listener.is_some_and(|listener| crate::controller::is_bon_fire_enabled(world, listener));
    let beheaded_by_squid = world.obj::<PygmyPygmyCoreFrame>(this).beheaded_by_squid;

    if bon_fire_enabled && !beheaded_by_squid {
        let bon_fire = listener.and_then(|listener| crate::controller::bon_fire_logic(world, listener, this));

        if let Some(bon_fire) = bon_fire {
            let on_fire = crate::logic::island_core::bon_fire_logic::is_on_fire(world, bon_fire);

            if on_fire {
                let fire_display = pg_engine::logic::display_object(world, bon_fire);
                let fire_pos = pg_engine::display::object(world, fire_display).pos;
                let pos = pg_engine::display::object(world, display).pos;

                let x_distance = (pos.x - fire_pos.x).abs();
                let height = pos.y - fire_pos.y;

                if x_distance < 20.0 && 40.0 < height && height < 130.0 {
                    world.obj_mut::<PygmyPygmyTouchDrag>(this).over_fire = true;
                }
            }
        }
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let healthy = !core.cooked && !core.killed_by_plague && !core.beheaded_by_squid;
    let pygmy_type = core.time_temple_pygmy_type;

    if healthy && matches!(pygmy_type, 0 | 2 | 4) {
        let shark = listener.and_then(|listener| crate::controller::shark_logic(world, listener, this));
        if let Some(shark) = shark {
            crate::logic::ocean_creatures::shark_logic::check_jump_for_pygmy(world, shark, this);
        }
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).spring_force = spring_force;

    ItemResult::Pending
}

/// `-[PygmyLogic startDrag]` @ 0x100132fdc
pub fn start_drag(world: &mut World, this: Entity) {
    stop_activities(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.drag_label_x_offset = 0.0;
    core.drag_label_y_offset = -64.0;
    core.dragging = true;
    core.apply_spring = true;
    core.touch_began_inside = false;
    let listener = core.pygmy_logic_listener;

    if let Some(listener) = listener {
        crate::controller::on_pygmy_drag(world, listener, this);
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let behavior = if core.cooked {
        "PygmyCookedDrag"
    } else if core.killed_by_plague {
        "PygmyPlagueBodyDrag"
    } else if core.cut_in_half_by_charlie {
        "HalfPygmyDrag"
    } else if core.time_temple_pygmy_type == 0 {
        "PygmyDrag"
    } else {
        "TemplePygmyDrag"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// The flag part of the grab condition of `touchBeganInside:allTouches:` (everything after
/// `isDraggingPygmy` and `canTouchPygmy`).
fn can_be_grabbed(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
    let sprite = world.obj::<SpriteLogic>(this);

    let busy = core.remove_from_story_mode
        || core.tilt_state != 0
        || story_mode
        || core.falling
        || core.doodle_jumping
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.resting_in_igloo
        || core.crapping
        || core.zombie_explode
        || core.is_ghost
        || core.touched_by_ghost
        || core.caught_by_monkey
        || core.monkey_on_back
        || core.eaten_by_moron_snake
        || core.wearing_water_wings
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.skewered
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.slamming_closet_temple_closet_door
        || core.crushed
        || core.caught_by_bird
        || core.tumble
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.beheaded_by_squid
        || core.swarmed
        || core.burnt_by_magnifying_glass
        || core.catch_shark
        || core.dropped_by_hurricane
        || sprite.blown_by_hurricane
        || sprite.picked_up_by_hurricane
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || core.active_touch.is_some();

    if busy {
        return false;
    }

    // A drowning pygmy can still be rescued, unless it is cooked, cut in half or a temple
    // pygmy that is not a plague body.
    let rescuable = !core.cooked
        && !core.cut_in_half_by_charlie
        && (core.killed_by_plague || core.time_temple_pygmy_type == 0);

    !core.drowning || rescuable
}

/// `-[PygmyLogic touchBeganInside:allTouches:]` @ 0x10013321c
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let display = pg_engine::logic::display_object(world, this);

    let mouth_filled_by_locusts = world.obj::<PygmyPygmyBehaviourItems>(this).mouth_filled_by_locusts;
    if mouth_filled_by_locusts {
        pg_engine::display::object_mut(world, display).is_selectable = false;
        pg_engine::logic::set_behavior(world, this, Some("PygmyMouthFilledByLocustsSplat"));
        return;
    }

    let mutated_by_locusts = world.obj::<PygmyPygmyCoreFrame>(this).mutated_by_locusts;
    if mutated_by_locusts {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;

        world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("SquashedLikeBug".to_string());
        pg_engine::display::object_mut(world, display).is_selectable = false;
        pg_engine::logic::set_behavior(world, this, Some("PygmyMutatedByLocustsSplat"));
        return;
    }

    let touch_drag = world.obj::<PygmyPygmyTouchDrag>(this);
    let the_runs = touch_drag.the_runs;
    let no_pee = touch_drag.no_pee;
    let konkey_dong = touch_drag.konkey_dong;

    if the_runs {
        let all_touches = pg_engine::touch::all_touches(world);
        touch_began_the_runs(world, this, touch, &all_touches);
        return;
    }

    if no_pee {
        touch_began_no_pee(world, this, touch);
        return;
    }

    if konkey_dong {
        let all_touches = pg_engine::touch::all_touches(world);
        touch_began_konkey_dong(world, this, touch, &all_touches);
        return;
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let is_ghost = core.is_ghost;
    let is_zombie = core.is_zombie;
    let sun_frozen = core.sun_frozen;
    let possessed_by_thetan = core.possessed_by_thetan;

    if is_ghost {
        touch_began_inside_ghost(world, this, touch, NO_ALL_TOUCHES);
        return;
    }

    if is_zombie {
        zombie_explode(world, this);
        return;
    }

    if sun_frozen {
        ice_shatter(world, this);
        return;
    }

    if possessed_by_thetan {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.possessed_by_thetan = false;
        let listener = core.pygmy_logic_listener;

        let apocalypse_active = listener.is_some_and(|listener| crate::controller::is_apocalypse_active(world, listener));
        let idol_manager = crate::managers::idol_manager::instance(world);
        let episode_event = if apocalypse_active {
            "ApocalypseThetanUnpossessPygmy"
        } else {
            "NormalThetanUnpossessPygmy"
        };

        if let Some(idol_manager) = idol_manager {
            crate::managers::idol_manager::trigger_episode_event(world, idol_manager, episode_event);
        }
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let kong_statue = core.kong_statue_logic;
    let kidnapped_by_konkey_dong = core.kidnapped_by_konkey_dong;

    if let Some(kong_statue) = kong_statue {
        if !kidnapped_by_konkey_dong {
            crate::logic::konkeydong::kong_statue_logic::remove_pygmy_from_kong_statue(world, kong_statue, this);

            let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
            core.kong_statue_logic = None;
            core.in_kong_statue = false;
        }
    }

    let ape_wench = world.obj::<PygmyPygmyCoreFrame>(this).ape_wench_logic;
    if let Some(ape_wench) = ape_wench {
        crate::logic::ape_ritual::ape_wench_logic::remove_pygmy_from_ape_wench(world, ape_wench, this);

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.ape_wench_logic = None;
        core.in_ape_wench = false;
    }

    let ape_bongo = world.obj::<PygmyPygmyCoreFrame>(this).ape_bongo_logic;
    if let Some(ape_bongo) = ape_bongo {
        crate::logic::ape_ritual::ape_bongo_logic::remove_pygmy_from_ape_bongo(world, ape_bongo, this);

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.ape_bongo_logic = None;
        core.in_ape_bongo = false;
    }

    let ape_altar = world.obj::<PygmyPygmyCoreFrame>(this).ape_altar_logic;
    if let Some(ape_altar) = ape_altar {
        crate::logic::ape_ritual::ape_altar_logic::remove_pygmy_from_ape_altar(world, ape_altar, this);

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.ape_altar_logic = None;
        core.in_ape_altar = false;
    }

    let go_go_cage = world.obj::<PygmyContainersAndCatchers>(this).go_go_cage_logic;
    if let Some(go_go_cage) = go_go_cage {
        crate::logic::apocalypse_dance::go_go_cage_logic::remove_pygmy_from_go_go_cage(world, go_go_cage, this);

        world.obj_mut::<PygmyContainersAndCatchers>(this).go_go_cage_logic = None;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).in_go_go_cage = false;
    }

    let dance_dance_execution = world.obj::<PygmyContainersAndCatchers>(this).dance_dance_execution_logic;
    if let Some(dance_dance_execution) = dance_dance_execution {
        crate::logic::apocalypse_dance::dance_dance_execution_logic::remove_pygmy_from_dance_dance_execution(
            world,
            dance_dance_execution,
            this,
        );

        world.obj_mut::<PygmyContainersAndCatchers>(this).dance_dance_execution_logic = None;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).in_dance_dance_execution = false;
    }

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let dragging_pygmy = listener.is_some_and(|listener| crate::controller::is_dragging_pygmy(world, listener));
    let can_grab = !dragging_pygmy
        && listener.is_some_and(|listener| crate::controller::can_touch_pygmy(world, listener))
        && can_be_grabbed(world, this);

    if can_grab {
        stop_holding_chum(world, this);
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
        stop_being_trapped_in_idol_pedestal(world, this);
        stop_being_on_top_apocalypse_time_temple_platform(world, this);
        stop_being_in_apocalypse_temple_fountain(world, this);
        stop_thetan_brawl(world, this);
        stop_being_on_closet_temple_thetan_meter(world, this);
        stop_being_on_closet_temple_couch(world, this);
        stop_being_in_closet_temple_closet_door(world, this);
        stop_being_held_in_toms_arms(world, this);
        stop_being_possessed_by_thetan(world, this);
        stop_holding_baby_pygmy(world, this);
        stop_being_eaten_by_locust_queen(world, this);
        stop_being_thrown_by_locusts(world, this);

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.drowning = false;
        core.active_touch = Some(touch);

        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;

        pg_engine::display::object_mut(world, display).set_alpha(1.0);

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.touch_began_inside = true;
        let on_fail_boat = core.on_fail_boat;

        if on_fail_boat {
            world.obj_mut::<PygmyPygmyCoreFrame>(this).on_fail_boat = false;

            let touch_pos = world.obj::<BcTouch>(touch).pos;
            world.obj_mut::<PygmyPygmyTouchDrag>(this).touch_screen_pos = touch_pos;

            let marker = listener.and_then(|listener| crate::controller::pygmy_drag_display_marker(world, listener));
            if let Some(marker) = marker {
                translate_and_move_to_display_marker(world, this, marker);
            }

            start_drag(world, this);
            return;
        }

        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        let behavior = if core.cooked {
            "PygmyCookedTouch"
        } else if core.killed_by_plague {
            "PygmyPlagueBodyTouch"
        } else if core.cut_in_half_by_charlie {
            "HalfPygmyTouch"
        } else if core.time_temple_pygmy_type == 0 {
            "PygmyTouch"
        } else {
            "TemplePygmyTouch"
        };

        pg_engine::logic::set_behavior(world, this, Some(behavior));
    }

    // A poke makes a pygmy hanging upside down let go.
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    if core.tilt_state != 5 || core.time_temple_pygmy_type != 0 {
        return;
    }

    stop_sleeping(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;

    let mut object = pg_engine::display::object_mut(world, display);
    object.rot.z = -180.0;
    let x_offset = if object.scale.x > 0.0 { -32.0 } else { 32.0 };
    object.pos.x += x_offset;
    object.pos.y += 32.0;
    let y_pos = object.pos.y;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.last_pos.y = y_pos;
    core.tilt_state = 0;
    core.sacrifice_id = Some("UpsideDownLoseGrip".to_string());

    pg_engine::logic::set_behavior(world, this, Some("PygmyFallUpsideDown"));
}

/// `-[PygmyLogic touchBeganOutside:allTouches:]` @ 0x100133df4
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let touch_drag = world.obj::<PygmyPygmyTouchDrag>(this);
    let the_runs = touch_drag.the_runs;
    let no_pee = touch_drag.no_pee;
    let konkey_dong = touch_drag.konkey_dong;

    if the_runs {
        let all_touches = pg_engine::touch::all_touches(world);
        touch_began_the_runs(world, this, touch, &all_touches);
        return;
    }

    if no_pee {
        touch_began_no_pee(world, this, touch);
        return;
    }

    if konkey_dong {
        let all_touches = pg_engine::touch::all_touches(world);
        touch_began_konkey_dong(world, this, touch, &all_touches);
        return;
    }

    let active_touch = world.obj::<PygmyPygmyCoreFrame>(this).active_touch;
    if active_touch.is_none() {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<PygmyPygmyTouchDrag>(this).touch_screen_pos = touch_pos;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).touch_began_inside = false;
    }
}

/// The flag part of the push condition of `touchMovedEntered:allTouches:` (everything after
/// `isDraggingPygmy` and `canTouchPygmy`).
fn can_be_pushed(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
    let sprite = world.obj::<SpriteLogic>(this);
    let touch_drag = world.obj::<PygmyPygmyTouchDrag>(this);

    let busy = core.remove_from_story_mode
        || core.tilt_state != 0
        || story_mode
        || core.dragging
        || core.cut_in_half_by_charlie
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
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
        || core.underwater
        || core.skewered
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.tumble
        || core.crushed
        || core.caught_by_bird
        || core.drowning
        || core.swarmed
        || core.burnt_by_magnifying_glass
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.beheaded_by_squid
        || core.cooked
        || core.killed_by_plague
        || core.catch_shark
        || core.dropped_by_hurricane
        || sprite.blown_by_hurricane
        || sprite.picked_up_by_hurricane
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || core.doodle_jumping
        || touch_drag.the_runs
        || touch_drag.no_pee
        || touch_drag.konkey_dong
        || core.falling;

    !busy
}

/// `-[PygmyLogic touchMovedEntered:allTouches:]` @ 0x100133eb4
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let active_touch = core.active_touch;
    let sun_frozen = core.sun_frozen;

    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    if sun_frozen {
        ice_shatter(world, this);
        return;
    }

    let display = pg_engine::logic::display_object(world, this);

    let old_screen_pos = world.obj::<PygmyPygmyTouchDrag>(this).touch_screen_pos;
    let old_point = view_to_display_group(world, display, old_screen_pos);

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<PygmyPygmyTouchDrag>(this).touch_screen_pos = touch_pos;
    let new_point = view_to_display_group(world, display, touch_pos);

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;

    let dragging_pygmy = listener.is_some_and(|listener| crate::controller::is_dragging_pygmy(world, listener));
    if dragging_pygmy {
        return;
    }

    let can_touch_pygmy = listener.is_some_and(|listener| crate::controller::can_touch_pygmy(world, listener));
    if !can_touch_pygmy || !can_be_pushed(world, this) {
        return;
    }

    let min_pos = pg_engine::display::min_pos(world, display);
    let max_pos = pg_engine::display::max_pos(world, display);

    // From above: the pygmy is pushed off its feet.
    if old_point.y >= max_pos.y {
        stop_activities(world, this);

        let mut object = pg_engine::display::object_mut(world, display);
        object.pos.y -= 1.0;
        let y_pos = object.pos.y;

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.last_pos.y = y_pos;
        core.falling = true;
        core.drowning = false;
        core.sacrifice_id = Some("PushedDrown".to_string());

        set_fall_behavior(world, this);
        return;
    }

    let delta_x = new_point.x - old_point.x;
    let delta_y = new_point.y - old_point.y;

    // From below: the pygmy is flicked into the air, away from the camera.
    if old_point.y <= min_pos.y {
        stop_activities(world, this);

        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = delta_x * 10.0;
        logic.pos_vel.y = delta_y * 40.0;
        logic.pos_vel.z = -400.0;

        let pos = pg_engine::display::object(world, display).pos;
        world.obj_mut::<PygmyPygmyBehaviourItems>(this).tumble_pos = pos;

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.falling = true;
        core.drowning = false;

        let location = listener.map_or(0, |listener| crate::controller::island_location_state(world, listener));
        let in_temple_room = (11..=17).contains(&location);

        if !in_temple_room {
            let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
            let behavior = if pygmy_type == 0 { "PygmyTumble" } else { "TemplePygmyTumble" };

            pg_engine::logic::set_behavior(world, this, Some(behavior));
            return;
        }

        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = delta_x * 5.0;
        logic.pos_vel.y = delta_y * 20.0;
        logic.pos_vel.z = 0.0;

        set_fall_behavior(world, this);
        return;
    }

    // From the side. The original compares old.x with max.x twice (`b.ge`, then `b.ls`), so
    // every ordered entry lands here and only a NaN coordinate returns.
    let side_ordered = old_point.x >= max_pos.x || old_point.x <= max_pos.x;
    if !side_ordered {
        return;
    }

    stop_activities(world, this);

    if delta_y <= 0.0 {
        let mut object = pg_engine::display::object_mut(world, display);
        object.pos.y -= 1.0;
        let y_pos = object.pos.y;

        world.obj_mut::<PygmyPygmyCoreFrame>(this).last_pos.y = y_pos;
    }

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = delta_x * 10.0;
    logic.pos_vel.y = delta_y * 20.0;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.falling = true;
    core.drowning = false;
    core.sacrifice_id = Some("PushedDrown".to_string());

    set_fall_behavior(world, this);
}

/// The flag part of the drag condition of `touchMovedExited:allTouches:` (everything after
/// `canTouchPygmy`).
fn can_start_drag(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
    let sprite = world.obj::<SpriteLogic>(this);
    let touch_drag = world.obj::<PygmyPygmyTouchDrag>(this);

    let busy = core.tilt_state != 0
        || story_mode
        || core.eaten_by_piranha
        || core.remove_from_story_mode
        || core.sucked_into_drain
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.resting_in_igloo
        || core.crapping
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
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.crushed
        || core.caught_by_bird
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.beheaded_by_squid
        || core.swarmed
        || core.burnt_by_magnifying_glass
        || core.catch_shark
        || sprite.blown_by_hurricane
        || sprite.picked_up_by_hurricane
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || core.doodle_jumping
        || touch_drag.the_runs
        || touch_drag.no_pee
        || touch_drag.konkey_dong
        || core.falling;

    !busy
}

/// `-[PygmyLogic touchMovedExited:allTouches:]` @ 0x100134b58
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let active_touch = core.active_touch;
    let pressed_ghost = core.is_ghost && core.touch_began_inside;

    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    if pressed_ghost {
        stop_banshee_scream(world, this);
    }

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let can_touch_pygmy = listener.is_some_and(|listener| crate::controller::can_touch_pygmy(world, listener));
    if !can_touch_pygmy || !can_start_drag(world, this) {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<PygmyPygmyTouchDrag>(this).touch_screen_pos = touch_pos;

    let dragging_pygmy = listener.is_some_and(|listener| crate::controller::is_dragging_pygmy(world, listener));
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    if !dragging_pygmy && core.tilt_state == 0 && !core.dragging && core.touch_began_inside {
        start_drag(world, this);
        return;
    }

    // Another pygmy is being dragged: this one, pressed meanwhile, only hops.
    let dragging_pygmy = listener.is_some_and(|listener| crate::controller::is_dragging_pygmy(world, listener));
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let hops = dragging_pygmy
        && !core.dragging
        && !core.underwater
        && !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.touch_began_inside;

    if hops {
        pg_engine::logic::logic_mut(world, this).pos_vel.y = 200.0;

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.falling = true;
        core.touch_began_inside = false;
        core.sacrifice_id = Some("DroppedDrown".to_string());

        set_fall_behavior(world, this);
    }
}

/// `-[PygmyLogic touchMovedInside:allTouches:]` @ 0x1001351d4
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[PygmyLogic touchMovedOutside:allTouches:]` @ 0x1001351e4
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let ladder_touch = world.obj::<PygmyPygmyTouchDrag>(this).konkey_dong_ladder_touch;

    if ladder_touch == Some(touch) {
        let group = pg_engine::library::display_group(world, "PocketGod");
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        let point = match group {
            Some(group) => pg_engine::display::view_to_display_object(world, group, touch_pos),
            None => Vec4::ZERO,
        };

        world.obj_mut::<PygmyMinigames>(this).konkey_dong_ladder_up = point.y > 160.0;
    }

    let active_touch = world.obj::<PygmyPygmyCoreFrame>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<PygmyPygmyTouchDrag>(this).touch_screen_pos = touch_pos;

    let is_ghost = world.obj::<PygmyPygmyCoreFrame>(this).is_ghost;
    if is_ghost {
        touch_moved_ghost(world, this, touch, NO_ALL_TOUCHES);
    }
}

/// `-[PygmyLogic touchEndedInside:allTouches:]` @ 0x100135320
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let konkey_dong = world.obj::<PygmyPygmyTouchDrag>(this).konkey_dong;

    if konkey_dong {
        let jump_touch = world.obj::<PygmyMinigames>(this).konkey_dong_jump_touch;

        if jump_touch == Some(touch) {
            let mut minigames = world.obj_mut::<PygmyMinigames>(this);
            minigames.konkey_dong_jump_touch = None;
            let holding_pick = minigames.konkey_dong_holding_pick;

            if !holding_pick {
                pg_engine::logic::set_behavior(world, this, Some("PygmyKonkeyDongJumpDown"));
            }
        }

        let ladder_touch = world.obj::<PygmyPygmyTouchDrag>(this).konkey_dong_ladder_touch;
        if ladder_touch == Some(touch) {
            world.obj_mut::<PygmyPygmyTouchDrag>(this).konkey_dong_ladder_touch = None;
        }

        return;
    }

    let active_touch = world.obj::<PygmyPygmyCoreFrame>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let touch_drag = world.obj::<PygmyPygmyTouchDrag>(this);
    let the_runs = touch_drag.the_runs;
    let no_pee = touch_drag.no_pee;

    if the_runs {
        if active_touch.is_none() {
            return;
        }

        world.obj_mut::<PygmyPygmyCoreFrame>(this).active_touch = None;
        pg_engine::logic::set_behavior(world, this, Some("PygmyTheRunsJumpDown"));
        return;
    }

    if no_pee {
        touch_ended_inside_no_pee(world, this, touch);
        return;
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    if core.is_ghost && core.touch_began_inside {
        let ghost = world.obj::<PygmyGhostZombieVampire>(this);
        let go_into_the_light = ghost.go_into_the_light;
        let the_light = ghost.the_light_logic;

        if !go_into_the_light {
            if let Some(the_light) = the_light {
                pg_engine::logic::set_behavior(world, the_light, Some("TheLightExit"));
            }
        }

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.active_touch = None;
        core.touch_began_inside = false;

        stop_banshee_scream(world, this);

        let path_point_count = world.obj::<PygmyGhostZombieVampire>(this).ghost_path_vector.len();
        if path_point_count > 1 {
            pg_engine::logic::set_behavior(world, this, Some("PygmyGhostFollowPath"));
        }

        let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_pygmy_ghost_drag_complete(world, listener, this);
        }

        return;
    }

    let display = pg_engine::logic::display_object(world, this);
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let dragging = core.dragging;
    let touch_began_inside = core.touch_began_inside;

    if dragging {
        release_drag(world, this, display);
    } else if touch_began_inside {
        release_tap(world, this);
    }

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.touch_began_inside = false;
    core.active_touch = None;
}

/// The release after a drag in `touchEndedInside:allTouches:`: the pygmy keeps the velocity
/// the spring gave it (the throw), unless the spider tree web or a jellyfish catches it.
fn release_drag(world: &mut World, this: Entity, display: Entity) {
    world.obj_mut::<PygmyPygmyTouchDrag>(this).over_fire = false;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.dragging = false;
    core.apply_spring = false;
    core.drag_label_x_offset = 0.0;
    core.drag_label_y_offset = 0.0;
    let listener = core.pygmy_logic_listener;

    let shark = listener.and_then(|listener| crate::controller::shark_logic(world, listener, this));
    if let Some(shark) = shark {
        crate::logic::ocean_creatures::shark_logic::cant_catch_pygmy(world, shark, this);
    }

    if let Some(listener) = listener {
        crate::controller::on_pygmy_drag_complete(world, listener, this);
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let normal_and_healthy = !core.cooked
        && !core.killed_by_plague
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == 0;

    let caught_by_web = normal_and_healthy
        && listener.is_some_and(|listener| crate::controller::is_spider_tree_enabled(world, listener))
        && check_spider_tree_collision(world, this);

    if caught_by_web {
        return;
    }

    let underwater = world.obj::<PygmyPygmyCoreFrame>(this).underwater;

    if underwater {
        let caught_by_jellyfish = check_jellyfish_collision(world, this);
        if caught_by_jellyfish {
            return;
        }

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.enable_island_side_collision = false;
        core.enable_island_floor_collision = false;

        pg_engine::display::object_mut(world, display).rot.z = 0.0;

        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;

        drop_underwater(world, this);
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;

    let marker = listener.and_then(|listener| crate::controller::pygmy_display_marker(world, listener));
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display));
    }

    update_label_layer(world, this);

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let behavior = if core.cooked {
        "PygmyCookedFall"
    } else if core.killed_by_plague {
        "PygmyPlagueBodyFall"
    } else if core.cut_in_half_by_charlie {
        "HalfPygmyFall"
    } else if core.time_temple_pygmy_type != 0 {
        "TemplePygmyFall"
    } else if core.beheaded_by_squid {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("LostMyHead".to_string());
        "PygmyBeheadedFall"
    } else {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("DroppedDrown".to_string());
        "PygmyFallFar"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// The release of a tap (pressed, never dragged) in `touchEndedInside:allTouches:`: a hop,
/// or a heart attack for the old fart.
fn release_tap(world: &mut World, this: Entity) {
    let underwater = world.obj::<PygmyPygmyCoreFrame>(this).underwater;

    if underwater {
        drop_underwater(world, this);
        return;
    }

    let swallowed_by_zombie_grave = check_for_zombie_grave(world, this);
    if swallowed_by_zombie_grave {
        return;
    }

    pg_engine::logic::logic_mut(world, this).pos_vel.y = 200.0;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let pygmy_type = core.time_temple_pygmy_type;

    let behavior = if core.cooked {
        "PygmyCookedFall"
    } else if core.killed_by_plague {
        "PygmyPlagueBodyFall"
    } else if core.cut_in_half_by_charlie {
        "HalfPygmyFall"
    } else if pygmy_type == 0 {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("DroppedDrown".to_string());
        "PygmyFall"
    } else if pygmy_type == 1 {
        world.obj_mut::<PygmyPygmyBehaviourItems>(this).had_heart_attack = true;

        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;

        "TempleOldPygmyHeartAttack"
    } else {
        "TemplePygmyFall"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic touchEndedOutside:allTouches:]` @ 0x100135954
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}

/// `-[PygmyLogic (Island) checkSpiderTreeCollision]` @ 0x1000bf554
pub fn check_spider_tree_collision(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let spider_tree = listener.and_then(|listener| crate::controller::spider_tree_logic(world, listener, this));

    let Some(spider_tree) = spider_tree else {
        return false;
    };

    let available = crate::logic::monsters::spider_tree_logic::is_available(world, spider_tree);
    if !available {
        return false;
    }

    let tree_display = pg_engine::logic::display_object(world, spider_tree);
    let display = pg_engine::logic::display_object(world, this);
    let pos = pg_engine::display::object(world, display).pos;
    let tree_pos = pg_engine::display::object(world, tree_display).pos;

    let delta_x = pos.x - (tree_pos.x + 130.0);
    let delta_y = pos.y - (tree_pos.y + 110.0);
    let distance_squared = delta_x * delta_x + delta_y * delta_y;
    let in_web = distance_squared < 900.0;

    if !in_web {
        return false;
    }

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.touch_began_inside = false;
    core.active_touch = None;
    core.trapped_in_spider_web = true;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 0.0;
    logic.pos_vel.y = 0.0;

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = false;

    pg_engine::display::object_mut(world, display).is_selectable = false;

    let marker = listener.and_then(|listener| crate::controller::pygmy_island_back_display_marker(world, listener));
    if let Some(marker) = marker {
        pg_engine::display::insert_before(world, marker, Some(display));
    }

    update_label_layer(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyStuckInWeb"));
    crate::logic::monsters::spider_tree_logic::trap_in_web(world, spider_tree, this);

    true
}
