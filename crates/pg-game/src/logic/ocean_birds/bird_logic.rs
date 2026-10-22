//! `BirdLogic` (superclass `SpriteLogic`).
//!
//! The seagull (`docs/re/logic-catalog.md` §4.3). It flies across the island at y 220, dives for
//! the fish (or for the pygmy eating it) and carries the catch off screen, drops a
//! `BirdPooLogic` when tapped, and once cooked by lightning or over the bonfire it is a throwable
//! of §2.2 (floor at y 70) that pygmies eat.

use crate::prelude::*;

use crate::logic::island_core::bon_fire_logic;
use crate::logic::island_core::coconut_logic::{
    ISLAND_Z_POS, THROWABLE_BOUNCE_SPEED, THROWABLE_GRAVITY_FORCE, bounce_throwable_velocity, integrate_throwable,
    is_throwable_out_of_range, reaches_island_floor, throwable_drag_z_rot, throwable_pos_force, throwable_spring_force,
    throwable_tilted_gravity_force,
};
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::ocean_birds::bird_poo_logic;
use crate::logic::ocean_fishing::fish_logic;
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::length3;
use pg_engine::touch::BcTouch;

/// `BirdLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct BirdLogic {
    /// `mBirdLogicListener`: `@"<BirdLogicListener>"`
    pub bird_logic_listener: Option<Entity>,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbExitIsland`: `B`
    pub exit_island: bool,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mSpringDelta`: `f`
    pub spring_delta: f32,

    /// `mCookTime`: `f`
    pub cook_time: f32,

    /// `mBirdCookState`: `i`
    pub bird_cook_state: i32,

    /// `mbInWater`: `B`
    pub in_water: bool,

    /// `mbFalling`: `B`
    pub falling: bool,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbDontHitUntilBounce`: `B`
    pub dont_hit_until_bounce: bool,

    /// `mbBeingEaten`: `B`
    pub being_eaten: bool,

    /// `mbOnIsland`: `B`
    pub on_island: bool,

    /// `mbEnableIslandFloorCollision`: `B`
    pub enable_island_floor_collision: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mbDivingForFood`: `B`
    pub diving_for_food: bool,

    /// `mbBirdPooHitPygmy`: `B`
    pub bird_poo_hit_pygmy: bool,

    /// `mBirdPooLogic`: `@"BirdPooLogic"`
    pub bird_poo_logic: Option<Entity>,

    /// `mCarryingFishLogic`: `@"FishLogic"`
    pub carrying_fish_logic: Option<Entity>,

    /// `mTargetPygmyLogic`: `@"PygmyLogic"`
    pub target_pygmy_logic: Option<Entity>,

    /// `mCarryingPygmyLogic`: `@"PygmyLogic"`
    pub carrying_pygmy_logic: Option<Entity>,
}

