//! `DanceBalloonLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::coconut_logic;
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::logic;
use pg_engine::math::length3;
use pg_engine::touch::BcTouch;

/// `DanceBalloonLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct DanceBalloonLogic {
    /// `mDanceBalloonLogicListener`: `@"<DanceBalloonLogicListener>"`
    pub dance_balloon_logic_listener: Option<Entity>,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mSpringDelta`: `f`
    pub spring_delta: f32,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbFalling`: `B`
    pub falling: bool,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,
}

pub const DANCE_BALLOON_LOGIC: Class = Class {
    name: "DanceBalloonLogic",
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

/// Half the side of the balloon's touch box (`setColXMin:` … `setColYMax:`).
const COLLISION_HALF_SIZE: f32 = 35.0;

/// `mGravityForce` while the balloon flies: helium lifts it.
const BUOYANCY_FORCE: Vec4 = Vec4::new(0.0, 600.0, 0.0, 1.0);

/// The drag spring of `drag:::` that hangs the balloon below the finger.
const DRAG_STIFFNESS: f32 = 50.0;
const DRAG_DAMPING: f32 = 6.0;

/// The balloon's edges are kept between x = 0 and this.
const SCREEN_WIDTH: f32 = 480.0;

/// Past these heights the balloon has left the screen and dies.
const MIN_Y_POS: f32 = -100.0;
const MAX_Y_POS: f32 = 820.0;

/// A pygmy inhales a dropped balloon within 35 points of its head (31 above its origin).
const PYGMY_HEAD_Y_OFFSET: f32 = 31.0;
const INHALE_DISTANCE_SQUARED: f32 = 1225.0;

/// Sequence-item handlers implemented by `DanceBalloonLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "drag" => seq_drag(world, this, call),
        "fall" => seq_fall(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[DanceBalloonLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&DANCE_BALLOON_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[DanceBalloonLogic initWithDisplayObject:]` @ 0x100169fd8
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(DanceBalloonLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let display_object = logic::display_object(world, this);
    let mut display = display::object_mut(world, display_object);
    display.col_x_min = -COLLISION_HALF_SIZE;
    display.col_x_max = COLLISION_HALF_SIZE;
    display.col_y_min = -COLLISION_HALF_SIZE;
    display.col_y_max = COLLISION_HALF_SIZE;
    display.is_selectable = true;
}

/// `-[DanceBalloonLogic setDanceBalloonLogicListener:]` @ 0x10016a0c4
pub fn set_dance_balloon_logic_listener(world: &mut World, this: Entity, dance_balloon_logic_listener: Entity) {
    world.obj_mut::<DanceBalloonLogic>(this).dance_balloon_logic_listener = Some(dance_balloon_logic_listener);
}

/// `-[DanceBalloonLogic isActive]` @ 0x10016a184
pub fn is_active(world: &mut World, this: Entity) -> bool {
    world.obj::<DanceBalloonLogic>(this).active
}

/// `-[DanceBalloonLogic enterIsland]` @ 0x10016a194
///
/// The balloon comes out of the bundle at rest and goes into the drag layer.
pub fn enter_island(world: &mut World, this: Entity) {
    world.obj_mut::<DanceBalloonLogic>(this).active = true;
    {
        let mut balloon_logic = logic::logic_mut(world, this);
        balloon_logic.pos_vel.x = 0.0;
        balloon_logic.pos_vel.y = 0.0;
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).pos.z = SPRITE_Z;

    let listener = {
        let mut balloon = world.obj_mut::<DanceBalloonLogic>(this);
        balloon.falling = false;
        balloon.apply_spring = false;
        balloon.active_touch = None;
        balloon.dance_balloon_logic_listener
    };

    let drag_display_marker = match listener {
        Some(listener) => crate::controller::pygmy_drag_display_marker(world, listener),
        None => None,
    };
    if let Some(drag_display_marker) = drag_display_marker {
        display::insert_after(world, drag_display_marker, Some(display_object));
    }

    logic::set_behavior(world, this, Some("DanceBalloonEnterIsland"));
}

/// `-[DanceBalloonLogic exitIsland]` @ 0x10016a268
pub fn exit_island(world: &mut World, this: Entity) {
    world.obj_mut::<DanceBalloonLogic>(this).active = false;
    logic::set_behavior(world, this, Some("DanceBalloonExitIsland"));
}

/// `-[DanceBalloonLogic onGameFrame:]` @ 0x10016a28c
///
/// Buoyancy plus the drag spring, integrated in place; the edges stay on screen, and the
/// balloon dies once it has left the screen.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    let active = world.obj::<DanceBalloonLogic>(this).active;
    if shutdown || !active {
        return;
    }

    let display_object = logic::display_object(world, this);

    world.obj_mut::<SpriteLogic>(this).gravity_force = BUOYANCY_FORCE;
    let gravity_force = BUOYANCY_FORCE;

    let mut pos_force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    pos_force.x = gravity_force.x + 0.0;
    pos_force.y = gravity_force.y + 0.0;
    pos_force.z = gravity_force.z + 0.0;

    let balloon = world.obj::<DanceBalloonLogic>(this);
    if balloon.apply_spring {
        let spring_force = balloon.spring_force;
        pos_force.x = spring_force.x + pos_force.x;
        pos_force.y = spring_force.y + pos_force.y;
        pos_force.z = spring_force.z + pos_force.z;
    }

    let pos_vel = {
        let mut balloon_logic = logic::logic_mut(world, this);
        balloon_logic.pos_force = pos_force;
        balloon_logic.pos_vel.x = balloon_logic.pos_vel.x + pos_force.x * dt;
        balloon_logic.pos_vel.y = balloon_logic.pos_vel.y + pos_force.y * dt;
        balloon_logic.pos_vel.z = balloon_logic.pos_vel.z + pos_force.z * dt;
        balloon_logic.pos_vel
    };
    {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = display.pos.x + pos_vel.x * dt;
        display.pos.y = display.pos.y + pos_vel.y * dt;
        display.pos.z = display.pos.z + pos_vel.z * dt;
    }

    // Both bounds are taken before the first correction moves the balloon.
    let min_pos = display::min_pos(world, display_object);
    let max_pos = display::max_pos(world, display_object);
    if min_pos.x < 0.0 {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = (display.pos.x - min_pos.x) + 0.0;
    }
    if max_pos.x > SCREEN_WIDTH {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = SCREEN_WIDTH - (max_pos.x - display.pos.x);
    }

    let y_pos = display::object(world, display_object).pos.y;
    if y_pos < MIN_Y_POS || y_pos > MAX_Y_POS {
        logic::set_behavior(world, this, Some("DanceBalloonDie"));
    }
}

/// `-[DanceBalloonLogic drag:::]` @ 0x10016a520
///
/// While dragged, a spring pulls the balloon towards the finger and turns it to hang from it.
pub fn seq_drag(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    let display = display::object(world, display_object);
    let pos = display.pos;
    let display_group = display.display_group;
    let touch_screen_pos = world.obj::<DanceBalloonLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    // Original quirk, kept: the compiled code subtracts the balloon's own z (not the touch's),
    // so the offset's z is 0.
    let subtracted_z = pos.z;
    let offset_x = pos.x - touch_pos.x;
    let offset_y = pos.y - touch_pos.y;
    let offset_z = pos.z - subtracted_z;
    let offset = Vec4::new(offset_x, offset_y, offset_z, pos.w);

    let length = length3(offset);
    if length == 0.0 {
        return ItemResult::Pending;
    }

    if offset.y != 0.0 {
        let angle = (-offset.y).atan2(-offset.x);
        let z_rot = (angle * 180.0) / std::f32::consts::PI + 90.0;
        display::object_mut(world, display_object).rot.z = z_rot;
    }

    let pos_vel = logic::logic(world, this).pos_vel;
    let spring_force = coconut_logic::throwable_spring_force(offset, length, pos_vel, DRAG_STIFFNESS, DRAG_DAMPING);
    world.obj_mut::<DanceBalloonLogic>(this).spring_force = spring_force;
    ItemResult::Pending
}

/// `-[DanceBalloonLogic fall:::]` @ 0x10016a6b0
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<DanceBalloonLogic>(this).falling = true;
    }
    ItemResult::Pending
}

/// `-[DanceBalloonLogic die:::]` @ 0x10016a6cc
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<DanceBalloonLogic>(this).active = false;

    let balloon = world.obj::<DanceBalloonLogic>(this);
    let dragging = balloon.dragging;
    let listener = balloon.dance_balloon_logic_listener;
    if dragging {
        world.obj_mut::<DanceBalloonLogic>(this).dragging = false;
        if let Some(listener) = listener {
            crate::controller::on_dance_balloon_drag_complete(world, listener, this);
        }
    }

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[DanceBalloonLogic checkPygmyCollision]` @ 0x10016a74c
///
/// A balloon let go near a pygmy's head is inhaled by the first pygmy that takes it.
pub fn check_pygmy_collision(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let balloon_pos = display::object(world, display_object).pos;
    let x_pos = balloon_pos.x;
    let y_pos = balloon_pos.y;

    let listener = world.obj::<DanceBalloonLogic>(this).dance_balloon_logic_listener;
    let pygmy_logic_array = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };

    for pygmy in pygmy_logic_array {
        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let x_distance = x_pos - (pygmy_pos.x + 0.0);
        let y_distance = y_pos - (pygmy_pos.y + PYGMY_HEAD_Y_OFFSET);
        let distance_squared = x_distance * x_distance + y_distance * y_distance;
        let within_reach = distance_squared < INHALE_DISTANCE_SQUARED;
        if !within_reach {
            continue;
        }

        let inhaled = crate::pygmy::inhale_dance_balloon(world, pygmy, this);
        if inhaled {
            logic::set_behavior(world, this, Some("DanceBalloonDie"));
            return;
        }
    }
}

/// `-[DanceBalloonLogic touchBeganInside:allTouches:]` @ 0x10016a974
///
/// Grabs the balloon unless it is falling or already held.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let balloon = world.obj::<DanceBalloonLogic>(this);
    if balloon.falling || balloon.active_touch.is_some() {
        return;
    }

    world.obj_mut::<DanceBalloonLogic>(this).active_touch = Some(touch);
    {
        let mut balloon_logic = logic::logic_mut(world, this);
        balloon_logic.pos_vel.x = 0.0;
        balloon_logic.pos_vel.y = 0.0;
    }

    let touch_screen_pos = world.obj::<BcTouch>(touch).pos;
    let listener = {
        let mut balloon = world.obj_mut::<DanceBalloonLogic>(this);
        balloon.touch_began_inside = true;
        balloon.touch_screen_pos = touch_screen_pos;
        balloon.dragging = true;
        balloon.apply_spring = true;
        balloon.touch_began_inside = false;
        balloon.dance_balloon_logic_listener
    };

    if let Some(listener) = listener {
        crate::controller::on_dance_balloon_drag(world, listener, this);
    }
    logic::set_behavior(world, this, Some("DanceBalloonDrag"));
}

/// `-[DanceBalloonLogic touchBeganOutside:allTouches:]` @ 0x10016aa68
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    if world.obj::<DanceBalloonLogic>(this).active_touch.is_some() {
        return;
    }

    let touch_screen_pos = world.obj::<BcTouch>(touch).pos;
    let mut balloon = world.obj_mut::<DanceBalloonLogic>(this);
    balloon.touch_screen_pos = touch_screen_pos;
    balloon.touch_began_inside = false;
}

