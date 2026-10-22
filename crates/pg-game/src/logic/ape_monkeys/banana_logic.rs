//! `BananaLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::coconut_logic;
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::length3;
use pg_engine::touch::BcTouch;

/// `BananaLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct BananaLogic {
    /// `mBananaLogicListener`: `@"<BananaLogicListener>"`
    pub banana_logic_listener: Option<Entity>,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mTiltState`: `i`
    pub tilt_state: i32,

    /// `mSlideFriction`: `f`
    pub slide_friction: f32,

    /// `mSpringDelta`: `f`
    pub spring_delta: f32,

    /// `mEatingPygmyLogic`: `@"PygmyLogic"`
    pub eating_pygmy_logic: Option<Entity>,

    /// `mEatingMonkeyLogic`: `@"MonkeyLogic"`
    pub eating_monkey_logic: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbOnIsland`: `B`
    pub on_island: bool,

    /// `mbEnableIslandFloorCollision`: `B`
    pub enable_island_floor_collision: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbDontHitUntilBounce`: `B`
    pub dont_hit_until_bounce: bool,

    /// `mbIsPeel`: `B`
    pub is_peel: bool,

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

pub const BANANA_LOGIC: Class = Class {
    name: "BananaLogic",
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
const FLOOR_Y: f32 = 98.0;

/// `DAT_10029d3f0`: the x velocity of a peel sent flying, by direction (left, right).
const PEEL_X_VEL: [f32; 2] = [-200.0, 200.0];

/// The spring of `drag:::` (100 / 8, `docs/re/logic-catalog.md` §2.2).
const DRAG_STIFFNESS: f32 = 100.0;
const DRAG_DAMPING: f32 = 8.0;

/// `checkPygmyCollision`: only a banana faster than this hits, within 20 (squared) of a point
/// 31 above the pygmy.
const PYGMY_HIT_SPEED: f32 = 200.0;
const PYGMY_HIT_DISTANCE_SQUARED: f32 = 400.0;
const PYGMY_HIT_Y_OFFSET: f32 = 31.0;

/// Sequence-item handlers implemented by `BananaLogic` (`respondsToSelector:` order).
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

/// `[[BananaLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&BANANA_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[mBananaLogicListener onBananaDragComplete:self]`.
fn notify_banana_drag_complete(world: &mut World, this: Entity) {
    let listener = world.obj::<BananaLogic>(this).banana_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_banana_drag_complete(world, listener, this);
    }
}

/// `-[BananaLogic initWithDisplayObject:]` @ 0x10001af08
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(BananaLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut banana = world.obj_mut::<BananaLogic>(this);
        banana.tilt_state = 0;
        banana.slide_friction = SLIDE_FRICTION;
    }

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[BananaLogic setBananaLogicListener:]` @ 0x10001af98
pub fn set_banana_logic_listener(world: &mut World, this: Entity, banana_logic_listener: Entity) {
    world.obj_mut::<BananaLogic>(this).banana_logic_listener = Some(banana_logic_listener);
}

/// `-[BananaLogic isActive]` @ 0x10001b058
pub fn is_active(world: &mut World, this: Entity) -> bool {
    world.obj::<BananaLogic>(this).active
}

/// `-[BananaLogic isSlippable]` @ 0x10001b068
pub fn is_slippable(world: &mut World, this: Entity) -> bool {
    let banana = world.obj::<BananaLogic>(this);
    banana.on_island && !banana.being_eaten && banana.is_peel && banana.active
}

/// `-[BananaLogic isEdible]` @ 0x10001b0b8
pub fn is_edible(world: &mut World, this: Entity) -> bool {
    let banana = world.obj::<BananaLogic>(this);
    banana.on_island && !banana.being_eaten && !banana.is_peel && banana.active
}

/// `-[BananaLogic thrownByCharlie:]` @ 0x10001b108
pub fn thrown_by_charlie(world: &mut World, this: Entity, thrown_by_charlie: Entity) {
    {
        let mut banana = world.obj_mut::<BananaLogic>(this);
        banana.eating_pygmy_logic = None;
        banana.eating_monkey_logic = None;
        banana.being_eaten = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<BananaLogic>(this).dont_hit_until_bounce = true;

    let charlie_display_object = logic::display_object(world, thrown_by_charlie);
    let charlie_scale_x = display::object(world, charlie_display_object).scale.x;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = charlie_scale_x * -200.0;
        state.pos_vel.y = 600.0;
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
    logic::set_behavior(world, this, Some("BananaDropWhileEating"));
}

/// `-[BananaLogic dropWhileEating]` @ 0x10001b1f4
pub fn drop_while_eating(world: &mut World, this: Entity) {
    {
        let mut banana = world.obj_mut::<BananaLogic>(this);
        banana.eating_pygmy_logic = None;
        banana.eating_monkey_logic = None;
        banana.being_eaten = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<BananaLogic>(this).dont_hit_until_bounce = true;
    logic::logic_mut(world, this).pos_vel.y = 400.0;
    logic::set_behavior(world, this, Some("BananaDropWhileEating"));
}

/// `-[BananaLogic slipOnBanana:]` @ 0x10001b268
pub fn slip_on_banana(world: &mut World, this: Entity, slip_on_banana: Entity) {
    let pygmy_display_object = logic::display_object(world, slip_on_banana);
    let pygmy_scale_x = display::object(world, pygmy_display_object).scale.x;
    world.obj_mut::<BananaLogic>(this).dont_hit_until_bounce = true;

    // The peel flies off the way the pygmy was walking.
    let facing_right = pygmy_scale_x > 0.0;
    let mut state = logic::logic_mut(world, this);
    state.pos_vel.y = 400.0;
    state.pos_vel.x = PEEL_X_VEL[usize::from(facing_right)];
}

/// `-[BananaLogic startEating:]` @ 0x10001b2ec
pub fn start_eating(world: &mut World, this: Entity, eating: Entity) {
    world.obj_mut::<BananaLogic>(this).being_eaten = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<BananaLogic>(this).eating_pygmy_logic = Some(eating);
}

/// `-[BananaLogic startEatingByMonkey:]` @ 0x10001b31c
pub fn start_eating_by_monkey(world: &mut World, this: Entity, eating_by_monkey: Entity) {
    world.obj_mut::<BananaLogic>(this).being_eaten = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<BananaLogic>(this).eating_monkey_logic = Some(eating_by_monkey);
}

/// `-[BananaLogic pickUpForEating]` @ 0x10001b34c
pub fn pick_up_for_eating(_world: &mut World, _this: Entity) {}

/// `-[BananaLogic finishEating]` @ 0x10001b350
pub fn finish_eating(world: &mut World, this: Entity) {
    world.obj_mut::<BananaLogic>(this).eating_pygmy_logic = None;
    logic::set_behavior(world, this, Some("BananaDie"));
}

/// `-[BananaLogic finishEatingByMonkey]` @ 0x10001b374
pub fn finish_eating_by_monkey(world: &mut World, this: Entity) {
    {
        let mut banana = world.obj_mut::<BananaLogic>(this);
        banana.is_peel = true;
        banana.eating_monkey_logic = None;
        banana.being_eaten = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<BananaLogic>(this).dont_hit_until_bounce = true;

    let display_object = logic::display_object(world, this);
    logic::logic_mut(world, this).pos_vel.y = 600.0;
    let x_pos = display::object(world, display_object).pos.x;

    // The monkey throws the peel towards the island's centre.
    let listener = world.obj::<BananaLogic>(this).banana_logic_listener;
    let island_center_x_pos = match listener {
        Some(listener) => crate::controller::island_center_x_pos(world, listener),
        None => 0.0,
    };
    let left_of_center = x_pos < island_center_x_pos;
    logic::logic_mut(world, this).pos_vel.x = PEEL_X_VEL[usize::from(left_of_center)];
    logic::set_behavior(world, this, Some("BananaFinishEatingByMonkey"));
}

/// `-[BananaLogic moveToIslandLayer]` @ 0x10001b464
pub fn move_to_island_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "SmallObjectsMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[BananaLogic moveToDragLayer]` @ 0x10001b4cc
pub fn move_to_drag_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[BananaLogic moveToBehindIslandLayer]` @ 0x10001b534
pub fn move_to_behind_island_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "IslandDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }
}

/// `-[BananaLogic reset]` @ 0x10001b59c
pub fn reset(world: &mut World, this: Entity) {
    let dragging = world.obj::<BananaLogic>(this).dragging;
    if dragging {
        world.obj_mut::<BananaLogic>(this).dragging = false;
        notify_banana_drag_complete(world, this);
    }

    let being_eaten = world.obj::<BananaLogic>(this).being_eaten;
    if being_eaten {
        let eating_pygmy = world.obj::<BananaLogic>(this).eating_pygmy_logic;
        if let Some(eating_pygmy) = eating_pygmy {
            crate::pygmy::stop_eating(world, eating_pygmy);
        }
        world.obj_mut::<BananaLogic>(this).eating_pygmy_logic = None;
    }

    let hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane_logic;
    if let Some(hurricane) = hurricane {
        crate::logic::sky_powers::hurricane_logic::drop_sprite_logic(world, hurricane, this);
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_hurricane_logic = None;
        sprite.picked_up_by_hurricane = false;
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).pos.z = coconut_logic::ISLAND_Z_POS;

    {
        let mut banana = world.obj_mut::<BananaLogic>(this);
        banana.is_peel = false;
        banana.falling = false;
        banana.in_water = false;
        banana.being_eaten = false;
        banana.apply_spring = false;
        banana.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<BananaLogic>(this).eating_pygmy_logic = None;
}

/// `-[BananaLogic enterIsland]` @ 0x10001b6fc
pub fn enter_island(world: &mut World, this: Entity) {
    reset(world, this);
    world.obj_mut::<BananaLogic>(this).active = true;
    move_to_drag_layer(world, this);

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;
}

/// `-[BananaLogic exitIsland]` @ 0x10001b76c
pub fn exit_island(world: &mut World, this: Entity) {
    reset(world, this);
    world.obj_mut::<BananaLogic>(this).active = false;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("BananaExitIsland"));
}

/// `-[BananaLogic checkPygmyCollision]` @ 0x10001b7e0
pub fn check_pygmy_collision(world: &mut World, this: Entity) {
    let banana = world.obj::<BananaLogic>(this);
    if banana.being_eaten || banana.dont_hit_until_bounce {
        return;
    }

    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    let x_pos = pos.x;
    let y_pos = pos.y;
    let pos_vel = logic::logic(world, this).pos_vel;
    let speed = length3(pos_vel);

    let listener = world.obj::<BananaLogic>(this).banana_logic_listener;
    let pygmy_logic_array = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmy_logic_array {
        let fast_enough = speed > PYGMY_HIT_SPEED;
        if !fast_enough {
            continue;
        }

        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let delta_x = x_pos - (pygmy_pos.x + 0.0);
        let delta_y = y_pos - (pygmy_pos.y + PYGMY_HIT_Y_OFFSET);
        let distance_squared = delta_x * delta_x + delta_y * delta_y;
        let within_reach = distance_squared < PYGMY_HIT_DISTANCE_SQUARED;
        if !within_reach {
            continue;
        }

        let hit = crate::pygmy::hit_by_banana(world, pygmy, this);
        if hit {
            return;
        }
    }
}

/// `-[BananaLogic isAccelEnabled]` @ 0x10001ba38
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<BananaLogic>(this).banana_logic_listener;
    let gravity_enabled = match listener {
        Some(listener) => crate::controller::is_gravity_enabled(world, listener),
        None => false,
    };
    if !gravity_enabled {
        return false;
    }
    sprite_logic::is_accel_enabled(world, this)
}

/// `-[BananaLogic onGameFrame:]` @ 0x10001baa8
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    let active = world.obj::<BananaLogic>(this).active;
    if shutdown || !active {
        return;
    }

    let display_object = logic::display_object(world, this);
    let previous_y = display::object(world, display_object).pos.y;

    let accel_enabled = is_accel_enabled(world, this);
    if accel_enabled {
        let gravity_force = coconut_logic::throwable_tilted_gravity_force(world);
        world.obj_mut::<SpriteLogic>(this).gravity_force = gravity_force;
        refresh_tilt_state(world, this);
    } else {
        world.obj_mut::<SpriteLogic>(this).gravity_force = coconut_logic::THROWABLE_GRAVITY_FORCE;
        undo_tilt_state(world, this);
    }

    let banana = world.obj::<BananaLogic>(this);
    let tilt_state = banana.tilt_state;
    let slide_friction = banana.slide_friction;
    let spring_force = banana.spring_force;
    let apply_spring = banana.apply_spring;

    let mut sprite = world.obj_mut::<SpriteLogic>(this);
    coconut_logic::apply_throwable_slide_friction(&mut sprite.gravity_force, tilt_state, slide_friction);
    let gravity_force = sprite.gravity_force;
    let ignore_gravity = sprite.ignore_gravity;

    let pos_force = coconut_logic::throwable_pos_force(gravity_force, ignore_gravity, spring_force, apply_spring);
    logic::logic_mut(world, this).pos_force = pos_force;
    coconut_logic::integrate_throwable(world, this, pos_force, dt);
    coconut_logic::clamp_throwable_to_screen(world, display_object);

    world.obj_mut::<BananaLogic>(this).island_floor_collision = false;
    let listener = world.obj::<BananaLogic>(this).banana_logic_listener;
    let mut island_min_x_pos = 0.0;
    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
        island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
    }

    let pos = display::object(world, display_object).pos;
    let reached_floor = coconut_logic::reaches_island_floor(previous_y, pos, island_min_x_pos, island_max_x_pos, FLOOR_Y);
    let enable_island_floor_collision = world.obj::<BananaLogic>(this).enable_island_floor_collision;
    if reached_floor && enable_island_floor_collision {
        // Unlike the coconut, every first contact plays the bounce, even one that lands.
        let on_island = world.obj::<BananaLogic>(this).on_island;
        let mut pos_vel = logic::logic(world, this).pos_vel;
        coconut_logic::bounce_throwable_velocity(&mut pos_vel, on_island);
        logic::logic_mut(world, this).pos_vel = pos_vel;
        if !on_island {
            let is_peel = world.obj::<BananaLogic>(this).is_peel;
            let behavior = if is_peel { "BananaPeelBounce" } else { "BananaBounce" };
            logic::set_behavior(world, this, Some(behavior));
        }

        let y_pos_vel = logic::logic(world, this).pos_vel.y;
        if y_pos_vel < coconut_logic::THROWABLE_BOUNCE_SPEED {
            logic::logic_mut(world, this).pos_vel.y = 0.0;
            let on_island = world.obj::<BananaLogic>(this).on_island;
            if !on_island {
                {
                    let mut banana = world.obj_mut::<BananaLogic>(this);
                    banana.falling = false;
                    banana.on_island = true;
                }
                logic::logic_mut(world, this).pos_vel.x = 0.0;
                world.obj_mut::<BananaLogic>(this).island_floor_collision = true;
            }
            display::object_mut(world, display_object).rot.z = 0.0;
        }

        world.obj_mut::<BananaLogic>(this).dont_hit_until_bounce = false;
        display::object_mut(world, display_object).pos.y = FLOOR_Y;
    } else {
        world.obj_mut::<BananaLogic>(this).on_island = false;
    }

    let y_pos = display::object(world, display_object).pos.y;
    if coconut_logic::is_throwable_out_of_range(y_pos) {
        logic::set_behavior(world, this, Some("BananaDie"));
    }

    check_pygmy_collision(world, this);
}

/// `-[BananaLogic refreshTiltState]` @ 0x10001c044
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let banana = world.obj::<BananaLogic>(this);
    if banana.dragging || banana.falling || !banana.on_island {
        return;
    }

    let tilt_state = coconut_logic::throwable_tilt_state(world);
    let current_tilt_state = world.obj::<BananaLogic>(this).tilt_state;
    if tilt_state != current_tilt_state {
        if tilt_state == 0 {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            logic::set_behavior(world, this, Some("BananaStopSlide"));
            world.obj_mut::<BananaLogic>(this).tilt_state = 0;
        } else if tilt_state == 1 {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            logic::set_behavior(world, this, Some("BananaTiltSlide"));
            world.obj_mut::<BananaLogic>(this).tilt_state = 1;
        }
    }

    if coconut_logic::is_island_tipped_over(world) {
        {
            let mut banana = world.obj_mut::<BananaLogic>(this);
            banana.falling = true;
            banana.tilt_state = 0;
        }
        logic::set_behavior(world, this, Some("BananaFall"));
    }
}

/// `-[BananaLogic undoTiltState]` @ 0x10001c224
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    let banana = world.obj::<BananaLogic>(this);
    if banana.dragging || banana.falling || !banana.on_island {
        return;
    }

    if banana.tilt_state == 1 {
        logic::logic_mut(world, this).pos_vel.x = 0.0;
        logic::set_behavior(world, this, Some("BananaStopSlide"));
    }
    world.obj_mut::<BananaLogic>(this).tilt_state = 0;
}

/// `-[BananaLogic slide:::]` @ 0x10001c2b8
pub fn seq_slide(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<BananaLogic>(this).enable_island_floor_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }
    ItemResult::Pending
}

/// `-[BananaLogic drag:::]` @ 0x10001c2e4
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<BananaLogic>(this).enable_island_floor_collision = false;
    }

    let display_object = logic::display_object(world, this);
    let display = display::object(world, display_object);
    let pos = display.pos;
    let display_group = display.display_group;
    let touch_screen_pos = world.obj::<BananaLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    // Original quirk, kept: the compiled code subtracts the banana's own z (not the touch's),
    // so the offset's z is 0.
    let subtracted_z = pos.z;
    let offset_x = pos.x - touch_pos.x;
    let offset_y = pos.y - touch_pos.y;
    let offset_z = pos.z - subtracted_z;
    let offset = Vec4::new(offset_x, offset_y, offset_z, pos.w);

    let length = length3(offset);
    if length != 0.0 {
        if offset.y != 0.0 {
            let z_rot = coconut_logic::throwable_drag_z_rot(offset);
            display::object_mut(world, display_object).rot.z = z_rot;
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let spring_force = coconut_logic::throwable_spring_force(offset, length, pos_vel, DRAG_STIFFNESS, DRAG_DAMPING);
        world.obj_mut::<BananaLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[BananaLogic fall:::]` @ 0x10001c494
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut banana = world.obj_mut::<BananaLogic>(this);
            if !banana.island_floor_collision {
                banana.on_island = false;
            }
            banana.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<BananaLogic>(this).falling = true;
    }

    let island_floor_collision = world.obj::<BananaLogic>(this).island_floor_collision;
    if !island_floor_collision {
        return ItemResult::Pending;
    }

    world.obj_mut::<BananaLogic>(this).falling = false;
    ItemResult::Done
}

/// `-[BananaLogic fallFromHurricane:::]` @ 0x10001c50c
pub fn seq_fall_from_hurricane(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<BananaLogic>(this).enable_island_floor_collision = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<BananaLogic>(this).falling = true;
    }
    ItemResult::Pending
}

/// `-[BananaLogic die:::]` @ 0x10001c544
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    reset(world, this);
    world.obj_mut::<BananaLogic>(this).active = false;

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[BananaLogic pickedUpByHurricane:]` @ 0x10001c5a0
pub fn picked_up_by_hurricane(world: &mut World, this: Entity, picked_up_by_hurricane: Entity) -> bool {
    let banana = world.obj::<BananaLogic>(this);
    let already_picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let game_frame_running = logic::logic(world, this).game_frame_running;
    let can_be_picked_up = !banana.touch_began_inside
        && !already_picked_up
        && !banana.dragging
        && !banana.falling
        && !banana.being_eaten
        && !banana.in_water
        && banana.active
        && game_frame_running;
    if !can_be_picked_up {
        return false;
    }

    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_hurricane_logic = Some(picked_up_by_hurricane);
        sprite.ignore_gravity = true;
        sprite.blown_by_hurricane = false;
        sprite.picked_up_by_hurricane = true;
    }
    world.obj_mut::<BananaLogic>(this).tilt_state = 0;
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

    let is_peel = world.obj::<BananaLogic>(this).is_peel;
    let behavior = if is_peel { "BananaPeelPickedUpByHurricane" } else { "BananaPickedUpByHurricane" };
    logic::set_behavior(world, this, Some(behavior));
    true
}

/// `-[BananaLogic droppedByHurricane:]` @ 0x10001c730
pub fn dropped_by_hurricane(world: &mut World, this: Entity, _dropped_by_hurricane: Entity) {
    let picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    if !picked_up {
        return;
    }

    world.obj_mut::<BananaLogic>(this).falling = true;
    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.ignore_gravity = false;
        sprite.blown_by_hurricane = false;
        sprite.picked_up_by_hurricane = false;
        sprite.picked_up_by_hurricane_logic = None;
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    let z_pos = display::object(world, display_object).pos.z;
    if z_pos < coconut_logic::ISLAND_Z_POS {
        move_to_behind_island_layer(world, this);
    }
    logic::set_behavior(world, this, Some("BananaDroppedByHurricane"));
}

/// `-[BananaLogic touchBeganInside:allTouches:]` @ 0x10001c824
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let banana = world.obj::<BananaLogic>(this);
    let busy = banana.falling || banana.in_water || banana.being_eaten || banana.active_touch.is_some();
    if busy {
        return;
    }

    world.obj_mut::<BananaLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut banana = world.obj_mut::<BananaLogic>(this);
        banana.touch_began_inside = true;
        banana.touch_screen_pos = touch_pos;
        banana.dragging = true;
        banana.apply_spring = true;
        // Original quirk: cleared again straight away.
        banana.touch_began_inside = false;
        banana.tilt_state = 0;
    }

    let listener = world.obj::<BananaLogic>(this).banana_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_banana_drag(world, listener, this);
    }

    let is_peel = world.obj::<BananaLogic>(this).is_peel;
    let behavior = if is_peel { "BananaPeelDrag" } else { "BananaDrag" };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[BananaLogic touchBeganOutside:allTouches:]` @ 0x10001c960
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BananaLogic>(this).active_touch;
    if active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let mut banana = world.obj_mut::<BananaLogic>(this);
    banana.touch_screen_pos = touch_pos;
    banana.touch_began_inside = false;
}

/// `-[BananaLogic touchMovedEntered:allTouches:]` @ 0x10001c9bc
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BananaLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<BananaLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[BananaLogic touchMovedExited:allTouches:]` @ 0x10001ca14
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BananaLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<BananaLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[BananaLogic touchMovedInside:allTouches:]` @ 0x10001ca6c
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[BananaLogic touchMovedOutside:allTouches:]` @ 0x10001ca7c
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BananaLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<BananaLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[BananaLogic touchEndedInside:allTouches:]` @ 0x10001cad4
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BananaLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    // The dragged banana is thrown.
    let dragging = world.obj::<BananaLogic>(this).dragging;
    if dragging {
        {
            let mut banana = world.obj_mut::<BananaLogic>(this);
            banana.falling = true;
            banana.dragging = false;
            banana.apply_spring = false;
        }
        move_to_island_layer(world, this);
        notify_banana_drag_complete(world, this);
        logic::set_behavior(world, this, Some("BananaFall"));
    }

    let mut banana = world.obj_mut::<BananaLogic>(this);
    banana.touch_began_inside = false;
    banana.active_touch = None;
}

/// `-[BananaLogic touchEndedOutside:allTouches:]` @ 0x10001cb9c
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
