//! `SwordFishLogic` (superclass `SpriteLogic`).
//!
//! The swordfish (`docs/re/logic-catalog.md` §4.3). It waits in the water until a pygmy fishing
//! through the ice hole catches it, and is then a throwable of §2.2 (slide friction 0.04, floor
//! at y 86) that pygmies eat and that knocks over the pygmies it hits fast enough.

use crate::prelude::*;

use crate::logic::island_core::coconut_logic::{
    ISLAND_Z_POS, THROWABLE_BOUNCE_SPEED, THROWABLE_GRAVITY_FORCE, apply_throwable_slide_friction,
    bounce_throwable_velocity, integrate_throwable, is_island_tipped_over, is_throwable_out_of_range,
    reaches_island_floor, throwable_drag_z_rot, throwable_pos_force, throwable_spring_force,
    throwable_tilt_state, throwable_tilted_gravity_force,
};
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::ocean_fishing::fish_logic;
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::length3;
use pg_engine::touch::BcTouch;

/// `SwordFishLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct SwordFishLogic {
    /// `mSwordFishLogicListener`: `@"<SwordFishLogicListener>"`
    pub sword_fish_logic_listener: Option<Entity>,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mEatingPygmyLogic`: `@"PygmyLogic"`
    pub eating_pygmy_logic: Option<Entity>,

    /// `mTiltState`: `i`
    pub tilt_state: i32,

    /// `mSlideFriction`: `f`
    pub slide_friction: f32,

    /// `mSpringDelta`: `f`
    pub spring_delta: f32,

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

    /// `mbInWater`: `B`
    pub in_water: bool,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbFalling`: `B`
    pub falling: bool,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,
}

