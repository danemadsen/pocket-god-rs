//! `JellyfishLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic_table;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::touch::BcTouch;
use pg_engine::{display, library, logic};

/// `JellyfishLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct JellyfishLogic {
    /// `mJellyfishLogicListener`: `@"<JellyfishLogicListener>"`
    pub jellyfish_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mTargetPos`: `{?="x"f"y"f"z"f"w"f}`
    pub target_pos: Vec4,

    /// `mbEnableScreenCollision`: `B`
    pub enable_screen_collision: bool,

    /// `mbCircleLeader`: `B`
    pub circle_leader: bool,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbDragAndMove`: `B`
    pub drag_and_move: bool,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,

    /// `mbCircleJerk`: `B`
    pub circle_jerk: bool,

    /// `mCircleJerkPos`: `{?="x"f"y"f"z"f"w"f}`
    pub circle_jerk_pos: Vec4,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,
}

pub const JELLYFISH_LOGIC: Class = Class {
    name: "JellyfishLogic",
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

/// Half the side of the jellyfish's square collision box.
const COLLISION_HALF_SIZE: f32 = 30.0;

/// The buoyancy that slowly sinks a free jellyfish.
const SWIM_GRAVITY: f32 = -30.0;

/// The gravity while it is dragged, against which the drag spring pulls.
const DRAG_GRAVITY: f32 = -3000.0;

/// The box the jellyfish stays in once it has entered (`mbEnableScreenCollision`).
const SCREEN_MIN_POS: f32 = 50.0;
const SCREEN_MAX_X_POS: f32 = 430.0;
const SCREEN_MAX_Y_POS: f32 = 280.0;

/// The height it swims up to when it leaves.
const EXIT_Y_POS: f32 = 420.0;

/// The radius of the circle the jellyfish form, and the size of the box they must gather in.
const CIRCLE_JERK_RADIUS: f32 = 60.0;

/// The bounds of the circle's centre.
const CIRCLE_JERK_MIN_POS: f32 = 100.0;
const CIRCLE_JERK_MAX_X_POS: f32 = 380.0;
const CIRCLE_JERK_MAX_Y_POS: f32 = 220.0;

/// The angle of the first place on the circle (π/2, `0x3fc90fdb`) and a full turn (2π,
/// `0x40c90fdb`).
const CIRCLE_JERK_START_ANGLE: f32 = std::f32::consts::FRAC_PI_2;
const FULL_TURN: f32 = std::f32::consts::TAU;

/// How close to the jellyfish's point of attack a pygmy must be.
const ATTACK_DISTANCE: f32 = 30.0;

/// The suffocating point is this far below the jellyfish's position, the stinging point this.
const SUFFOCATE_Y_OFFSET: f32 = -20.0;
const STING_Y_OFFSET: f32 = -80.0;

/// Sequence-item handlers implemented by `JellyfishLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "createCircleJerk" => seq_create_circle_jerk(world, this, call),
        "swimToTarget" => seq_swim_to_target(world, this, call),
        "enterIslandComplete" => seq_enter_island_complete(world, this, call),
        "swim" => seq_swim(world, this, call),
        "drag" => seq_drag(world, this, call),
        "onJellyfishAttackComplete" => seq_on_jellyfish_attack_complete(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[JellyfishLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&JELLYFISH_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[JellyfishLogic initWithDisplayObject:]` @ 0x10013b768
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(JellyfishLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let own_display_object = logic::display_object(world, this);
    let mut object = display::object_mut(world, own_display_object);
    object.col_x_min = -COLLISION_HALF_SIZE;
    object.col_x_max = COLLISION_HALF_SIZE;
    object.col_y_min = -COLLISION_HALF_SIZE;
    object.col_y_max = COLLISION_HALF_SIZE;
}

/// `-[JellyfishLogic setJellyfishLogicListener:]` @ 0x10013b834
pub fn set_jellyfish_logic_listener(world: &mut World, this: Entity, jellyfish_logic_listener: Entity) {
    world.obj_mut::<JellyfishLogic>(this).jellyfish_logic_listener = Some(jellyfish_logic_listener);
}

/// `-[JellyfishLogic enterIsland:]` @ 0x10013b890
///
/// Rises into view from below the screen toward a random height.
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    if world.obj::<JellyfishLogic>(this).active {
        return;
    }
    {
        let mut jellyfish = world.obj_mut::<JellyfishLogic>(this);
        jellyfish.active = true;
        jellyfish.enable_screen_collision = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<JellyfishLogic>(this).circle_jerk = false;

    let own_display_object = logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, own_display_object);
        object.rot.z = 0.0;
        object.is_selectable = true;
    }
    place_below_screen(world, this);

    let marker = library::display_marker(world, "CoralReefObjectDisplayMarker");
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(own_display_object));
    }
    logic::set_behavior(world, this, Some("JellyfishEnterIsland"));
}

/// `-[JellyfishLogic exitIsland:]` @ 0x10013ba88
///
/// Swims up out of the screen (`exit_island` YES: dies on the way), letting go of whatever it
/// holds.
pub fn exit_island(world: &mut World, this: Entity, exit_island: bool) {
    if !world.obj::<JellyfishLogic>(this).active {
        return;
    }
    {
        let mut jellyfish = world.obj_mut::<JellyfishLogic>(this);
        jellyfish.active = false;
        jellyfish.enable_screen_collision = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let own_display_object = logic::display_object(world, this);
    let pos_x = display::object(world, own_display_object).pos.x;
    {
        let mut jellyfish = world.obj_mut::<JellyfishLogic>(this);
        jellyfish.target_pos.x = pos_x;
        jellyfish.target_pos.y = EXIT_Y_POS;
    }
    display::object_mut(world, own_display_object).is_selectable = false;

    let (circle_jerk, circle_leader) = {
        let jellyfish = world.obj::<JellyfishLogic>(this);
        (jellyfish.circle_jerk, jellyfish.circle_leader)
    };
    if circle_jerk && circle_leader {
        let circle_effect = library::logic(world, logic_table::JELLYFISH_CIRCLE_JERK);
        logic::set_behavior(world, circle_effect, Some("JellyfishCircleJerkExit"));
    }

    if world.obj::<JellyfishLogic>(this).dragging {
        {
            let mut jellyfish = world.obj_mut::<JellyfishLogic>(this);
            jellyfish.dragging = false;
            jellyfish.apply_spring = false;
        }
        notify_drag_complete(world, this);
    }

    let pygmy = world.obj::<JellyfishLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::jellyfish_attack_complete(world, pygmy, this);
        world.obj_mut::<JellyfishLogic>(this).pygmy_logic = None;
    }

    let behavior = if exit_island { "JellyfishDie" } else { "JellyfishExitIsland" };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[JellyfishLogic exitIslandNow]` @ 0x10013bc3c
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[JellyfishLogic isCircleJerkLeader]` @ 0x10013bc50
pub fn is_circle_jerk_leader(world: &mut World, this: Entity) -> bool {
    let jellyfish = world.obj::<JellyfishLogic>(this);
    jellyfish.circle_leader && jellyfish.circle_jerk
}

/// `-[JellyfishLogic circleJerkPos]` @ 0x10013bc80
///
/// The original returns a pointer to the ivar, which its callers only read.
pub fn circle_jerk_pos(world: &mut World, this: Entity) -> Vec4 {
    world.obj::<JellyfishLogic>(this).circle_jerk_pos
}

/// `-[JellyfishLogic canInteractWithPygmy]` @ 0x10013bc90
pub fn can_interact_with_pygmy(world: &mut World, this: Entity) -> bool {
    let jellyfish = world.obj::<JellyfishLogic>(this);
    jellyfish.pygmy_logic.is_none() && !jellyfish.dragging && !jellyfish.circle_jerk
}

/// `-[JellyfishLogic canFormCircleJerk]` @ 0x10013bcd0
pub fn can_form_circle_jerk(world: &mut World, this: Entity) -> bool {
    let jellyfish = world.obj::<JellyfishLogic>(this);
    jellyfish.pygmy_logic.is_none() && !jellyfish.dragging && !jellyfish.circle_jerk
}

/// `-[JellyfishLogic checkForCircleJerk]` @ 0x10013bd10
///
/// When every jellyfish is free and they all lie within a 60 × 60 box, they form a circle of
/// radius 60 around the box's centre (kept inside x 100–380, y 100–220), this one leading.
pub fn check_for_circle_jerk(world: &mut World, this: Entity) {
    let jellyfish_array = listener_jellyfish_logic_array(world, this);

    let mut max_x_pos = 0.0_f32;
    let mut min_x_pos = 480.0_f32;
    let mut max_y_pos = 0.0_f32;
    let mut min_y_pos = 320.0_f32;
    for jellyfish in &jellyfish_array {
        if !can_form_circle_jerk(world, *jellyfish) {
            return;
        }

        let jellyfish_display_object = logic::display_object(world, *jellyfish);
        let pos = display::object(world, jellyfish_display_object).pos;
        if max_x_pos < pos.x {
            max_x_pos = pos.x;
        }
        if min_x_pos > pos.x {
            min_x_pos = pos.x;
        }
        if max_y_pos < pos.y {
            max_y_pos = pos.y;
        }
        if min_y_pos > pos.y {
            min_y_pos = pos.y;
        }
    }

    {
        let mut own = world.obj_mut::<JellyfishLogic>(this);
        own.circle_jerk_pos.x = (max_x_pos + min_x_pos) * 0.5;
        own.circle_jerk_pos.y = (max_y_pos + min_y_pos) * 0.5;
        if own.circle_jerk_pos.x < CIRCLE_JERK_MIN_POS {
            own.circle_jerk_pos.x = CIRCLE_JERK_MIN_POS;
        }
        if CIRCLE_JERK_MAX_X_POS < own.circle_jerk_pos.x {
            own.circle_jerk_pos.x = CIRCLE_JERK_MAX_X_POS;
        }
        if own.circle_jerk_pos.y < CIRCLE_JERK_MIN_POS {
            own.circle_jerk_pos.y = CIRCLE_JERK_MIN_POS;
        }
        if CIRCLE_JERK_MAX_Y_POS < own.circle_jerk_pos.y {
            own.circle_jerk_pos.y = CIRCLE_JERK_MAX_Y_POS;
        }
    }

    if !(max_x_pos - min_x_pos < CIRCLE_JERK_RADIUS) || !(max_y_pos - min_y_pos < CIRCLE_JERK_RADIUS) {
        return;
    }

    let jellyfish_count = jellyfish_array.len();
    let mut angle = CIRCLE_JERK_START_ANGLE;
    for jellyfish in &jellyfish_array {
        let center = world.obj::<JellyfishLogic>(this).circle_jerk_pos;
        let (sin, cos) = angle.sin_cos();
        let x_pos = center.x + cos * CIRCLE_JERK_RADIUS;
        let y_pos = center.y + sin * CIRCLE_JERK_RADIUS;
        start_circle_jerk_x_pos(world, *jellyfish, x_pos, y_pos);
        if *jellyfish == this {
            set_circle_leader(world, *jellyfish);
        }
        angle = angle + FULL_TURN / jellyfish_count as f32;
    }
}

/// `-[JellyfishLogic startCircleJerkXPos:yPos:]` @ 0x10013c0f4
pub fn start_circle_jerk_x_pos(world: &mut World, this: Entity, circle_jerk_x_pos: f32, y_pos: f32) {
    {
        let mut jellyfish = world.obj_mut::<JellyfishLogic>(this);
        jellyfish.circle_jerk = true;
        jellyfish.circle_leader = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    {
        let mut jellyfish = world.obj_mut::<JellyfishLogic>(this);
        jellyfish.target_pos.x = circle_jerk_x_pos;
        jellyfish.target_pos.y = y_pos;
    }
    logic::set_behavior(world, this, Some("JellyfishCircleJerk"));
}

/// `-[JellyfishLogic setCircleLeader]` @ 0x10013c15c
pub fn set_circle_leader(world: &mut World, this: Entity) {
    world.obj_mut::<JellyfishLogic>(this).circle_leader = true;
    logic::set_behavior(world, this, Some("JellyfishCircleJerkLeader"));
}

/// `-[JellyfishLogic stopCircleJerk]` @ 0x10013c184
pub fn stop_circle_jerk(world: &mut World, this: Entity) {
    world.obj_mut::<JellyfishLogic>(this).circle_jerk = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let own_display_object = logic::display_object(world, this);
    display::object_mut(world, own_display_object).rot.z = 0.0;

    if world.obj::<JellyfishLogic>(this).circle_leader {
        let circle_effect = library::logic(world, logic_table::JELLYFISH_CIRCLE_JERK);
        logic::set_behavior(world, circle_effect, Some("JellyfishCircleJerkExit"));
    }
    logic::set_behavior(world, this, Some("JellyfishIdle"));
}

/// `-[JellyfishLogic onGameFrame:]` @ 0x10013c24c
///
/// Its own integration (no `[super onGameFrame:]`): the velocity first, then the position with
/// the new velocity, held inside the screen once it has entered.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }
    let own_display_object = logic::display_object(world, this);

    let (dragging, apply_spring, spring_force, enable_screen_collision) = {
        let jellyfish = world.obj::<JellyfishLogic>(this);
        (jellyfish.dragging, jellyfish.apply_spring, jellyfish.spring_force, jellyfish.enable_screen_collision)
    };
    let gravity_y = if dragging { DRAG_GRAVITY } else { SWIM_GRAVITY };
    let gravity_force = Vec4::new(0.0, gravity_y, 0.0, 1.0);
    let ignore_gravity = {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.gravity_force = gravity_force;
        sprite.ignore_gravity
    };

    let mut pos_force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    if !ignore_gravity {
        pos_force.x = gravity_force.x + pos_force.x;
        pos_force.y = gravity_force.y + pos_force.y;
        pos_force.z = gravity_force.z + pos_force.z;
    }
    if apply_spring {
        pos_force.x = spring_force.x + pos_force.x;
        pos_force.y = spring_force.y + pos_force.y;
        pos_force.z = spring_force.z + pos_force.z;
    }

    let pos_vel = {
        let mut state = logic::logic_mut(world, this);
        state.pos_force = pos_force;
        state.pos_vel.x = state.pos_vel.x + pos_force.x * dt;
        state.pos_vel.y = state.pos_vel.y + pos_force.y * dt;
        state.pos_vel.z = state.pos_vel.z + pos_force.z * dt;
        state.pos_vel
    };

    let pos = {
        let mut object = display::object_mut(world, own_display_object);
        object.pos.x = object.pos.x + pos_vel.x * dt;
        object.pos.y = object.pos.y + pos_vel.y * dt;
        object.pos.z = object.pos.z + pos_vel.z * dt;
        object.pos
    };

    if !enable_screen_collision {
        return;
    }

    if pos.x < SCREEN_MIN_POS {
        logic::logic_mut(world, this).pos_vel.x = 0.0;
        display::object_mut(world, own_display_object).pos.x = SCREEN_MIN_POS;
    } else if SCREEN_MAX_X_POS < pos.x {
        logic::logic_mut(world, this).pos_vel.x = 0.0;
        display::object_mut(world, own_display_object).pos.x = SCREEN_MAX_X_POS;
    }

    if pos.y < SCREEN_MIN_POS {
        logic::logic_mut(world, this).pos_vel.y = 0.0;
        display::object_mut(world, own_display_object).pos.y = SCREEN_MIN_POS;
    } else if SCREEN_MAX_Y_POS < pos.y {
        logic::logic_mut(world, this).pos_vel.y = 0.0;
        display::object_mut(world, own_display_object).pos.y = SCREEN_MAX_Y_POS;
    }
}

/// `-[JellyfishLogic createCircleJerk:::]` @ 0x10013c498
///
/// The circle effect appears at the circle's centre.
pub fn seq_create_circle_jerk(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let circle_effect = library::logic(world, logic_table::JELLYFISH_CIRCLE_JERK);
    let circle_effect_display_object = logic::display_object(world, circle_effect);

    let center = world.obj::<JellyfishLogic>(this).circle_jerk_pos;
    {
        let mut object = display::object_mut(world, circle_effect_display_object);
        object.pos.x = center.x;
        object.pos.y = center.y;
        object.pos.z = SPRITE_Z;
    }

    let marker = library::display_marker(world, "CoralReefObjectDisplayMarker");
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(circle_effect_display_object));
    }
    logic::set_behavior(world, circle_effect, Some("JellyfishCircleJerkInit"));
    ItemResult::Done
}

/// `-[JellyfishLogic swimToTarget:::]` @ 0x10013c59c
///
/// Eases 10 % per frame toward `mTargetPos` for one second.
pub fn seq_swim_to_target(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let own_display_object = logic::display_object(world, this);
    if call.first_time {
        logic::logic_mut(world, this).stall_time = 1.0;
    }

    let target_pos = world.obj::<JellyfishLogic>(this).target_pos;
    {
        let mut object = display::object_mut(world, own_display_object);
        object.pos.x = object.pos.x + (target_pos.x - object.pos.x) * 0.1;
        object.pos.y = object.pos.y + (target_pos.y - object.pos.y) * 0.1;
    }

    let stall_time = logic::logic(world, this).stall_time;
    if stall_time <= 0.0 {
        return ItemResult::Done;
    }
    logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    ItemResult::Pending
}

/// `-[JellyfishLogic enterIslandComplete:::]` @ 0x10013c688
pub fn seq_enter_island_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<JellyfishLogic>(this).enable_screen_collision = true;
    ItemResult::Done
}

/// `-[JellyfishLogic swim:::]` @ 0x10013c6b0
///
/// For a random `[minTime, maxTime]` (hundredths), rises at 50 while drifting 10 sideways,
/// turned toward the middle near the edges.
pub fn seq_swim(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let own_display_object = logic::display_object(world, this);

    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let min_hundredths = pg_engine::math::trunc_i32(min_time * 100.0);
        let random = world.resource_mut::<Random>().arc4random();
        let max_hundredths = pg_engine::math::trunc_i32(max_time * 100.0);
        let range = 1_i32.wrapping_sub(min_hundredths).wrapping_add(max_hundredths) as u32;
        let offset = if range == 0 { random } else { random % range };
        let hundredths = offset.wrapping_add(min_hundredths as u32) as i32;
        logic::logic_mut(world, this).stall_time = hundredths as f32 / 100.0;

        let direction = world.resource_mut::<Random>().arc4random();
        let pos_x = display::object(world, own_display_object).pos.x;
        logic::logic_mut(world, this).pos_vel.y = 50.0;

        let even_direction = (direction & 1) == 0;
        let swim_left = (50.0 < pos_x && even_direction) || 430.0 <= pos_x;
        let (x_scale, x_vel) = if swim_left { (-1.0, -10.0) } else { (1.0, 10.0) };
        display::object_mut(world, own_display_object).scale.x = x_scale;
        logic::logic_mut(world, this).pos_vel.x = x_vel;
    }

    let stall_time = logic::logic(world, this).stall_time;
    let result = if stall_time <= 0.0 {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    let mut state = logic::logic_mut(world, this);
    state.stall_time = state.stall_time - call.delta_time;
    result
}

/// `-[JellyfishLogic drag:::]` @ 0x10013c87c
///
/// Springs toward the finger, turning to face the way it is pulled (`JellyfishDragMove`) and
/// back upright when the pull is straight up or down (`JellyfishDrag`). Never completes.
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let own_display_object = logic::display_object(world, this);
    let (pos, display_group) = {
        let object = display::object(world, own_display_object);
        (object.pos, object.display_group)
    };
    let touch_screen_pos = world.obj::<JellyfishLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    // The original subtracts the jellyfish's own z, so the spring never acts in depth.
    let mut delta = pos;
    delta.x = pos.x - touch_pos.x;
    delta.y = pos.y - touch_pos.y;
    delta.z = pos.z - pos.z;

    let distance = pg_engine::math::length3(delta);
    if distance == 0.0 {
        return ItemResult::Pending;
    }

    if delta.y != 0.0 {
        let rot_z = display::object(world, own_display_object).rot.z;
        let pull_angle = (-delta.y).atan2(-delta.x);
        let angle = pg_engine::math::rad_to_deg(pull_angle) + -90.0;
        let turn = angle - rot_z;

        if angle.abs() > 5.0 && turn.abs() > 5.0 {
            let x_scale = if turn > 0.0 { -1.0 } else { 1.0 };
            display::object_mut(world, own_display_object).scale.x = x_scale;

            if !world.obj::<JellyfishLogic>(this).drag_and_move {
                world.obj_mut::<JellyfishLogic>(this).drag_and_move = true;
                logic::set_behavior(world, this, Some("JellyfishDragMove"));
            }
        } else if angle.abs() < 5.0 && turn.abs() < 5.0 {
            if world.obj::<JellyfishLogic>(this).drag_and_move {
                world.obj_mut::<JellyfishLogic>(this).drag_and_move = false;
                logic::set_behavior(world, this, Some("JellyfishDrag"));
            }
        }
        display::object_mut(world, own_display_object).rot.z = angle;
    }

    let pos_vel = logic::logic(world, this).pos_vel;
    let spring = (distance * -100.0) / distance;
    let mut spring_force = delta;
    spring_force.x = -8.0 * pos_vel.x + spring * delta.x;
    spring_force.y = -8.0 * pos_vel.y + spring * delta.y;
    spring_force.z = -8.0 * pos_vel.z + spring * delta.z;
    world.obj_mut::<JellyfishLogic>(this).spring_force = spring_force;
    ItemResult::Pending
}

/// `-[JellyfishLogic onJellyfishAttackComplete:]` @ 0x10013cae0
pub fn on_jellyfish_attack_complete(world: &mut World, this: Entity, _jellyfish_attack_complete: Entity) {
    world.obj_mut::<JellyfishLogic>(this).pygmy_logic = None;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let own_display_object = logic::display_object(world, this);
    display::object_mut(world, own_display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("JellyfishIdle"));
}

/// `-[JellyfishLogic cancelJellyfishAttack:]` @ 0x10013cb54
pub fn cancel_jellyfish_attack(world: &mut World, this: Entity, _cancel_jellyfish_attack: Entity) {
    world.obj_mut::<JellyfishLogic>(this).pygmy_logic = None;
}

/// `-[JellyfishLogic onJellyfishSuffocateAttackComplete:]` @ 0x10013cb64
///
/// After suffocating a pygmy the jellyfish starts over from below the screen.
pub fn on_jellyfish_suffocate_attack_complete(world: &mut World, this: Entity, _jellyfish_suffocate_attack_complete: Entity) {
    {
        let mut jellyfish = world.obj_mut::<JellyfishLogic>(this);
        jellyfish.pygmy_logic = None;
        jellyfish.enable_screen_collision = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<JellyfishLogic>(this).circle_jerk = false;

    let own_display_object = logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, own_display_object);
        object.rot.z = 0.0;
        object.is_selectable = true;
    }
    place_below_screen(world, this);
    logic::set_behavior(world, this, Some("JellyfishEnterIsland"));
}

/// `-[JellyfishLogic onJellyfishAttackComplete:::]` @ 0x10013ccf0
pub fn seq_on_jellyfish_attack_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let own_display_object = logic::display_object(world, this);
    display::object_mut(world, own_display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("JellyfishIdle"));

    let pygmy = world.obj::<JellyfishLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::jellyfish_attack_complete(world, pygmy, this);
    }
    world.obj_mut::<JellyfishLogic>(this).pygmy_logic = None;
    ItemResult::Done
}

/// `-[JellyfishLogic die:::]` @ 0x10013cd84
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
    ItemResult::Done
}

/// `-[JellyfishLogic checkPygmyCollision]` @ 0x10013ce1c
///
/// Dropped onto a pygmy: within 30 of the point 20 below it the jellyfish suffocates the
/// pygmy, within 30 of the point 80 below it stings it.
pub fn check_pygmy_collision(world: &mut World, this: Entity) -> bool {
    let own_display_object = logic::display_object(world, this);
    let pos = display::object(world, own_display_object).pos;
    let suffocate_y_pos = pos.y + SUFFOCATE_Y_OFFSET;
    let sting_y_pos = pos.y + STING_Y_OFFSET;

    let listener = world.obj::<JellyfishLogic>(this).jellyfish_logic_listener;
    let pygmies = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };

    for pygmy in pygmies {
        if !crate::pygmy::can_be_hit_by_jellyfish(world, pygmy, this) {
            continue;
        }

        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;

        let suffocate_dx = pygmy_pos.x - pos.x;
        let suffocate_dy = pygmy_pos.y - suffocate_y_pos;
        let suffocate_distance = (suffocate_dx * suffocate_dx + suffocate_dy * suffocate_dy).sqrt();
        if suffocate_distance < ATTACK_DISTANCE {
            world.obj_mut::<JellyfishLogic>(this).pygmy_logic = Some(pygmy);
            crate::pygmy::suffocated_by_jellyfish(world, pygmy, this);
            hold_still_for_attack(world, this);
            logic::set_behavior(world, this, Some("JellyfishSuffocatePygmy"));
            return true;
        }

        let sting_dx = pygmy_pos.x - pos.x;
        let sting_dy = pygmy_pos.y - sting_y_pos;
        let sting_distance = (sting_dx * sting_dx + sting_dy * sting_dy).sqrt();
        if sting_distance < ATTACK_DISTANCE {
            world.obj_mut::<JellyfishLogic>(this).pygmy_logic = Some(pygmy);
            crate::pygmy::stung_by_jellyfish(world, pygmy, this);
            hold_still_for_attack(world, this);
            logic::set_behavior(world, this, Some("JellyfishStingPygmy"));
            return true;
        }
    }
    false
}

/// `-[JellyfishLogic eatPygmyLogic:]` @ 0x10013d148
pub fn eat_pygmy_logic(world: &mut World, this: Entity, eat_pygmy_logic: Entity) {
    world.obj_mut::<JellyfishLogic>(this).pygmy_logic = Some(eat_pygmy_logic);
    hold_still_for_attack(world, this);
    logic::set_behavior(world, this, Some("JellyfishEatPygmy"));
}

/// `-[JellyfishLogic drinkPygmyLogic:]` @ 0x10013d1d4
pub fn drink_pygmy_logic(world: &mut World, this: Entity, drink_pygmy_logic: Entity) {
    world.obj_mut::<JellyfishLogic>(this).pygmy_logic = Some(drink_pygmy_logic);
    hold_still_for_attack(world, this);
    logic::set_behavior(world, this, Some("JellyfishDrinkPygmy"));
}

/// `-[JellyfishLogic shockPygmyLogic:]` @ 0x10013d260
///
/// The circle shocks a pygmy: the effect and every jellyfish flash.
pub fn shock_pygmy_logic(world: &mut World, this: Entity, _shock_pygmy_logic: Entity) {
    let circle_effect = library::logic(world, logic_table::JELLYFISH_CIRCLE_JERK);
    logic::set_behavior(world, circle_effect, Some("JellyfishCircleJerkShock"));

    let jellyfish_array = listener_jellyfish_logic_array(world, this);
    for jellyfish in jellyfish_array {
        logic::set_behavior(world, jellyfish, Some("JellyfishCircleJerkSoloShock"));
    }
}

/// `-[JellyfishLogic touchBeganInside:allTouches:]` @ 0x10013d3d4
///
/// Picked up: a circle breaks up, and the jellyfish is dragged above the piranhas.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    if world.obj::<JellyfishLogic>(this).active_touch.is_some() {
        return;
    }
    world.obj_mut::<JellyfishLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<JellyfishLogic>(this).touch_screen_pos = touch_pos;

    if world.obj::<JellyfishLogic>(this).circle_jerk {
        let jellyfish_array = listener_jellyfish_logic_array(world, this);
        for jellyfish in jellyfish_array {
            stop_circle_jerk(world, jellyfish);
        }
    }

    {
        let mut jellyfish = world.obj_mut::<JellyfishLogic>(this);
        jellyfish.dragging = true;
        jellyfish.drag_and_move = false;
        jellyfish.apply_spring = true;
    }

    let own_display_object = logic::display_object(world, this);
    let marker = library::display_marker(world, "UnderwaterPiranhaDisplayMarker");
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(own_display_object));
    }

    let listener = world.obj::<JellyfishLogic>(this).jellyfish_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_jellyfish_drag(world, listener, this);
    }
    logic::set_behavior(world, this, Some("JellyfishDrag"));
}

/// `-[JellyfishLogic touchBeganOutside:allTouches:]` @ 0x10013d610
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[JellyfishLogic touchMovedEntered:allTouches:]` @ 0x10013d614
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[JellyfishLogic touchMovedExited:allTouches:]` @ 0x10013d618
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[JellyfishLogic touchMovedInside:allTouches:]` @ 0x10013d61c
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[JellyfishLogic touchMovedOutside:allTouches:]` @ 0x10013d62c
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    if !is_own_touch(world, this, touch) {
        return;
    }
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<JellyfishLogic>(this).touch_screen_pos = touch_pos;
}

/// `-[JellyfishLogic touchEndedInside:allTouches:]` @ 0x10013d684
///
/// Dropped: it attacks a pygmy it lands on, and may complete a circle.
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    if !is_own_touch(world, this, touch) {
        return;
    }

    if world.obj::<JellyfishLogic>(this).dragging {
        {
            let mut jellyfish = world.obj_mut::<JellyfishLogic>(this);
            jellyfish.dragging = false;
            jellyfish.apply_spring = false;
        }
        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }

        let own_display_object = logic::display_object(world, this);
        display::object_mut(world, own_display_object).rot.z = 0.0;
        let marker = library::display_marker(world, "CoralReefObjectDisplayMarker");
        if let Some(marker) = marker {
            display::insert_before(world, marker, Some(own_display_object));
        }
        notify_drag_complete(world, this);

        if !check_pygmy_collision(world, this) {
            logic::set_behavior(world, this, Some("JellyfishIdle"));
        }
        check_for_circle_jerk(world, this);
    }
    world.obj_mut::<JellyfishLogic>(this).active_touch = None;
}

/// `-[JellyfishLogic touchEndedOutside:allTouches:]` @ 0x10013d7cc
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}

/// The start of `enterIsland:` and `onJellyfishSuffocateAttackComplete:`: a random spot below
/// the screen, standing still, heading for a random height above it.
fn place_below_screen(world: &mut World, this: Entity) {
    let own_display_object = logic::display_object(world, this);

    let x_random = world.resource_mut::<Random>().arc4random();
    let x_pos = (x_random % 3801 + 500) as i32 as f32 / 10.0;
    display::object_mut(world, own_display_object).pos.x = x_pos;

    let y_random = world.resource_mut::<Random>().arc4random();
    let y_pos = ((y_random % 1501) as i32 - 2000) as f32 / 10.0;
    display::object_mut(world, own_display_object).pos.y = y_pos;

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<JellyfishLogic>(this).target_pos.x = x_pos;

    let target_random = world.resource_mut::<Random>().arc4random();
    let target_y_pos = (target_random % 1501 + 500) as i32 as f32 / 10.0;
    world.obj_mut::<JellyfishLogic>(this).target_pos.y = target_y_pos;
}

/// What every attack does first: stop, give gravity up and stop being selectable.
fn hold_still_for_attack(world: &mut World, this: Entity) {
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let own_display_object = logic::display_object(world, this);
    display::object_mut(world, own_display_object).is_selectable = false;
}

/// `mActiveTouch == nil || mActiveTouch == touch`.
fn is_own_touch(world: &World, this: Entity, touch: Entity) -> bool {
    let active_touch = world.obj::<JellyfishLogic>(this).active_touch;
    match active_touch {
        Some(active_touch) => active_touch == touch,
        None => true,
    }
}

/// `[mJellyfishLogicListener onJellyfishDragComplete:self]`.
fn notify_drag_complete(world: &mut World, this: Entity) {
    let listener = world.obj::<JellyfishLogic>(this).jellyfish_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_jellyfish_drag_complete(world, listener, this);
    }
}

/// `[mJellyfishLogicListener jellyfishLogicArray:self]` (empty from a nil listener).
fn listener_jellyfish_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let listener = world.obj::<JellyfishLogic>(this).jellyfish_logic_listener;
    match listener {
        Some(listener) => crate::controller::jellyfish_logic_array(world, listener, this),
        None => Vec::new(),
    }
}
