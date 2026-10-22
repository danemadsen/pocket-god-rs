//! `TilePlugLogic` (superclass `SpriteLogic`).
//!
//! The stone plug of the drain in the underwater ruins. The drain lifts it out onto the lifting
//! finger; dragged, it hangs on a spring and sinks under water gravity, and dropped onto the
//! drain it plugs it again. Its listener is the view controller.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::underwater_paindrain::tile_drain_logic;
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::length3;
use pg_engine::touch::BcTouch;

/// `TilePlugLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct TilePlugLogic {
    /// `mTilePlugLogicListener`: `@"<TilePlugLogicListener>"`
    pub tile_plug_logic_listener: Option<Entity>,

    /// `mbUnplugged`: `B`
    pub unplugged: bool,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,

    /// `mbPainDrainActive`: `B`
    pub pain_drain_active: bool,

    /// `mPainDrainPygmyLogic`: `@"PygmyLogic"`
    pub pain_drain_pygmy_logic: Option<Entity>,

    /// `mPlatformYPosVel`: `f`
    pub platform_y_pos_vel: f32,

    /// `mPainDrainPlatformLogicArray`: `@"NSMutableArray"`
    pub pain_drain_platform_logic_array: Vec<Entity>,
}

pub const TILE_PLUG_LOGIC: Class = Class {
    name: "TilePlugLogic",
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

/// Water gravity, and the stronger pull while the plug hangs on the finger.
const GRAVITY_Y_FORCE: f32 = -30.0;
const DRAG_GRAVITY_Y_FORCE: f32 = -3000.0;

/// The sea floor the plug rests on.
const FLOOR_Y_POS: f32 = 50.0;

/// The right edge the plug stays inside.
const SCREEN_MAX_X_POS: f32 = 480.0;

/// The drag spring of `drag:::`.
const DRAG_STIFFNESS: f32 = 50.0;
const DRAG_DAMPING: f32 = 6.0;

/// `unplug:touch:` puts the plug this far above the drain.
const LIFTED_Y_OFFSET: f32 = 50.0;

/// `fall:::` plugs a drain within these distances.
const PLUG_X_DISTANCE: f32 = 15.0;
const PLUG_Y_DISTANCE: f32 = 50.0;

/// Sequence-item handlers implemented by `TilePlugLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "drag" => seq_drag(world, this, call),
        "fall" => seq_fall(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[TilePlugLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&TILE_PLUG_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[TilePlugLogic initWithDisplayObject:]` @ 0x10011fea8
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(TilePlugLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
    logic::set_behavior(world, this, Some("TilePlugInit"));
}

/// `-[TilePlugLogic setTilePlugLogicListener:]` @ 0x10011ff44
pub fn set_tile_plug_logic_listener(world: &mut World, this: Entity, tile_plug_logic_listener: Entity) {
    world.obj_mut::<TilePlugLogic>(this).tile_plug_logic_listener = Some(tile_plug_logic_listener);
}

/// `-[TilePlugLogic dealloc]` @ 0x100120970
///
/// For the plug's only owner, the view controller, which releases it in its own `dealloc`.
pub fn dealloc(world: &mut World, this: Entity) {
    // `[mTilePlugLogicListener release]` needs no port; `[super dealloc]`.
    logic::dealloc(world, this);
}

/// `-[TilePlugLogic onGameFrame:]` @ 0x10011ffa0
///
/// Water gravity `(0, −30)` (`−3000` while dragged) plus the drag spring, integrated. Let go,
/// the plug straightens up; it stays on screen and comes to rest on the sea floor at y = 50 (no
/// `[super onGameFrame:]`).
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;

    let (dragging, apply_spring, spring_force) = {
        let plug = world.obj::<TilePlugLogic>(this);
        (plug.dragging, plug.apply_spring, plug.spring_force)
    };

    let gravity_y_force = if dragging { DRAG_GRAVITY_Y_FORCE } else { GRAVITY_Y_FORCE };
    let gravity_force = Vec4::new(0.0, gravity_y_force, 0.0, 1.0);
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

    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = object.pos.x + pos_vel.x * dt;
        object.pos.y = object.pos.y + pos_vel.y * dt;
        object.pos.z = object.pos.z + pos_vel.z * dt;
    }

    let min_pos = display::min_pos(world, display_object);
    let max_pos = display::max_pos(world, display_object);
    if !dragging {
        let mut object = display::object_mut(world, display_object);
        object.rot.z = object.rot.z + (0.0 - object.rot.z) * 0.1;
    }
    if min_pos.x < 0.0 {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = (object.pos.x - min_pos.x) + 0.0;
    }
    if max_pos.x > SCREEN_MAX_X_POS {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = SCREEN_MAX_X_POS - (max_pos.x - object.pos.x);
    }

    let y_pos = display::object(world, display_object).pos.y;
    if y_pos < FLOOR_Y_POS {
        logic::logic_mut(world, this).pos_vel.y = 0.0;
        display::object_mut(world, display_object).pos.y = FLOOR_Y_POS;

        let island_floor_collision = world.obj::<TilePlugLogic>(this).island_floor_collision;
        if !island_floor_collision && !dragging {
            world.obj_mut::<TilePlugLogic>(this).island_floor_collision = true;
            logic::set_behavior(world, this, Some("TilePlugHitGround"));
        }
    } else {
        world.obj_mut::<TilePlugLogic>(this).island_floor_collision = false;
    }
}

/// `-[TilePlugLogic plug:]` @ 0x1001202e0
///
/// Pain Drain started: the drain swallows the plug.
pub fn plug(world: &mut World, this: Entity, _tile_drain: Entity) {
    world.obj_mut::<TilePlugLogic>(this).unplugged = false;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("TilePlugInDrain"));
}

/// `-[TilePlugLogic unplug:touch:]` @ 0x100120344
///
/// The drain lifted the plug: it appears 50 above the drain, hanging on the lifting finger's
/// spring, just before `UnderwaterPygmyDisplayMarker`.
pub fn unplug(world: &mut World, this: Entity, tile_drain: Entity, touch: Entity) {
    let display_object = logic::display_object(world, this);
    let drain_display_object = logic::display_object(world, tile_drain);
    let drain_pos = display::object(world, drain_display_object).pos;
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = drain_pos.x;
        object.pos.y = drain_pos.y + LIFTED_Y_OFFSET;
    }
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let listener = {
        let mut plug = world.obj_mut::<TilePlugLogic>(this);
        plug.touch_screen_pos = touch_pos;
        plug.unplugged = true;
        plug.dragging = true;
        plug.apply_spring = true;
        plug.active_touch = Some(touch);
        plug.tile_plug_logic_listener
    };
    if let Some(listener) = listener {
        crate::controller::on_tile_plug_drag(world, listener, this);
    }

    let pygmy_marker = pg_engine::library::display_marker(world, "UnderwaterPygmyDisplayMarker");
    if let Some(pygmy_marker) = pygmy_marker {
        display::insert_before(world, pygmy_marker, Some(display_object));
    }
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("TilePlugLifted"));
}

/// `-[TilePlugLogic enterIsland:]` @ 0x1001204d0
pub fn enter_island(_world: &mut World, _this: Entity, _startup: bool) {}

/// `-[TilePlugLogic exitIsland]` @ 0x1001204d4
///
/// A plug out of the drain goes back in (`TilePlugTransitionOut`), letting go of the finger.
pub fn exit_island(world: &mut World, this: Entity) {
    let unplugged = world.obj::<TilePlugLogic>(this).unplugged;
    if !unplugged {
        return;
    }

    world.obj_mut::<TilePlugLogic>(this).unplugged = false;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("TilePlugTransitionOut"));

    let dragging = world.obj::<TilePlugLogic>(this).dragging;
    if !dragging {
        return;
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    let listener = {
        let mut plug = world.obj_mut::<TilePlugLogic>(this);
        plug.dragging = false;
        plug.apply_spring = false;
        plug.active_touch = None;
        plug.tile_plug_logic_listener
    };
    if let Some(listener) = listener {
        crate::controller::on_tile_plug_drag_complete(world, listener, this);
    }
}

/// `-[TilePlugLogic drag:::]` @ 0x1001205ac
///
/// A damped spring toward the finger (stiffness 50, damping 6), applied by `onGameFrame:`; the
/// plug leans along the pull. The first call sets the origin from `[1] xOrigin` and
/// `[0] yOrigin`.
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

        let x_origin = call.float("xOrigin");
        let y_origin = call.float("yOrigin");
        let mut object = display::object_mut(world, display_object);
        object.origin_pos.x = x_origin;
        object.origin_pos.y = y_origin;
    }

    let (pos, display_group) = {
        let object = display::object(world, display_object);
        (object.pos, object.display_group)
    };
    let touch_screen_pos = world.obj::<TilePlugLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    // Original bug, kept: the machine code reloads `pos.z` over the touch's z, so the z offset
    // is always `pos.z - pos.z`.
    let offset = Vec4::new(pos.x - touch_pos.x, pos.y - touch_pos.y, pos.z - pos.z, pos.w);
    let distance = length3(offset);
    if distance == 0.0 {
        return ItemResult::Pending;
    }

    if offset.y != 0.0 {
        let angle = (-offset.y).atan2(-offset.x);
        let z_rot = (angle * 180.0) / std::f32::consts::PI + -90.0;
        display::object_mut(world, display_object).rot.z = z_rot;
    }

    let pos_vel = logic::logic(world, this).pos_vel;
    let stretch = (distance * -DRAG_STIFFNESS) / distance;
    let spring_x = -DRAG_DAMPING * pos_vel.x + stretch * offset.x;
    let spring_y = -DRAG_DAMPING * pos_vel.y + stretch * offset.y;
    let spring_z = -DRAG_DAMPING * pos_vel.z + stretch * offset.z;
    world.obj_mut::<TilePlugLogic>(this).spring_force = Vec4::new(spring_x, spring_y, spring_z, offset.w);
    ItemResult::Pending
}

/// `-[TilePlugLogic fall:::]` @ 0x1001207c4
///
/// While the plug sinks: within 15 (x) and 50 (y) of a drain that can take it, it plugs the drain.
pub fn seq_fall(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<TilePlugLogic>(this).tile_plug_logic_listener;
    let tile_drain = match listener {
        Some(listener) => crate::controller::tile_drain_logic(world, listener, this),
        None => None,
    };
    // The original reads the drain's position unchecked; the controller always has a drain.
    let Some(tile_drain) = tile_drain else {
        return ItemResult::Pending;
    };

    let drain_display_object = logic::display_object(world, tile_drain);
    let drain_pos = display::object(world, drain_display_object).pos;
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;

    let x_distance = (drain_pos.x - pos.x).abs();
    let y_distance = (drain_pos.y - pos.y).abs();
    let over_drain = x_distance < PLUG_X_DISTANCE && y_distance < PLUG_Y_DISTANCE;
    if !over_drain {
        return ItemResult::Pending;
    }

    let can_plug_drain = tile_drain_logic::can_plug_drain(world, tile_drain);
    if !can_plug_drain {
        return ItemResult::Pending;
    }

    world.obj_mut::<TilePlugLogic>(this).unplugged = false;
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("TilePlugInDrain"));
    tile_drain_logic::plug_drain(world, tile_drain, this);
    ItemResult::Replaced
}

/// `-[TilePlugLogic die:::]` @ 0x100120914
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    logic::set_behavior(world, this, Some("TilePlugInit"));
    ItemResult::Done
}

/// `-[TilePlugLogic touchBeganInside:allTouches:]` @ 0x1001209d4
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let listener = {
        let mut plug = world.obj_mut::<TilePlugLogic>(this);
        plug.touch_screen_pos = touch_pos;
        plug.dragging = true;
        plug.apply_spring = true;
        plug.active_touch = Some(touch);
        plug.tile_plug_logic_listener
    };
    if let Some(listener) = listener {
        crate::controller::on_tile_plug_drag(world, listener, this);
    }
    logic::set_behavior(world, this, Some("TilePlugDrag"));
}

/// `-[TilePlugLogic touchBeganOutside:allTouches:]` @ 0x100120a94
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[TilePlugLogic touchMovedEntered:allTouches:]` @ 0x100120a98
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[TilePlugLogic touchMovedExited:allTouches:]` @ 0x100120a9c
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[TilePlugLogic touchMovedInside:allTouches:]` @ 0x100120aa0
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[TilePlugLogic touchMovedOutside:allTouches:]` @ 0x100120ab0
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<TilePlugLogic>(this).active_touch;
    let touch_is_ours = active_touch.is_none() || active_touch == Some(touch);
    if !touch_is_ours {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<TilePlugLogic>(this).touch_screen_pos = touch_pos;
}

/// `-[TilePlugLogic touchEndedInside:allTouches:]` @ 0x100120b08
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<TilePlugLogic>(this).active_touch;
    if active_touch != Some(touch) {
        return;
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    let listener = {
        let mut plug = world.obj_mut::<TilePlugLogic>(this);
        plug.dragging = false;
        plug.apply_spring = false;
        plug.active_touch = None;
        plug.tile_plug_logic_listener
    };
    if let Some(listener) = listener {
        crate::controller::on_tile_plug_drag_complete(world, listener, this);
    }
    logic::set_behavior(world, this, Some("TilePlugFall"));
}

/// `-[TilePlugLogic touchEndedOutside:allTouches:]` @ 0x100120bb0
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