pub const SWORD_FISH_LOGIC: Class = Class {
    name: "SwordFishLogic",
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

/// `mSlideFriction` (`0x3d23d70a`).
const SLIDE_FRICTION: f32 = 0.04;

/// The island floor (`setYPos:` in `onGameFrame:`).
const FLOOR_Y: f32 = 86.0;

/// The surface: `slide:::`, `fall:::` and (in front of the island) `fallFromHurricane:::` splash
/// below it, and `die:::` puts the swordfish back there.
const WATER_LINE_Y: f32 = 20.0;

/// `fallFromHurricane:::` splashes below this y behind the island.
const BEHIND_ISLAND_WATER_LINE_Y: f32 = 86.0;

/// `catch` (`DAT_10029d378`): flung up and toward the island centre, indexed by
/// `x > islandCenterXPos`.
const CATCH_X_VELS: [f32; 2] = [200.0, -200.0];
const CATCH_Y_VEL: f32 = 1000.0;

/// `dropWhileEating`: the swordfish hops up.
const DROP_Y_VEL: f32 = 400.0;

/// The spring of `drag:::`.
const DRAG_STIFFNESS: f32 = 100.0;
const DRAG_DAMPING: f32 = 8.0;

/// `checkPygmyCollision`: faster than this, the point 40 below the swordfish's origin hits a
/// pygmy whose point 31 above its origin lies within 30 (squared distance 900).
const PYGMY_HIT_MIN_SPEED: f32 = 200.0;
const PYGMY_HIT_DISTANCE_SQUARED: f32 = 900.0;
const PYGMY_HIT_Y_OFFSET: f32 = 31.0;
const HIT_POINT_Y_OFFSET: f32 = -40.0;

/// Sequence-item handlers implemented by `SwordFishLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "slide" => seq_slide(world, this, call),
        "drag" => seq_drag(world, this, call),
        "fall" => seq_fall(world, this, call),
        "fallFromHurricane" => seq_fall_from_hurricane(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[SwordFishLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&SWORD_FISH_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[SwordFishLogic initWithDisplayObject:]` @ 0x1000c15c0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(SwordFishLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut sword_fish = world.obj_mut::<SwordFishLogic>(this);
        sword_fish.in_water = true;
        sword_fish.tilt_state = 0;
        sword_fish.slide_friction = SLIDE_FRICTION;
    }

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    logic::set_behavior(world, this, Some("SwordFishIdle"));
}

/// `-[SwordFishLogic setSwordFishLogicListener:]` @ 0x1000c167c
pub fn set_sword_fish_logic_listener(world: &mut World, this: Entity, sword_fish_logic_listener: Entity) {
    world.obj_mut::<SwordFishLogic>(this).sword_fish_logic_listener = Some(sword_fish_logic_listener);
}

/// Re-inserts the swordfish's display object after the display marker `marker_id`.
fn insert_after_marker(world: &mut World, this: Entity, marker_id: &str) {
    let marker = pg_engine::library::display_marker(world, marker_id);
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `[mSwordFishLogicListener onSwordFishDragComplete:self]`.
fn send_drag_complete(world: &mut World, this: Entity) {
    let listener = world.obj::<SwordFishLogic>(this).sword_fish_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_sword_fish_drag_complete(world, listener, this);
    }
}

/// `[mSwordFishLogicListener isOceanEnabled]` (NO without a listener).
fn is_ocean_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<SwordFishLogic>(this).sword_fish_logic_listener;
    match listener {
        Some(listener) => crate::controller::is_ocean_enabled(world, listener),
        None => false,
    }
}

/// `-[SwordFishLogic exitIsland:]` @ 0x1000c16d8
///
/// A swordfish in the water stays unless `now`; otherwise it falls through the island
/// (`SwordFishFallIgnoreIsland`), or ends at once with `now` (`SwordFishDie`).
pub fn exit_island(world: &mut World, this: Entity, now: bool) {
    let in_water = world.obj::<SwordFishLogic>(this).in_water;
    if in_water && !now {
        return;
    }

    let dragging = world.obj::<SwordFishLogic>(this).dragging;
    if dragging {
        {
            let mut sword_fish = world.obj_mut::<SwordFishLogic>(this);
            sword_fish.falling = true;
            sword_fish.dragging = false;
            sword_fish.apply_spring = false;
        }
        send_drag_complete(world, this);
    }

    insert_after_marker(world, this, "DragDisplayMarker");
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<SwordFishLogic>(this).enable_island_floor_collision = false;

    let behavior = if now { "SwordFishDie" } else { "SwordFishFallIgnoreIsland" };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[SwordFishLogic moveToWaterLayer]` @ 0x1000c1864
///
/// The water layer is the drag layer.
pub fn move_to_water_layer(world: &mut World, this: Entity) {
    insert_after_marker(world, this, "DragDisplayMarker");
}

/// `-[SwordFishLogic moveToDragLayer]` @ 0x1000c18cc
pub fn move_to_drag_layer(world: &mut World, this: Entity) {
    insert_after_marker(world, this, "DragDisplayMarker");
}

/// `-[SwordFishLogic moveToIslandLayer]` @ 0x1000c1934
pub fn move_to_island_layer(world: &mut World, this: Entity) {
    insert_after_marker(world, this, "SmallObjectsMarker");
}

/// `-[SwordFishLogic moveToBehindIslandLayer]` @ 0x1000c199c
pub fn move_to_behind_island_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "IslandDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }
}

/// `-[SwordFishLogic canBeCaught]` @ 0x1000c1a04
pub fn can_be_caught(world: &mut World, this: Entity) -> bool {
    world.obj::<SwordFishLogic>(this).in_water
}

/// `-[SwordFishLogic catch]` @ 0x1000c1a14
///
/// Pulled out of the ice hole, the swordfish is flung up toward the island centre.
pub fn catch(world: &mut World, this: Entity) {
    {
        let mut sword_fish = world.obj_mut::<SwordFishLogic>(this);
        sword_fish.in_water = false;
        sword_fish.falling = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut sword_fish = world.obj_mut::<SwordFishLogic>(this);
        sword_fish.dont_hit_until_bounce = true;
        sword_fish.tilt_state = 0;
    }
    logic::logic_mut(world, this).pos_vel.y = CATCH_Y_VEL;

    let display_object = logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;
    let listener = world.obj::<SwordFishLogic>(this).sword_fish_logic_listener;
    let island_center_x_pos = match listener {
        Some(listener) => crate::controller::island_center_x_pos(world, listener),
        None => 0.0,
    };
    let right_of_center = x_pos > island_center_x_pos;
    logic::logic_mut(world, this).pos_vel.x = CATCH_X_VELS[usize::from(right_of_center)];

    display::object_mut(world, display_object).scale.x = -1.0;
    move_to_island_layer(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("SwordFishCatch"));
}

/// `-[SwordFishLogic isEdible]` @ 0x1000c1b50
pub fn is_edible(world: &mut World, this: Entity) -> bool {
    let sword_fish = world.obj::<SwordFishLogic>(this);
    sword_fish.on_island && !sword_fish.being_eaten && !sword_fish.in_water && !sword_fish.touch_began_inside
}

/// `-[SwordFishLogic isBeingEaten]` @ 0x1000c1ba0
pub fn is_being_eaten(world: &mut World, this: Entity) -> bool {
    world.obj::<SwordFishLogic>(this).being_eaten
}

/// `-[SwordFishLogic eatingPygmyLogic]` @ 0x1000c1bb0
pub fn eating_pygmy_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<SwordFishLogic>(this).eating_pygmy_logic
}

/// `-[SwordFishLogic dropWhileEating]` @ 0x1000c1bc0
pub fn drop_while_eating(world: &mut World, this: Entity) {
    {
        let mut sword_fish = world.obj_mut::<SwordFishLogic>(this);
        sword_fish.eating_pygmy_logic = None;
        sword_fish.being_eaten = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<SwordFishLogic>(this).dont_hit_until_bounce = true;
    logic::logic_mut(world, this).pos_vel.y = DROP_Y_VEL;
    logic::set_behavior(world, this, Some("SwordFishDropWhileEating"));
}

/// `-[SwordFishLogic startEating:]` @ 0x1000c1c28
pub fn start_eating(world: &mut World, this: Entity, eating: Entity) {
    world.obj_mut::<SwordFishLogic>(this).being_eaten = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<SwordFishLogic>(this).eating_pygmy_logic = Some(eating);
}

/// `-[SwordFishLogic pickUpForEating]` @ 0x1000c1c58
pub fn pick_up_for_eating(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("SwordFishEaten"));
}

/// `-[SwordFishLogic finishEating]` @ 0x1000c1c70
pub fn finish_eating(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("SwordFishDie"));
}

