//! `SharkLogic` (superclass `SpriteLogic`).
//!
//! The shark (`docs/re/logic-catalog.md` §4.3). It swims in the sea, leaps at pygmies standing
//! near the water and eats them, bites an idle fishing bob, and can be dragged out of the water
//! and thrown (§2.2, island floor at y 80) to land head-first on a pygmy.

use crate::prelude::*;

use crate::logic::island_core::coconut_logic::{
    THROWABLE_GRAVITY_FORCE, integrate_throwable, is_throwable_out_of_range, throwable_drag_z_rot,
    throwable_pos_force, throwable_spring_force, throwable_tilted_gravity_force,
};
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::ocean_fishing::{fish_logic, fishing_bob_logic};
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::length3;
use pg_engine::touch::BcTouch;

/// `SharkLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct SharkLogic {
    /// `mSharkLogicListener`: `@"<SharkLogicListener>"`
    pub shark_logic_listener: Option<Entity>,

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

    /// `mSlideFriction`: `f`
    pub slide_friction: f32,

    /// `mSpringDelta`: `f`
    pub spring_delta: f32,

    /// `mEatenPygmyLogic`: `@"PygmyLogic"`
    pub eaten_pygmy_logic: Option<Entity>,

    /// `mCatchPygmyLogic`: `@"PygmyLogic"`
    pub catch_pygmy_logic: Option<Entity>,

    /// `mbPygmyHasPlague`: `B`
    pub pygmy_has_plague: bool,

    /// `mbWaitingToJump`: `B`
    pub waiting_to_jump: bool,

    /// `mbDropOnHead`: `B`
    pub drop_on_head: bool,

    /// `mbHitIsland`: `B`
    pub hit_island: bool,

    /// `mbEnableIslandFloorCollision`: `B`
    pub enable_island_floor_collision: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbInWater`: `B`
    pub in_water: bool,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbFalling`: `B`
    pub falling: bool,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,
}

