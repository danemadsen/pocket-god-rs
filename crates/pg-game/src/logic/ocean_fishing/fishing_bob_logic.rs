//! `FishingBobLogic` (superclass `SpriteLogic`).
//!
//! The bob at the end of the fishing line (`docs/re/logic-catalog.md` §4.3). The fishing pole
//! creates it in `startFishing:`; the player drags it sideways along the water, the fish and
//! the shark hook themselves on it, and `moveTowardPygmy:::` reels a hooked fish in.

use crate::prelude::*;

use crate::logic::island_core::coconut_logic::{
    THROWABLE_GRAVITY_FORCE, clamp_throwable_to_screen, integrate_throwable, throwable_tilted_gravity_force,
};
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::ocean_fishing::{fish_logic, fishing_pole_logic};
use pg_engine::display;
use pg_engine::logic;
use pg_engine::touch::BcTouch;

/// `FishingBobLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct FishingBobLogic {
    /// `mFishingBobLogicListener`: `@"<FishingBobLogicListener>"`
    pub fishing_bob_logic_listener: Option<Entity>,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mTiltState`: `i`
    pub tilt_state: i32,

    /// `mDragPos`: `{?="x"f"y"f"z"f"w"f}`
    pub drag_pos: Vec4,

    /// `mLastDragXPos`: `f`
    pub last_drag_x_pos: f32,

    /// `mDragXVel`: `f`
    pub drag_x_vel: f32,

    /// `mHookedFishLogic`: `@"FishLogic"`
    pub hooked_fish_logic: Option<Entity>,

    /// `mHookedSharkLogic`: `@"SharkLogic"`
    pub hooked_shark_logic: Option<Entity>,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbFishingIdle`: `B`
    pub fishing_idle: bool,
}

