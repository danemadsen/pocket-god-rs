//! `FishLogic` (superclass `SpriteLogic`).
//!
//! The fish (`docs/re/logic-catalog.md` §4.3). It swims along the surface until it bites an
//! idle fishing bob, is reeled in and flung onto the island, and is then a throwable of §2.2
//! (slide friction 0.04, floor at y 76) that cooks over the bonfire, is eaten by pygmies,
//! carried off by birds, or dropped into the igloo or the ice hole.

use crate::prelude::*;

use crate::logic::ice_island::{ice_hole_logic, igloo_logic};
use crate::logic::island_core::bon_fire_logic;
use crate::logic::island_core::coconut_logic::{
    ISLAND_Z_POS, THROWABLE_GRAVITY_FORCE, apply_throwable_slide_friction, bounce_throwable_velocity,
    integrate_throwable, is_island_tipped_over, is_throwable_out_of_range, throwable_drag_z_rot,
    throwable_pos_force, throwable_spring_force, throwable_tilted_gravity_force,
};
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::ocean_fishing::fishing_bob_logic;
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::length3;
use pg_engine::touch::BcTouch;
use pg_engine::view::BcView;

/// `FishLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct FishLogic {
    /// `mFishLogicListener`: `@"<FishLogicListener>"`
    pub fish_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mEatingPygmyLogic`: `@"PygmyLogic"`
    pub eating_pygmy_logic: Option<Entity>,

    /// `mCaughtBirdLogic`: `@"BirdLogic"`
    pub caught_bird_logic: Option<Entity>,

    /// `mTiltState`: `i`
    pub tilt_state: i32,

    /// `mSlideFriction`: `f`
    pub slide_friction: f32,

    /// `mSpringDelta`: `f`
    pub spring_delta: f32,

    /// `mCookTime`: `f`
    pub cook_time: f32,

    /// `mFishCookState`: `i`
    pub fish_cook_state: i32,

    /// `mFishingBobLogic`: `@"FishingBobLogic"`
    pub fishing_bob_logic: Option<Entity>,

    /// `mbInIgloo`: `B`
    pub in_igloo: bool,

    /// `mIglooLogic`: `@"IglooLogic"`
    pub igloo_logic: Option<Entity>,

    /// `mbOnIsland`: `B`
    pub on_island: bool,

    /// `mbEnableIslandFloorCollision`: `B`
    pub enable_island_floor_collision: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mbDontHitUntilBounce`: `B`
    pub dont_hit_until_bounce: bool,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbBeingEaten`: `B`
    pub being_eaten: bool,

    /// `mbCaughtByBird`: `B`
    pub caught_by_bird: bool,

    /// `mbInWater`: `B`
    pub in_water: bool,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbFalling`: `B`
    pub falling: bool,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,
}

