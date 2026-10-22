//! `ApocalypseGemLogic` (superclass `SpriteLogic`).
//!
//! A throwable (`docs/re/logic-catalog.md` §2.2); the physics steps it shares with the other
//! throwables come from `CoconutLogic`'s `throwable_*` helpers.

use crate::prelude::*;

use crate::logic::apocalypse_temples::apocalypse_gem_pedestal_logic;
use crate::logic::island_core::coconut_logic::{
    THROWABLE_GRAVITY_FORCE, apply_throwable_slide_friction, integrate_throwable, is_island_tipped_over, throwable_drag_offset,
    throwable_drag_z_rot, throwable_pos_force, throwable_spring_force, throwable_tilt_state, throwable_tilted_gravity_force,
};
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::length3;
use pg_engine::touch::BcTouch;

/// `ApocalypseGemLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ApocalypseGemLogic {
    /// `mApocalypseGemLogicListener`: `@"<ApocalypseGemLogicListener>"`
    pub apocalypse_gem_logic_listener: Option<Entity>,

    /// `mGemIndex`: `i`
    pub gem_index: i32,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mApocalypseGemType`: `i`
    pub apocalypse_gem_type: i32,

    /// `mApocalypseGemPedestalLogic`: `@"ApocalypseGemPedestalLogic"`
    pub apocalypse_gem_pedestal_logic: Option<Entity>,

    /// `mbDontHitUntilBounce`: `B`
    pub dont_hit_until_bounce: bool,

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

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbFalling`: `B`
    pub falling: bool,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,

    /// `mbInWater`: `B`
    pub in_water: bool,
}