pub const BIRD_LOGIC: Class = Class {
    name: "BirdLogic",
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

/// `mBirdCookState`: raw, cooked or burnt.
const COOK_STATE_RAW: i32 = 0;
const COOK_STATE_COOKED: i32 = 1;
const COOK_STATE_BURNT: i32 = 2;

/// The animation channels a catch hangs on: a pygmy on 1, the fish on 2.
const PYGMY_CHANNEL: usize = 1;
const FISH_CHANNEL: usize = 2;

/// The flight height (`enterIsland:`, and `fly:::` on its first frame).
const FLIGHT_Y_POS: f32 = 220.0;

/// The island floor (`setYPos:` in `onGameFrame:`).
const FLOOR_Y: f32 = 70.0;

/// `fall:::` and (in front of the island) `fallFromHurricane:::` splash below this y.
const WATER_LINE_Y: f32 = 20.0;

/// `fallFromHurricane:::` splashes below this y behind the island.
const BEHIND_ISLAND_WATER_LINE_Y: f32 = 86.0;

/// Dives aim from the beak, 20 ahead (along `xScale`) and 36 below the origin, at 150.
const BEAK_X_OFFSET: f32 = 20.0;
const BEAK_Y_OFFSET: f32 = -36.0;
const DIVE_SPEED: f32 = 150.0;

/// `fly:::` dives for a catchable fish less than 40 ahead.
const FISH_SIGHT_RANGE: f32 = 40.0;

/// `fly:::` turns around after a dropping hit a pygmy once its bounds pass these x.
const TURN_AROUND_MAX_X_POS: f32 = 80.0;
const TURN_AROUND_MIN_X_POS: f32 = 400.0;

/// Off screen: the right edge left of −40 or the left edge right of 520.
const OFF_SCREEN_LEFT_X_POS: f32 = -40.0;
const OFF_SCREEN_RIGHT_X_POS: f32 = 520.0;

/// The speed of both axes while flying off screen with or without a catch.
const FLY_OFF_SPEED: f32 = 150.0;

/// `thrownByCharlie:`: thrown away from Charlie's facing, upwards.
const CHARLIE_THROW_X_VEL: f32 = -200.0;
const CHARLIE_THROW_Y_VEL: f32 = 600.0;

/// `dropWhileEating`: the bird hops up.
const DROP_Y_VEL: f32 = 400.0;

/// The spring of `drag:::`.
const DRAG_STIFFNESS: f32 = 100.0;
const DRAG_DAMPING: f32 = 8.0;

/// `checkPygmyCollision`: faster than this, a cooked bird hits a pygmy whose point 31 above its
/// origin lies within 20 points (squared distance 400).
const PYGMY_HIT_MIN_SPEED: f32 = 200.0;
const PYGMY_HIT_DISTANCE_SQUARED: f32 = 400.0;
const PYGMY_HIT_Y_OFFSET: f32 = 31.0;

/// Cooking in `onGameFrame:`: dragged within 20 of the fire's x and 30–130 above it; cooked
/// after 2 s, burnt after 4 s.
const COOK_X_REACH: f32 = 20.0;
const COOK_MIN_HEIGHT: f32 = 30.0;
const COOK_MAX_HEIGHT: f32 = 130.0;
const COOKED_TIME: f32 = 2.0;
const BURNT_TIME: f32 = 4.0;

/// The dropping's horizontal speed per unit of the bird's `xScale`.
const BIRD_POO_X_SPEED: f32 = 150.0;

/// Sequence-item handlers implemented by `BirdLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "fly" => seq_fly(world, this, call),
        "diveForFish" => seq_dive_for_fish(world, this, call),
        "carryFishOffScreen" => seq_carry_fish_off_screen(world, this, call),
        "diveForPygmy" => seq_dive_for_pygmy(world, this, call),
        "carryPygmyOffScreen" => seq_carry_pygmy_off_screen(world, this, call),
        "dropAndFlyOffScreen" => seq_drop_and_fly_off_screen(world, this, call),
        "die" => seq_die(world, this, call),
        "fall" => seq_fall(world, this, call),
        "fallFromHurricane" => seq_fall_from_hurricane(world, this, call),
        "drag" => seq_drag(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[BirdLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&BIRD_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[BirdLogic initWithDisplayObject:]` @ 0x100091e54
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(BirdLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[BirdLogic setBirdLogicListener:]` @ 0x100091ec4
pub fn set_bird_logic_listener(world: &mut World, this: Entity, bird_logic_listener: Entity) {
    world.obj_mut::<BirdLogic>(this).bird_logic_listener = Some(bird_logic_listener);
}

/// `-[BirdLogic dropPygmyLogic:]` @ 0x100091f20
pub fn drop_pygmy_logic(world: &mut World, this: Entity, drop_pygmy_logic: Entity) {
    let carrying_pygmy = world.obj::<BirdLogic>(this).carrying_pygmy_logic;
    if carrying_pygmy != Some(drop_pygmy_logic) {
        return;
    }

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    world.obj_mut::<BirdLogic>(this).carrying_pygmy_logic = None;
}

/// `-[BirdLogic dropFood]` @ 0x100091f78
pub fn drop_food(world: &mut World, this: Entity) {
    let carrying_fish = world.obj::<BirdLogic>(this).carrying_fish_logic;
    if let Some(carrying_fish) = carrying_fish {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, FISH_CHANNEL);
        fish_logic::dropped_by_bird(world, carrying_fish, this);
        world.obj_mut::<BirdLogic>(this).carrying_fish_logic = None;
    }

    let carrying_pygmy = world.obj::<BirdLogic>(this).carrying_pygmy_logic;
    if let Some(carrying_pygmy) = carrying_pygmy {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        crate::pygmy::dropped_by_bird(world, carrying_pygmy, this);
        world.obj_mut::<BirdLogic>(this).carrying_pygmy_logic = None;
    }
}

/// `-[BirdLogic isEdible]` @ 0x100092034
pub fn is_edible(world: &mut World, this: Entity) -> bool {
    let bird = world.obj::<BirdLogic>(this);
    bird.on_island && !bird.being_eaten && bird.bird_cook_state != COOK_STATE_RAW
}

/// `-[BirdLogic isBurnt]` @ 0x100092074
pub fn is_burnt(world: &mut World, this: Entity) -> bool {
    world.obj::<BirdLogic>(this).bird_cook_state == COOK_STATE_BURNT
}

/// `-[BirdLogic isCooked]` @ 0x10009208c
pub fn is_cooked(world: &mut World, this: Entity) -> bool {
    world.obj::<BirdLogic>(this).bird_cook_state == COOK_STATE_COOKED
}

/// `-[BirdLogic moveToDragLayer]` @ 0x1000920a4
pub fn move_to_drag_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[BirdLogic moveToIslandLayer]` @ 0x10009210c
///
/// The island layer is just before the drag marker.
pub fn move_to_island_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }
}

/// `-[BirdLogic moveToBehindIslandLayer]` @ 0x100092174
///
/// The marker's index is taken before the bird leaves its group.
pub fn move_to_behind_island_layer(world: &mut World, this: Entity) {
    let island_display_group = pg_engine::library::display_group(world, "IslandDisplayGroup");
    let marker_index = match island_display_group {
        Some(island_display_group) => {
            display::group::get_display_object_index(world, island_display_group, "IslandDisplayMarker")
        }
        None => 0,
    };

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    display::move_to_display_group_at_index(world, display_object, island_display_group, marker_index);
}

/// `-[BirdLogic enterIsland:]` @ 0x100092220
///
/// The startup flag is ignored: the bird always starts over from `BirdIdle`.
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    let active = world.obj::<BirdLogic>(this).active;
    if active {
        return;
    }
    world.obj_mut::<BirdLogic>(this).active = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let display_object = logic::display_object(world, this);
    {
        let mut display = display::object_mut(world, display_object);
        display.pos.y = FLIGHT_Y_POS;
        display.pos.z = ISLAND_Z_POS;
        display.rot.z = 0.0;
        display.scale.x = 1.0;
        display.scale.y = 1.0;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    move_to_island_layer(world, this);

    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("BirdIdle"));
}

/// `-[BirdLogic exitIsland:]` @ 0x100092350
///
/// `now` ends the bird at once (`BirdDie`). Otherwise the first call lets a cooked bird that is
/// not in the water fall through the island, and a raw bird flies on.
pub fn exit_island(world: &mut World, this: Entity, now: bool) {
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;

    let dragging = world.obj::<BirdLogic>(this).dragging;
    if dragging {
        {
            let mut bird = world.obj_mut::<BirdLogic>(this);
            bird.falling = true;
            bird.dragging = false;
            bird.apply_spring = false;
        }
        let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_bird_drag_complete(world, listener, this);
        }
    }

    if now {
        world.obj_mut::<BirdLogic>(this).exit_island = true;
        logic::set_behavior(world, this, Some("BirdDie"));
        return;
    }

    let already_exiting = world.obj::<BirdLogic>(this).exit_island;
    if already_exiting {
        return;
    }
    world.obj_mut::<BirdLogic>(this).exit_island = true;

    let bird = world.obj::<BirdLogic>(this);
    if bird.in_water || bird.bird_cook_state == COOK_STATE_RAW {
        return;
    }
    logic::set_behavior(world, this, Some("BirdFallIgnoreIsland"));
}

/// `-[BirdLogic exitIslandNow]` @ 0x100092464
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[BirdLogic isAccelEnabled]` @ 0x1000924f4
///
/// The bird never slides with the device's tilt.
pub fn is_accel_enabled(_world: &mut World, _this: Entity) -> bool {
    false
}

/// `-[BirdLogic isBeingEaten]` @ 0x1000924fc
pub fn is_being_eaten(world: &mut World, this: Entity) -> bool {
    world.obj::<BirdLogic>(this).being_eaten
}

/// The behaviour of a bird dropped while being eaten: cooked, or else burnt.
fn drop_while_eating_behavior(cook_state: i32) -> &'static str {
    if cook_state == COOK_STATE_COOKED {
        "BirdCookedDropWhileEating"
    } else {
        "BirdBurntDropWhileEating"
    }
}

/// `-[BirdLogic thrownByCharlie:]` @ 0x10009250c
pub fn thrown_by_charlie(world: &mut World, this: Entity, thrown_by_charlie: Entity) {
    world.obj_mut::<BirdLogic>(this).being_eaten = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<BirdLogic>(this).dont_hit_until_bounce = true;

    let charlie_display_object = logic::display_object(world, thrown_by_charlie);
    let charlie_x_scale = display::object(world, charlie_display_object).scale.x;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = charlie_x_scale * CHARLIE_THROW_X_VEL;
        state.pos_vel.y = CHARLIE_THROW_Y_VEL;
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;

    let cook_state = world.obj::<BirdLogic>(this).bird_cook_state;
    let behavior = drop_while_eating_behavior(cook_state);
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[BirdLogic dropWhileEating]` @ 0x100092600
pub fn drop_while_eating(world: &mut World, this: Entity) {
    world.obj_mut::<BirdLogic>(this).being_eaten = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<BirdLogic>(this).dont_hit_until_bounce = true;
    logic::logic_mut(world, this).pos_vel.y = DROP_Y_VEL;

    let cook_state = world.obj::<BirdLogic>(this).bird_cook_state;
    let behavior = drop_while_eating_behavior(cook_state);
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[BirdLogic startEating:]` @ 0x10009267c
pub fn start_eating(world: &mut World, this: Entity, _eating: Entity) {
    world.obj_mut::<BirdLogic>(this).being_eaten = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
}

/// `-[BirdLogic pickUpForEating]` @ 0x1000926a0
pub fn pick_up_for_eating(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("BirdCookedEaten"));
}

/// `-[BirdLogic finishEating]` @ 0x1000926b8
pub fn finish_eating(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("BirdDie"));
}

