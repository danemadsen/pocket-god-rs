//! `MoronSnakeLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::coconut_logic::{
    THROWABLE_GRAVITY_FORCE, integrate_throwable, is_island_tipped_over, is_throwable_out_of_range,
    throwable_drag_z_rot, throwable_pos_force, throwable_spring_force,
};
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::moronpests_chase::moron_turtle_logic;
use crate::logic::moronpests_quiz::moron_pests_question_screen_logic;
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::logic;
use pg_engine::math::{length3, normalize3, trunc_i32};
use pg_engine::touch::BcTouch;
use pg_engine::view::BcView;

/// `MoronSnakeLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct MoronSnakeLogic {
    /// `mMoronSnakeLogicListener`: `@"<MoronSnakeLogicListener>"`
    pub moron_snake_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mEatingPygmyLogic`: `@"PygmyLogic"`
    pub eating_pygmy_logic: Option<Entity>,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mLastPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_pos: Vec4,

    /// `mTiltState`: `i`
    pub tilt_state: i32,

    /// `mSlideFriction`: `f`
    pub slide_friction: f32,

    /// `mSpringDelta`: `f`
    pub spring_delta: f32,

    /// `mbDisableCollision`: `B`
    pub disable_collision: bool,

    /// `mbOnIsland`: `B`
    pub on_island: bool,

    /// `mbEnableIslandFloorCollision`: `B`
    pub enable_island_floor_collision: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mbEnableIslandSideCollision`: `B`
    pub enable_island_side_collision: bool,

    /// `mbIslandSideCollision`: `B`
    pub island_side_collision: bool,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbFalling`: `B`
    pub falling: bool,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,

    /// `mbInWater`: `B`
    pub in_water: bool,

    /// `mDifficulty`: `f`
    pub difficulty: f32,

    /// `mbMoronPests`: `B`
    pub moron_pests: bool,

    /// `mMoronTurtleLogic`: `@"MoronTurtleLogic"`
    pub moron_turtle_logic: Option<Entity>,

    /// `mMoronPestsPygmyLogic`: `@"PygmyLogic"`
    pub moron_pests_pygmy_logic: Option<Entity>,

    /// `mMoronPestsBackgroundLogic`: `@"MultiModelLogic"`
    pub moron_pests_background_logic: Option<Entity>,

    /// `mMoronPestsRibBackgroundLogic`: `@"MultiModelLogic"`
    pub moron_pests_rib_background_logic: Option<Entity>,

    /// `mMoronPestsDisplayGroup`: `@"BCDisplayGroup"`
    pub moron_pests_display_group: Option<Entity>,

    /// `mMoronPestsQuestionScreenLogic`: `@"MoronPestsQuestionScreenLogic"`
    pub moron_pests_question_screen_logic: Option<Entity>,
}

pub const MORON_SNAKE_LOGIC: Class = Class {
    name: "MoronSnakeLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
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

/// `mSlideFriction` (`0x3ccccccd`).
const SLIDE_FRICTION: f32 = 0.025;

/// The touch box: ±32 around the snake on both axes.
const COLLISION_HALF_SIZE: f32 = 32.0;

/// The island's ends: the snake turns at them and falls off beyond them.
const ISLAND_MIN_X_POS: f32 = 32.0;
const ISLAND_MAX_X_POS: f32 = 420.0;

/// The island floor.
const FLOOR_Y_POS: f32 = 85.0;

/// `fall:::` splashes below this y.
const WATER_LINE_Y_POS: f32 = 10.0;

/// The spring of `drag:::`.
const DRAG_STIFFNESS: f32 = 100.0;
const DRAG_DAMPING: f32 = 8.0;

/// `checkPygmyCollision`: a falling snake within 30 of a point 31 above a pygmy eats it.
const EAT_DISTANCE: f32 = 30.0;
const EAT_Y_OFFSET: f32 = 31.0;
const EAT_MIN_FALL_SPEED: f32 = -20.0;

/// `timeTemplePygmyType` of a baby pygmy.
const TIME_TEMPLE_PYGMY_TYPE_BABY: i32 = 2;

/// `thrownByBaby:` velocity: x per unit of the baby's x scale, and y.
const BABY_THROW_X_SPEED: f32 = -300.0;
const BABY_THROW_Y_SPEED: f32 = 800.0;

/// Where `enterIsland:` drops the snake in.
const ENTER_X_POS: f32 = 200.0;
const ENTER_Y_POS: f32 = 400.0;

/// Moron Pests: the score (and question difficulty) per point the display group scrolled left.
const DIFFICULTY_PER_SCROLL: f32 = -0.3125;

/// Moron Pests parallax: the backgrounds scroll at these fractions of the display group.
const BACKGROUND_PARALLAX: f32 = -0.4;
const RIB_BACKGROUND_PARALLAX: f32 = -0.3;

/// Sequence-item handlers implemented by `MoronSnakeLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "resetLastPos" => seq_reset_last_pos(world, this, call),
        "whichWayDoIGo" => seq_which_way_do_i_go(world, this, call),
        "slither" => seq_slither(world, this, call),
        "idle" => seq_idle(world, this, call),
        "drag" => seq_drag(world, this, call),
        "fall" => seq_fall(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `MoronSnakeLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animThrowPygmy" => anim_throw_pygmy(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[MoronSnakeLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&MORON_SNAKE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[MoronSnakeLogic initWithDisplayObject:]` @ 0x10018bf1c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(MoronSnakeLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().moron_snake_logic = Some(this);

    {
        let mut snake = world.obj_mut::<MoronSnakeLogic>(this);
        snake.tilt_state = 0;
        snake.slide_friction = SLIDE_FRICTION;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<MoronSnakeLogic>(this).enable_island_floor_collision = true;

    let moron_turtle_logic = pg_engine::library::logic(world, crate::logic_table::MORON_TURTLE);
    world.obj_mut::<MoronSnakeLogic>(this).moron_turtle_logic = Some(moron_turtle_logic);

    let question_screen_logic = pg_engine::library::logic(world, crate::logic_table::MORON_PESTS_QUESTION_SCREEN);
    world.obj_mut::<MoronSnakeLogic>(this).moron_pests_question_screen_logic = Some(question_screen_logic);

    let display_group = pg_engine::library::display_group(world, "MoronPestsDisplayGroup");
    world.obj_mut::<MoronSnakeLogic>(this).moron_pests_display_group = display_group;

    let background_logic = pg_engine::library::logic(world, crate::logic_table::MORON_PESTS_BACKGROUND);
    world.obj_mut::<MoronSnakeLogic>(this).moron_pests_background_logic = Some(background_logic);

    let rib_background_logic = pg_engine::library::logic(world, crate::logic_table::RIB_BACKGROUND);
    world.obj_mut::<MoronSnakeLogic>(this).moron_pests_rib_background_logic = Some(rib_background_logic);

    let display_object = logic::display_object(world, this);
    let mut display = display::object_mut(world, display_object);
    display.col_x_min = -COLLISION_HALF_SIZE;
    display.col_x_max = COLLISION_HALF_SIZE;
    display.col_y_min = -COLLISION_HALF_SIZE;
    display.col_y_max = COLLISION_HALF_SIZE;
}

/// `-[MoronSnakeLogic setMoronSnakeLogicListener:]` @ 0x10018c11c
pub fn set_moron_snake_logic_listener(world: &mut World, this: Entity, moron_snake_logic_listener: Entity) {
    world.obj_mut::<MoronSnakeLogic>(this).moron_snake_logic_listener = Some(moron_snake_logic_listener);
}

/// `-[MoronSnakeLogic thrownByBaby:]` @ 0x10018c178
pub fn thrown_by_baby(world: &mut World, this: Entity, baby_pygmy_logic: Entity) {
    let baby_display_object = logic::display_object(world, baby_pygmy_logic);
    let baby_x_scale = display::object(world, baby_display_object).scale.x;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = baby_x_scale * BABY_THROW_X_SPEED;
        state.pos_vel.y = BABY_THROW_Y_SPEED;
    }

    {
        let mut snake = world.obj_mut::<MoronSnakeLogic>(this);
        snake.falling = true;
        snake.eating_pygmy_logic = None;
    }

    logic::set_behavior(world, this, Some("MoronSnakeThrownByBaby"));
}

/// `-[MoronSnakeLogic stopInteractingWithObjects]` @ 0x10018c218
pub fn stop_interacting_with_objects(_world: &mut World, _this: Entity) {}

/// `-[MoronSnakeLogic animThrowPygmy]` @ 0x10018c21c
pub fn anim_throw_pygmy(_world: &mut World, _this: Entity) {}

/// `-[MoronSnakeLogic enterIsland:]` @ 0x10018c220
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    let active = world.obj::<MoronSnakeLogic>(this).active;
    if active {
        return;
    }

    world.obj_mut::<MoronSnakeLogic>(this).active = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    {
        let mut snake = world.obj_mut::<MoronSnakeLogic>(this);
        snake.disable_collision = true;
        snake.in_water = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut snake = world.obj_mut::<MoronSnakeLogic>(this);
        snake.enable_island_side_collision = false;
        snake.enable_island_floor_collision = false;
    }

    let display_object = logic::display_object(world, this);
    {
        let mut display = display::object_mut(world, display_object);
        display.rot.z = 0.0;
        display.pos.x = ENTER_X_POS;
        display.pos.y = ENTER_Y_POS;
        display.pos.z = SPRITE_Z;
        display.scale.x = 1.0;
        display.is_selectable = true;
    }

    let marker = pg_engine::library::display_marker(world, "SmallObjectsMarker");
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }

    logic::set_behavior(world, this, Some("MoronSnakeEnterIsland"));
}