pub const APOCALYPSE_GEM_LOGIC: Class = Class {
    name: "ApocalypseGemLogic",
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

/// The island floor of `onGameFrame:`.
const FLOOR_Y: f32 = 92.0;

/// `fall:::` splashes below this y when the ocean is enabled.
const WATER_LINE_Y: f32 = 40.0;

/// The spring of `drag:::`.
const DRAG_STIFFNESS: f32 = 100.0;
const DRAG_DAMPING: f32 = 8.0;

/// `checkForApocalypseGemPedestalDuringFall`: a falling gem lands in a pedestal when it is
/// within 20 (on each axis) of the point (22, 51) from the pedestal's position.
const PEDESTAL_X_OFFSET: f32 = 22.0;
const PEDESTAL_Y_OFFSET: f32 = 51.0;
const PEDESTAL_HIT_DISTANCE: f32 = 20.0;

/// Sequence-item handlers implemented by `ApocalypseGemLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "slide" => seq_slide(world, this, call),
        "setAnimationBasedOnGemType" => seq_set_animation_based_on_gem_type(world, this, call),
        "resetLastPos" => seq_reset_last_pos(world, this, call),
        "drag" => seq_drag(world, this, call),
        "fall" => seq_fall(world, this, call),
        "checkRespawn" => seq_check_respawn(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[ApocalypseGemLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&APOCALYPSE_GEM_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// The listener (the view controller).
fn gem_logic_listener(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<ApocalypseGemLogic>(this).apocalypse_gem_logic_listener
}

/// `[[mApocalypseGemLogicListener pygmyDisplayMarker] insertBefore:mDisplayObject]`.
fn insert_before_pygmy_display_marker(world: &mut World, this: Entity) {
    let Some(listener) = gem_logic_listener(world, this) else {
        return;
    };
    let marker = crate::controller::pygmy_display_marker(world, listener);
    if let Some(marker) = marker {
        let display_object = logic::display_object(world, this);
        display::insert_before(world, marker, Some(display_object));
    }
}

/// `[[mApocalypseGemLogicListener pygmyDragDisplayMarker] insertAfter:mDisplayObject]`.
fn insert_after_pygmy_drag_display_marker(world: &mut World, this: Entity) {
    let Some(listener) = gem_logic_listener(world, this) else {
        return;
    };
    let marker = crate::controller::pygmy_drag_display_marker(world, listener);
    if let Some(marker) = marker {
        let display_object = logic::display_object(world, this);
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `[mApocalypseGemLogicListener onApocalypseGemDragComplete:self]`.
fn send_drag_complete(world: &mut World, this: Entity) {
    if let Some(listener) = gem_logic_listener(world, this) {
        crate::controller::on_apocalypse_gem_drag_complete(world, listener, this);
    }
}

/// The gem leaves its pedestal: `mbDontHitUntilBounce = YES`,
/// `[mApocalypseGemPedestalLogic ejectApocalypseGemFromApocalypseGemPedestal:self]`, then
/// `mApocalypseGemPedestalLogic = nil` (`deactivate`, `die:::`, `touchBeganInside:allTouches:`).
fn leave_apocalypse_gem_pedestal(world: &mut World, this: Entity) {
    let pedestal = world.obj::<ApocalypseGemLogic>(this).apocalypse_gem_pedestal_logic;
    let Some(pedestal) = pedestal else {
        return;
    };

    world.obj_mut::<ApocalypseGemLogic>(this).dont_hit_until_bounce = true;
    apocalypse_gem_pedestal_logic::eject_apocalypse_gem_from_apocalypse_gem_pedestal(world, pedestal, this);
    world.obj_mut::<ApocalypseGemLogic>(this).apocalypse_gem_pedestal_logic = None;
}

/// `-[ApocalypseGemLogic initWithDisplayObject:]` @ 0x10000e894
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ApocalypseGemLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut gem = world.obj_mut::<ApocalypseGemLogic>(this);
        gem.tilt_state = 0;
        gem.slide_friction = SLIDE_FRICTION;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<ApocalypseGemLogic>(this).enable_island_floor_collision = true;

    let own_display_object = logic::display_object(world, this);
    let mut display = display::object_mut(world, own_display_object);
    display.col_x_min = -20.0;
    display.col_x_max = 20.0;
    display.col_y_min = -10.0;
    display.col_y_max = 30.0;
}

/// `-[ApocalypseGemLogic setApocalypseGemLogicListener:]` @ 0x10000e990
pub fn set_apocalypse_gem_logic_listener(world: &mut World, this: Entity, apocalypse_gem_logic_listener: Entity) {
    world.obj_mut::<ApocalypseGemLogic>(this).apocalypse_gem_logic_listener = Some(apocalypse_gem_logic_listener);
}

/// `-[ApocalypseGemLogic setApocalypseGemType:]` @ 0x10000e9ec
pub fn set_apocalypse_gem_type(world: &mut World, this: Entity, apocalypse_gem_type: i32) {
    world.obj_mut::<ApocalypseGemLogic>(this).apocalypse_gem_type = apocalypse_gem_type;
}

/// `-[ApocalypseGemLogic apocalypseGemType]` @ 0x10000e9fc
pub fn apocalypse_gem_type(world: &mut World, this: Entity) -> i32 {
    world.obj::<ApocalypseGemLogic>(this).apocalypse_gem_type
}

/// `-[ApocalypseGemLogic prepareToEndApocalypse]` @ 0x10000ea0c
pub fn prepare_to_end_apocalypse(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
}

/// `-[ApocalypseGemLogic isActive]` @ 0x10000ea30
pub fn is_active(world: &mut World, this: Entity) -> bool {
    world.obj::<ApocalypseGemLogic>(this).active
}

/// `-[ApocalypseGemLogic ejectFromApocalypseGemPedestal]` @ 0x10000ea40
pub fn eject_from_apocalypse_gem_pedestal(world: &mut World, this: Entity) {
    let pedestal = world.obj::<ApocalypseGemLogic>(this).apocalypse_gem_pedestal_logic;
    if pedestal.is_none() {
        return;
    }

    world.obj_mut::<ApocalypseGemLogic>(this).apocalypse_gem_pedestal_logic = None;
    logic::set_behavior(world, this, Some("ApocalypseGemDie"));
}

/// `-[ApocalypseGemLogic checkForApocalypseGemPedestalDuringFall]` @ 0x10000ea70
pub fn check_for_apocalypse_gem_pedestal_during_fall(world: &mut World, this: Entity) -> bool {
    let dont_hit_until_bounce = world.obj::<ApocalypseGemLogic>(this).dont_hit_until_bounce;
    if dont_hit_until_bounce {
        return false;
    }

    let Some(listener) = gem_logic_listener(world, this) else {
        return false;
    };
    let pedestal_enabled = crate::controller::is_apocalypse_gem_pedestal_enabled(world, listener);
    if !pedestal_enabled {
        return false;
    }

    let pedestals = crate::controller::apocalypse_gem_pedestal_logic_array(world, listener, this);
    let own_display_object = logic::display_object(world, this);
    for pedestal in pedestals {
        let available = apocalypse_gem_pedestal_logic::is_available(world, pedestal);
        if !available {
            continue;
        }

        let pedestal_display_object = logic::display_object(world, pedestal);
        let pedestal_pos = display::object(world, pedestal_display_object).pos;
        let pos = display::object(world, own_display_object).pos;

        let pos_vel = logic::logic(world, this).pos_vel;
        if !(pos_vel.y < 0.0) {
            continue;
        }

        let x_distance = (pos.x - (pedestal_pos.x + PEDESTAL_X_OFFSET)).abs();
        let y_distance = (pos.y - (pedestal_pos.y + PEDESTAL_Y_OFFSET)).abs();
        let hit = x_distance <= PEDESTAL_HIT_DISTANCE && y_distance <= PEDESTAL_HIT_DISTANCE;
        if !hit {
            continue;
        }

        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        {
            let mut gem = world.obj_mut::<ApocalypseGemLogic>(this);
            gem.enable_island_floor_collision = false;
            gem.apocalypse_gem_pedestal_logic = Some(pedestal);
        }
        apocalypse_gem_pedestal_logic::apocalypse_gem_fall_into_apocalypse_gem_pedestal(world, pedestal, this);

        let seated_pedestal = world.obj::<ApocalypseGemLogic>(this).apocalypse_gem_pedestal_logic;
        if let Some(seated_pedestal) = seated_pedestal {
            let seated_display_object = logic::display_object(world, seated_pedestal);
            display::insert_after(world, seated_display_object, Some(own_display_object));
        }
        logic::set_behavior(world, this, Some("ApocalypseGemFallIntoApocalypseGemPedestal"));
        return true;
    }
    false
}

/// `-[ApocalypseGemLogic refreshDisplayMarker]` @ 0x10000ed4c
pub fn refresh_display_marker(world: &mut World, this: Entity) {
    let pedestal = world.obj::<ApocalypseGemLogic>(this).apocalypse_gem_pedestal_logic;
    if pedestal.is_some() {
        return;
    }
    insert_before_pygmy_display_marker(world, this);
}

/// `-[ApocalypseGemLogic activate]` @ 0x10000edbc
pub fn activate(world: &mut World, this: Entity) {
    let gem = world.obj::<ApocalypseGemLogic>(this);
    if gem.falling || gem.active {
        return;
    }

    world.obj_mut::<ApocalypseGemLogic>(this).active = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<ApocalypseGemLogic>(this).in_water = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<ApocalypseGemLogic>(this).enable_island_floor_collision = false;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;

    // A random x on the island, 15 in from each end, in tenths: `arc4random() % range` (u32).
    let listener = gem_logic_listener(world, this);
    let mut island_min_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
    }
    let start_tenths = ((island_min_x_pos + 15.0) * 10.0) as i32;
    let random = world.resource_mut::<Random>().arc4random();

    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
    }
    let end_tenths = ((island_max_x_pos + -15.0) * 10.0) as i32;

    let mut island_min_x_pos_again = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos_again = crate::controller::island_min_x_pos(world, listener);
    }
    let range_start_tenths = ((island_min_x_pos_again + 15.0) * 10.0) as i32;

    let range = (end_tenths.wrapping_add(1) as u32).wrapping_sub(range_start_tenths as u32);
    let offset = if range == 0 { random } else { random % range };
    let x_tenths = offset.wrapping_add(start_tenths as u32) as i32;

    {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = x_tenths as f32 / 10.0;
        display.pos.y = 400.0;
        display.pos.z = -277.12;
    }
    world.obj_mut::<ApocalypseGemLogic>(this).dont_hit_until_bounce = true;
    {
        let mut display = display::object_mut(world, display_object);
        display.scale.x = 1.0;
        display.is_selectable = true;
    }

    insert_before_pygmy_display_marker(world, this);
    logic::set_behavior(world, this, Some("ApocalypseGemEnterIsland"));
}

/// `-[ApocalypseGemLogic deactivate]` @ 0x10000efe8
pub fn deactivate(world: &mut World, this: Entity) {
    let active = world.obj::<ApocalypseGemLogic>(this).active;
    if !active {
        return;
    }

    world.obj_mut::<ApocalypseGemLogic>(this).active = false;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;

    leave_apocalypse_gem_pedestal(world, this);

    let dragging = world.obj::<ApocalypseGemLogic>(this).dragging;
    if dragging {
        {
            let mut gem = world.obj_mut::<ApocalypseGemLogic>(this);
            gem.falling = true;
            gem.dragging = false;
            gem.apply_spring = false;
        }
        send_drag_complete(world, this);
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<ApocalypseGemLogic>(this).enable_island_floor_collision = false;

    let in_water = world.obj::<ApocalypseGemLogic>(this).in_water;
    if !in_water {
        insert_after_pygmy_drag_display_marker(world, this);
        logic::set_behavior(world, this, Some("ApocalypseGemFallIgnoreIsland"));
    }
}

/// `-[ApocalypseGemLogic refreshTiltState]` @ 0x10000f1b8
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let gem = world.obj::<ApocalypseGemLogic>(this);
    if gem.dragging || gem.falling || !gem.on_island {
        return;
    }

    let tilt_state = throwable_tilt_state(world);
    let current_tilt_state = world.obj::<ApocalypseGemLogic>(this).tilt_state;
    if tilt_state != current_tilt_state {
        if tilt_state == 0 {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            logic::set_behavior(world, this, Some("ApocalypseGemStopSlide"));
            world.obj_mut::<ApocalypseGemLogic>(this).tilt_state = 0;
        } else if tilt_state == 1 {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            logic::set_behavior(world, this, Some("ApocalypseGemTiltSlide"));
            world.obj_mut::<ApocalypseGemLogic>(this).tilt_state = 1;
        }
    }

    if is_island_tipped_over(world) {
        {
            let mut gem = world.obj_mut::<ApocalypseGemLogic>(this);
            gem.falling = true;
            gem.tilt_state = 0;
        }
        logic::set_behavior(world, this, Some("ApocalypseGemFall"));
    }
}

/// `-[ApocalypseGemLogic undoTiltState]` @ 0x10000f398
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    let gem = world.obj::<ApocalypseGemLogic>(this);
    if gem.dragging || gem.falling || !gem.on_island {
        return;
    }

    if gem.tilt_state == 1 {
        logic::logic_mut(world, this).pos_vel.x = 0.0;
        logic::set_behavior(world, this, Some("ApocalypseGemStopSlide"));
    }
    world.obj_mut::<ApocalypseGemLogic>(this).tilt_state = 0;
}