pub const FISHING_BOB_LOGIC: Class = Class {
    name: "FishingBobLogic",
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

/// `onGameFrame:`: a released bob faster than this (either way) loses 5 % of its speed a frame.
const DRAG_SLOW_DOWN_SPEED: f32 = 10.0;
const DRAG_SLOW_DOWN_FACTOR: f32 = 0.95;

/// `drag:::`: the release speed is the last frame's drag step times this.
const DRAG_X_VEL_FACTOR: f32 = 20.0;

/// `moveTowardPygmy:::`: the catch is reeled in to this far in front of the pygmy.
const REEL_IN_X_OFFSET: f32 = 40.0;

/// Sequence-item handlers implemented by `FishingBobLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "moveTowardPygmy" => seq_move_toward_pygmy(world, this, call),
        "drag" => seq_drag(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[FishingBobLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&FISHING_BOB_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[FishingBobLogic initWithDisplayObject:]` @ 0x1000d6038
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(FishingBobLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<FishingBobLogic>(this).tilt_state = 0;
}

/// `-[FishingBobLogic setFishingBobLogicListener:]` @ 0x1000d6084
pub fn set_fishing_bob_logic_listener(world: &mut World, this: Entity, fishing_bob_logic_listener: Entity) {
    world.obj_mut::<FishingBobLogic>(this).fishing_bob_logic_listener = Some(fishing_bob_logic_listener);
}

/// `-[FishingBobLogic initBobTexture]` @ 0x1000d6144
pub fn init_bob_texture(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("FishingBobInit"));
}

/// `-[FishingBobLogic startFishingIdle]` @ 0x1000d615c
pub fn start_fishing_idle(world: &mut World, this: Entity) {
    world.obj_mut::<FishingBobLogic>(this).fishing_idle = true;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
    logic::set_behavior(world, this, Some("FishingBobIdle"));
}

/// `-[FishingBobLogic dropWhileFishing]` @ 0x1000d61c4
pub fn drop_while_fishing(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("FishingBobNothing"));

    let hooked_fish = world.obj::<FishingBobLogic>(this).hooked_fish_logic;
    if let Some(hooked_fish) = hooked_fish {
        fish_logic::unhook(world, hooked_fish);
    }
}

/// `-[FishingBobLogic isUnhooked]` @ 0x1000d6210
pub fn is_unhooked(world: &mut World, this: Entity) -> bool {
    let bob = world.obj::<FishingBobLogic>(this);
    bob.fishing_idle && bob.hooked_fish_logic.is_none() && bob.hooked_shark_logic.is_none() && bob.active_touch.is_none()
}

/// `[mFishingBobLogicListener fishingPoleLogic:self]`: the view controller's fishing pole (nil
/// without a listener).
fn listener_fishing_pole_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<FishingBobLogic>(this).fishing_bob_logic_listener?;
    crate::controller::fishing_pole_logic(world, listener, this)
}

/// `[pole pygmyLogic]` for a pole that may be nil.
fn fishing_pole_pygmy_logic(world: &mut World, fishing_pole: Option<Entity>) -> Option<Entity> {
    let fishing_pole = fishing_pole?;
    fishing_pole_logic::pygmy_logic(world, fishing_pole)
}

/// `-[FishingBobLogic hookFish:]` @ 0x1000d6260
pub fn hook_fish(world: &mut World, this: Entity, hook_fish: Entity) {
    world.obj_mut::<FishingBobLogic>(this).hooked_fish_logic = Some(hook_fish);

    let fishing_pole = listener_fishing_pole_logic(world, this);
    let pygmy = fishing_pole_pygmy_logic(world, fishing_pole);

    logic::set_behavior(world, this, Some("FishingBobPulling"));
    if let Some(pygmy) = pygmy {
        crate::pygmy::pull_pole(world, pygmy);
    }
}

/// `-[FishingBobLogic hookShark:]` @ 0x1000d62e8
pub fn hook_shark(world: &mut World, this: Entity, hook_shark: Entity) -> bool {
    let fishing_pole = listener_fishing_pole_logic(world, this);
    let pygmy = fishing_pole_pygmy_logic(world, fishing_pole);

    // A nil pygmy answers an x scale of 0.
    let pygmy_x_scale = match pygmy {
        Some(pygmy) => {
            let pygmy_display_object = logic::display_object(world, pygmy);
            display::object(world, pygmy_display_object).scale.x
        }
        None => 0.0,
    };
    let shark_display_object = logic::display_object(world, hook_shark);
    let shark_x_scale = display::object(world, shark_display_object).scale.x;

    // Only a shark facing the same way as the fishing pygmy bites.
    let same_facing = 0.0 <= pygmy_x_scale * shark_x_scale;
    if same_facing {
        world.obj_mut::<FishingBobLogic>(this).hooked_shark_logic = Some(hook_shark);
        logic::set_behavior(world, this, Some("FishingBobCatchShark"));

        if let Some(fishing_pole) = fishing_pole {
            fishing_pole_logic::catch_shark(world, fishing_pole);
        }
        if let Some(pygmy) = pygmy {
            crate::pygmy::catch_shark(world, pygmy);
        }
    }
    same_facing
}

/// `-[FishingBobLogic isAccelEnabled]` @ 0x1000d63fc
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<FishingBobLogic>(this).fishing_bob_logic_listener;
    let gravity_enabled = match listener {
        Some(listener) => crate::controller::is_gravity_enabled(world, listener),
        None => false,
    };
    if !gravity_enabled {
        return false;
    }
    sprite_logic::is_accel_enabled(world, this)
}

/// `-[FishingBobLogic onGameFrame:]` @ 0x1000d646c
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let accel_enabled = is_accel_enabled(world, this);
    if accel_enabled {
        let gravity_force = throwable_tilted_gravity_force(world);
        world.obj_mut::<SpriteLogic>(this).gravity_force = gravity_force;
        refresh_tilt_state(world, this);
    } else {
        world.obj_mut::<SpriteLogic>(this).gravity_force = THROWABLE_GRAVITY_FORCE;
        undo_tilt_state(world, this);
    }

    // A NaN speed counts as fast.
    let x_vel = logic::logic(world, this).pos_vel.x;
    let slow = (-DRAG_SLOW_DOWN_SPEED..=DRAG_SLOW_DOWN_SPEED).contains(&x_vel);
    if !slow {
        logic::logic_mut(world, this).pos_vel.x = x_vel * DRAG_SLOW_DOWN_FACTOR;
    }

    // The gravity force is computed but never applied: the bob floats.
    let pos_force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    logic::logic_mut(world, this).pos_force = pos_force;
    integrate_throwable(world, this, pos_force, dt);

    let display_object = logic::display_object(world, this);
    clamp_throwable_to_screen(world, display_object);
}

/// `-[FishingBobLogic refreshTiltState]` @ 0x1000d6738
pub fn refresh_tilt_state(_world: &mut World, _this: Entity) {}

/// `-[FishingBobLogic undoTiltState]` @ 0x1000d673c
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    let mut bob = world.obj_mut::<FishingBobLogic>(this);
    if !bob.dragging {
        bob.tilt_state = 0;
    }
}

/// `-[FishingBobLogic moveTowardPygmy:::]` @ 0x1000d675c
pub fn seq_move_toward_pygmy(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let fishing_pole = listener_fishing_pole_logic(world, this);
    let pygmy = fishing_pole_pygmy_logic(world, fishing_pole);
    let Some(pygmy) = pygmy else {
        // PORT: the original reads the position of a nil pygmy's display object (a crash); the
        // pulling sequence only runs while a pygmy holds the pole, so the port just waits.
        return ItemResult::Pending;
    };

    let pygmy_display_object = logic::display_object(world, pygmy);
    let pygmy_display = display::object(world, pygmy_display_object);
    let pygmy_x_pos = pygmy_display.pos.x;
    let pygmy_x_scale = pygmy_display.scale.x;

    let reel_in_offset = if pygmy_x_scale <= 0.0 { -REEL_IN_X_OFFSET } else { REEL_IN_X_OFFSET };
    let target_x_pos = pygmy_x_pos + reel_in_offset;

    let display_object = logic::display_object(world, this);
    if call.first_time {
        let x_pos = display::object(world, display_object).pos.x;
        logic::logic_mut(world, this).pos_vel.x = target_x_pos - x_pos;
    }

    let x_vel = logic::logic(world, this).pos_vel.x;
    let x_pos = display::object(world, display_object).pos.x;
    let passed_moving_left = x_vel < 0.0 && x_pos < target_x_pos;
    let passed_moving_right = 0.0 < x_vel && target_x_pos < x_pos;
    let arrived = passed_moving_left || passed_moving_right || x_vel == 0.0;
    if !arrived {
        return ItemResult::Pending;
    }

    let hooked_fish = world.obj::<FishingBobLogic>(this).hooked_fish_logic;
    if let Some(hooked_fish) = hooked_fish {
        fish_logic::catch(world, hooked_fish);
    }
    world.obj_mut::<FishingBobLogic>(this).hooked_fish_logic = None;

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<FishingBobLogic>(this).fishing_idle = false;
    display::object_mut(world, display_object).pos.x = target_x_pos;

    crate::pygmy::catch_fish(world, pygmy);
    if let Some(fishing_pole) = fishing_pole {
        fishing_pole_logic::catch_fish(world, fishing_pole);
    }
    logic::set_behavior(world, this, Some("FishingBobCatchFish"));
    ItemResult::Done
}

/// `-[FishingBobLogic drag:::]` @ 0x1000d6934
pub fn seq_drag(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let bob = world.obj::<FishingBobLogic>(this);
    let drag_x_pos = bob.drag_pos.x;
    let last_drag_x_pos = bob.last_drag_x_pos;

    let drag_x_vel = (drag_x_pos - last_drag_x_pos) * DRAG_X_VEL_FACTOR;
    world.obj_mut::<FishingBobLogic>(this).drag_x_vel = drag_x_vel;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).pos.x = drag_x_pos;
    world.obj_mut::<FishingBobLogic>(this).last_drag_x_pos = drag_x_pos;
    ItemResult::Pending
}

/// `[[mDisplayObject displayGroup] viewToDisplayObject:[touch pos]]`: the touch in the bob's
/// display group (a nil group answers the zero point).
fn touch_drag_pos(world: &World, this: Entity, touch: Entity) -> Vec4 {
    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;
    let touch_pos = world.obj::<BcTouch>(touch).pos;

    match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_pos),
        None => Vec4::ZERO,
    }
}

/// `-[FishingBobLogic touchBeganInside:allTouches:]` @ 0x1000d69b4
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let bob = world.obj::<FishingBobLogic>(this);
    if bob.active_touch.is_some() || bob.hooked_fish_logic.is_some() {
        return;
    }

    world.obj_mut::<FishingBobLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    {
        let mut bob = world.obj_mut::<FishingBobLogic>(this);
        bob.dragging = true;
        bob.touch_began_inside = true;
        bob.tilt_state = 0;
    }

    let drag_pos = touch_drag_pos(world, this, touch);
    world.obj_mut::<FishingBobLogic>(this).drag_pos = drag_pos;

    let listener = world.obj::<FishingBobLogic>(this).fishing_bob_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_fishing_bob_drag(world, listener, this);
    }
    logic::set_behavior(world, this, Some("FishingBobDrag"));
}

/// `-[FishingBobLogic touchBeganOutside:allTouches:]` @ 0x1000d6af0
pub fn touch_began_outside(world: &mut World, this: Entity, _touch: Entity) {
    let mut bob = world.obj_mut::<FishingBobLogic>(this);
    if bob.active_touch.is_none() {
        bob.touch_began_inside = false;
    }
}

/// `-[FishingBobLogic touchMovedEntered:allTouches:]` @ 0x1000d6b10
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let bob = world.obj::<FishingBobLogic>(this);
    let own_touch = bob.active_touch.is_none() || bob.active_touch == Some(touch);
    if own_touch && bob.dragging {
        let drag_pos = touch_drag_pos(world, this, touch);
        world.obj_mut::<FishingBobLogic>(this).drag_pos = drag_pos;
    }
}

/// `-[FishingBobLogic touchMovedExited:allTouches:]` @ 0x1000d6bc0
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[FishingBobLogic touchMovedInside:allTouches:]` @ 0x1000d6bc4
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[FishingBobLogic touchMovedOutside:allTouches:]` @ 0x1000d6bd4
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let bob = world.obj::<FishingBobLogic>(this);
    let own_touch = bob.active_touch.is_none() || bob.active_touch == Some(touch);
    if own_touch && bob.dragging {
        let drag_pos = touch_drag_pos(world, this, touch);
        world.obj_mut::<FishingBobLogic>(this).drag_pos = drag_pos;
    }
}

/// `-[FishingBobLogic touchEndedInside:allTouches:]` @ 0x1000d6c84
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<FishingBobLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let dragging = world.obj::<FishingBobLogic>(this).dragging;
    if dragging {
        world.obj_mut::<FishingBobLogic>(this).dragging = false;

        // Released with the speed of the last drag step.
        let drag_x_vel = world.obj::<FishingBobLogic>(this).drag_x_vel;
        logic::logic_mut(world, this).pos_vel.x = drag_x_vel;

        let listener = world.obj::<FishingBobLogic>(this).fishing_bob_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_fishing_bob_drag_complete(world, listener, this);
        }
        logic::set_behavior(world, this, Some("FishingBobNoDrag"));
    }

    let mut bob = world.obj_mut::<FishingBobLogic>(this);
    bob.touch_began_inside = false;
    bob.active_touch = None;
}

/// `-[FishingBobLogic touchEndedOutside:allTouches:]` @ 0x1000d6d38
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