/// `-[BirdLogic checkPygmyCollision]` @ 0x1000926d0
///
/// The first pygmy in reach that takes the hit ends the check; the bird itself flies on.
pub fn check_pygmy_collision(world: &mut World, this: Entity) {
    let bird = world.obj::<BirdLogic>(this);
    if bird.being_eaten || bird.dont_hit_until_bounce || bird.bird_cook_state == COOK_STATE_RAW {
        return;
    }

    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    let pos_vel = logic::logic(world, this).pos_vel;
    let speed = length3(pos_vel);

    let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
    let pygmy_logic_array = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmy_logic_array {
        // The speed is tested per pygmy in the original.
        let fast = PYGMY_HIT_MIN_SPEED < speed;
        if !fast {
            continue;
        }

        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let delta_x = pos.x - (pygmy_pos.x + 0.0);
        let delta_y = pos.y - (pygmy_pos.y + PYGMY_HIT_Y_OFFSET);
        let distance_squared = delta_x * delta_x + delta_y * delta_y;
        let within_reach = distance_squared < PYGMY_HIT_DISTANCE_SQUARED;
        if !within_reach {
            continue;
        }

        let hit = crate::pygmy::hit_by_cooked_bird(world, pygmy, this);
        if hit {
            return;
        }
    }
}

/// `-[BirdLogic onGameFrame:]` @ 0x100092938
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let display_object = logic::display_object(world, this);
    let previous_y = display::object(world, display_object).pos.y;

    let accel_enabled = is_accel_enabled(world, this);
    let gravity_force = if accel_enabled {
        throwable_tilted_gravity_force(world)
    } else {
        THROWABLE_GRAVITY_FORCE
    };
    world.obj_mut::<SpriteLogic>(this).gravity_force = gravity_force;

    let ignore_gravity = world.obj::<SpriteLogic>(this).ignore_gravity;
    let bird = world.obj::<BirdLogic>(this);
    let spring_force = bird.spring_force;
    let apply_spring = bird.apply_spring;

    let pos_force = throwable_pos_force(gravity_force, ignore_gravity, spring_force, apply_spring);
    logic::logic_mut(world, this).pos_force = pos_force;
    integrate_throwable(world, this, pos_force, dt);

    // Unlike the other throwables, the bird tests the play area before the floor.
    let y_pos = display::object(world, display_object).pos.y;
    if is_throwable_out_of_range(y_pos) {
        logic::set_behavior(world, this, Some("BirdDie"));
    }

    world.obj_mut::<BirdLogic>(this).island_floor_collision = false;
    let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
    let mut island_min_x_pos = 0.0;
    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
        island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
    }

    let pos = display::object(world, display_object).pos;
    let reached_floor = reaches_island_floor(previous_y, pos, island_min_x_pos, island_max_x_pos, FLOOR_Y);
    let enable_island_floor_collision = world.obj::<BirdLogic>(this).enable_island_floor_collision;
    if reached_floor && enable_island_floor_collision {
        land_on_island_floor(world, this, display_object);
    } else {
        world.obj_mut::<BirdLogic>(this).on_island = false;
    }

    cook_over_bon_fire(world, this, dt);
    check_pygmy_collision(world, this);
}