pub const SHARK_LOGIC: Class = Class {
    name: "SharkLogic",
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

/// The island floor: a thrown shark whose origin drops below it over the island bounces, and a
/// dragged shark released above it may hit the island. The mouth must be at least this high to
/// land on a pygmy's head.
const FLOOR_Y: f32 = 80.0;

/// The floor bounce of `onGameFrame:`: x keeps 90 %, y reverses at 70 % (`0x3f666666`,
/// `0xbf333333`), at most 700 upwards.
const BOUNCE_X_VEL_FACTOR: f32 = 0.9;
const BOUNCE_Y_VEL_FACTOR: f32 = -0.7;
const BOUNCE_MAX_Y_VEL: f32 = 700.0;

/// The screen width: the shark out of the water is kept on it, and `swim:::` ends beyond it.
const SCREEN_WIDTH: f32 = 480.0;

/// `checkPygmyCollision:`: the mouth lies (34, 18) from the origin (`DAT_10029d448`), the
/// pygmy's head 31 above the pygmy's origin (`DAT_10029d450`).
const MOUTH_X_OFFSET: f32 = 34.0;
const MOUTH_Y_OFFSET: f32 = 18.0;
const PYGMY_HEAD_Y_OFFSET: f32 = 31.0;

/// `checkPygmyCollision:`: a falling, nearly upright shark dropping slowly sideways lands on a
/// head within 20 on both axes.
const DROP_ON_HEAD_REACH: f32 = 20.0;
const DROP_ON_HEAD_MAX_TILT: f32 = 30.0;
const DROP_ON_HEAD_MAX_Y_VEL: f32 = -20.0;
const DROP_ON_HEAD_MAX_X_SPEED: f32 = 30.0;

/// `checkPygmyCollision:`: a dragged shark hits pygmies within 30 (squared 900), knocking them
/// away at three times the x distance.
const DRAG_HIT_DISTANCE_SQUARED: f32 = 900.0;
const DRAG_HIT_X_VEL_FACTOR: f32 = -3.0;

/// The spring of `drag:::`.
const DRAG_STIFFNESS: f32 = 100.0;
const DRAG_DAMPING: f32 = 8.0;

/// `setCatchPosition:::` and `checkCatchPygmy:::`: the pygmy must stand between these heights,
/// and the shark waits 50 behind it (against its x scale), within 10.
const CATCH_MIN_Y: f32 = 60.0;
const CATCH_MAX_Y: f32 = 150.0;
const CATCH_X_OFFSET: f32 = -50.0;
const CATCH_REACH: f32 = 10.0;

/// `checkJumpForPygmy:`: the pygmy must be no higher than 150 and less than 100 ahead.
const JUMP_MAX_Y: f32 = 150.0;
const JUMP_REACH: f32 = 100.0;

/// `launchIntoAir:::` ends above this height.
const LAUNCH_TOP_Y: f32 = 960.0;

/// `fall:::` splashes once the origin drops below this height.
const WATER_LINE_Y: f32 = 10.0;

/// `swim:::`: an idle bob within this x distance hooks the shark.
const HOOK_REACH: f32 = 16.0;

/// `launchOffCharlie`: launched straight up.
const CHARLIE_LAUNCH_Y_VEL: f32 = 1000.0;

/// `touchEndedInside:allTouches:`: a tap that did not drag bounces the shark up.
const TAP_Y_VEL: f32 = 200.0;

/// `storyModeSharkSnap`: x is `(arc4random() % 2801 + 1000) / 10`, 100 to 380.
const STORY_SNAP_X_RANGE: u32 = 2801;
const STORY_SNAP_X_BASE: u32 = 1000;

/// Sequence-item handlers implemented by `SharkLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "resetDropOnHead" => seq_reset_drop_on_head(world, this, call),
        "enableJump" => seq_enable_jump(world, this, call),
        "disableJump" => seq_disable_jump(world, this, call),
        "setCatchPosition" => seq_set_catch_position(world, this, call),
        "checkForPlaguePygmy" => seq_check_for_plague_pygmy(world, this, call),
        "checkCatchPygmy" => seq_check_catch_pygmy(world, this, call),
        "drag" => seq_drag(world, this, call),
        "launchIntoAir" => seq_launch_into_air(world, this, call),
        "fall" => seq_fall(world, this, call),
        "swim" => seq_swim(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[SharkLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&SHARK_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[SharkLogic initWithDisplayObject:]` @ 0x1000281b0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(SharkLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<SharkLogic>(this).in_water = true;

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[SharkLogic setSharkLogicListener:]` @ 0x100028230
pub fn set_shark_logic_listener(world: &mut World, this: Entity, shark_logic_listener: Entity) {
    world.obj_mut::<SharkLogic>(this).shark_logic_listener = Some(shark_logic_listener);
}

/// Re-inserts the shark's display object after `DragDisplayMarker`, in front of the island.
fn insert_after_drag_display_marker(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `[mSharkLogicListener islandMinXPos]` and `islandMaxXPos` (both 0 without a listener).
fn island_x_range(world: &mut World, this: Entity) -> (f32, f32) {
    let listener = world.obj::<SharkLogic>(this).shark_logic_listener;
    let Some(listener) = listener else {
        return (0.0, 0.0);
    };

    let island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
    let island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
    (island_min_x_pos, island_max_x_pos)
}

/// `[mSharkLogicListener onSharkDragComplete:self]`.
fn send_drag_complete(world: &mut World, this: Entity) {
    let listener = world.obj::<SharkLogic>(this).shark_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_shark_drag_complete(world, listener, this);
    }
}

/// `[mEatenPygmyLogic uneatenByShark:self]`, then the shark forgets the pygmy (nothing without
/// one).
fn release_eaten_pygmy(world: &mut World, this: Entity) {
    let eaten_pygmy = world.obj::<SharkLogic>(this).eaten_pygmy_logic;
    if let Some(eaten_pygmy) = eaten_pygmy {
        crate::pygmy::uneaten_by_shark(world, eaten_pygmy, this);
        world.obj_mut::<SharkLogic>(this).eaten_pygmy_logic = None;
    }
}

/// `-[SharkLogic enterIsland:]` @ 0x10002828c
///
/// The shark ignores the startup flag: it always starts in the water.
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    let active = world.obj::<SharkLogic>(this).active;
    if active {
        return;
    }

    world.obj_mut::<SharkLogic>(this).active = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    insert_after_drag_display_marker(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("SharkIdle"));
}

/// `-[SharkLogic exitIsland:]` @ 0x100028394
///
/// `now` ends the shark at once (`SharkDie`); otherwise a shark out of the water falls through
/// the island once (`SharkFallIgnoreIsland`) and one in the water stays.
pub fn exit_island(world: &mut World, this: Entity, now: bool) {
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;

    let dragging = world.obj::<SharkLogic>(this).dragging;
    if dragging {
        {
            let mut shark = world.obj_mut::<SharkLogic>(this);
            shark.falling = true;
            shark.dragging = false;
            shark.apply_spring = false;
        }
        send_drag_complete(world, this);
    }

    release_eaten_pygmy(world, this);
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<SharkLogic>(this).enable_island_floor_collision = false;

    if now {
        world.obj_mut::<SharkLogic>(this).exit_island = true;
        logic::set_behavior(world, this, Some("SharkDie"));
        return;
    }

    let already_exiting = world.obj::<SharkLogic>(this).exit_island;
    if already_exiting {
        return;
    }
    world.obj_mut::<SharkLogic>(this).exit_island = true;

    let in_water = world.obj::<SharkLogic>(this).in_water;
    if in_water {
        return;
    }

    insert_after_drag_display_marker(world, this);
    logic::set_behavior(world, this, Some("SharkFallIgnoreIsland"));
}

/// `-[SharkLogic exitIslandNow]` @ 0x10002852c
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[SharkLogic checkPygmyCollision:]` @ 0x1000285a4
///
/// Over the island, a shark falling onto a pygmy's head lands there and starts eating it (the
/// first pygmy that accepts ends the check); a dragged shark knocks over every pygmy it touches.
pub fn check_pygmy_collision(world: &mut World, this: Entity, _dt: f32) {
    let display_object = logic::display_object(world, this);
    let display = display::object(world, display_object);
    let pos = display.pos;
    let origin_pos = display.origin_pos;
    let z_rot = display.rot.z;
    let (island_min_x_pos, island_max_x_pos) = island_x_range(world, this);

    let eaten_pygmy = world.obj::<SharkLogic>(this).eaten_pygmy_logic;
    if eaten_pygmy.is_some() {
        return;
    }

    let mouth_x = pos.x + origin_pos.x + MOUTH_X_OFFSET;
    let mouth_y = pos.y + origin_pos.y + MOUTH_Y_OFFSET;
    let drop_on_head = world.obj::<SharkLogic>(this).drop_on_head;
    if drop_on_head {
        return;
    }

    // The machine code branches on `x < min` and `x > max`, so NaN passes.
    let off_island = mouth_x < island_min_x_pos || mouth_x > island_max_x_pos;
    if off_island || mouth_y < FLOOR_Y {
        return;
    }

    let listener = world.obj::<SharkLogic>(this).shark_logic_listener;
    let pygmy_logic_array = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmy_logic_array {
        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let delta_x = mouth_x - (pygmy_pos.x + 0.0);
        let delta_y = mouth_y - (pygmy_pos.y + PYGMY_HEAD_Y_OFFSET);

        let dragging = world.obj::<SharkLogic>(this).dragging;
        if dragging {
            let distance_squared = delta_x * delta_x + delta_y * delta_y;
            if distance_squared < DRAG_HIT_DISTANCE_SQUARED {
                let x_vel = DRAG_HIT_X_VEL_FACTOR * delta_x;
                crate::pygmy::hit_by_shark(world, pygmy, this, x_vel);
            }
            continue;
        }

        let over_head = delta_x.abs() < DROP_ON_HEAD_REACH && delta_y.abs() < DROP_ON_HEAD_REACH;
        let falling = world.obj::<SharkLogic>(this).falling;
        let upright = -DROP_ON_HEAD_MAX_TILT < z_rot && z_rot < DROP_ON_HEAD_MAX_TILT;
        if !over_head || !falling || !upright {
            continue;
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let dropping = pos_vel.y < DROP_ON_HEAD_MAX_Y_VEL && pos_vel.x.abs() < DROP_ON_HEAD_MAX_X_SPEED;
        if !dropping {
            continue;
        }

        let eaten = crate::pygmy::eaten_by_shark(world, pygmy, this);
        if !eaten {
            continue;
        }

        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }
        display::object_mut(world, display_object).rot.z = 0.0;

        // The pygmy's x is read after it has taken the shark onto its head.
        let pygmy_x_pos = display::object(world, pygmy_display_object).pos.x;
        display::object_mut(world, display_object).pos.x = pygmy_x_pos;

        world.obj_mut::<SharkLogic>(this).drop_on_head = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        world.obj_mut::<SharkLogic>(this).eaten_pygmy_logic = Some(pygmy);
        logic::set_behavior(world, this, Some("SharkOnPygmyHead"));

        let pygmy_display_object = logic::display_object(world, pygmy);
        display::insert_after(world, pygmy_display_object, Some(display_object));
        return;
    }
}

/// `-[SharkLogic isAccelEnabled]` @ 0x1000289e8
pub fn is_accel_enabled(_world: &mut World, _this: Entity) -> bool {
    false
}

/// `-[SharkLogic onGameFrame:]` @ 0x1000289f0
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let display_object = logic::display_object(world, this);
    let previous_y = display::object(world, display_object).pos.y;

    let accel_enabled = is_accel_enabled(world, this);
    let gravity_force = if accel_enabled { throwable_tilted_gravity_force(world) } else { THROWABLE_GRAVITY_FORCE };
    world.obj_mut::<SpriteLogic>(this).gravity_force = gravity_force;

    let ignore_gravity = world.obj::<SpriteLogic>(this).ignore_gravity;
    let shark = world.obj::<SharkLogic>(this);
    let spring_force = shark.spring_force;
    let apply_spring = shark.apply_spring;

    let pos_force = throwable_pos_force(gravity_force, ignore_gravity, spring_force, apply_spring);
    logic::logic_mut(world, this).pos_force = pos_force;
    integrate_throwable(world, this, pos_force, dt);

    // The bounds are measured in the water too; only a shark out of it stays on the screen.
    let min_pos = display::min_pos(world, display_object);
    let max_pos = display::max_pos(world, display_object);
    let in_water = world.obj::<SharkLogic>(this).in_water;
    if !in_water {
        if min_pos.x < 0.0 {
            let mut display = display::object_mut(world, display_object);
            display.pos.x = (display.pos.x - min_pos.x) + 0.0;
        }
        if max_pos.x > SCREEN_WIDTH {
            let mut display = display::object_mut(world, display_object);
            display.pos.x = SCREEN_WIDTH - (max_pos.x - display.pos.x);
        }
    }

    // On a pygmy's head the shark follows the pygmy and faces its way.
    let eaten_pygmy = world.obj::<SharkLogic>(this).eaten_pygmy_logic;
    if let Some(eaten_pygmy) = eaten_pygmy {
        let pygmy_display_object = logic::display_object(world, eaten_pygmy);
        let pygmy_display = display::object(world, pygmy_display_object);
        let pygmy_x_pos = pygmy_display.pos.x;
        let pygmy_x_scale = pygmy_display.scale.x;

        let mut display = display::object_mut(world, display_object);
        display.pos.x = pygmy_x_pos;
        display.scale.x = pygmy_x_scale;
    }

    let (island_min_x_pos, island_max_x_pos) = island_x_range(world, this);
    let display = display::object(world, display_object);
    let pos = display.pos;
    let origin_y = display.origin_pos.y;
    let over_island = island_min_x_pos <= pos.x && pos.x <= island_max_x_pos;
    let crossed_floor = FLOOR_Y <= previous_y && pos.y + origin_y < FLOOR_Y;
    let enable_island_floor_collision = world.obj::<SharkLogic>(this).enable_island_floor_collision;
    if over_island && crossed_floor && enable_island_floor_collision {
        bounce_off_island(world, this);
    }

    let y_pos = display::object(world, display_object).pos.y;
    if is_throwable_out_of_range(y_pos) {
        logic::set_behavior(world, this, Some("SharkDie"));
    }

    check_pygmy_collision(world, this, dt);
}

/// The island floor contact of `onGameFrame:`: the shark bounces off once (the floor collision
/// switches itself off), swallowing if it carried a pygmy down on its head.
fn bounce_off_island(world: &mut World, this: Entity) {
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = state.pos_vel.x * BOUNCE_X_VEL_FACTOR;

        // `fcsel gt`: a NaN velocity stays NaN.
        let bounce_y_vel = state.pos_vel.y * BOUNCE_Y_VEL_FACTOR;
        state.pos_vel.y = if bounce_y_vel > BOUNCE_MAX_Y_VEL { BOUNCE_MAX_Y_VEL } else { bounce_y_vel };
    }

    let drop_on_head = {
        let mut shark = world.obj_mut::<SharkLogic>(this);
        shark.hit_island = true;
        shark.enable_island_floor_collision = false;
        shark.drop_on_head
    };

    let behavior = if drop_on_head { "SharkSwallowBounce" } else { "SharkBounce" };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[SharkLogic launchOffCharlie]` @ 0x100028f30
pub fn launch_off_charlie(world: &mut World, this: Entity) {
    logic::logic_mut(world, this).pos_vel.y = CHARLIE_LAUNCH_Y_VEL;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<SharkLogic>(this).eaten_pygmy_logic = None;

    insert_after_drag_display_marker(world, this);
    logic::set_behavior(world, this, Some("SharkLaunchOffCharlie"));
}

/// `-[SharkLogic swallowPygmy]` @ 0x100028fec
pub fn swallow_pygmy(world: &mut World, this: Entity) {
    world.obj_mut::<SharkLogic>(this).eaten_pygmy_logic = None;

    insert_after_drag_display_marker(world, this);
    logic::set_behavior(world, this, Some("SharkSwallowPygmy"));
}

/// `-[SharkLogic letGoOfPygmy]` @ 0x10002907c
pub fn let_go_of_pygmy(world: &mut World, this: Entity) {
    world.obj_mut::<SharkLogic>(this).eaten_pygmy_logic = None;

    insert_after_drag_display_marker(world, this);
    logic::set_behavior(world, this, Some("SharkLetGoOfPygmy"));
}

/// `-[SharkLogic storyModeSharkSnap]` @ 0x10002910c
pub fn story_mode_shark_snap(world: &mut World, this: Entity) {
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let display_object = logic::display_object(world, this);
    let random = world.resource_mut::<Random>().arc4random_mod(STORY_SNAP_X_RANGE);
    let x_pos = (random + STORY_SNAP_X_BASE) as f32 / 10.0;
    display::object_mut(world, display_object).pos.x = x_pos;

    insert_after_drag_display_marker(world, this);
    logic::set_behavior(world, this, Some("SharkStoryModeSharkSnap"));
}

/// `-[SharkLogic resetDropOnHead:::]` @ 0x100029208
pub fn seq_reset_drop_on_head(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SharkLogic>(this).drop_on_head = false;
    ItemResult::Done
}

/// `-[SharkLogic enableJump:::]` @ 0x10002921c
pub fn seq_enable_jump(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SharkLogic>(this).waiting_to_jump = true;
    ItemResult::Done
}

/// `-[SharkLogic disableJump:::]` @ 0x100029234
pub fn seq_disable_jump(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SharkLogic>(this).waiting_to_jump = false;
    ItemResult::Done
}

/// `-[SharkLogic setCatchPosition:::]` @ 0x100029248
///
/// The shark stops 50 behind the pygmy it leapt at; without one in reach it dies.
pub fn seq_set_catch_position(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let catch_pygmy = world.obj::<SharkLogic>(this).catch_pygmy_logic;
    if let Some(catch_pygmy) = catch_pygmy {
        let pygmy_display_object = logic::display_object(world, catch_pygmy);
        let pygmy_y_pos = display::object(world, pygmy_display_object).pos.y;
        let out_of_reach = pygmy_y_pos < CATCH_MIN_Y || pygmy_y_pos > CATCH_MAX_Y;
        if out_of_reach {
            world.obj_mut::<SharkLogic>(this).catch_pygmy_logic = None;
        } else {
            {
                let mut state = logic::logic_mut(world, this);
                state.pos_vel.x = 0.0;
                state.pos_vel.y = 0.0;
            }

            let display_object = logic::display_object(world, this);
            let pygmy_x_pos = display::object(world, pygmy_display_object).pos.x;
            let x_scale = display::object(world, display_object).scale.x;
            display::object_mut(world, display_object).pos.x = pygmy_x_pos + x_scale * CATCH_X_OFFSET;
            return ItemResult::Done;
        }
    }

    logic::set_behavior(world, this, Some("SharkDie"));
    ItemResult::Replaced
}

/// `-[SharkLogic checkForPlaguePygmy:::]` @ 0x100029384
pub fn seq_check_for_plague_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let pygmy_has_plague = world.obj::<SharkLogic>(this).pygmy_has_plague;
    if pygmy_has_plague {
        logic::set_behavior(world, this, Some("SharkEatPlaguePygmy"));
    }
    ItemResult::Done
}

/// `-[SharkLogic checkCatchPygmy:::]` @ 0x1000293c0
///
/// The jaws close: a pygmy still in place is caught (a baby time-temple pygmy has its own
/// catch); otherwise the listener counts a snap.
pub fn seq_check_catch_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let catch_pygmy = world.obj::<SharkLogic>(this).catch_pygmy_logic;
    let Some(catch_pygmy) = catch_pygmy else {
        return ItemResult::Done;
    };

    let pygmy_display_object = logic::display_object(world, catch_pygmy);
    let pygmy_pos = display::object(world, pygmy_display_object).pos;
    let display_object = logic::display_object(world, this);
    let display = display::object(world, display_object);
    let x_pos = display.pos.x;
    let x_scale = display.scale.x;

    let catch_x_pos = pygmy_pos.x + x_scale * CATCH_X_OFFSET;
    let distance = (x_pos - catch_x_pos).abs();
    let catchable = CATCH_MIN_Y < pygmy_pos.y && pygmy_pos.y < CATCH_MAX_Y && distance < CATCH_REACH;
    if !catchable {
        let listener = world.obj::<SharkLogic>(this).shark_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_shark_snap(world, listener, this);
        }
        return ItemResult::Done;
    }

    crate::pygmy::caught_by_shark(world, catch_pygmy, this);
    let disease_type = sprite_logic::apocalypse_disease_type(world, catch_pygmy);
    world.obj_mut::<SharkLogic>(this).pygmy_has_plague = disease_type == 1;

    let pygmy_type = crate::pygmy::time_temple_pygmy_type(world, catch_pygmy);
    let behavior = if pygmy_type == 2 { "SharkCatchPygmyBaby" } else { "SharkCatchPygmy" };
    logic::set_behavior(world, this, Some(behavior));
    ItemResult::Done
}