pub const FISH_LOGIC: Class = Class {
    name: "FishLogic",
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

/// `mFishCookState`: raw, cooked or burnt.
const COOK_STATE_RAW: i32 = 0;
const COOK_STATE_COOKED: i32 = 1;
const COOK_STATE_BURNT: i32 = 2;

/// `mSlideFriction` (`0x3d23d70a`).
const SLIDE_FRICTION: f32 = 0.04;

/// The island floor (`setYPos:` in `onGameFrame:`).
const FLOOR_Y: f32 = 76.0;

/// A floor contact this fast upwards (after the contact's damping) bounces.
const BOUNCE_SPEED: f32 = 100.0;

/// The surface: `enterIsland:` puts the fish there, and `slide:::`, `fall:::` and (in front of
/// the island) `fallFromHurricane:::` splash below it.
const WATER_LINE_Y: f32 = 20.0;

/// `fallFromHurricane:::` splashes below this y behind the island.
const BEHIND_ISLAND_WATER_LINE_Y: f32 = 86.0;

/// `swim:::`: the speed, the start x off either side of the screen and the bob's bite range.
const SWIM_SPEED: f32 = 50.0;
const SWIM_START_LEFT_X_POS: f32 = -40.0;
const SWIM_START_RIGHT_X_POS: f32 = 520.0;
const HOOK_REACH: f32 = 16.0;

/// The screen width the swim leaves.
const SCREEN_WIDTH: f32 = 480.0;

/// `catch` (`DAT_10029d378`): flung up and toward the island centre, indexed by
/// `x > islandCenterXPos`.
const CATCH_X_VELS: [f32; 2] = [200.0, -200.0];
const CATCH_Y_VEL: f32 = 1000.0;

/// `tossFromIgloo` and `dropWhileEating`: the fish hops up.
const DROP_Y_VEL: f32 = 400.0;

/// `thrownByCharlie:`: thrown away from Charlie's facing, upwards.
const CHARLIE_THROW_X_VEL: f32 = -200.0;
const CHARLIE_THROW_Y_VEL: f32 = 600.0;

/// The spring of `drag:::`.
const DRAG_STIFFNESS: f32 = 100.0;
const DRAG_DAMPING: f32 = 8.0;

/// `checkPygmyCollision`: faster than this, the fish hits a pygmy whose point 31 above its
/// origin lies within 20 points (squared distance 400).
const PYGMY_HIT_MIN_SPEED: f32 = 200.0;
const PYGMY_HIT_DISTANCE_SQUARED: f32 = 400.0;
const PYGMY_HIT_Y_OFFSET: f32 = 31.0;

/// Cooking in `onGameFrame:`: within 20 of the fire's x, and 30–130 above it unless lying on the
/// island; cooked after 2 s, burnt after 4 s.
const COOK_X_REACH: f32 = 20.0;
const COOK_MIN_HEIGHT: f32 = 30.0;
const COOK_MAX_HEIGHT: f32 = 130.0;
const COOKED_TIME: f32 = 2.0;
const BURNT_TIME: f32 = 4.0;

/// `checkForIglooDuringFall`: the igloo's door, 48 above its origin, takes a fish falling within
/// 30 of it on both axes.
const IGLOO_DOOR_Y_OFFSET: f32 = 48.0;
const IGLOO_DOOR_REACH: f32 = 30.0;

/// `checkForIceHoleDuringFall`: the ice hole takes a fish falling within 30 across and 10 up.
const ICE_HOLE_X_REACH: f32 = 30.0;
const ICE_HOLE_Y_REACH: f32 = 10.0;

/// Sequence-item handlers implemented by `FishLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "slide" => seq_slide(world, this, call),
        "swim" => seq_swim(world, this, call),
        "drag" => seq_drag(world, this, call),
        "fall" => seq_fall(world, this, call),
        "fallFromHurricane" => seq_fall_from_hurricane(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[FishLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&FISH_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[FishLogic initWithDisplayObject:]` @ 0x100010ce4
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(FishLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut fish = world.obj_mut::<FishLogic>(this);
        fish.in_water = true;
        fish.tilt_state = 0;
        fish.slide_friction = SLIDE_FRICTION;
    }

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[FishLogic setFishLogicListener:]` @ 0x100010d84
pub fn set_fish_logic_listener(world: &mut World, this: Entity, fish_logic_listener: Entity) {
    world.obj_mut::<FishLogic>(this).fish_logic_listener = Some(fish_logic_listener);
}

/// Re-inserts the fish's display object after the display marker `marker_id`.
fn insert_after_marker(world: &mut World, this: Entity, marker_id: &str) {
    let marker = pg_engine::library::display_marker(world, marker_id);
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[FishLogic moveToWaterLayer]` @ 0x100010e44
///
/// The water layer is the drag layer.
pub fn move_to_water_layer(world: &mut World, this: Entity) {
    insert_after_marker(world, this, "DragDisplayMarker");
}

/// `-[FishLogic moveToDragLayer]` @ 0x100010eac
pub fn move_to_drag_layer(world: &mut World, this: Entity) {
    insert_after_marker(world, this, "DragDisplayMarker");
}

/// `-[FishLogic moveToIslandLayer]` @ 0x100010f14
pub fn move_to_island_layer(world: &mut World, this: Entity) {
    insert_after_marker(world, this, "SmallObjectsMarker");
}

/// `-[FishLogic moveToBehindIslandLayer]` @ 0x100010f7c
pub fn move_to_behind_island_layer(world: &mut World, this: Entity) {
    let island_display_group = pg_engine::library::display_group(world, "IslandDisplayGroup");
    let marker_index = match island_display_group {
        Some(island_display_group) => display::group::get_display_object_index(world, island_display_group, "IslandDisplayMarker"),
        None => 0,
    };

    let display_object = logic::display_object(world, this);
    display::move_to_display_group_at_index(world, display_object, island_display_group, marker_index);
}

/// `-[FishLogic enterIsland:]` @ 0x100011008
///
/// The fish ignores the startup flag: it always starts in the water.
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    let active = world.obj::<FishLogic>(this).active;
    if active {
        return;
    }
    world.obj_mut::<FishLogic>(this).active = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let display_object = logic::display_object(world, this);
    {
        let mut display = display::object_mut(world, display_object);
        display.pos.y = WATER_LINE_Y;
        display.pos.z = ISLAND_Z_POS;
        display.rot.z = 0.0;
        display.scale.x = 1.0;
        display.scale.y = 1.0;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    move_to_water_layer(world, this);

    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("FishIdle"));
}

/// `-[FishLogic unhook]` @ 0x100011138
pub fn unhook(world: &mut World, this: Entity) {
    world.obj_mut::<FishLogic>(this).fishing_bob_logic = None;
    logic::set_behavior(world, this, Some("FishUnhook"));
}

/// `-[FishLogic catch]` @ 0x10001115c
pub fn catch(world: &mut World, this: Entity) {
    {
        let mut fish = world.obj_mut::<FishLogic>(this);
        fish.in_water = false;
        fish.falling = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut fish = world.obj_mut::<FishLogic>(this);
        fish.dont_hit_until_bounce = true;
        fish.fishing_bob_logic = None;
        fish.tilt_state = 0;
    }
    logic::logic_mut(world, this).pos_vel.y = CATCH_Y_VEL;

    let display_object = logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;
    let listener = world.obj::<FishLogic>(this).fish_logic_listener;
    let island_center_x_pos = match listener {
        Some(listener) => crate::controller::island_center_x_pos(world, listener),
        None => 0.0,
    };
    let right_of_center = x_pos > island_center_x_pos;
    logic::logic_mut(world, this).pos_vel.x = CATCH_X_VELS[usize::from(right_of_center)];

    move_to_island_layer(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("FishCatch"));
}

/// `-[FishLogic isEdible]` @ 0x10001128c
pub fn is_edible(world: &mut World, this: Entity) -> bool {
    let fish = world.obj::<FishLogic>(this);
    fish.on_island && !fish.being_eaten && !fish.caught_by_bird && !fish.in_igloo && !fish.touch_began_inside
}

/// `-[FishLogic isBurnt]` @ 0x1000112ec
pub fn is_burnt(world: &mut World, this: Entity) -> bool {
    world.obj::<FishLogic>(this).fish_cook_state == COOK_STATE_BURNT
}

/// `-[FishLogic isCooked]` @ 0x100011304
pub fn is_cooked(world: &mut World, this: Entity) -> bool {
    world.obj::<FishLogic>(this).fish_cook_state == COOK_STATE_COOKED
}

/// `-[FishLogic canBeCaughtByBird:]` @ 0x10001131c
pub fn can_be_caught_by_bird(world: &mut World, this: Entity, _can_be_caught_by_bird: Entity) -> bool {
    let fish = world.obj::<FishLogic>(this);
    (fish.on_island || fish.being_eaten) && !fish.in_igloo && !fish.touch_began_inside
}

/// `-[FishLogic caughtByBird:]` @ 0x10001136c
pub fn caught_by_bird(world: &mut World, this: Entity, caught_by_bird: Entity) {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut fish = world.obj_mut::<FishLogic>(this);
    fish.caught_bird_logic = Some(caught_by_bird);
    fish.caught_by_bird = true;
}

/// `-[FishLogic caughtByBirdComplete:]` @ 0x10001139c
pub fn caught_by_bird_complete(world: &mut World, this: Entity, _caught_by_bird_complete: Entity) {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let mut fish = world.obj_mut::<FishLogic>(this);
    fish.caught_bird_logic = None;
    fish.caught_by_bird = false;
}

/// `-[FishLogic droppedByBird:]` @ 0x1000113c8
pub fn dropped_by_bird(world: &mut World, this: Entity, _dropped_by_bird: Entity) {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut fish = world.obj_mut::<FishLogic>(this);
        fish.caught_bird_logic = None;
        fish.caught_by_bird = false;
    }
    logic::set_behavior(world, this, Some("FishFall"));
}

/// `-[FishLogic isBeingEaten]` @ 0x100011408
pub fn is_being_eaten(world: &mut World, this: Entity) -> bool {
    world.obj::<FishLogic>(this).being_eaten
}

/// `-[FishLogic eatingPygmyLogic]` @ 0x100011418
pub fn eating_pygmy_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<FishLogic>(this).eating_pygmy_logic
}

/// `-[FishLogic tossFromIgloo]` @ 0x100011428
pub fn toss_from_igloo(world: &mut World, this: Entity) {
    {
        let mut fish = world.obj_mut::<FishLogic>(this);
        fish.in_igloo = false;
        fish.igloo_logic = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut fish = world.obj_mut::<FishLogic>(this);
        fish.dont_hit_until_bounce = true;
        fish.enable_island_floor_collision = true;
    }
    logic::logic_mut(world, this).pos_vel.y = DROP_Y_VEL;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("FishDropWhileEating"));
}

/// The behaviour of a fish dropped while being eaten, by cook state (anything not raw or cooked
/// counts as burnt).
fn drop_while_eating_behavior(cook_state: i32) -> &'static str {
    if cook_state == COOK_STATE_RAW {
        return "FishDropWhileEating";
    }
    if cook_state == COOK_STATE_COOKED {
        return "FishCookedDropWhileEating";
    }
    "FishBurntDropWhileEating"
}

/// `-[FishLogic thrownByCharlie:]` @ 0x1000114dc
pub fn thrown_by_charlie(world: &mut World, this: Entity, thrown_by_charlie: Entity) {
    {
        let mut fish = world.obj_mut::<FishLogic>(this);
        fish.eating_pygmy_logic = None;
        fish.being_eaten = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<FishLogic>(this).dont_hit_until_bounce = true;

    let charlie_display_object = logic::display_object(world, thrown_by_charlie);
    let charlie_x_scale = display::object(world, charlie_display_object).scale.x;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = charlie_x_scale * CHARLIE_THROW_X_VEL;
        state.pos_vel.y = CHARLIE_THROW_Y_VEL;
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;

    let cook_state = world.obj::<FishLogic>(this).fish_cook_state;
    let behavior = drop_while_eating_behavior(cook_state);
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[FishLogic dropWhileEating]` @ 0x1000115f8
pub fn drop_while_eating(world: &mut World, this: Entity) {
    {
        let mut fish = world.obj_mut::<FishLogic>(this);
        fish.eating_pygmy_logic = None;
        fish.being_eaten = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<FishLogic>(this).dont_hit_until_bounce = true;
    logic::logic_mut(world, this).pos_vel.y = DROP_Y_VEL;

    let cook_state = world.obj::<FishLogic>(this).fish_cook_state;
    let behavior = drop_while_eating_behavior(cook_state);
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[FishLogic startEating:]` @ 0x10001169c
pub fn start_eating(world: &mut World, this: Entity, eating: Entity) {
    world.obj_mut::<FishLogic>(this).being_eaten = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<FishLogic>(this).eating_pygmy_logic = Some(eating);
}

/// `-[FishLogic pickUpForEating]` @ 0x1000116cc
pub fn pick_up_for_eating(world: &mut World, this: Entity) {
    let cook_state = world.obj::<FishLogic>(this).fish_cook_state;
    let behavior = if cook_state != COOK_STATE_RAW { "FishCookedEaten" } else { "FishEaten" };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[FishLogic finishEating]` @ 0x100011700
pub fn finish_eating(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("FishDie"));
}

/// `-[FishLogic finishEatingInIgloo]` @ 0x100011718
pub fn finish_eating_in_igloo(world: &mut World, this: Entity) {
    {
        let mut fish = world.obj_mut::<FishLogic>(this);
        fish.in_igloo = false;
        fish.igloo_logic = None;
    }
    logic::set_behavior(world, this, Some("FishDie"));
}

/// `-[FishLogic checkPygmyCollision]` @ 0x100011748
///
/// The first pygmy in reach that takes the hit ends the check; the fish itself flies on.
pub fn check_pygmy_collision(world: &mut World, this: Entity) {
    let fish = world.obj::<FishLogic>(this);
    if fish.being_eaten || fish.dont_hit_until_bounce || fish.caught_by_bird {
        return;
    }

    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    let speed = length3(logic::logic(world, this).pos_vel);

    let listener = world.obj::<FishLogic>(this).fish_logic_listener;
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

        let hit = crate::pygmy::hit_by_fish(world, pygmy, this);
        if hit {
            return;
        }
    }
}

/// `-[FishLogic isAccelEnabled]` @ 0x1000119b0
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<FishLogic>(this).fish_logic_listener;
    let gravity_enabled = match listener {
        Some(listener) => crate::controller::is_gravity_enabled(world, listener),
        None => false,
    };
    if !gravity_enabled {
        return false;
    }
    sprite_logic::is_accel_enabled(world, this)
}

/// `-[FishLogic onGameFrame:]` @ 0x100011a20
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let display_object = logic::display_object(world, this);
    let previous_y = display::object(world, display_object).pos.y;

    let accel_enabled = is_accel_enabled(world, this);
    if accel_enabled {
        let gravity_force = throwable_tilted_gravity_force(world);
        world.obj_mut::<SpriteLogic>(this).gravity_force = gravity_force;
        refresh_tilt_state(world, this);
    } else {
        world.obj_mut::<SpriteLogic>(this).gravity_force = THROWABLE_GRAVITY_FORCE;
        undo_tilt_state(world, this);
    }

    let fish = world.obj::<FishLogic>(this);
    let tilt_state = fish.tilt_state;
    let slide_friction = fish.slide_friction;
    let spring_force = fish.spring_force;
    let apply_spring = fish.apply_spring;

    let mut sprite = world.obj_mut::<SpriteLogic>(this);
    apply_throwable_slide_friction(&mut sprite.gravity_force, tilt_state, slide_friction);
    let gravity_force = sprite.gravity_force;
    let ignore_gravity = sprite.ignore_gravity;

    let pos_force = throwable_pos_force(gravity_force, ignore_gravity, spring_force, apply_spring);
    logic::logic_mut(world, this).pos_force = pos_force;
    integrate_throwable(world, this, pos_force, dt);

    // A hooked fish hangs under the bob.
    let fishing_bob = world.obj::<FishLogic>(this).fishing_bob_logic;
    if let Some(fishing_bob) = fishing_bob {
        let bob_display_object = logic::display_object(world, fishing_bob);
        let bob_x_pos = display::object(world, bob_display_object).pos.x;
        display::object_mut(world, display_object).pos.x = bob_x_pos;
    }

    world.obj_mut::<FishLogic>(this).island_floor_collision = false;
    let listener = world.obj::<FishLogic>(this).fish_logic_listener;
    let mut island_min_x_pos = 0.0;
    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
        island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
    }

    let pos = display::object(world, display_object).pos;
    let enable_island_floor_collision = world.obj::<FishLogic>(this).enable_island_floor_collision;
    let missed_floor = FLOOR_Y <= pos.y
        || pos.x < island_min_x_pos
        || island_max_x_pos < pos.x
        || previous_y < FLOOR_Y
        || !enable_island_floor_collision;
    if missed_floor {
        world.obj_mut::<FishLogic>(this).on_island = false;
    } else {
        land_on_island_floor(world, this, display_object);
    }

    let y_pos = display::object(world, display_object).pos.y;
    if is_throwable_out_of_range(y_pos) {
        logic::set_behavior(world, this, Some("FishDie"));
    }

    cook_over_bon_fire(world, this, dt);
    check_pygmy_collision(world, this);
}

/// The island floor contact of `onGameFrame:`: on the first contact the fish bounces or lands
/// (with the behaviours of its cook state); every contact puts it on the floor.
fn land_on_island_floor(world: &mut World, this: Entity, display_object: Entity) {
    let on_island = world.obj::<FishLogic>(this).on_island;
    let mut pos_vel = logic::logic(world, this).pos_vel;
    bounce_throwable_velocity(&mut pos_vel, on_island);
    logic::logic_mut(world, this).pos_vel = pos_vel;

    let cook_state = world.obj::<FishLogic>(this).fish_cook_state;
    if BOUNCE_SPEED <= pos_vel.y {
        let behavior = match cook_state {
            COOK_STATE_BURNT => Some("FishBurntBounce"),
            COOK_STATE_COOKED => Some("FishCookedBounce"),
            COOK_STATE_RAW => Some("FishBounce"),
            _ => None,
        };
        if let Some(behavior) = behavior {
            logic::set_behavior(world, this, Some(behavior));
        }
    } else {
        logic::logic_mut(world, this).pos_vel.y = 0.0;
        if !on_island {
            world.obj_mut::<FishLogic>(this).on_island = true;
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            world.obj_mut::<FishLogic>(this).island_floor_collision = true;

            let behavior = match cook_state {
                COOK_STATE_BURNT => Some("FishBurntOnGround"),
                COOK_STATE_COOKED => Some("FishCookedOnGround"),
                COOK_STATE_RAW => Some("FishOnGround"),
                _ => None,
            };
            if let Some(behavior) = behavior {
                logic::set_behavior(world, this, Some(behavior));
            }
        }
        display::object_mut(world, display_object).rot.z = 0.0;
    }

    world.obj_mut::<FishLogic>(this).dont_hit_until_bounce = false;
    display::object_mut(world, display_object).pos.y = FLOOR_Y;
}

/// The cooking of `onGameFrame:`: a dragged fish held over a burning bonfire, or one lying next
/// to it on the island, cooks after 2 s and burns after 4 s (`mCookTime` keeps counting).
fn cook_over_bon_fire(world: &mut World, this: Entity, dt: f32) {
    let fish = world.obj::<FishLogic>(this);
    let near_fire_possible = fish.dragging || fish.on_island;
    let listener = fish.fish_logic_listener;
    if !near_fire_possible {
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
    let on_island = world.obj::<FishLogic>(this).on_island;
    if !within_reach || !(above_fire || on_island) {
        return;
    }

    let cook_time = world.obj::<FishLogic>(this).cook_time + dt;
    world.obj_mut::<FishLogic>(this).cook_time = cook_time;

    let fish = world.obj::<FishLogic>(this);
    let cook_state = fish.fish_cook_state;
    let dragging = fish.dragging;
    if COOKED_TIME < cook_time && cook_state == COOK_STATE_RAW {
        world.obj_mut::<FishLogic>(this).fish_cook_state = COOK_STATE_COOKED;
        let behavior = if dragging { "FishCook" } else { "FishOnGroundCook" };
        logic::set_behavior(world, this, Some(behavior));
        return;
    }
    if BURNT_TIME < cook_time && cook_state == COOK_STATE_COOKED {
        world.obj_mut::<FishLogic>(this).fish_cook_state = COOK_STATE_BURNT;
        let behavior = if dragging { "FishBurn" } else { "FishOnGroundBurn" };
        logic::set_behavior(world, this, Some(behavior));
    }
}

/// The tilt state for `BCView.rawAccelRotDegrees.z`: 0 beyond ±130° (also for NaN), 1 between
/// ±90° and ±130°, 2 within ±90°.
fn raw_roll_tilt_state(world: &World) -> i32 {
    let z = world.resource::<BcView>().raw_accel_rot_degrees.z;
    let within_130 = -130.0 <= z && z <= 130.0;
    if !within_130 {
        return 0;
    }
    let within_90 = z <= 90.0 && -90.0 <= z;
    if within_90 { 2 } else { 1 }
}

/// `-[FishLogic refreshTiltState]` @ 0x100012190
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let fish = world.obj::<FishLogic>(this);
    if fish.dragging || fish.falling || !fish.on_island {
        return;
    }

    let tilt_state = raw_roll_tilt_state(world);
    let current_tilt_state = world.obj::<FishLogic>(this).tilt_state;
    if tilt_state != current_tilt_state {
        if tilt_state == 0 {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            logic::set_behavior(world, this, Some("FishStopSlide"));
            world.obj_mut::<FishLogic>(this).tilt_state = 0;
        } else if tilt_state == 1 {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            logic::set_behavior(world, this, Some("FishTiltSlide"));
            world.obj_mut::<FishLogic>(this).tilt_state = 1;
        }
    }

    if is_island_tipped_over(world) {
        {
            let mut fish = world.obj_mut::<FishLogic>(this);
            fish.falling = true;
            fish.tilt_state = 0;
        }
        logic::set_behavior(world, this, Some("FishFall"));
    }
}

/// `-[FishLogic undoTiltState]` @ 0x100012370
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    let fish = world.obj::<FishLogic>(this);
    if fish.dragging || fish.falling || !fish.on_island {
        return;
    }

    if fish.tilt_state == 1 {
        logic::logic_mut(world, this).pos_vel.x = 0.0;
        logic::set_behavior(world, this, Some("FishStopSlide"));
    }
    world.obj_mut::<FishLogic>(this).tilt_state = 0;
}