/// The island floor contact of `onGameFrame:`: on the first contact the bird bounces
/// (`BirdCookedBounce`) or lands; every contact puts it on the floor.
fn land_on_island_floor(world: &mut World, this: Entity, display_object: Entity) {
    let on_island = world.obj::<BirdLogic>(this).on_island;
    let mut pos_vel = logic::logic(world, this).pos_vel;
    bounce_throwable_velocity(&mut pos_vel, on_island);
    logic::logic_mut(world, this).pos_vel = pos_vel;

    if pos_vel.y < THROWABLE_BOUNCE_SPEED {
        logic::logic_mut(world, this).pos_vel.y = 0.0;
        if !on_island {
            world.obj_mut::<BirdLogic>(this).on_island = true;
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            world.obj_mut::<BirdLogic>(this).island_floor_collision = true;
        }
        display::object_mut(world, display_object).rot.z = 0.0;
    } else {
        logic::set_behavior(world, this, Some("BirdCookedBounce"));
    }

    world.obj_mut::<BirdLogic>(this).dont_hit_until_bounce = false;
    display::object_mut(world, display_object).pos.y = FLOOR_Y;
}

/// The cooking of `onGameFrame:`: a bird dragged over a burning bonfire cooks after 2 s and
/// burns after 4 s (`mCookTime` keeps counting).
fn cook_over_bon_fire(world: &mut World, this: Entity, dt: f32) {
    let bird = world.obj::<BirdLogic>(this);
    let dragging = bird.dragging;
    let listener = bird.bird_logic_listener;
    if !dragging {
        return;
    }
    let Some(listener) = listener else {
        return;
    };

    let bon_fire_enabled = crate::controller::is_bon_fire_enabled(world, listener);
    if !bon_fire_enabled {
        return;
    }
    let bon_fire = crate::controller::bon_fire_logic(world, listener, this);
    let Some(bon_fire) = bon_fire else {
        return;
    };
    let on_fire = bon_fire_logic::is_on_fire(world, bon_fire);
    if !on_fire {
        return;
    }

    let bon_fire_display_object = logic::display_object(world, bon_fire);
    let bon_fire_pos = display::object(world, bon_fire_display_object).pos;
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;

    let within_reach = (pos.x - bon_fire_pos.x).abs() < COOK_X_REACH;
    let height = pos.y - bon_fire_pos.y;
    let above_fire = COOK_MIN_HEIGHT < height && height < COOK_MAX_HEIGHT;
    if !within_reach || !above_fire {
        return;
    }

    let cook_time = world.obj::<BirdLogic>(this).cook_time + dt;
    world.obj_mut::<BirdLogic>(this).cook_time = cook_time;

    let cook_state = world.obj::<BirdLogic>(this).bird_cook_state;
    if COOKED_TIME < cook_time && cook_state == COOK_STATE_RAW {
        world.obj_mut::<BirdLogic>(this).bird_cook_state = COOK_STATE_COOKED;
        logic::set_behavior(world, this, Some("BirdCook"));
        return;
    }
    if BURNT_TIME < cook_time && cook_state == COOK_STATE_COOKED {
        world.obj_mut::<BirdLogic>(this).bird_cook_state = COOK_STATE_BURNT;
        logic::set_behavior(world, this, Some("BirdBurn"));
    }
}

/// The dive velocity of `fly:::`, `diveForFish:::` and `diveForPygmy:::`: 150 along the line from
/// the beak to the target's origin.
fn dive_toward(world: &mut World, this: Entity, target_display_object: Entity) {
    let target_pos = display::object(world, target_display_object).pos;
    let display_object = logic::display_object(world, this);
    let display = display::object(world, display_object);
    let pos = display.pos;
    let x_scale = display.scale.x;

    let delta_x = target_pos.x - (pos.x + x_scale * BEAK_X_OFFSET);
    let delta_y = target_pos.y - (pos.y + BEAK_Y_OFFSET);
    let angle = delta_y.atan2(delta_x);
    let (sin, cos) = angle.sin_cos();

    let mut state = logic::logic_mut(world, this);
    state.pos_vel.x = cos * DIVE_SPEED;
    state.pos_vel.y = sin * DIVE_SPEED;
}

/// `-[BirdLogic fly:::]` @ 0x100092f04
///
/// The first frame puts the bird at flight height. It dives at a catchable fish just ahead,
/// turns around near the edge it flies to once a dropping has hit a pygmy, and completes off
/// screen.
pub fn seq_fly(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    // The bounds are measured before the first frame's climb.
    let display_object = logic::display_object(world, this);
    let x_scale = display::object(world, display_object).scale.x;
    let min_pos = display::min_pos(world, display_object);
    let max_pos = display::max_pos(world, display_object);

    let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
    let fish = match listener {
        Some(listener) => crate::controller::fish_logic(world, listener, this),
        None => None,
    };

    if call.first_time {
        display::object_mut(world, display_object).pos.y = FLIGHT_Y_POS;
    }

    if let Some(fish) = fish
        && fish_logic::can_be_caught_by_bird(world, fish, this)
    {
        let fish_display_object = logic::display_object(world, fish);
        let fish_x_pos = display::object(world, fish_display_object).pos.x;
        let x_pos = display::object(world, display_object).pos.x;
        let distance = fish_x_pos - x_pos;

        let ahead_right = 0.0 < x_scale && distance < FISH_SIGHT_RANGE && 0.0 < distance;
        let ahead_left = x_scale < 0.0 && -FISH_SIGHT_RANGE < distance && distance < 0.0;
        if ahead_right || ahead_left {
            dive_toward(world, this, fish_display_object);
            world.obj_mut::<BirdLogic>(this).diving_for_food = true;
            logic::set_behavior(world, this, Some("BirdCatchFood"));
            return ItemResult::Replaced;
        }
    }

    let bird_poo_hit_pygmy = world.obj::<BirdLogic>(this).bird_poo_hit_pygmy;
    if bird_poo_hit_pygmy {
        let near_left_edge = x_scale < 0.0 && max_pos.x < TURN_AROUND_MAX_X_POS;
        let near_right_edge = 0.0 < x_scale && TURN_AROUND_MIN_X_POS < min_pos.x;
        if near_left_edge || near_right_edge {
            world.obj_mut::<BirdLogic>(this).bird_poo_hit_pygmy = false;
            logic::set_behavior(world, this, Some("BirdTurnAround"));
            return ItemResult::Replaced;
        }
    }

    let off_left = x_scale < 0.0 && max_pos.x < OFF_SCREEN_LEFT_X_POS;
    let off_right = 0.0 < x_scale && OFF_SCREEN_RIGHT_X_POS < min_pos.x;
    if !off_left && !off_right {
        return ItemResult::Pending;
    }

    if let Some(listener) = listener {
        crate::controller::on_bird_fly_off_screen(world, listener, this);
    }
    ItemResult::Done
}