/// `-[SharkLogic checkJumpForPygmy:]` @ 0x100029554
///
/// A waiting shark leaps at a pygmy near the water less than 100 ahead of its mouth.
pub fn check_jump_for_pygmy(world: &mut World, this: Entity, pygmy: Entity) {
    let waiting_to_jump = world.obj::<SharkLogic>(this).waiting_to_jump;
    if !waiting_to_jump {
        return;
    }

    let pygmy_display_object = logic::display_object(world, pygmy);
    let pygmy_pos = display::object(world, pygmy_display_object).pos;
    let display_object = logic::display_object(world, this);
    let display = display::object(world, display_object);
    let x_pos = display.pos.x;
    let x_scale = display.scale.x;

    // `b.gt`: a NaN height passes.
    if pygmy_pos.y > JUMP_MAX_Y {
        return;
    }

    let delta_x = x_pos - pygmy_pos.x;
    let ahead_to_the_left = x_scale < 0.0 && 0.0 < delta_x && delta_x < JUMP_REACH;
    let ahead_to_the_right = 0.0 < x_scale && delta_x < 0.0 && -JUMP_REACH < delta_x;
    if !ahead_to_the_left && !ahead_to_the_right {
        return;
    }

    {
        let mut shark = world.obj_mut::<SharkLogic>(this);
        shark.waiting_to_jump = false;
        shark.catch_pygmy_logic = Some(pygmy);
    }
    logic::set_behavior(world, this, Some("SharkJumpForPygmy"));
}