/// `-[ApocalypseGemLogic isAccelEnabled]` @ 0x10000f42c
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let gravity_enabled = match gem_logic_listener(world, this) {
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

/// `-[ApocalypseGemLogic onGameFrame:]` @ 0x10000f4c0
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let display_object = logic::display_object(world, this);

    let accel_enabled = is_accel_enabled(world, this);
    if accel_enabled {
        let gravity_force = throwable_tilted_gravity_force(world);
        world.obj_mut::<SpriteLogic>(this).gravity_force = gravity_force;
        refresh_tilt_state(world, this);
    } else {
        world.obj_mut::<SpriteLogic>(this).gravity_force = THROWABLE_GRAVITY_FORCE;
        undo_tilt_state(world, this);
    }

    let gem = world.obj::<ApocalypseGemLogic>(this);
    let tilt_state = gem.tilt_state;
    let slide_friction = gem.slide_friction;
    let spring_force = gem.spring_force;
    let apply_spring = gem.apply_spring;

    let mut sprite = world.obj_mut::<SpriteLogic>(this);
    apply_throwable_slide_friction(&mut sprite.gravity_force, tilt_state, slide_friction);
    let gravity_force = sprite.gravity_force;
    let ignore_gravity = sprite.ignore_gravity;

    let pos_force = throwable_pos_force(gravity_force, ignore_gravity, spring_force, apply_spring);
    logic::logic_mut(world, this).pos_force = pos_force;
    integrate_throwable(world, this, pos_force, dt);

    world.obj_mut::<ApocalypseGemLogic>(this).island_floor_collision = false;
    let listener = gem_logic_listener(world, this);
    let mut island_min_x_pos = 0.0;
    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
        island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
    }

    let pos = display::object(world, display_object).pos;
    let gem = world.obj::<ApocalypseGemLogic>(this);
    let within_island = pos.x >= island_min_x_pos && pos.x <= island_max_x_pos;
    let reached_floor = within_island && gem.last_pos.y >= FLOOR_Y && pos.y < FLOOR_Y;
    let enable_island_floor_collision = gem.enable_island_floor_collision;

    let mut y_pos = pos.y;
    if reached_floor && enable_island_floor_collision {
        let on_island = world.obj::<ApocalypseGemLogic>(this).on_island;
        {
            let mut state = logic::logic_mut(world, this);
            if on_island {
                state.pos_vel.y = 0.0;
            } else {
                state.pos_vel.x = state.pos_vel.x * 0.5;
                state.pos_vel.y = state.pos_vel.y * -0.4;
            }
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let settled = pos_vel.x.abs() < 30.0 && pos_vel.y < 30.0;
        if settled {
            logic::logic_mut(world, this).pos_vel.y = 0.0;
            world.obj_mut::<ApocalypseGemLogic>(this).dont_hit_until_bounce = false;
            if !on_island {
                world.obj_mut::<ApocalypseGemLogic>(this).on_island = true;
                logic::logic_mut(world, this).pos_vel.x = 0.0;
                world.obj_mut::<ApocalypseGemLogic>(this).island_floor_collision = true;
            }
        } else if pos_vel.y > 100.0 {
            logic::set_behavior(world, this, Some("ApocalypseGemBounce"));
        }

        {
            let mut display = display::object_mut(world, display_object);
            display.rot.z = 0.0;
            display.pos.y = FLOOR_Y;
        }
        y_pos = FLOOR_Y;
    } else {
        world.obj_mut::<ApocalypseGemLogic>(this).on_island = false;
    }

    if y_pos < -100.0 || y_pos > 820.0 {
        let idol_manager = crate::managers::idol_manager::instance(world);
        if let Some(idol_manager) = idol_manager {
            crate::managers::idol_manager::trigger_episode_event(world, idol_manager, "ThrowGemAway");
        }
        logic::set_behavior(world, this, Some("ApocalypseGemDie"));
    }

    let last_pos = display::object(world, display_object).pos;
    world.obj_mut::<ApocalypseGemLogic>(this).last_pos = last_pos;
}