/// `-[BirdLogic diveForFish:::]` @ 0x1000931fc
///
/// Steers at the fish; takes the pygmy eating it instead, gives up when it cannot be caught,
/// and catches it once the beak is below it.
pub fn seq_dive_for_fish(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
    let fish = match listener {
        Some(listener) => crate::controller::fish_logic(world, listener, this),
        None => None,
    };
    let Some(fish) = fish else {
        // PORT: the original reads a nil fish's position (a crash); without a fish the dive
        // ends the way an uncatchable one does.
        logic::set_behavior(world, this, Some("BirdAbortDive"));
        return ItemResult::Replaced;
    };

    let fish_display_object = logic::display_object(world, fish);
    dive_toward(world, this, fish_display_object);

    let fish_being_eaten = fish_logic::is_being_eaten(world, fish);
    if fish_being_eaten {
        let eating_pygmy = fish_logic::eating_pygmy_logic(world, fish);
        world.obj_mut::<BirdLogic>(this).target_pygmy_logic = eating_pygmy;
        logic::set_behavior(world, this, Some("BirdCatchPygmy"));
        return ItemResult::Replaced;
    }

    let catchable = fish_logic::can_be_caught_by_bird(world, fish, this);
    if !catchable {
        logic::set_behavior(world, this, Some("BirdAbortDive"));
        return ItemResult::Replaced;
    }

    let display_object = logic::display_object(world, this);
    let beak_y_pos = display::object(world, display_object).pos.y + BEAK_Y_OFFSET;
    let fish_y_pos = display::object(world, fish_display_object).pos.y;
    let reached = beak_y_pos < fish_y_pos;
    if !reached {
        return ItemResult::Pending;
    }

    {
        let mut bird = world.obj_mut::<BirdLogic>(this);
        bird.diving_for_food = false;
        bird.carrying_fish_logic = Some(fish);
    }
    fish_logic::caught_by_bird(world, fish, this);

    let animation = logic::animation(world, this);
    let fish_display_object = logic::display_object(world, fish);
    pg_engine::animation::set_tweenable(world, animation, Some(fish_display_object), FISH_CHANNEL);
    ItemResult::Done
}

/// The flight of `carryFishOffScreen:::`, `carryPygmyOffScreen:::` and `dropAndFlyOffScreen:::`:
/// the first frame sets off at 150 up and forward (along `xScale`); once past the screen edge it
/// flies to, the bird stops and this answers YES.
fn fly_off_screen(world: &mut World, this: Entity, first_time: bool) -> bool {
    let display_object = logic::display_object(world, this);
    let min_pos = display::min_pos(world, display_object);
    let max_pos = display::max_pos(world, display_object);

    if first_time {
        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = FLY_OFF_SPEED;
            state.pos_vel.y = FLY_OFF_SPEED;
        }
        let x_scale = display::object(world, display_object).scale.x;
        if x_scale < 0.0 {
            logic::logic_mut(world, this).pos_vel.x = -FLY_OFF_SPEED;
        }
    }

    let x_vel = logic::logic(world, this).pos_vel.x;
    let off_left = x_vel < 0.0 && max_pos.x < OFF_SCREEN_LEFT_X_POS;
    let off_right = 0.0 < x_vel && OFF_SCREEN_RIGHT_X_POS < min_pos.x;
    if !off_left && !off_right {
        return false;
    }

    let mut state = logic::logic_mut(world, this);
    state.pos_vel.x = 0.0;
    state.pos_vel.y = 0.0;
    true
}

/// `-[BirdLogic carryFishOffScreen:::]` @ 0x100093428
pub fn seq_carry_fish_off_screen(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let off_screen = fly_off_screen(world, this, call.first_time);
    if !off_screen {
        return ItemResult::Pending;
    }

    let carrying_fish = world.obj::<BirdLogic>(this).carrying_fish_logic;
    if let Some(carrying_fish) = carrying_fish {
        fish_logic::caught_by_bird_complete(world, carrying_fish, this);
    }
    world.obj_mut::<BirdLogic>(this).carrying_fish_logic = None;

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, FISH_CHANNEL);
    ItemResult::Done
}

/// `-[BirdLogic diveForPygmy:::]` @ 0x10009357c
///
/// Steers at the pygmy that was eating the fish; gives up when it cannot be caught, and catches
/// it once the beak is below it.
pub fn seq_dive_for_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let target_pygmy = world.obj::<BirdLogic>(this).target_pygmy_logic;
    let Some(target_pygmy) = target_pygmy else {
        // PORT: the original reads a nil target's position (a crash); without a target the dive
        // ends the way an uncatchable one does.
        logic::set_behavior(world, this, Some("BirdAbortDive"));
        return ItemResult::Replaced;
    };

    let target_display_object = logic::display_object(world, target_pygmy);
    dive_toward(world, this, target_display_object);

    let catchable = crate::pygmy::can_be_caught_by_bird(world, target_pygmy, this);
    if !catchable {
        world.obj_mut::<BirdLogic>(this).target_pygmy_logic = None;
        logic::set_behavior(world, this, Some("BirdAbortDive"));
        return ItemResult::Replaced;
    }

    let display_object = logic::display_object(world, this);
    let beak_y_pos = display::object(world, display_object).pos.y + BEAK_Y_OFFSET;
    let target_y_pos = display::object(world, target_display_object).pos.y;
    let reached = beak_y_pos < target_y_pos;
    if !reached {
        return ItemResult::Pending;
    }

    {
        let mut bird = world.obj_mut::<BirdLogic>(this);
        bird.diving_for_food = false;
        bird.carrying_pygmy_logic = Some(target_pygmy);
        bird.target_pygmy_logic = None;
    }
    crate::pygmy::caught_by_bird(world, target_pygmy, this);

    // The pygmy may already have let go (`dropPygmyLogic:`): the ivar is read again.
    let carrying_pygmy = world.obj::<BirdLogic>(this).carrying_pygmy_logic;
    let carrying_display_object = carrying_pygmy.map(|pygmy| logic::display_object(world, pygmy));
    let animation = logic::animation(world, this);
    pg_engine::animation::set_tweenable(world, animation, carrying_display_object, PYGMY_CHANNEL);
    ItemResult::Done
}