/// `-[SharkLogic cantCatchPygmy:]` @ 0x10002968c
pub fn cant_catch_pygmy(world: &mut World, this: Entity, pygmy: Entity) {
    let catch_pygmy = world.obj::<SharkLogic>(this).catch_pygmy_logic;
    if catch_pygmy == Some(pygmy) {
        world.obj_mut::<SharkLogic>(this).catch_pygmy_logic = None;
    }
}

/// `-[SharkLogic drag:::]` @ 0x1000296a8
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SharkLogic>(this).apply_spring = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<SharkLogic>(this).enable_island_floor_collision = false;
        insert_after_drag_display_marker(world, this);
    }

    let display_object = logic::display_object(world, this);
    let touch_screen_pos = world.obj::<SharkLogic>(this).touch_screen_pos;
    let offset = fish_logic::drag_offset(world, display_object, touch_screen_pos);
    let length = length3(offset);
    if length != 0.0 {
        if offset.y != 0.0 {
            let z_rot = throwable_drag_z_rot(offset);
            display::object_mut(world, display_object).rot.z = z_rot;
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let spring_force = throwable_spring_force(offset, length, pos_vel, DRAG_STIFFNESS, DRAG_DAMPING);
        world.obj_mut::<SharkLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[SharkLogic launchIntoAir:::]` @ 0x1000298b8
pub fn seq_launch_into_air(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SharkLogic>(this).falling = true;
    }

    let display_object = logic::display_object(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    if y_pos > LAUNCH_TOP_Y {
        return ItemResult::Done;
    }
    ItemResult::Pending
}

/// `-[SharkLogic fall:::]` @ 0x100029914
///
/// The shark falls until its origin drops below the water line, then splashes back in.
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<SharkLogic>(this).falling = true;
    }

    let display_object = logic::display_object(world, this);
    let display = display::object(world, display_object);
    let y_with_origin = display.pos.y + display.origin_pos.y;
    let below_water_line = y_with_origin < WATER_LINE_Y;
    if !below_water_line {
        return ItemResult::Pending;
    }

    {
        let mut shark = world.obj_mut::<SharkLogic>(this);
        shark.falling = false;
        shark.in_water = true;
        shark.hit_island = false;
    }
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    {
        let mut display = display::object_mut(world, display_object);
        display.pos.y = WATER_LINE_Y - display.origin_pos.y;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<SharkLogic>(this).enable_island_floor_collision = false;

    display::object_mut(world, display_object).rot.z = 0.0;
    logic::set_behavior(world, this, Some("SharkSplash"));
    ItemResult::Replaced
}

/// `-[SharkLogic swim:::]` @ 0x100029a50
///
/// The shark swims (its sequence sets the velocity) until an idle fishing bob within reach
/// hooks it or it has left the screen on the side it swims to.
pub fn seq_swim(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    let min_pos = display::min_pos(world, display_object);
    let max_pos = display::max_pos(world, display_object);

    let listener = world.obj::<SharkLogic>(this).shark_logic_listener;
    let fishing_bob = match listener {
        Some(listener) => crate::controller::fishing_bob_logic(world, listener, this),
        None => None,
    };

    let mut result = ItemResult::Pending;
    if let Some(fishing_bob) = fishing_bob
        && fishing_bob_logic::is_unhooked(world, fishing_bob)
    {
        let bob_display_object = logic::display_object(world, fishing_bob);
        let x_pos = display::object(world, display_object).pos.x;
        let bob_x_pos = display::object(world, bob_display_object).pos.x;
        let within_reach = (x_pos - bob_x_pos).abs() <= HOOK_REACH;
        if within_reach && fishing_bob_logic::hook_shark(world, fishing_bob, this) {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
            result = ItemResult::Done;
        }
    }

    let x_vel = logic::logic(world, this).pos_vel.x;
    let gone_left = x_vel < 0.0 && max_pos.x < 0.0;
    let gone_right = 0.0 < x_vel && SCREEN_WIDTH < min_pos.x;
    if gone_left || gone_right {
        logic::logic_mut(world, this).pos_vel.x = 0.0;
        result = ItemResult::Done;
    }
    result
}

/// `-[SharkLogic die:::]` @ 0x100029bd0
///
/// The shark leaves the screen and, while sharks are enabled outside story mode, starts over in
/// the water.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);

    let dragging = world.obj::<SharkLogic>(this).dragging;
    if dragging {
        world.obj_mut::<SharkLogic>(this).dragging = false;
        send_drag_complete(world, this);
    }

    release_eaten_pygmy(world, this);
    {
        let mut shark = world.obj_mut::<SharkLogic>(this);
        shark.active = false;
        shark.exit_island = false;
        shark.in_water = true;
        shark.falling = false;
        shark.apply_spring = false;
        shark.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<SharkLogic>(this).eaten_pygmy_logic = None;

    let listener = world.obj::<SharkLogic>(this).shark_logic_listener;
    let Some(listener) = listener else {
        return ItemResult::Done;
    };
    let shark_enabled = crate::controller::is_shark_enabled(world, listener);
    if shark_enabled {
        let story_active = crate::controller::is_story_active(world, listener);
        if !story_active {
            enter_island(world, this, false);
        }
    }
    ItemResult::Done
}

/// `-[SharkLogic touchBeganInside:allTouches:]` @ 0x100029d1c
///
/// Grabbing the shark in the water drops a pygmy it is eating and starts a drag.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let shark = world.obj::<SharkLogic>(this);
    let can_be_grabbed = !shark.falling && shark.in_water && shark.active_touch.is_none();
    let eaten_pygmy = shark.eaten_pygmy_logic;
    if !can_be_grabbed {
        return;
    }

    if eaten_pygmy.is_some() {
        insert_after_drag_display_marker(world, this);
        release_eaten_pygmy(world, this);
    }

    world.obj_mut::<SharkLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut shark = world.obj_mut::<SharkLogic>(this);
        shark.touch_began_inside = true;
        shark.touch_screen_pos = touch_pos;
        shark.in_water = false;
        shark.dragging = true;
        // Original quirk: cleared again straight away.
        shark.touch_began_inside = false;
        shark.enable_island_floor_collision = false;
    }

    let listener = world.obj::<SharkLogic>(this).shark_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_shark_drag(world, listener, this);
    }
    logic::set_behavior(world, this, Some("SharkDrag"));
}

/// `-[SharkLogic touchBeganOutside:allTouches:]` @ 0x100029eb4
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<SharkLogic>(this).active_touch;
    if active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let mut shark = world.obj_mut::<SharkLogic>(this);
    shark.touch_screen_pos = touch_pos;
    shark.touch_began_inside = false;
}

/// `-[SharkLogic touchMovedEntered:allTouches:]` @ 0x100029f10
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<SharkLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<SharkLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[SharkLogic touchMovedExited:allTouches:]` @ 0x100029f68
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<SharkLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<SharkLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[SharkLogic touchMovedInside:allTouches:]` @ 0x100029fc0
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[SharkLogic touchMovedOutside:allTouches:]` @ 0x100029fd0
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<SharkLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<SharkLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[SharkLogic touchEndedInside:allTouches:]` @ 0x10002a028
///
/// Releasing a dragged shark throws it (over the island it may hit the floor); a tap that never
/// became a drag bounces it up.
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<SharkLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let shark = world.obj::<SharkLogic>(this);
    let dragging = shark.dragging;
    let touch_began_inside = shark.touch_began_inside;
    if dragging {
        {
            let mut shark = world.obj_mut::<SharkLogic>(this);
            shark.falling = true;
            shark.dragging = false;
            shark.apply_spring = false;
        }

        let display_object = logic::display_object(world, this);
        let y_pos = display::object(world, display_object).pos.y;
        if y_pos > FLOOR_Y {
            world.obj_mut::<SharkLogic>(this).enable_island_floor_collision = true;
        }

        insert_after_drag_display_marker(world, this);
        send_drag_complete(world, this);
        logic::set_behavior(world, this, Some("SharkFall"));
    } else if touch_began_inside {
        logic::logic_mut(world, this).pos_vel.y = TAP_Y_VEL;
        world.obj_mut::<SharkLogic>(this).falling = true;
        logic::set_behavior(world, this, Some("SharkFall"));
    }

    let mut shark = world.obj_mut::<SharkLogic>(this);
    shark.touch_began_inside = false;
    shark.active_touch = None;
}

/// `-[SharkLogic touchEndedOutside:allTouches:]` @ 0x10002a1a0
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