/// `-[MoronSnakeLogic exitIsland:]` @ 0x10018c394
pub fn exit_island(world: &mut World, this: Entity, now: bool) {
    let active = world.obj::<MoronSnakeLogic>(this).active;
    if !active {
        return;
    }

    world.obj_mut::<MoronSnakeLogic>(this).active = false;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    stop_interacting_with_objects(world, this);

    let dragging = world.obj::<MoronSnakeLogic>(this).dragging;
    if dragging {
        {
            let mut snake = world.obj_mut::<MoronSnakeLogic>(this);
            snake.falling = true;
            snake.dragging = false;
            snake.apply_spring = false;
        }

        let listener = world.obj::<MoronSnakeLogic>(this).moron_snake_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_moron_snake_drag_complete(world, listener, this);
        }
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<MoronSnakeLogic>(this).enable_island_floor_collision = false;

    if now {
        logic::set_behavior(world, this, Some("MoronSnakeDie"));
        return;
    }

    let in_water = world.obj::<MoronSnakeLogic>(this).in_water;
    if in_water {
        return;
    }

    let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }

    logic::set_behavior(world, this, Some("MoronSnakeFallIgnoreIsland"));
}

/// `-[MoronSnakeLogic exitIslandNow]` @ 0x10018c50c
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[MoronSnakeLogic checkPygmyCollision]` @ 0x10018c584
pub fn check_pygmy_collision(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);

    let snake = world.obj::<MoronSnakeLogic>(this);
    let can_eat = snake.falling && snake.eating_pygmy_logic.is_none() && !snake.disable_collision;
    let listener = snake.moron_snake_logic_listener;
    if !can_eat {
        return;
    }

    // `[nil pygmyLogicArray:]` enumerates nothing.
    let pygmy_logic_array = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };

    for pygmy_logic in pygmy_logic_array {
        let pygmy_display_object = logic::display_object(world, pygmy_logic);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let pos = display::object(world, display_object).pos;
        let y_vel = logic::logic(world, this).pos_vel.y;

        let delta_x = pos.x - (pygmy_pos.x + 0.0);
        let delta_y = pos.y - (pygmy_pos.y + EAT_Y_OFFSET);
        let within_reach = delta_x.abs() < EAT_DISTANCE && delta_y.abs() < EAT_DISTANCE;
        let falling_fast = y_vel < EAT_MIN_FALL_SPEED;
        if !within_reach || !falling_fast {
            continue;
        }

        let eaten = crate::pygmy::eaten_by_moron_snake(world, pygmy_logic, this);
        if !eaten {
            continue;
        }

        // The pygmy's scale and position are read again after it was eaten.
        let pygmy_display = display::object(world, pygmy_display_object);
        let pygmy_x_scale = pygmy_display.scale.x;
        let pygmy_x_pos = pygmy_display.pos.x;

        display::object_mut(world, display_object).scale.x = pygmy_x_scale;
        world.obj_mut::<MoronSnakeLogic>(this).falling = false;
        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }
        display::object_mut(world, display_object).pos.x = pygmy_x_pos;
        world.obj_mut::<MoronSnakeLogic>(this).eating_pygmy_logic = Some(pygmy_logic);

        let time_temple_pygmy_type = crate::pygmy::time_temple_pygmy_type(world, pygmy_logic);
        let behavior = if time_temple_pygmy_type == TIME_TEMPLE_PYGMY_TYPE_BABY {
            "MoronSnakeEatPygmyBaby"
        } else {
            "MoronSnakeEatPygmy"
        };
        logic::set_behavior(world, this, Some(behavior));
        return;
    }
}

/// `-[MoronSnakeLogic refreshTiltState]` @ 0x10018c890
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let ignore_gravity = world.obj::<SpriteLogic>(this).ignore_gravity;
    let dragging = world.obj::<MoronSnakeLogic>(this).dragging;
    if ignore_gravity || dragging {
        return;
    }

    if !is_island_tipped_over(world) {
        return;
    }

    world.obj_mut::<MoronSnakeLogic>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<MoronSnakeLogic>(this).tilt_state = 0;
    stop_interacting_with_objects(world, this);
    logic::set_behavior(world, this, Some("MoronSnakeFall"));
}

/// `-[MoronSnakeLogic undoTiltState]` @ 0x10018c974
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    world.obj_mut::<MoronSnakeLogic>(this).tilt_state = 0;
}

/// `-[MoronSnakeLogic isAccelEnabled]` @ 0x10018c984
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<MoronSnakeLogic>(this).moron_snake_logic_listener;
    let gravity_enabled = match listener {
        Some(listener) => crate::controller::is_gravity_enabled(world, listener),
        None => false,
    };
    if !gravity_enabled {
        return false;
    }

    let display_object = logic::display_object(world, this);
    let selectable = display::object(world, display_object).is_selectable;
    if !selectable {
        return false;
    }

    sprite_logic::is_accel_enabled(world, this)
}

/// `-[MoronSnakeLogic onGameFrameMoronPests:]` @ 0x10018ca18
pub fn on_game_frame_moron_pests(world: &mut World, this: Entity, _dt: f32) {
    let snake = world.obj::<MoronSnakeLogic>(this);
    let display_group = snake.moron_pests_display_group;
    let listener = snake.moron_snake_logic_listener;
    let question_screen_logic = snake.moron_pests_question_screen_logic;
    let background_logic = snake.moron_pests_background_logic;
    let rib_background_logic = snake.moron_pests_rib_background_logic;

    // The library always has the group (the original would read through NULL otherwise).
    let Some(display_group) = display_group else {
        return;
    };

    let group_x_pos = display::object(world, display_group).pos.x;
    let difficulty = group_x_pos * DIFFICULTY_PER_SCROLL;
    world.obj_mut::<MoronSnakeLogic>(this).difficulty = difficulty;

    if let Some(listener) = listener {
        let score = trunc_i32(difficulty);
        crate::controller::on_moron_pests_score(world, listener, score);
    }

    if let Some(question_screen_logic) = question_screen_logic {
        let difficulty = world.obj::<MoronSnakeLogic>(this).difficulty;
        moron_pests_question_screen_logic::set_difficulty(world, question_screen_logic, difficulty);
    }

    // The group's x is read again through its live position.
    if let Some(background_logic) = background_logic {
        let group_x_pos = display::object(world, display_group).pos.x;
        let background_display_object = logic::display_object(world, background_logic);
        display::object_mut(world, background_display_object).pos.x = group_x_pos * BACKGROUND_PARALLAX;
    }

    if let Some(rib_background_logic) = rib_background_logic {
        let group_x_pos = display::object(world, display_group).pos.x;
        let rib_background_display_object = logic::display_object(world, rib_background_logic);
        display::object_mut(world, rib_background_display_object).pos.x = group_x_pos * RIB_BACKGROUND_PARALLAX;
    }
}

/// `-[MoronSnakeLogic onGameFrame:]` @ 0x10018cb30
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let moron_pests = world.obj::<MoronSnakeLogic>(this).moron_pests;
    if moron_pests {
        on_game_frame_moron_pests(world, this, dt);
        return;
    }

    let display_object = logic::display_object(world, this);

    let accel_enabled = is_accel_enabled(world, this);
    if accel_enabled {
        let gravity_force = tilted_gravity_force(world);
        world.obj_mut::<SpriteLogic>(this).gravity_force = gravity_force;
        refresh_tilt_state(world, this);
    } else {
        world.obj_mut::<SpriteLogic>(this).gravity_force = THROWABLE_GRAVITY_FORCE;
        undo_tilt_state(world, this);
    }

    let sprite = world.obj::<SpriteLogic>(this);
    let gravity_force = sprite.gravity_force;
    let ignore_gravity = sprite.ignore_gravity;
    let snake = world.obj::<MoronSnakeLogic>(this);
    let spring_force = snake.spring_force;
    let apply_spring = snake.apply_spring;

    let pos_force = throwable_pos_force(gravity_force, ignore_gravity, spring_force, apply_spring);
    logic::logic_mut(world, this).pos_force = pos_force;
    integrate_throwable(world, this, pos_force, dt);

    stop_at_island_ends(world, this, display_object);
    land_on_island_floor(world, this, display_object);

    let y_pos = display::object(world, display_object).pos.y;
    if is_throwable_out_of_range(y_pos) {
        logic::set_behavior(world, this, Some("MoronSnakeDie"));
    }

    check_pygmy_collision(world, this);

    let pos = display::object(world, display_object).pos;
    world.obj_mut::<MoronSnakeLogic>(this).last_pos = pos;
}

/// The gravity of `onGameFrame:` while the accelerometer tilts the world: `(−accelForce.y,
/// accelForce.x, 0, 1)` normalised (`FUN_1001a157c`, w included), then y and z times 3000 and
/// x cleared, so the snake only ever falls straight down.
fn tilted_gravity_force(world: &World) -> Vec4 {
    let accel_force = world.resource::<BcView>().accel_force;
    let tilted = Vec4::new(-accel_force.y, accel_force.x, 0.0, 1.0);
    let normalized = normalize3(tilted);

    let gravity_y = normalized.y * 3000.0;
    let gravity_z = normalized.z * 3000.0;
    Vec4::new(0.0, gravity_y, gravity_z, normalized.w)
}

/// The side collision of `onGameFrame:`: a slithering snake that crosses an end of the island
/// this frame stops there.
fn stop_at_island_ends(world: &mut World, this: Entity, display_object: Entity) {
    world.obj_mut::<MoronSnakeLogic>(this).island_side_collision = false;

    let snake = world.obj::<MoronSnakeLogic>(this);
    let enable_island_side_collision = snake.enable_island_side_collision;
    let last_x_pos = snake.last_pos.x;
    if !enable_island_side_collision {
        return;
    }

    let x_pos = display::object(world, display_object).pos.x;
    let crossed_min = x_pos < ISLAND_MIN_X_POS && last_x_pos >= ISLAND_MIN_X_POS;
    let crossed_max = x_pos > ISLAND_MAX_X_POS && last_x_pos <= ISLAND_MAX_X_POS;
    let end_x_pos = if crossed_min {
        ISLAND_MIN_X_POS
    } else if crossed_max {
        ISLAND_MAX_X_POS
    } else {
        return;
    };

    logic::logic_mut(world, this).pos_vel.x = 0.0;
    display::object_mut(world, display_object).pos.x = end_x_pos;
    world.obj_mut::<MoronSnakeLogic>(this).island_side_collision = true;
}

/// The floor collision of `onGameFrame:`: a snake over the island that crosses the floor this
/// frame lands on it upright and can eat pygmies again.
fn land_on_island_floor(world: &mut World, this: Entity, display_object: Entity) {
    world.obj_mut::<MoronSnakeLogic>(this).island_floor_collision = false;

    let pos = display::object(world, display_object).pos;
    let snake = world.obj::<MoronSnakeLogic>(this);
    let last_y_pos = snake.last_pos.y;
    let enable_island_floor_collision = snake.enable_island_floor_collision;

    let over_island = pos.x >= ISLAND_MIN_X_POS && pos.x <= ISLAND_MAX_X_POS;
    let crossed_floor = last_y_pos >= FLOOR_Y_POS && pos.y < FLOOR_Y_POS;
    if !over_island || !crossed_floor || !enable_island_floor_collision {
        world.obj_mut::<MoronSnakeLogic>(this).on_island = false;
        return;
    }

    {
        let mut snake = world.obj_mut::<MoronSnakeLogic>(this);
        if !snake.on_island {
            snake.on_island = true;
            snake.island_floor_collision = true;
        }
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    {
        let mut display = display::object_mut(world, display_object);
        display.pos.y = FLOOR_Y_POS;
        display.rot.z = 0.0;
    }

    world.obj_mut::<MoronSnakeLogic>(this).disable_collision = false;
}

/// `-[MoronSnakeLogic resetLastPos:::]` @ 0x10018cfac
pub fn seq_reset_last_pos(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    world.obj_mut::<MoronSnakeLogic>(this).last_pos = pos;
    ItemResult::Done
}

/// `-[MoronSnakeLogic whichWayDoIGo:::]` @ 0x10018cffc
pub fn seq_which_way_do_i_go(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    let random = world.resource_mut::<Random>().arc4random();

    let display = display::object(world, display_object);
    let x_pos = display.pos.x;
    let x_scale = display.scale.x;

    // A coin toss picks the heading, except at the island's ends, which send the snake back.
    let mut head_right = random & 1 != 0;
    if x_pos == ISLAND_MIN_X_POS {
        head_right = true;
    }
    if x_pos == ISLAND_MAX_X_POS {
        head_right = false;
    }

    let facing_heading = if head_right { x_scale > 0.0 } else { x_scale < 0.0 };
    let behavior = if facing_heading { "MoronSnakeSlither" } else { "MoronSnakeTurn" };
    logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `-[MoronSnakeLogic slither:::]` @ 0x10018d0e8
pub fn seq_slither(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = logic::random_scaled_time(min_time, max_time, 100.0, random);
        logic::logic_mut(world, this).stall_time = stall_time;
        world.obj_mut::<MoronSnakeLogic>(this).enable_island_side_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let island_side_collision = world.obj::<MoronSnakeLogic>(this).island_side_collision;
    let stall_time = logic::logic(world, this).stall_time;
    let result = if island_side_collision || stall_time <= 0.0 {
        logic::logic_mut(world, this).pos_vel.x = 0.0;
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    // The timer runs down even on the frame the slither ends.
    let delta_time = call.delta_time;
    logic::logic_mut(world, this).stall_time -= delta_time;
    result
}

/// `-[MoronSnakeLogic idle:::]` @ 0x10018d238
pub fn seq_idle(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = logic::random_scaled_time(min_time, max_time, 100.0, random);
        logic::logic_mut(world, this).stall_time = stall_time;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let stall_time = logic::logic(world, this).stall_time;
    if stall_time <= 0.0 {
        return ItemResult::Done;
    }

    let delta_time = call.delta_time;
    logic::logic_mut(world, this).stall_time -= delta_time;
    ItemResult::Pending
}

/// `-[MoronSnakeLogic drag:::]` @ 0x10018d34c
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<MoronSnakeLogic>(this).enable_island_floor_collision = false;
    }

    let display_object = logic::display_object(world, this);
    let display = display::object(world, display_object);
    let pos = display.pos;
    let display_group = display.display_group;

    // `[nil viewToDisplayObject:]` answers the zero point.
    let touch_screen_pos = world.obj::<MoronSnakeLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    // Original bug, kept: the converted touch point's z is overwritten with `pos.z` before the
    // subtraction (`ldr s2, [x20, #8]` after `viewToDisplayObject:`), so the offset has no z.
    let offset_x = pos.x - touch_pos.x;
    let offset_y = pos.y - touch_pos.y;
    let offset_z = pos.z - pos.z;
    let offset = Vec4::new(offset_x, offset_y, offset_z, pos.w);

    let length = length3(offset);
    if length != 0.0 {
        if offset.y != 0.0 {
            let z_rot = throwable_drag_z_rot(offset);
            display::object_mut(world, display_object).rot.z = z_rot;
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let spring_force = throwable_spring_force(offset, length, pos_vel, DRAG_STIFFNESS, DRAG_DAMPING);
        world.obj_mut::<MoronSnakeLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[MoronSnakeLogic fall:::]` @ 0x10018d4fc
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut snake = world.obj_mut::<MoronSnakeLogic>(this);
            if !snake.island_floor_collision {
                snake.on_island = false;
            }
            snake.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<MoronSnakeLogic>(this).falling = true;

        let ignore_island = call.bool("ignoreIsland");
        if ignore_island {
            world.obj_mut::<MoronSnakeLogic>(this).enable_island_floor_collision = false;
        }
    }

    let display_object = logic::display_object(world, this);
    let island_floor_collision = world.obj::<MoronSnakeLogic>(this).island_floor_collision;
    if island_floor_collision {
        let mut snake = world.obj_mut::<MoronSnakeLogic>(this);
        snake.falling = false;
        snake.enable_island_side_collision = true;
        return ItemResult::Done;
    }

    let y_pos = display::object(world, display_object).pos.y;
    if y_pos < WATER_LINE_Y_POS {
        splash(world, this, display_object);
        return ItemResult::Replaced;
    }
    ItemResult::Pending
}

/// The water landing of `fall:::`: the snake stops in the water and plays `MoronSnakeSplash`.
fn splash(world: &mut World, this: Entity, display_object: Entity) {
    {
        let mut snake = world.obj_mut::<MoronSnakeLogic>(this);
        snake.falling = false;
        snake.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
        state.rot_vel.z = 0.0;
    }

    display::object_mut(world, display_object).rot.z = 0.0;
    logic::set_behavior(world, this, Some("MoronSnakeSplash"));
}

/// `-[MoronSnakeLogic die:::]` @ 0x10018d680
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    stop_interacting_with_objects(world, this);

    let dragging = world.obj::<MoronSnakeLogic>(this).dragging;
    if dragging {
        world.obj_mut::<MoronSnakeLogic>(this).dragging = false;

        let listener = world.obj::<MoronSnakeLogic>(this).moron_snake_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_moron_snake_drag_complete(world, listener, this);
        }
    }

    {
        let mut snake = world.obj_mut::<MoronSnakeLogic>(this);
        snake.active = false;
        snake.falling = false;
        snake.in_water = false;
        snake.apply_spring = false;
        snake.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    // `[nil isMoronSnakeEnabled]` is NO.
    let listener = world.obj::<MoronSnakeLogic>(this).moron_snake_logic_listener;
    let Some(listener) = listener else {
        return ItemResult::Done;
    };

    let moron_snake_enabled = crate::controller::is_moron_snake_enabled(world, listener);
    if moron_snake_enabled {
        let story_active = crate::controller::is_story_active(world, listener);
        if !story_active {
            enter_island(world, this, false);
        }
    }
    ItemResult::Done
}

/// `-[MoronSnakeLogic touchBeganInside:allTouches:]` @ 0x10018d7bc
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<MoronSnakeLogic>(this).moron_snake_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };
    if story_active {
        return;
    }

    let snake = world.obj::<MoronSnakeLogic>(this);
    let busy = snake.eating_pygmy_logic.is_some() || snake.falling || snake.in_water || snake.active_touch.is_some();
    if busy {
        return;
    }

    stop_interacting_with_objects(world, this);
    world.obj_mut::<MoronSnakeLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut snake = world.obj_mut::<MoronSnakeLogic>(this);
        snake.touch_screen_pos = touch_pos;
        snake.tilt_state = 0;
        snake.dragging = true;
        snake.apply_spring = true;
        snake.enable_island_floor_collision = false;
        snake.enable_island_side_collision = false;
    }

    if let Some(listener) = listener {
        crate::controller::on_moron_snake_drag(world, listener, this);
    }
    logic::set_behavior(world, this, Some("MoronSnakeDrag"));
}

/// `-[MoronSnakeLogic touchBeganOutside:allTouches:]` @ 0x10018d914
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<MoronSnakeLogic>(this).active_touch;
    if active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<MoronSnakeLogic>(this).touch_screen_pos = touch_pos;
}

/// `-[MoronSnakeLogic touchMovedEntered:allTouches:]` @ 0x10018d964
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    follow_touch(world, this, touch);
}

/// `-[MoronSnakeLogic touchMovedExited:allTouches:]` @ 0x10018d9bc
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    follow_touch(world, this, touch);
}

/// `-[MoronSnakeLogic touchMovedInside:allTouches:]` @ 0x10018da14
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[MoronSnakeLogic touchMovedOutside:allTouches:]` @ 0x10018da24
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    follow_touch(world, this, touch);
}

/// The body shared by the `touchMoved…` handlers: the dragging finger (or any finger while none
/// drags) becomes the drag target.
fn follow_touch(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<MoronSnakeLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<MoronSnakeLogic>(this).touch_screen_pos = touch_pos;
}

/// `-[MoronSnakeLogic touchEndedInside:allTouches:]` @ 0x10018da7c
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<MoronSnakeLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let dragging = world.obj::<MoronSnakeLogic>(this).dragging;
    if dragging {
        {
            let mut snake = world.obj_mut::<MoronSnakeLogic>(this);
            snake.falling = true;
            snake.dragging = false;
            snake.apply_spring = false;
        }

        let display_object = logic::display_object(world, this);
        let marker = pg_engine::library::display_marker(world, "SmallObjectsMarker");
        if let Some(marker) = marker {
            display::insert_before(world, marker, Some(display_object));
        }

        let listener = world.obj::<MoronSnakeLogic>(this).moron_snake_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_moron_snake_drag_complete(world, listener, this);
        }
        logic::set_behavior(world, this, Some("MoronSnakeFall"));
    }

    world.obj_mut::<MoronSnakeLogic>(this).active_touch = None;
}

/// `-[MoronSnakeLogic touchEndedOutside:allTouches:]` @ 0x10018db74
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}

/// `-[MoronSnakeLogic eatPygmyComplete:]` @ 0x10018db84
pub fn eat_pygmy_complete(world: &mut World, this: Entity, _pygmy_logic: Entity) {
    world.obj_mut::<MoronSnakeLogic>(this).eating_pygmy_logic = None;
    logic::set_behavior(world, this, Some("MoronSnakeIdle"));
}

/// `-[MoronSnakeLogic startMoronPests:]` @ 0x10018dba8
pub fn start_moron_pests(world: &mut World, this: Entity, pygmy_logic: Entity) {
    {
        let mut snake = world.obj_mut::<MoronSnakeLogic>(this);
        snake.moron_pests = true;
        snake.difficulty = 0.0;
        snake.moron_pests_pygmy_logic = Some(pygmy_logic);
    }

    crate::pygmy::start_moron_pests(world, pygmy_logic);

    // The original reads mMoronPestsPygmyLogic again for each call; nothing above changes it.
    let snake = world.obj::<MoronSnakeLogic>(this);
    let moron_turtle_logic = snake.moron_turtle_logic;
    let question_screen_logic = snake.moron_pests_question_screen_logic;
    let listener = snake.moron_snake_logic_listener;

    if let Some(moron_turtle_logic) = moron_turtle_logic {
        moron_turtle_logic::start_moron_pests(world, moron_turtle_logic, pygmy_logic);
    }

    if let Some(question_screen_logic) = question_screen_logic {
        moron_pests_question_screen_logic::set_moron_pests_pygmy_logic(world, question_screen_logic, pygmy_logic);
    }

    if let Some(listener) = listener {
        crate::controller::on_moron_pests_start(world, listener, this, pygmy_logic);
    }
}

/// `-[MoronSnakeLogic stopMoronPests]` @ 0x10018dc60
pub fn stop_moron_pests(world: &mut World, this: Entity) {
    pg_engine::view::remove_game_frame_listener(world, this);

    {
        let mut snake = world.obj_mut::<MoronSnakeLogic>(this);
        snake.moron_pests = false;
        snake.moron_pests_pygmy_logic = None;
    }

    let question_screen_logic = world.obj::<MoronSnakeLogic>(this).moron_pests_question_screen_logic;
    if let Some(question_screen_logic) = question_screen_logic {
        let class = isa(world, question_screen_logic);
        (class.close_screen_now)(world, question_screen_logic);
    }

    let listener = world.obj::<MoronSnakeLogic>(this).moron_snake_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_moron_pests_stop(world, listener, this);
    }
}