/// `-[BirdLogic carryPygmyOffScreen:::]` @ 0x10009374c
pub fn seq_carry_pygmy_off_screen(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let off_screen = fly_off_screen(world, this, call.first_time);
    if !off_screen {
        return ItemResult::Pending;
    }

    let carrying_pygmy = world.obj::<BirdLogic>(this).carrying_pygmy_logic;
    if let Some(carrying_pygmy) = carrying_pygmy {
        crate::pygmy::caught_by_bird_complete(world, carrying_pygmy, this);
    }
    world.obj_mut::<BirdLogic>(this).carrying_pygmy_logic = None;

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    ItemResult::Done
}

/// `-[BirdLogic dropAndFlyOffScreen:::]` @ 0x10009389c
pub fn seq_drop_and_fly_off_screen(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let off_screen = fly_off_screen(world, this, call.first_time);
    if off_screen { ItemResult::Done } else { ItemResult::Pending }
}

/// `-[BirdLogic die:::]` @ 0x1000939a4
///
/// The bird leaves the screen, lets go of its catch, forgets its state and, while birds are
/// enabled outside a story, starts over.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);

    // The original repeats the steps of `dropFood` here.
    drop_food(world, this);

    let dragging = world.obj::<BirdLogic>(this).dragging;
    if dragging {
        world.obj_mut::<BirdLogic>(this).dragging = false;
        let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_bird_drag_complete(world, listener, this);
        }
    }

    {
        let mut bird = world.obj_mut::<BirdLogic>(this);
        bird.active = false;
        bird.exit_island = false;
        bird.cook_time = 0.0;
        bird.bird_cook_state = COOK_STATE_RAW;
        bird.in_water = false;
        bird.falling = false;
        bird.being_eaten = false;
        bird.apply_spring = false;
        bird.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<BirdLogic>(this).enable_island_floor_collision = false;

    let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
    let Some(listener) = listener else {
        return ItemResult::Done;
    };
    let bird_enabled = crate::controller::is_bird_enabled(world, listener);
    if bird_enabled {
        let story_active = crate::controller::is_story_active(world, listener);
        if !story_active {
            enter_island(world, this, false);
        }
    }
    ItemResult::Done
}

/// The splash of `fall:::` and `fallFromHurricane:::`: the bird stops in the water at
/// `water_line_y` and plays `BirdSplash` (raw) or `BirdCookedSplash`.
fn splash(world: &mut World, this: Entity, water_line_y: f32) {
    {
        let mut bird = world.obj_mut::<BirdLogic>(this);
        bird.falling = false;
        bird.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let display_object = logic::display_object(world, this);
    {
        let mut display = display::object_mut(world, display_object);
        display.pos.y = water_line_y;
        display.rot.z = 0.0;
    }

    let cook_state = world.obj::<BirdLogic>(this).bird_cook_state;
    let behavior = if cook_state == COOK_STATE_RAW { "BirdSplash" } else { "BirdCookedSplash" };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[BirdLogic fall:::]` @ 0x100093b94
///
/// `[0] ignoreIsland` falls through the island floor.
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut bird = world.obj_mut::<BirdLogic>(this);
            if !bird.island_floor_collision {
                bird.on_island = false;
            }
            bird.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<BirdLogic>(this).falling = true;

        let ignore_island = call.bool("ignoreIsland");
        if ignore_island {
            world.obj_mut::<BirdLogic>(this).enable_island_floor_collision = false;
        }
    }

    let island_floor_collision = world.obj::<BirdLogic>(this).island_floor_collision;
    if island_floor_collision {
        world.obj_mut::<BirdLogic>(this).falling = false;
        return ItemResult::Done;
    }

    let display_object = logic::display_object(world, this);
    let ocean_enabled = is_ocean_enabled(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    let below_water_line = y_pos < WATER_LINE_Y;
    if !ocean_enabled || !below_water_line {
        return ItemResult::Pending;
    }

    splash(world, this, WATER_LINE_Y);
    ItemResult::Replaced
}

/// `-[BirdLogic fallFromHurricane:::]` @ 0x100093d44
pub fn seq_fall_from_hurricane(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<BirdLogic>(this).enable_island_floor_collision = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<BirdLogic>(this).falling = true;
    }

    let display_object = logic::display_object(world, this);
    let z_pos = display::object(world, display_object).pos.z;
    let behind_island = z_pos < ISLAND_Z_POS;
    let water_line_y = if behind_island { BEHIND_ISLAND_WATER_LINE_Y } else { WATER_LINE_Y };

    let ocean_enabled = is_ocean_enabled(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    let below_water_line = y_pos < water_line_y;
    if !ocean_enabled || !below_water_line {
        return ItemResult::Pending;
    }

    splash(world, this, water_line_y);
    ItemResult::Replaced
}

/// `-[BirdLogic drag:::]` @ 0x100093ec0
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<BirdLogic>(this).enable_island_floor_collision = false;
        move_to_drag_layer(world, this);
    }

    // The same offset as the fish's: the machine code reloads `pos.z` over the touch point's z.
    let display_object = logic::display_object(world, this);
    let touch_screen_pos = world.obj::<BirdLogic>(this).touch_screen_pos;
    let offset = fish_logic::drag_offset(world, display_object, touch_screen_pos);
    let length = length3(offset);
    if length != 0.0 {
        if offset.y != 0.0 {
            let z_rot = throwable_drag_z_rot(offset);
            display::object_mut(world, display_object).rot.z = z_rot;
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let spring_force = throwable_spring_force(offset, length, pos_vel, DRAG_STIFFNESS, DRAG_DAMPING);
        world.obj_mut::<BirdLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[BirdLogic isOceanEnabled]` @ 0x100094088
pub fn is_ocean_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
    match listener {
        Some(listener) => crate::controller::is_ocean_enabled(world, listener),
        None => false,
    }
}

/// `-[BirdLogic onBirdPooDie:]` @ 0x1000940a4
///
/// The bird held the dropping's last reference: it is deallocated here (its display object only
/// leaves the display tree).
pub fn on_bird_poo_die(world: &mut World, this: Entity, _bird_poo_die: Entity) {
    let bird_poo = world.obj::<BirdLogic>(this).bird_poo_logic;
    if let Some(bird_poo) = bird_poo {
        let bird_poo_display_object = logic::display_object(world, bird_poo);
        display::remove_from_display_group(world, bird_poo_display_object);
        logic::shutdown(world, bird_poo);
        logic::dealloc(world, bird_poo);
    }
    world.obj_mut::<BirdLogic>(this).bird_poo_logic = None;
}

/// `-[BirdLogic onBirdPooHitPygmy:]` @ 0x100094118
pub fn on_bird_poo_hit_pygmy(world: &mut World, this: Entity, _bird_poo_hit_pygmy: Entity) {
    world.obj_mut::<BirdLogic>(this).bird_poo_hit_pygmy = true;

    let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_bird_poo_hit_pygmy(world, listener, this);
    }
}

