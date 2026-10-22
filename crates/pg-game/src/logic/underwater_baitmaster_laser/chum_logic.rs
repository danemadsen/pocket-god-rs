//! `ChumLogic` (superclass `SpriteLogic`).
//!
//! The chum spilled from the bucket: it sinks, can be dragged, held by a pygmy and eaten by
//! the laser shark, and fades away on the sea floor. Its listener is the `ChumBucketLogic`.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::underwater_baitmaster_laser::chum_bucket_logic;
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::{length3, rad_to_deg};
use pg_engine::touch::BcTouch;

/// `ChumLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ChumLogic {
    /// `mChumLogicListener`: `@"<ChumLogicListener>"`
    pub chum_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbFade`: `B`
    pub fade: bool,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,

    /// `mHoldingPygmyLogic`: `@"PygmyLogic"`
    pub holding_pygmy_logic: Option<Entity>,
}

pub const CHUM_LOGIC: Class = Class {
    name: "ChumLogic",
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

/// The sea floor the chum settles on.
const FLOOR_Y_POS: f32 = 30.0;

/// Gravity while the chum sinks freely.
const SINK_GRAVITY_Y: f32 = -15.0;

/// Gravity while the chum is dragged.
const DRAG_GRAVITY_Y: f32 = -3000.0;

/// Sequence-item handlers implemented by `ChumLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "drag" => seq_drag(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[ChumLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&CHUM_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[ChumLogic initWithDisplayObject:]` @ 0x10008e1c8
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ChumLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
    logic::set_behavior(world, this, Some("ChumInit"));

    let mut object = display::object_mut(world, own_display_object);
    object.is_selectable = true;
    object.origin_pos.x = -20.0;
    object.origin_pos.y = -10.0;
    object.col_x_min = -10.0;
    object.col_x_max = 50.0;
    object.col_y_min = -10.0;
    object.col_y_max = 30.0;
}

/// `-[ChumLogic setChumLogicListener:]` @ 0x10008e31c
pub fn set_chum_logic_listener(world: &mut World, this: Entity, chum_logic_listener: Entity) {
    world.obj_mut::<ChumLogic>(this).chum_logic_listener = Some(chum_logic_listener);
}

/// `-[ChumLogic enterIsland]` @ 0x10008e378
pub fn enter_island(world: &mut World, this: Entity) {
    let active = world.obj::<ChumLogic>(this).active;
    if active {
        return;
    }
    world.obj_mut::<ChumLogic>(this).active = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let display_object = logic::display_object(world, this);
    let marker = pg_engine::library::display_marker(world, "TsunamiDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[ChumLogic exitIsland]` @ 0x10008e414
pub fn exit_island(world: &mut World, this: Entity) {
    let active = world.obj::<ChumLogic>(this).active;
    if !active {
        return;
    }
    world.obj_mut::<ChumLogic>(this).active = false;
    release_from_pygmy(world, this);

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `[mHoldingPygmyLogic dropChum:self]; mHoldingPygmyLogic = nil` when a pygmy holds the chum.
fn release_from_pygmy(world: &mut World, this: Entity) {
    let holding_pygmy_logic = world.obj::<ChumLogic>(this).holding_pygmy_logic;
    if let Some(holding_pygmy_logic) = holding_pygmy_logic {
        crate::pygmy::drop_chum(world, holding_pygmy_logic, this);
        world.obj_mut::<ChumLogic>(this).holding_pygmy_logic = None;
    }
}

/// `-[ChumLogic holdingPygmyLogic]` @ 0x10008e494
pub fn holding_pygmy_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<ChumLogic>(this).holding_pygmy_logic
}

/// `-[ChumLogic isPygmyHoldingChum]` @ 0x10008e4a4
pub fn is_pygmy_holding_chum(world: &mut World, this: Entity) -> bool {
    world.obj::<ChumLogic>(this).holding_pygmy_logic.is_some()
}

/// `-[ChumLogic canBeFedToShark]` @ 0x10008e4bc
///
/// Not while the finger holds it.
pub fn can_be_fed_to_shark(world: &mut World, this: Entity) -> bool {
    world.obj::<ChumLogic>(this).active_touch.is_none()
}

/// `-[ChumLogic eatenByShark]` @ 0x10008e4d4
pub fn eaten_by_shark(world: &mut World, this: Entity) {
    release_from_pygmy(world, this);
    logic::set_behavior(world, this, Some("ChumEaten"));
}

/// `-[ChumLogic dropChum:]` @ 0x10008e530
///
/// The pygmy holding the chum lets it go.
pub fn drop_chum(world: &mut World, this: Entity, _drop_chum: Entity) {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<ChumLogic>(this).holding_pygmy_logic = None;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("ChumInit"));
}

/// `-[ChumLogic onGameFrame:]` @ 0x10008e5a4
///
/// Sinks (pulled by the finger while dragged), levels out when released, stays on screen and
/// fades away once it lies on the sea floor untouched.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;

    let (active_touch, apply_spring, spring_force) = {
        let chum = world.obj::<ChumLogic>(this);
        (chum.active_touch, chum.apply_spring, chum.spring_force)
    };

    let gravity_y = if active_touch.is_none() { SINK_GRAVITY_Y } else { DRAG_GRAVITY_Y };
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

    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = object.pos.x + pos_vel.x * dt;
        object.pos.y = object.pos.y + pos_vel.y * dt;
        object.pos.z = object.pos.z + pos_vel.z * dt;
        if active_touch.is_none() {
            object.rot.z = object.rot.z + object.rot.z * -0.5;
        }
    }

    let min_pos = display::min_pos(world, display_object);
    let max_pos = display::max_pos(world, display_object);
    if min_pos.x < 0.0 {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = (object.pos.x - min_pos.x) + 0.0;
    }
    if max_pos.x > 480.0 {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = 480.0 - (max_pos.x - object.pos.x);
    }

    let y_pos = display::object(world, display_object).pos.y;
    if y_pos < FLOOR_Y_POS {
        logic::logic_mut(world, this).pos_vel.y = 0.0;
        display::object_mut(world, display_object).pos.y = FLOOR_Y_POS;

        let fade = world.obj::<ChumLogic>(this).fade;
        if active_touch.is_none() && !fade {
            world.obj_mut::<ChumLogic>(this).fade = true;
            logic::set_behavior(world, this, Some("ChumFade"));
            display::object_mut(world, display_object).is_selectable = false;
        }
    }
}