/// `-[SwordFishLogic checkPygmyCollision]` @ 0x1000c1c88
///
/// The first pygmy in reach that takes the hit ends the check; the swordfish itself flies on.
pub fn check_pygmy_collision(world: &mut World, this: Entity) {
    let sword_fish = world.obj::<SwordFishLogic>(this);
    if sword_fish.being_eaten || sword_fish.dont_hit_until_bounce {
        return;
    }

    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    let speed = length3(logic::logic(world, this).pos_vel);
    let hit_x = pos.x + 0.0;
    let hit_y = pos.y + HIT_POINT_Y_OFFSET;

    let listener = world.obj::<SwordFishLogic>(this).sword_fish_logic_listener;
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
        let delta_x = hit_x - (pygmy_pos.x + 0.0);
        let delta_y = hit_y - (pygmy_pos.y + PYGMY_HIT_Y_OFFSET);
        let distance_squared = delta_x * delta_x + delta_y * delta_y;
        let within_reach = distance_squared < PYGMY_HIT_DISTANCE_SQUARED;
        if !within_reach {
            continue;
        }

        let hit = crate::pygmy::hit_by_sword_fish(world, pygmy, this);
        if hit {
            return;
        }
    }
}

/// `-[SwordFishLogic isAccelEnabled]` @ 0x1000c1eec
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<SwordFishLogic>(this).sword_fish_logic_listener;
    let gravity_enabled = match listener {
        Some(listener) => crate::controller::is_gravity_enabled(world, listener),
        None => false,
    };
    if !gravity_enabled {
        return false;
    }
    sprite_logic::is_accel_enabled(world, this)
}