/// `-[BirdLogic onBirdPooMissPygmy:]` @ 0x10009414c
pub fn on_bird_poo_miss_pygmy(world: &mut World, this: Entity, _bird_poo_miss_pygmy: Entity) {
    let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_bird_poo_miss_pygmy(world, listener, this);
    }
}

/// `-[BirdLogic hitByLightning]` @ 0x100094170
///
/// Only a raw bird is struck: it drops its catch, ends a drag and falls burnt.
pub fn hit_by_lightning(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
    let bird_enabled = match listener {
        Some(listener) => crate::controller::is_bird_enabled(world, listener),
        None => false,
    };
    if !bird_enabled {
        return false;
    }
    let cook_state = world.obj::<BirdLogic>(this).bird_cook_state;
    if cook_state != COOK_STATE_RAW {
        return false;
    }

    drop_food(world, this);

    let dragging = world.obj::<BirdLogic>(this).dragging;
    if dragging {
        {
            let mut bird = world.obj_mut::<BirdLogic>(this);
            bird.dragging = false;
            bird.apply_spring = false;
        }
        if let Some(listener) = listener {
            crate::controller::on_bird_drag_complete(world, listener, this);
        }
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<BirdLogic>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    move_to_island_layer(world, this);
    world.obj_mut::<BirdLogic>(this).bird_cook_state = COOK_STATE_BURNT;

    let picked_up_by_hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let behavior = if picked_up_by_hurricane {
        "BirdHitByLightningHurricane"
    } else {
        "BirdHitByLightning"
    };
    logic::set_behavior(world, this, Some(behavior));
    true
}

/// `-[BirdLogic pickedUpByHurricane:]` @ 0x1000942a8
///
/// Like the fish, the bird does not remember the hurricane (`mPickedUpByHurricaneLogic` stays
/// nil).
pub fn picked_up_by_hurricane(world: &mut World, this: Entity, _picked_up_by_hurricane: Entity) -> bool {
    let bird = world.obj::<BirdLogic>(this);
    let already_picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let game_frame_running = logic::logic(world, this).game_frame_running;
    let can_be_picked_up = !bird.touch_began_inside
        && !already_picked_up
        && game_frame_running
        && !bird.dragging
        && !bird.falling
        && !bird.being_eaten
        && !bird.in_water;
    if !can_be_picked_up {
        return false;
    }

    drop_food(world, this);
    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.ignore_gravity = true;
        sprite.blown_by_hurricane = false;
        sprite.picked_up_by_hurricane = true;
    }
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let display_object = logic::display_object(world, this);
    {
        let mut display = display::object_mut(world, display_object);
        display.rot.z = 0.0;
        display.is_selectable = false;
    }
    move_to_island_layer(world, this);

    let cook_state = world.obj::<BirdLogic>(this).bird_cook_state;
    let behavior = if cook_state == COOK_STATE_RAW {
        "BirdPickedUpByHurricane"
    } else {
        "BirdCookedPickedUpByHurricane"
    };
    logic::set_behavior(world, this, Some(behavior));
    true
}

/// `-[BirdLogic droppedByHurricane:]` @ 0x100094420
pub fn dropped_by_hurricane(world: &mut World, this: Entity, _dropped_by_hurricane: Entity) {
    let picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    if !picked_up {
        return;
    }

    world.obj_mut::<BirdLogic>(this).falling = true;
    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.ignore_gravity = false;
        sprite.blown_by_hurricane = false;
        sprite.picked_up_by_hurricane = false;
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    let z_pos = display::object(world, display_object).pos.z;
    if z_pos < ISLAND_Z_POS {
        move_to_behind_island_layer(world, this);
    }
    logic::set_behavior(world, this, Some("BirdDroppedByHurricane"));
}

/// `-[BirdLogic islandMinXPos]` @ 0x100094504
pub fn island_min_x_pos(world: &mut World, this: Entity) -> f32 {
    let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
    match listener {
        Some(listener) => crate::controller::island_min_x_pos(world, listener),
        None => 0.0,
    }
}

/// `-[BirdLogic islandMaxXPos]` @ 0x100094520
pub fn island_max_x_pos(world: &mut World, this: Entity) -> f32 {
    let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
    match listener {
        Some(listener) => crate::controller::island_max_x_pos(world, listener),
        None => 0.0,
    }
}