/// `-[ChumLogic drag:::]` @ 0x10008e8dc
///
/// A damped spring toward the finger (stiffness 100, damping 8), the chum hanging from it.
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        logic::logic_mut(world, this).rot_vel.z = 0.0;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let display_object = logic::display_object(world, this);
    let (pos, display_group) = {
        let object = display::object(world, display_object);
        (object.pos, object.display_group)
    };
    let touch_screen_pos = world.obj::<ChumLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    let offset = Vec4::new(pos.x - touch_pos.x, pos.y - touch_pos.y, pos.z - touch_pos.z, pos.w);
    let distance = length3(offset);
    if distance != 0.0 {
        if offset.y != 0.0 {
            let angle = (-offset.y).atan2(-offset.x);
            let z_rot = rad_to_deg(angle) - 90.0;
            display::object_mut(world, display_object).rot.z = z_rot;
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let stiffness = (distance * -100.0) / distance;
        let spring_force = Vec4::new(
            -8.0 * pos_vel.x + stiffness * offset.x,
            -8.0 * pos_vel.y + stiffness * offset.y,
            -8.0 * pos_vel.z + stiffness * offset.z,
            offset.w,
        );
        world.obj_mut::<ChumLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[ChumLogic die:::]` @ 0x10008ea94
///
/// The bucket releases the chum in `onChumDie:`; nothing may touch `this` afterwards.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    release_from_pygmy(world, this);

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);

    let listener = world.obj::<ChumLogic>(this).chum_logic_listener;
    if let Some(listener) = listener {
        chum_bucket_logic::on_chum_die(world, listener, this);
    }
    ItemResult::Done
}

/// `-[ChumLogic touchBeganInside:allTouches:]` @ 0x10008eb80
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut chum = world.obj_mut::<ChumLogic>(this);
        chum.touch_screen_pos = touch_pos;
        chum.apply_spring = true;
        chum.active_touch = Some(touch);
    }
    logic::set_behavior(world, this, Some("ChumDrag"));
}

/// `-[ChumLogic touchBeganOutside:allTouches:]` @ 0x10008ec14
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ChumLogic touchMovedEntered:allTouches:]` @ 0x10008ec18
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ChumLogic touchMovedExited:allTouches:]` @ 0x10008ec1c
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ChumLogic touchMovedInside:allTouches:]` @ 0x10008ec20
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[ChumLogic touchMovedOutside:allTouches:]` @ 0x10008ec30
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<ChumLogic>(this).active_touch;
    let touch_is_ours = active_touch.is_none() || active_touch == Some(touch);
    if !touch_is_ours {
        return;
    }
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<ChumLogic>(this).touch_screen_pos = touch_pos;
}

/// `-[ChumLogic touchEndedInside:allTouches:]` @ 0x10008ec88
///
/// Released within 30 of a pygmy's hands (31 above its position) that can hold it, the chum is
/// handed to that pygmy.
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<ChumLogic>(this).active_touch;
    if active_touch.is_none() || active_touch != Some(touch) {
        return;
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    let listener = {
        let mut chum = world.obj_mut::<ChumLogic>(this);
        chum.active_touch = None;
        chum.apply_spring = false;
        chum.chum_logic_listener
    };

    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;

    let pygmies = match listener {
        Some(listener) => chum_bucket_logic::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmies {
        if !crate::pygmy::can_hold_chum(world, pygmy, this) {
            continue;
        }

        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let offset_x = pos.x - (pygmy_pos.x + 0.0);
        let offset_y = pos.y - (pygmy_pos.y + 31.0);
        if offset_x * offset_x + offset_y * offset_y < 900.0 {
            world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
            world.obj_mut::<ChumLogic>(this).holding_pygmy_logic = Some(pygmy);
            display::object_mut(world, display_object).is_selectable = false;
            logic::set_behavior(world, this, Some("ChumHold"));
            crate::pygmy::hold_chum(world, pygmy, this);
            return;
        }
    }
}

/// `-[ChumLogic touchEndedOutside:allTouches:]` @ 0x10008ef00
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