/// `-[MoronSnakeLogic pygmyIntroComplete]` @ 0x10018dcf8
pub fn pygmy_intro_complete(world: &mut World, this: Entity) {
    pg_engine::view::add_game_frame_listener(world, this);

    let question_screen_logic = world.obj::<MoronSnakeLogic>(this).moron_pests_question_screen_logic;
    if let Some(question_screen_logic) = question_screen_logic {
        moron_pests_question_screen_logic::open_moron_pests_question(world, question_screen_logic);
    }

    let listener = world.obj::<MoronSnakeLogic>(this).moron_snake_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_moron_pests_start_moving_camera(world, listener);
    }
}

/// `-[MoronSnakeLogic onMoronTurtleCatchPygmy]` @ 0x10018dd74
pub fn on_moron_turtle_catch_pygmy(world: &mut World, this: Entity) {
    let question_screen_logic = world.obj::<MoronSnakeLogic>(this).moron_pests_question_screen_logic;
    if let Some(question_screen_logic) = question_screen_logic {
        moron_pests_question_screen_logic::close_screen(world, question_screen_logic);
    }
}

/// `-[MoronSnakeLogic onMoronPestsStopMovingCamera]` @ 0x10018dd90
pub fn on_moron_pests_stop_moving_camera(world: &mut World, this: Entity) {
    let listener = world.obj::<MoronSnakeLogic>(this).moron_snake_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_moron_pests_stop_moving_camera(world, listener);
    }
}