/// `-[ApocalypseGemLogic slide:::]` @ 0x10000f9b4
pub fn seq_slide(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<ApocalypseGemLogic>(this).enable_island_floor_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }
    ItemResult::Pending
}

/// `-[ApocalypseGemLogic setAnimationBasedOnGemType:::]` @ 0x10000f9e0
pub fn seq_set_animation_based_on_gem_type(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let gem_type = world.obj::<ApocalypseGemLogic>(this).apocalypse_gem_type;
    let animation_id = gem_type_animation_id(call, gem_type);

    let animation = logic::animation(world, this);
    pg_engine::animation::play_animation_id(world, animation, animation_id);
    ItemResult::Done
}

/// The `setAnimationBasedOnGemType:::` parameter for gem type `gem_type` (`gem1` … `gem7`; type
/// 0 counts as 1, other types are nil). Shared with `ApocalypseGemPedestalLogic`.
pub(crate) fn gem_type_animation_id<'a>(call: &'a SequenceCall, gem_type: i32) -> Option<&'a str> {
    match gem_type {
        0 | 1 => call.param("gem1"),
        2 => call.param("gem2"),
        3 => call.param("gem3"),
        4 => call.param("gem4"),
        5 => call.param("gem5"),
        6 => call.param("gem6"),
        7 => call.param("gem7"),
        _ => None,
    }
}