/// `-[SwordFishLogic onGameFrame:]` @ 0x1000c1f5c
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

    let sword_fish = world.obj::<SwordFishLogic>(this);
    let tilt_state = sword_fish.tilt_state;
    let slide_friction = sword_fish.slide_friction;
    let spring_force = sword_fish.spring_force;
    let apply_spring = sword_fish.apply_spring;

    let mut sprite = world.obj_mut::<SpriteLogic>(this);
    apply_throwable_slide_friction(&mut sprite.gravity_force, tilt_state, slide_friction);
    let gravity_force = sprite.gravity_force;
    let ignore_gravity = sprite.ignore_gravity;

    let pos_force = throwable_pos_force(gravity_force, ignore_gravity, spring_force, apply_spring);
    logic::logic_mut(world, this).pos_force = pos_force;
    integrate_throwable(world, this, pos_force, dt);

    world.obj_mut::<SwordFishLogic>(this).island_floor_collision = false;
    let listener = world.obj::<SwordFishLogic>(this).sword_fish_logic_listener;
    let mut island_min_x_pos = 0.0;
    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
        island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
    }

    let pos = display::object(world, display_object).pos;
    let reached_floor = reaches_island_floor(previous_y, pos, island_min_x_pos, island_max_x_pos, FLOOR_Y);
    let enable_island_floor_collision = world.obj::<SwordFishLogic>(this).enable_island_floor_collision;
    if reached_floor && enable_island_floor_collision {
        land_on_island_floor(world, this, display_object);
    } else {
        world.obj_mut::<SwordFishLogic>(this).on_island = false;
    }

    let y_pos = display::object(world, display_object).pos.y;
    if is_throwable_out_of_range(y_pos) {
        logic::set_behavior(world, this, Some("SwordFishDie"));
    }

    check_pygmy_collision(world, this);
}

/// The island floor contact of `onGameFrame:`: on the first contact the swordfish bounces or
/// lands; every contact puts it on the floor.
fn land_on_island_floor(world: &mut World, this: Entity, display_object: Entity) {
    let on_island = world.obj::<SwordFishLogic>(this).on_island;
    let mut pos_vel = logic::logic(world, this).pos_vel;
    bounce_throwable_velocity(&mut pos_vel, on_island);
    logic::logic_mut(world, this).pos_vel = pos_vel;

    if pos_vel.y < THROWABLE_BOUNCE_SPEED {
        logic::logic_mut(world, this).pos_vel.y = 0.0;
        if !on_island {
            world.obj_mut::<SwordFishLogic>(this).on_island = true;
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            world.obj_mut::<SwordFishLogic>(this).island_floor_collision = true;
            logic::set_behavior(world, this, Some("SwordFishOnGround"));
        }
        display::object_mut(world, display_object).rot.z = 0.0;
    } else {
        logic::set_behavior(world, this, Some("SwordFishBounce"));
    }

    world.obj_mut::<SwordFishLogic>(this).dont_hit_until_bounce = false;
    display::object_mut(world, display_object).pos.y = FLOOR_Y;
}

/// `-[SwordFishLogic refreshTiltState]` @ 0x1000c244c
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let sword_fish = world.obj::<SwordFishLogic>(this);
    if sword_fish.dragging || sword_fish.falling || !sword_fish.on_island {
        return;
    }

    let tilt_state = throwable_tilt_state(world);
    let current_tilt_state = world.obj::<SwordFishLogic>(this).tilt_state;
    if tilt_state != current_tilt_state {
        if tilt_state == 0 {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            logic::set_behavior(world, this, Some("SwordFishStopSlide"));
            world.obj_mut::<SwordFishLogic>(this).tilt_state = 0;
        } else if tilt_state == 1 {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            logic::set_behavior(world, this, Some("SwordFishTiltSlide"));
            world.obj_mut::<SwordFishLogic>(this).tilt_state = 1;
        }
    }

    if is_island_tipped_over(world) {
        {
            let mut sword_fish = world.obj_mut::<SwordFishLogic>(this);
            sword_fish.falling = true;
            sword_fish.tilt_state = 0;
        }
        logic::set_behavior(world, this, Some("SwordFishFall"));
    }
}

/// `-[SwordFishLogic undoTiltState]` @ 0x1000c262c
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    let sword_fish = world.obj::<SwordFishLogic>(this);
    if sword_fish.dragging || sword_fish.falling || !sword_fish.on_island {
        return;
    }

    if sword_fish.tilt_state == 1 {
        logic::logic_mut(world, this).pos_vel.x = 0.0;
        logic::set_behavior(world, this, Some("SwordFishStopSlide"));
    }
    world.obj_mut::<SwordFishLogic>(this).tilt_state = 0;
}