/// `[mFishLogicListener isOceanEnabled]` (NO without a listener).
fn is_ocean_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<FishLogic>(this).fish_logic_listener;
    match listener {
        Some(listener) => crate::controller::is_ocean_enabled(world, listener),
        None => false,
    }
}

/// The splash of `slide:::`, `fall:::` and `fallFromHurricane:::`: the fish stops in the water
/// and plays `FishSplash`.
fn splash(world: &mut World, this: Entity) {
    {
        let mut fish = world.obj_mut::<FishLogic>(this);
        fish.falling = false;
        fish.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<FishLogic>(this).tilt_state = 0;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
    logic::set_behavior(world, this, Some("FishSplash"));
}

/// `-[FishLogic slide:::]` @ 0x100012404
pub fn seq_slide(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<FishLogic>(this).enable_island_floor_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let display_object = logic::display_object(world, this);
    let ocean_enabled = is_ocean_enabled(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    if !ocean_enabled || WATER_LINE_Y <= y_pos {
        return ItemResult::Pending;
    }

    splash(world, this);
    ItemResult::Replaced
}

/// `-[FishLogic swim:::]` @ 0x10001252c
///
/// `[2] noReset` keeps the x the fish has (an unhooked fish swims on from the bob).
pub fn seq_swim(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    // The bounds are measured before the first frame's turn.
    let display_object = logic::display_object(world, this);
    let min_pos = display::min_pos(world, display_object);
    let max_pos = display::max_pos(world, display_object);

    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        world.obj_mut::<FishLogic>(this).enable_island_floor_collision = false;
        let no_reset = call.bool("noReset");

        let random = world.resource_mut::<Random>().arc4random();
        let swims_left = (random & 1) == 0;
        if swims_left {
            display::object_mut(world, display_object).scale.x = -1.0;
            logic::logic_mut(world, this).pos_vel.x = -SWIM_SPEED;
            if !no_reset {
                display::object_mut(world, display_object).pos.x = SWIM_START_RIGHT_X_POS;
            }
        } else {
            display::object_mut(world, display_object).scale.x = 1.0;
            logic::logic_mut(world, this).pos_vel.x = SWIM_SPEED;
            if !no_reset {
                display::object_mut(world, display_object).pos.x = SWIM_START_LEFT_X_POS;
            }
        }
    }

    let ocean_enabled = is_ocean_enabled(world, this);
    if !ocean_enabled {
        logic::set_behavior(world, this, Some("FishIdle"));
        return ItemResult::Replaced;
    }

    // An idle bob within reach hooks the fish.
    let listener = world.obj::<FishLogic>(this).fish_logic_listener;
    let fishing_bob = match listener {
        Some(listener) => crate::controller::fishing_bob_logic(world, listener, this),
        None => None,
    };
    if let Some(fishing_bob) = fishing_bob
        && fishing_bob_logic::is_unhooked(world, fishing_bob)
    {
        let x_pos = display::object(world, display_object).pos.x;
        let bob_display_object = logic::display_object(world, fishing_bob);
        let bob_x_pos = display::object(world, bob_display_object).pos.x;
        if (x_pos - bob_x_pos).abs() <= HOOK_REACH {
            {
                let mut state = logic::logic_mut(world, this);
                state.pos_vel.x = 0.0;
                state.pos_vel.y = 0.0;
            }
            world.obj_mut::<FishLogic>(this).fishing_bob_logic = Some(fishing_bob);
            fishing_bob_logic::hook_fish(world, fishing_bob, this);
            logic::set_behavior(world, this, Some("FishGetHooked"));
            return ItemResult::Replaced;
        }
    }

    // Out of the screen on the side it swims to: the swim ends.
    let x_vel = logic::logic(world, this).pos_vel.x;
    let left_off_left = x_vel < 0.0 && max_pos.x < 0.0;
    let left_off_right = 0.0 < x_vel && SCREEN_WIDTH < min_pos.x;
    if left_off_left || left_off_right {
        logic::logic_mut(world, this).pos_vel.x = 0.0;
        return ItemResult::Done;
    }
    ItemResult::Pending
}

/// The offset of `drag:::` (the fish's and the fishing pole's): the display position minus the
/// touch point in the display group's space (`viewToDisplayObject:`; a nil group answers the
/// zero point). The machine code reloads `pos.z` over the touch point's z, so the offset's z is
/// `pos.z − pos.z`; w is `pos.w`.
pub(crate) fn drag_offset(world: &World, display_object: Entity, touch_screen_pos: Vec4) -> Vec4 {
    let display = display::object(world, display_object);
    let pos = display.pos;
    let display_group = display.display_group;

    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    let offset_x = pos.x - touch_pos.x;
    let offset_y = pos.y - touch_pos.y;
    let offset_z = pos.z - pos.z;
    Vec4::new(offset_x, offset_y, offset_z, pos.w)
}

/// `-[FishLogic drag:::]` @ 0x1000127c8
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<FishLogic>(this).enable_island_floor_collision = false;
        move_to_drag_layer(world, this);
    }

    let display_object = logic::display_object(world, this);
    let touch_screen_pos = world.obj::<FishLogic>(this).touch_screen_pos;
    let offset = drag_offset(world, display_object, touch_screen_pos);
    let length = length3(offset);
    if length != 0.0 {
        if offset.y != 0.0 {
            let z_rot = throwable_drag_z_rot(offset);
            display::object_mut(world, display_object).rot.z = z_rot;
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let spring_force = throwable_spring_force(offset, length, pos_vel, DRAG_STIFFNESS, DRAG_DAMPING);
        world.obj_mut::<FishLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[FishLogic checkForIglooDuringFall]` @ 0x100012990
///
/// A raw fish falling at the island's depth onto the igloo's door drops inside.
pub fn check_for_igloo_during_fall(world: &mut World, this: Entity) -> bool {
    let display_object = logic::display_object(world, this);
    let z_pos = display::object(world, display_object).pos.z;
    let fish = world.obj::<FishLogic>(this);
    let raw = fish.fish_cook_state == COOK_STATE_RAW;
    let dont_hit_until_bounce = fish.dont_hit_until_bounce;
    let listener = fish.fish_logic_listener;
    if !raw || dont_hit_until_bounce || z_pos != ISLAND_Z_POS {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };
    let igloo_enabled = crate::controller::is_igloo_enabled(world, listener);
    if !igloo_enabled {
        return false;
    }
    let igloo = crate::controller::igloo_logic(world, listener, this);
    let Some(igloo) = igloo else {
        return false;
    };
    let available = igloo_logic::is_fish_fall_available(world, igloo, this);
    if !available {
        return false;
    }

    let igloo_display_object = logic::display_object(world, igloo);
    let igloo_pos = display::object(world, igloo_display_object).pos;
    let pos = display::object(world, display_object).pos;
    let distance_x = (pos.x - igloo_pos.x).abs();
    let distance_y = (pos.y - (igloo_pos.y + IGLOO_DOOR_Y_OFFSET)).abs();
    let y_vel = logic::logic(world, this).pos_vel.y;
    let falls_in = y_vel < 0.0 && distance_x <= IGLOO_DOOR_REACH && distance_y <= IGLOO_DOOR_REACH;
    if !falls_in {
        return false;
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut fish = world.obj_mut::<FishLogic>(this);
        fish.enable_island_floor_collision = false;
        fish.in_igloo = true;
        fish.igloo_logic = Some(igloo);
    }
    igloo_logic::fish_fall_into_igloo(world, igloo, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("FishFallIntoIgloo"));
    true
}

/// `-[FishLogic checkForIceHoleDuringFall]` @ 0x100012b6c
///
/// A raw fish falling at the island's depth into the ice hole is gone.
pub fn check_for_ice_hole_during_fall(world: &mut World, this: Entity) -> bool {
    let display_object = logic::display_object(world, this);
    let listener = world.obj::<FishLogic>(this).fish_logic_listener;
    // The hole is fetched before anything is tested.
    let ice_hole = match listener {
        Some(listener) => crate::controller::ice_hole_logic(world, listener, this),
        None => None,
    };

    let z_pos = display::object(world, display_object).pos.z;
    let fish = world.obj::<FishLogic>(this);
    let raw = fish.fish_cook_state == COOK_STATE_RAW;
    let dont_hit_until_bounce = fish.dont_hit_until_bounce;
    if !raw || dont_hit_until_bounce || z_pos != ISLAND_Z_POS {
        return false;
    }

    let Some(listener) = listener else {
        return false;
    };
    let ice_hole_enabled = crate::controller::is_ice_hole_enabled(world, listener);
    if !ice_hole_enabled {
        return false;
    }
    let Some(ice_hole) = ice_hole else {
        return false;
    };
    let available = ice_hole_logic::is_available_for_falling(world, ice_hole);
    if !available {
        return false;
    }

    let ice_hole_display_object = logic::display_object(world, ice_hole);
    let ice_hole_pos = display::object(world, ice_hole_display_object).pos;
    let pos = display::object(world, display_object).pos;
    let distance_x = (pos.x - ice_hole_pos.x).abs();
    let distance_y = (pos.y - (ice_hole_pos.y + 0.0)).abs();
    let y_vel = logic::logic(world, this).pos_vel.y;
    let falls_in = y_vel < 0.0 && distance_x <= ICE_HOLE_X_REACH && distance_y <= ICE_HOLE_Y_REACH;
    if !falls_in {
        return false;
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<FishLogic>(this).enable_island_floor_collision = false;
    ice_hole_logic::fish_fall_into_ice_hole(world, ice_hole, this);
    logic::set_behavior(world, this, Some("FishDie"));
    true
}

/// `-[FishLogic fall:::]` @ 0x100012d18
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut fish = world.obj_mut::<FishLogic>(this);
            if !fish.island_floor_collision {
                fish.on_island = false;
            }
            fish.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<FishLogic>(this).falling = true;
    }

    let fell_into_igloo = check_for_igloo_during_fall(world, this);
    let taken_in = fell_into_igloo || check_for_ice_hole_during_fall(world, this);
    if taken_in {
        world.obj_mut::<FishLogic>(this).falling = false;
        return ItemResult::Replaced;
    }

    let island_floor_collision = world.obj::<FishLogic>(this).island_floor_collision;
    if island_floor_collision {
        world.obj_mut::<FishLogic>(this).falling = false;
        return ItemResult::Done;
    }

    let display_object = logic::display_object(world, this);
    let ocean_enabled = is_ocean_enabled(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    if !ocean_enabled || WATER_LINE_Y <= y_pos {
        return ItemResult::Pending;
    }

    // The original sets y between the velocity and the tilt state; nothing reads it there.
    display::object_mut(world, display_object).pos.y = WATER_LINE_Y;
    splash(world, this);
    ItemResult::Replaced
}

/// `-[FishLogic fallFromHurricane:::]` @ 0x100012ed4
pub fn seq_fall_from_hurricane(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<FishLogic>(this).enable_island_floor_collision = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<FishLogic>(this).falling = true;
    }

    let display_object = logic::display_object(world, this);
    let z_pos = display::object(world, display_object).pos.z;
    let water_line_y = if ISLAND_Z_POS <= z_pos { WATER_LINE_Y } else { BEHIND_ISLAND_WATER_LINE_Y };

    let ocean_enabled = is_ocean_enabled(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    if !ocean_enabled || water_line_y <= y_pos {
        return ItemResult::Pending;
    }

    // The original sets y between the velocity and the tilt state; nothing reads it there.
    display::object_mut(world, display_object).pos.y = water_line_y;
    splash(world, this);
    ItemResult::Replaced
}

/// `-[FishLogic die:::]` @ 0x100013040
///
/// The fish leaves the screen and, while fish are enabled, starts over in the water.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<FishLogic>(this).active = false;
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);

    let listener = world.obj::<FishLogic>(this).fish_logic_listener;
    let dragging = world.obj::<FishLogic>(this).dragging;
    if dragging {
        world.obj_mut::<FishLogic>(this).dragging = false;
        if let Some(listener) = listener {
            crate::controller::on_fish_drag_complete(world, listener, this);
        }
    }

    {
        let mut fish = world.obj_mut::<FishLogic>(this);
        fish.cook_time = 0.0;
        fish.fish_cook_state = COOK_STATE_RAW;
        fish.in_water = true;
        fish.falling = false;
        fish.being_eaten = false;
        fish.apply_spring = false;
        fish.dragging = false;
        fish.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut fish = world.obj_mut::<FishLogic>(this);
        fish.fishing_bob_logic = None;
        fish.eating_pygmy_logic = None;
    }

    let fish_enabled = match listener {
        Some(listener) => crate::controller::is_fish_enabled(world, listener),
        None => false,
    };
    if fish_enabled {
        enter_island(world, this, false);
    }
    ItemResult::Done
}

/// `-[FishLogic pickedUpByHurricane:]` @ 0x100013184
///
/// Unlike the coconut, the fish does not remember the hurricane (`mPickedUpByHurricaneLogic`
/// stays nil).
pub fn picked_up_by_hurricane(world: &mut World, this: Entity, _picked_up_by_hurricane: Entity) -> bool {
    let fish = world.obj::<FishLogic>(this);
    let already_picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let can_be_picked_up = !fish.touch_began_inside
        && !already_picked_up
        && !fish.dragging
        && !fish.falling
        && !fish.being_eaten
        && !fish.caught_by_bird
        && !fish.in_igloo
        && !fish.in_water;
    if !can_be_picked_up {
        return false;
    }

    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.ignore_gravity = true;
        sprite.blown_by_hurricane = false;
        sprite.picked_up_by_hurricane = true;
    }
    world.obj_mut::<FishLogic>(this).tilt_state = 0;
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
    logic::set_behavior(world, this, Some("FishPickedUpByHurricane"));
    true
}

/// `-[FishLogic droppedByHurricane:]` @ 0x1000132ec
pub fn dropped_by_hurricane(world: &mut World, this: Entity, _dropped_by_hurricane: Entity) {
    let picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    if !picked_up {
        return;
    }

    world.obj_mut::<FishLogic>(this).falling = true;
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
    logic::set_behavior(world, this, Some("FishDroppedByHurricane"));
}

/// `-[FishLogic touchBeganInside:allTouches:]` @ 0x1000133d0
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<FishLogic>(this).fish_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };

    let fish = world.obj::<FishLogic>(this);
    let busy = fish.falling || fish.in_water || fish.being_eaten || fish.caught_by_bird || fish.active_touch.is_some();
    if story_active || busy {
        return;
    }

    world.obj_mut::<FishLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut fish = world.obj_mut::<FishLogic>(this);
        fish.touch_began_inside = true;
        fish.touch_screen_pos = touch_pos;
        fish.dragging = true;
        fish.apply_spring = true;
        // Original quirk: cleared again straight away.
        fish.touch_began_inside = false;
        fish.tilt_state = 0;
    }

    if let Some(listener) = listener {
        crate::controller::on_fish_drag(world, listener, this);
    }

    let cook_state = world.obj::<FishLogic>(this).fish_cook_state;
    let behavior = match cook_state {
        COOK_STATE_BURNT => Some("FishBurntDrag"),
        COOK_STATE_COOKED => Some("FishCookedDrag"),
        COOK_STATE_RAW => Some("FishDrag"),
        _ => None,
    };
    if let Some(behavior) = behavior {
        logic::set_behavior(world, this, Some(behavior));
    }
}

/// `-[FishLogic touchBeganOutside:allTouches:]` @ 0x100013578
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<FishLogic>(this).active_touch;
    if active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let mut fish = world.obj_mut::<FishLogic>(this);
    fish.touch_screen_pos = touch_pos;
    fish.touch_began_inside = false;
}

/// `-[FishLogic touchMovedEntered:allTouches:]` @ 0x1000135d4
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<FishLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<FishLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[FishLogic touchMovedExited:allTouches:]` @ 0x10001362c
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<FishLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<FishLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[FishLogic touchMovedInside:allTouches:]` @ 0x100013684
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[FishLogic touchMovedOutside:allTouches:]` @ 0x100013694
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<FishLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<FishLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[FishLogic touchEndedInside:allTouches:]` @ 0x1000136ec
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<FishLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let dragging = world.obj::<FishLogic>(this).dragging;
    if dragging {
        {
            let mut fish = world.obj_mut::<FishLogic>(this);
            fish.falling = true;
            fish.dragging = false;
            fish.apply_spring = false;
        }
        move_to_island_layer(world, this);

        let listener = world.obj::<FishLogic>(this).fish_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_fish_drag_complete(world, listener, this);
        }
        logic::set_behavior(world, this, Some("FishFall"));
    }

    let mut fish = world.obj_mut::<FishLogic>(this);
    fish.touch_began_inside = false;
    fish.active_touch = None;
}

/// `-[FishLogic touchEndedOutside:allTouches:]` @ 0x1000137b4
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