/// `-[DanceBalloonLogic touchMovedEntered:allTouches:]` @ 0x10016aac4
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    follow_touch(world, this, touch);
}

/// `-[DanceBalloonLogic touchMovedExited:allTouches:]` @ 0x10016ab1c
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    follow_touch(world, this, touch);
}

/// `-[DanceBalloonLogic touchMovedInside:allTouches:]` @ 0x10016ab74
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[DanceBalloonLogic touchMovedOutside:allTouches:]` @ 0x10016ab84
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    follow_touch(world, this, touch);
}

/// `-[DanceBalloonLogic touchEndedInside:allTouches:]` @ 0x10016abdc
///
/// Letting go of a dragged balloon drops it; it is inhaled at once if it is near a pygmy.
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    if !is_own_touch(world, this, touch) {
        return;
    }

    let balloon = world.obj::<DanceBalloonLogic>(this);
    let dragging = balloon.dragging;
    let listener = balloon.dance_balloon_logic_listener;
    if dragging {
        {
            let mut balloon = world.obj_mut::<DanceBalloonLogic>(this);
            balloon.falling = true;
            balloon.dragging = false;
            balloon.apply_spring = false;
        }
        if let Some(listener) = listener {
            crate::controller::on_dance_balloon_drag_complete(world, listener, this);
        }
        logic::set_behavior(world, this, Some("DanceBalloonFall"));
        check_pygmy_collision(world, this);
    }

    let mut balloon = world.obj_mut::<DanceBalloonLogic>(this);
    balloon.touch_began_inside = false;
    balloon.active_touch = None;
}

/// `-[DanceBalloonLogic touchEndedOutside:allTouches:]` @ 0x10016aca4
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}

/// `mActiveTouch == nil || mActiveTouch == touch`: the touch moves or ends the balloon's drag.
fn is_own_touch(world: &World, this: Entity, touch: Entity) -> bool {
    let active_touch = world.obj::<DanceBalloonLogic>(this).active_touch;
    match active_touch {
        Some(active_touch) => active_touch == touch,
        None => true,
    }
}

/// The body of the `touchMoved…` handlers: the spring target follows the balloon's touch.
fn follow_touch(world: &mut World, this: Entity, touch: Entity) {
    if !is_own_touch(world, this, touch) {
        return;
    }

    let touch_screen_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<DanceBalloonLogic>(this).touch_screen_pos = touch_screen_pos;
}