/// `-[ApocalypseGemLogic resetLastPos:::]` @ 0x10000fab4
pub fn seq_reset_last_pos(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    world.obj_mut::<ApocalypseGemLogic>(this).last_pos = pos;
    ItemResult::Done
}

/// `-[ApocalypseGemLogic drag:::]` @ 0x10000fb04
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<ApocalypseGemLogic>(this).enable_island_floor_collision = false;
    }

    let display_object = logic::display_object(world, this);
    let touch_screen_pos = world.obj::<ApocalypseGemLogic>(this).touch_screen_pos;
    let offset = throwable_drag_offset(world, display_object, touch_screen_pos);
    let length = length3(offset);
    if length != 0.0 {
        if offset.y != 0.0 {
            let z_rot = throwable_drag_z_rot(offset);
            display::object_mut(world, display_object).rot.z = z_rot;
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let spring_force = throwable_spring_force(offset, length, pos_vel, DRAG_STIFFNESS, DRAG_DAMPING);
        world.obj_mut::<ApocalypseGemLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[ApocalypseGemLogic fall:::]` @ 0x10000fcb4
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut gem = world.obj_mut::<ApocalypseGemLogic>(this);
            if !gem.island_floor_collision {
                gem.on_island = false;
            }
            gem.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<ApocalypseGemLogic>(this).falling = true;

        let ignore_island = call.bool("ignoreIsland");
        if ignore_island {
            world.obj_mut::<ApocalypseGemLogic>(this).enable_island_floor_collision = false;
        }
    }

    let landed_in_pedestal = check_for_apocalypse_gem_pedestal_during_fall(world, this);
    if landed_in_pedestal {
        world.obj_mut::<ApocalypseGemLogic>(this).falling = false;
        return ItemResult::Replaced;
    }

    let display_object = logic::display_object(world, this);
    let island_floor_collision = world.obj::<ApocalypseGemLogic>(this).island_floor_collision;
    if island_floor_collision {
        world.obj_mut::<ApocalypseGemLogic>(this).falling = false;
        return ItemResult::Done;
    }

    let ocean_enabled = match gem_logic_listener(world, this) {
        Some(listener) => crate::controller::is_ocean_enabled(world, listener),
        None => false,
    };
    let y_pos = display::object(world, display_object).pos.y;
    let in_water = ocean_enabled && y_pos < WATER_LINE_Y;
    if !in_water {
        return ItemResult::Pending;
    }

    {
        let mut gem = world.obj_mut::<ApocalypseGemLogic>(this);
        gem.falling = false;
        gem.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    display::object_mut(world, display_object).rot.z = 0.0;
    logic::set_behavior(world, this, Some("ApocalypseGemSplash"));

    let idol_manager = crate::managers::idol_manager::instance(world);
    if let Some(idol_manager) = idol_manager {
        crate::managers::idol_manager::trigger_episode_event(world, idol_manager, "ThrowGemAway");
    }
    ItemResult::Replaced
}

/// `-[ApocalypseGemLogic checkRespawn:::]` @ 0x10000fe9c
pub fn seq_check_respawn(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    ItemResult::Done
}

/// `-[ApocalypseGemLogic die:::]` @ 0x10000fea4
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);

    let dragging = world.obj::<ApocalypseGemLogic>(this).dragging;
    if dragging {
        world.obj_mut::<ApocalypseGemLogic>(this).dragging = false;
        send_drag_complete(world, this);
    }

    leave_apocalypse_gem_pedestal(world, this);

    {
        let mut gem = world.obj_mut::<ApocalypseGemLogic>(this);
        gem.active = false;
        gem.falling = false;
        gem.in_water = false;
        gem.apply_spring = false;
        gem.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    if let Some(listener) = gem_logic_listener(world, this) {
        crate::controller::on_apocalypse_gem_die(world, listener, this);
    }
    ItemResult::Done
}

/// `-[ApocalypseGemLogic touchBeganInside:allTouches:]` @ 0x10000ffd8
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = gem_logic_listener(world, this);
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };
    if story_active {
        return;
    }

    // A tap on a seated gem takes it out of its pedestal.
    leave_apocalypse_gem_pedestal(world, this);

    let gem = world.obj::<ApocalypseGemLogic>(this);
    if gem.falling || gem.in_water || gem.active_touch.is_some() {
        return;
    }

    world.obj_mut::<ApocalypseGemLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut gem = world.obj_mut::<ApocalypseGemLogic>(this);
        gem.touch_screen_pos = touch_pos;
        gem.tilt_state = 0;
        gem.dont_hit_until_bounce = false;
        gem.dragging = true;
        gem.apply_spring = true;
        gem.enable_island_floor_collision = false;
    }

    if let Some(listener) = listener {
        crate::controller::on_apocalypse_gem_drag(world, listener, this);
    }
    insert_after_pygmy_drag_display_marker(world, this);
    logic::set_behavior(world, this, Some("ApocalypseGemDrag"));
}

/// `-[ApocalypseGemLogic touchBeganOutside:allTouches:]` @ 0x10001017c
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<ApocalypseGemLogic>(this).active_touch;
    if active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<ApocalypseGemLogic>(this).touch_screen_pos = touch_pos;
}

/// `-[ApocalypseGemLogic touchMovedEntered:allTouches:]` @ 0x1000101cc
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<ApocalypseGemLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<ApocalypseGemLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[ApocalypseGemLogic touchMovedExited:allTouches:]` @ 0x100010224
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<ApocalypseGemLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<ApocalypseGemLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[ApocalypseGemLogic touchMovedInside:allTouches:]` @ 0x10001027c
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[ApocalypseGemLogic touchMovedOutside:allTouches:]` @ 0x10001028c
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<ApocalypseGemLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<ApocalypseGemLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[ApocalypseGemLogic touchEndedInside:allTouches:]` @ 0x1000102e4
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<ApocalypseGemLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let dragging = world.obj::<ApocalypseGemLogic>(this).dragging;
    if dragging {
        {
            let mut gem = world.obj_mut::<ApocalypseGemLogic>(this);
            gem.falling = true;
            gem.dragging = false;
            gem.apply_spring = false;
        }
        insert_before_pygmy_display_marker(world, this);
        send_drag_complete(world, this);
        logic::set_behavior(world, this, Some("ApocalypseGemFall"));
    }

    world.obj_mut::<ApocalypseGemLogic>(this).active_touch = None;
}

/// `-[ApocalypseGemLogic touchEndedOutside:allTouches:]` @ 0x1000103c8
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