/// `-[BirdLogic pygmyLogicArray:]` @ 0x10009453c
///
/// Forwards to the listener with `self` as the argument (the caller's is ignored).
pub fn pygmy_logic_array(world: &mut World, this: Entity, _pygmy_logic_array: Entity) -> Vec<Entity> {
    let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
    let Some(listener) = listener else {
        return Vec::new();
    };
    crate::controller::pygmy_logic_array(world, listener, this)
}

/// `-[BirdLogic startDrag:]` @ 0x100094560
pub fn start_drag(world: &mut World, this: Entity, touch: Entity) {
    let bird = world.obj::<BirdLogic>(this);
    if bird.falling || bird.in_water || bird.active_touch.is_some() {
        return;
    }

    world.obj_mut::<BirdLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut bird = world.obj_mut::<BirdLogic>(this);
        bird.dragging = true;
        bird.apply_spring = true;
        bird.touch_began_inside = false;
        bird.touch_screen_pos = touch_pos;
    }

    let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_bird_drag(world, listener, this);
    }

    let cook_state = world.obj::<BirdLogic>(this).bird_cook_state;
    let behavior = if cook_state == COOK_STATE_RAW { "BirdDrag" } else { "BirdCookedDrag" };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[BirdLogic touchBeganInside:allTouches:]` @ 0x100094670
///
/// A cooked bird is dragged at once. A tap on a raw bird aborts a dive, makes it drop its catch,
/// or arms a dropping for the release.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let bird = world.obj::<BirdLogic>(this);
    if bird.in_water || bird.falling {
        return;
    }
    let cook_state = bird.bird_cook_state;
    let diving_for_food = bird.diving_for_food;
    let carrying = bird.carrying_fish_logic.is_some() || bird.carrying_pygmy_logic.is_some();

    if cook_state != COOK_STATE_RAW {
        start_drag(world, this, touch);
        return;
    }

    if diving_for_food {
        world.obj_mut::<BirdLogic>(this).diving_for_food = false;
    } else if carrying {
        drop_food(world, this);
    } else {
        world.obj_mut::<BirdLogic>(this).touch_began_inside = true;
        return;
    }
    logic::set_behavior(world, this, Some("BirdAbortDive"));
}

/// `-[BirdLogic touchBeganOutside:allTouches:]` @ 0x10009474c
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BirdLogic>(this).active_touch;
    if active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<BirdLogic>(this).touch_screen_pos = touch_pos;
}

/// `-[BirdLogic touchMovedEntered:allTouches:]` @ 0x10009479c
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[BirdLogic touchMovedExited:allTouches:]` @ 0x1000947a0
///
/// Leaving the bird with an armed tap drags it instead.
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let bird = world.obj::<BirdLogic>(this);
    let own_touch = bird.active_touch.is_none() || bird.active_touch == Some(touch);
    if own_touch && bird.touch_began_inside {
        start_drag(world, this, touch);
    }
}

/// `-[BirdLogic touchMovedInside:allTouches:]` @ 0x1000947dc
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[BirdLogic touchMovedOutside:allTouches:]` @ 0x1000947ec
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let bird = world.obj::<BirdLogic>(this);
    let own_touch = bird.active_touch.is_none() || bird.active_touch == Some(touch);
    if own_touch && bird.dragging {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<BirdLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// The dropping of `touchEndedInside:allTouches:`: a `BirdPoo` model at the bird, just after the
/// lava marker, falling forward at `xScale`·150 while the bird plays `BirdMakeFeces`.
fn make_bird_poo(world: &mut World, this: Entity) {
    let bird_poo_model = display::multi_model::spawn(world, Some("BirdPoo"));
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    {
        let mut bird_poo_object = display::object_mut(world, bird_poo_model);
        bird_poo_object.pos.x = pos.x;
        bird_poo_object.pos.y = pos.y;
        bird_poo_object.pos.z = ISLAND_Z_POS;
    }
    let marker = pg_engine::library::display_marker(world, "LavaDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(bird_poo_model));
    }

    let bird_poo = bird_poo_logic::spawn(world, bird_poo_model);
    world.obj_mut::<BirdLogic>(this).bird_poo_logic = Some(bird_poo);
    bird_poo_logic::set_bird_poo_logic_listener(world, bird_poo, this);

    let x_scale = display::object(world, display_object).scale.x;
    logic::logic_mut(world, bird_poo).pos_vel.x = x_scale * BIRD_POO_X_SPEED;
    logic::set_behavior(world, this, Some("BirdMakeFeces"));
}

/// `-[BirdLogic touchEndedInside:allTouches:]` @ 0x100094854
///
/// Releasing an armed tap drops a dropping (one at a time); releasing a drag lets the bird fall.
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BirdLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let touch_began_inside = world.obj::<BirdLogic>(this).touch_began_inside;
    if touch_began_inside {
        world.obj_mut::<BirdLogic>(this).touch_began_inside = false;
        let bird_poo = world.obj::<BirdLogic>(this).bird_poo_logic;
        if bird_poo.is_none() {
            make_bird_poo(world, this);
        }
    }

    let dragging = world.obj::<BirdLogic>(this).dragging;
    if dragging {
        {
            let mut bird = world.obj_mut::<BirdLogic>(this);
            bird.falling = true;
            bird.dragging = false;
            bird.apply_spring = false;
        }
        move_to_island_layer(world, this);

        let listener = world.obj::<BirdLogic>(this).bird_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_bird_drag_complete(world, listener, this);
        }

        let cook_state = world.obj::<BirdLogic>(this).bird_cook_state;
        let behavior = if cook_state == COOK_STATE_RAW { "BirdFallIgnoreIsland" } else { "BirdCookedFall" };
        logic::set_behavior(world, this, Some(behavior));
    }

    world.obj_mut::<BirdLogic>(this).active_touch = None;
}

/// `-[BirdLogic touchEndedOutside:allTouches:]` @ 0x100094af4
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