/// `-[MoronSnakeLogic onMoronPestsQuestionCorrect]` @ 0x10018ddac
pub fn on_moron_pests_question_correct(world: &mut World, this: Entity) {
    let question_screen_logic = world.obj::<MoronSnakeLogic>(this).moron_pests_question_screen_logic;
    if let Some(question_screen_logic) = question_screen_logic {
        moron_pests_question_screen_logic::open_moron_pests_question(world, question_screen_logic);
    }

    let pygmy_logic = world.obj::<MoronSnakeLogic>(this).moron_pests_pygmy_logic;
    if let Some(pygmy_logic) = pygmy_logic {
        crate::pygmy::on_moron_pests_answer_correct(world, pygmy_logic);
    }
}

/// `-[MoronSnakeLogic onMoronPestsQuestionIncorrect]` @ 0x10018ddfc
pub fn on_moron_pests_question_incorrect(world: &mut World, this: Entity) {
    let question_screen_logic = world.obj::<MoronSnakeLogic>(this).moron_pests_question_screen_logic;
    if let Some(question_screen_logic) = question_screen_logic {
        moron_pests_question_screen_logic::open_moron_pests_question(world, question_screen_logic);
    }

    let pygmy_logic = world.obj::<MoronSnakeLogic>(this).moron_pests_pygmy_logic;
    if let Some(pygmy_logic) = pygmy_logic {
        crate::pygmy::on_moron_pests_answer_incorrect(world, pygmy_logic);
    }
}

/// `+[MoronSnakeLogic instance]` @ 0x10018bf10
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().moron_snake_logic
}