/// The splash of `slide:::`, `fall:::` and `fallFromHurricane:::`: the swordfish stops in the
/// water and plays `SwordFishSplash`.
fn splash(world: &mut World, this: Entity) {
    {
        let mut sword_fish = world.obj_mut::<SwordFishLogic>(this);
        sword_fish.falling = false;
        sword_fish.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<SwordFishLogic>(this).tilt_state = 0;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
    logic::set_behavior(world, this, Some("SwordFishSplash"));
}

/// `-[SwordFishLogic slide:::]` @ 0x1000c26c0
pub fn seq_slide(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SwordFishLogic>(this).enable_island_floor_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let display_object = logic::display_object(world, this);
    let ocean_enabled = is_ocean_enabled(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    let below_water_line = y_pos < WATER_LINE_Y;
    if !ocean_enabled || !below_water_line {
        return ItemResult::Pending;
    }

    splash(world, this);
    ItemResult::Replaced
}

/// `-[SwordFishLogic drag:::]` @ 0x1000c27e8
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<SwordFishLogic>(this).enable_island_floor_collision = false;
        move_to_drag_layer(world, this);
    }

    let display_object = logic::display_object(world, this);
    let touch_screen_pos = world.obj::<SwordFishLogic>(this).touch_screen_pos;
    let offset = fish_logic::drag_offset(world, display_object, touch_screen_pos);
    let length = length3(offset);
    if length != 0.0 {
        if offset.y != 0.0 {
            let z_rot = throwable_drag_z_rot(offset);
            display::object_mut(world, display_object).rot.z = z_rot;
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let spring_force = throwable_spring_force(offset, length, pos_vel, DRAG_STIFFNESS, DRAG_DAMPING);
        world.obj_mut::<SwordFishLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[SwordFishLogic fall:::]` @ 0x1000c29b0
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut sword_fish = world.obj_mut::<SwordFishLogic>(this);
            if !sword_fish.island_floor_collision {
                sword_fish.on_island = false;
            }
            sword_fish.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<SwordFishLogic>(this).falling = true;
    }

    let island_floor_collision = world.obj::<SwordFishLogic>(this).island_floor_collision;
    if island_floor_collision {
        world.obj_mut::<SwordFishLogic>(this).falling = false;
        return ItemResult::Done;
    }

    let display_object = logic::display_object(world, this);
    let ocean_enabled = is_ocean_enabled(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    let below_water_line = y_pos < WATER_LINE_Y;
    if !ocean_enabled || !below_water_line {
        return ItemResult::Pending;
    }

    // The original sets y between the velocity and the tilt state; nothing reads it there.
    display::object_mut(world, display_object).pos.y = WATER_LINE_Y;
    splash(world, this);
    ItemResult::Replaced
}

/// `-[SwordFishLogic fallFromHurricane:::]` @ 0x1000c2b2c
pub fn seq_fall_from_hurricane(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SwordFishLogic>(this).enable_island_floor_collision = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<SwordFishLogic>(this).falling = true;
    }

    let display_object = logic::display_object(world, this);
    let z_pos = display::object(world, display_object).pos.z;
    let water_line_y = if z_pos < ISLAND_Z_POS { BEHIND_ISLAND_WATER_LINE_Y } else { WATER_LINE_Y };

    let ocean_enabled = is_ocean_enabled(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    let below_water_line = y_pos < water_line_y;
    if !ocean_enabled || !below_water_line {
        return ItemResult::Pending;
    }

    // The original sets y between the velocity and the tilt state; nothing reads it there.
    display::object_mut(world, display_object).pos.y = water_line_y;
    splash(world, this);
    ItemResult::Replaced
}

/// `-[SwordFishLogic die:::]` @ 0x1000c2c98
///
/// The swordfish goes back to the water at rest, unscaled and unrotated, and idles there.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
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
    move_to_water_layer(world, this);

    let dragging = world.obj::<SwordFishLogic>(this).dragging;
    if dragging {
        world.obj_mut::<SwordFishLogic>(this).dragging = false;
        send_drag_complete(world, this);
    }

    {
        let mut sword_fish = world.obj_mut::<SwordFishLogic>(this);
        sword_fish.in_water = true;
        sword_fish.falling = false;
        sword_fish.being_eaten = false;
        sword_fish.apply_spring = false;
        sword_fish.dragging = false;
        sword_fish.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<SwordFishLogic>(this).eating_pygmy_logic = None;
    logic::set_behavior(world, this, Some("SwordFishIdle"));
    ItemResult::Done
}

/// `-[SwordFishLogic pickedUpByHurricane:]` @ 0x1000c2e0c
///
/// Like the fish, the swordfish does not remember the hurricane (`mPickedUpByHurricaneLogic`
/// stays nil).
pub fn picked_up_by_hurricane(world: &mut World, this: Entity, _hurricane: Entity) -> bool {
    let sword_fish = world.obj::<SwordFishLogic>(this);
    let already_picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let can_be_picked_up = !sword_fish.touch_began_inside
        && !already_picked_up
        && !sword_fish.dragging
        && !sword_fish.falling
        && !sword_fish.being_eaten
        && !sword_fish.in_water;
    if !can_be_picked_up {
        return false;
    }

    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.ignore_gravity = true;
        sprite.blown_by_hurricane = false;
        sprite.picked_up_by_hurricane = true;
    }
    world.obj_mut::<SwordFishLogic>(this).tilt_state = 0;
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
    logic::set_behavior(world, this, Some("SwordFishPickedUpByHurricane"));
    true
}

/// `-[SwordFishLogic droppedByHurricane:]` @ 0x1000c2f54
pub fn dropped_by_hurricane(world: &mut World, this: Entity, _hurricane: Entity) {
    let picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    if !picked_up {
        return;
    }

    world.obj_mut::<SwordFishLogic>(this).falling = true;
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
    logic::set_behavior(world, this, Some("SwordFishDroppedByHurricane"));
}

/// `-[SwordFishLogic touchBeganInside:allTouches:]` @ 0x1000c3038
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<SwordFishLogic>(this).sword_fish_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };

    let sword_fish = world.obj::<SwordFishLogic>(this);
    let busy = sword_fish.falling || sword_fish.in_water || sword_fish.being_eaten || sword_fish.active_touch.is_some();
    if story_active || busy {
        return;
    }

    world.obj_mut::<SwordFishLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut sword_fish = world.obj_mut::<SwordFishLogic>(this);
        sword_fish.touch_began_inside = true;
        sword_fish.touch_screen_pos = touch_pos;
        sword_fish.dragging = true;
        sword_fish.apply_spring = true;
        // Original quirk: cleared again straight away.
        sword_fish.touch_began_inside = false;
        sword_fish.tilt_state = 0;
    }

    if let Some(listener) = listener {
        crate::controller::on_sword_fish_drag(world, listener, this);
    }
    logic::set_behavior(world, this, Some("SwordFishDrag"));
}

/// `-[SwordFishLogic touchBeganOutside:allTouches:]` @ 0x1000c3180
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<SwordFishLogic>(this).active_touch;
    if active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let mut sword_fish = world.obj_mut::<SwordFishLogic>(this);
    sword_fish.touch_screen_pos = touch_pos;
    sword_fish.touch_began_inside = false;
}

/// `-[SwordFishLogic touchMovedEntered:allTouches:]` @ 0x1000c31dc
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<SwordFishLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<SwordFishLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[SwordFishLogic touchMovedExited:allTouches:]` @ 0x1000c3234
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<SwordFishLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<SwordFishLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[SwordFishLogic touchMovedInside:allTouches:]` @ 0x1000c328c
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[SwordFishLogic touchMovedOutside:allTouches:]` @ 0x1000c329c
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<SwordFishLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<SwordFishLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[SwordFishLogic touchEndedInside:allTouches:]` @ 0x1000c32f4
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<SwordFishLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let dragging = world.obj::<SwordFishLogic>(this).dragging;
    if dragging {
        {
            let mut sword_fish = world.obj_mut::<SwordFishLogic>(this);
            sword_fish.falling = true;
            sword_fish.dragging = false;
            sword_fish.apply_spring = false;
        }
        move_to_island_layer(world, this);
        send_drag_complete(world, this);
        logic::set_behavior(world, this, Some("SwordFishFall"));
    }

    let mut sword_fish = world.obj_mut::<SwordFishLogic>(this);
    sword_fish.touch_began_inside = false;
    sword_fish.active_touch = None;
}

/// `-[SwordFishLogic touchEndedOutside:allTouches:]` @ 0x1000c33bc
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
