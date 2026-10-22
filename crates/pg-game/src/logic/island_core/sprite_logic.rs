//! `SpriteLogic` (superclass `BCLogic`).

use crate::prelude::*;

use crate::logic::apocalypse_temples::apocalypse_cloud_logic;
use pg_engine::display;
use pg_engine::logic;
use pg_engine::view::BcView;

/// `SpriteLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct SpriteLogic {
    /// `mGravityForce`: `{?="x"f"y"f"z"f"w"f}`
    pub gravity_force: Vec4,

    /// `mbIgnoreGravity`: `B`
    pub ignore_gravity: bool,

    /// `mbBlownByHurricane`: `B`
    pub blown_by_hurricane: bool,

    /// `mbPickedUpByHurricane`: `B`
    pub picked_up_by_hurricane: bool,

    /// `mPickedUpByHurricaneLogic`: `@"HurricaneLogic"`
    pub picked_up_by_hurricane_logic: Option<Entity>,

    /// `mHurricaneSpinState`: `{HurricaneSpinState="mStall"i"mRadius"f"mRadiusRot"f"mRadiusRotVel"f"mYPos"f"mYPosRot"f"mYPosRotVel"f"mAngle"f"mAngleVel"f"mStrength"f"mStrengthVel"f}`
    pub hurricane_spin_state: HurricaneSpinState,

    /// `mbBlownByWhirlPool`: `B`
    pub blown_by_whirl_pool: bool,

    /// `mbPickedUpByWhirlPool`: `B`
    pub picked_up_by_whirl_pool: bool,

    /// `mPickedUpByWhirlPoolLogic`: `@"WhirlPoolLogic"`
    pub picked_up_by_whirl_pool_logic: Option<Entity>,

    /// `mWhirlPoolSpinState`: `{WhirlPoolSpinState="mStall"i"mRadius"f"mTargetRadius"f"mRadiusRot"f"mRadiusRotVel"f"mAngle"f"mAngleVel"f"mStrength"f"mStrengthVel"f}`
    pub whirl_pool_spin_state: WhirlPoolSpinState,

    /// `mApocalypseCloudStyleId`: `@"NSString"`
    pub apocalypse_cloud_style_id: Option<String>,

    /// `mApocalypseCloudLogic`: `@"ApocalypseCloudLogic"`
    pub apocalypse_cloud_logic: Option<Entity>,

    /// `mApocalypseDiseaseType`: `i`
    pub apocalypse_disease_type: i32,
}

pub const SPRITE_LOGIC: Class = Class {
    name: "SpriteLogic",
    superclass: Some(&pg_engine::logic::BC_LOGIC),
    sequence_item,
    on_game_frame,
    ..pg_engine::logic::BC_LOGIC
};

/// `mGravityForce` as `initWithDisplayObject:` sets it: `(0, -3000, 0, 1)`.
const INITIAL_GRAVITY_FORCE: Vec4 = Vec4::new(0.0, -3000.0, 0.0, 1.0);

/// Sequence-item handlers implemented by `SpriteLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "triggerEpisodeEvent" => seq_trigger_episode_event(world, this, call),
        "setApocalypseCloudDragOffset" => seq_set_apocalypse_cloud_drag_offset(world, this, call),
        _ => return (pg_engine::logic::BC_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[SpriteLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&SPRITE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[SpriteLogic initWithDisplayObject:]` @ 0x100199178
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(SpriteLogic::default());
    logic::init_with_display_object(world, this, display_object);

    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.ignore_gravity = true;
        sprite.apocalypse_cloud_style_id = Some(String::from("Default"));
        sprite.gravity_force = INITIAL_GRAVITY_FORCE;
    }

    logic::start_game_frame(world, this);
}

/// `-[SpriteLogic setApocalypseCloudStyleId:]` @ 0x100199288
pub fn set_apocalypse_cloud_style_id(world: &mut World, this: Entity, apocalypse_cloud_style_id: &str) {
    let style_id = apocalypse_cloud_style_id.to_owned();
    world.obj_mut::<SpriteLogic>(this).apocalypse_cloud_style_id = Some(style_id);
}

/// `-[SpriteLogic isBlownByHurricane]` @ 0x1001992e4
pub fn is_blown_by_hurricane(world: &mut World, this: Entity) -> bool {
    world.obj::<SpriteLogic>(this).blown_by_hurricane
}

/// `-[SpriteLogic isPickedUpByHurricane]` @ 0x1001992f4
pub fn is_picked_up_by_hurricane(world: &mut World, this: Entity) -> bool {
    world.obj::<SpriteLogic>(this).picked_up_by_hurricane
}

/// `-[SpriteLogic blownByHurricane:]` @ 0x100199304
pub fn blown_by_hurricane(_world: &mut World, _this: Entity, _blown_by_hurricane: Entity) -> bool {
    false
}

/// `-[SpriteLogic unblownByHurricane:]` @ 0x10019930c
pub fn unblown_by_hurricane(_world: &mut World, _this: Entity, _unblown_by_hurricane: Entity) {}

/// `-[SpriteLogic pickedUpByHurricane:]` @ 0x100199310
pub fn picked_up_by_hurricane(_world: &mut World, _this: Entity, _picked_up_by_hurricane: Entity) -> bool {
    false
}

/// `-[SpriteLogic droppedByHurricane:]` @ 0x100199318
pub fn dropped_by_hurricane(_world: &mut World, _this: Entity, _dropped_by_hurricane: Entity) {}

/// `-[SpriteLogic hurricaneSpinState]` @ 0x10019931c
///
/// The original returns a pointer into the ivars, which the hurricane updates in place; the
/// port hands out the ivar itself.
pub fn hurricane_spin_state(world: &mut World, this: Entity) -> Mut<'_, HurricaneSpinState> {
    let sprite = world.obj_mut::<SpriteLogic>(this);
    sprite.map_unchanged(|sprite| &mut sprite.hurricane_spin_state)
}

/// `-[SpriteLogic isPickedUpByWhirlPool]` @ 0x10019932c
pub fn is_picked_up_by_whirl_pool(world: &mut World, this: Entity) -> bool {
    world.obj::<SpriteLogic>(this).picked_up_by_whirl_pool
}

/// `-[SpriteLogic pickedUpByWhirlPool:]` @ 0x10019933c
pub fn picked_up_by_whirl_pool(_world: &mut World, _this: Entity, _picked_up_by_whirl_pool: Entity) -> bool {
    false
}

/// `-[SpriteLogic droppedByWhirlPool:]` @ 0x100199344
pub fn dropped_by_whirl_pool(_world: &mut World, _this: Entity, _dropped_by_whirl_pool: Entity) {}

/// `-[SpriteLogic whirlPoolSpinState]` @ 0x100199348
///
/// The original returns a pointer into the ivars, which the whirlpool updates in place; the
/// port hands out the ivar itself.
pub fn whirl_pool_spin_state(world: &mut World, this: Entity) -> Mut<'_, WhirlPoolSpinState> {
    let sprite = world.obj_mut::<SpriteLogic>(this);
    sprite.map_unchanged(|sprite| &mut sprite.whirl_pool_spin_state)
}

/// `-[SpriteLogic isAccelEnabled]` @ 0x100199358
pub fn is_accel_enabled(world: &mut World, _this: Entity) -> bool {
    let view = world.resource::<BcView>();

    let raw_rot = view.raw_accel_rot_degrees;
    let x_lying_flat = raw_rot.x < -80.0 && raw_rot.x > -100.0;
    let y_lying_flat = raw_rot.y < -80.0 && raw_rot.y > -100.0;
    if x_lying_flat && y_lying_flat {
        return false;
    }

    let raw_force = view.raw_accel_force;
    raw_force.x != 0.0 && raw_force.y != 0.0 && raw_force.z != 0.0
}

/// `-[SpriteLogic onGameFrame:]` @ 0x100199420
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;
    let pos_vel = state.pos_vel;

    let sprite = world.obj::<SpriteLogic>(this);
    let mut pos_force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    if !sprite.ignore_gravity {
        let gravity_force = sprite.gravity_force;
        pos_force.x = gravity_force.x + pos_force.x;
        pos_force.y = gravity_force.y + pos_force.y;
        pos_force.z = gravity_force.z + pos_force.z;
    }
    logic::logic_mut(world, this).pos_force = pos_force;

    // Explicit Euler: the position moves with the old velocity.
    {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = display.pos.x + pos_vel.x * dt;
        display.pos.y = display.pos.y + pos_vel.y * dt;
        display.pos.z = display.pos.z + pos_vel.z * dt;
    }

    let mut state = logic::logic_mut(world, this);
    state.pos_vel.x = pos_vel.x + pos_force.x * dt;
    state.pos_vel.y = pos_vel.y + pos_force.y * dt;
    state.pos_vel.z = pos_vel.z + pos_force.z * dt;
}

/// `-[SpriteLogic triggerEpisodeEvent:::]` @ 0x100199570
pub fn seq_trigger_episode_event(world: &mut World, _this: Entity, call: &mut SequenceCall) -> ItemResult {
    // Every shipped item names its event; "" stands in for a nil one.
    let event = call.param("event").unwrap_or("");
    let idol_manager = crate::managers::idol_manager::instance(world);
    if let Some(idol_manager) = idol_manager {
        crate::managers::idol_manager::trigger_episode_event(world, idol_manager, event);
    }
    ItemResult::Done
}

/// `-[SpriteLogic setApocalypseCloudDragOffset:::]` @ 0x1001995c0
pub fn seq_set_apocalypse_cloud_drag_offset(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let cloud = world.obj::<SpriteLogic>(this).apocalypse_cloud_logic;
    if let Some(cloud) = cloud {
        let drag_x_offset = call.float("dragXOffset");
        let drag_y_offset = call.float("dragYOffset");
        apocalypse_cloud_logic::set_drag_x_offset(world, cloud, drag_x_offset, drag_y_offset);
    }
    ItemResult::Done
}

/// `-[SpriteLogic apocalypseDiseaseType]` @ 0x100199650
pub fn apocalypse_disease_type(world: &mut World, this: Entity) -> i32 {
    world.obj::<SpriteLogic>(this).apocalypse_disease_type
}

/// `-[SpriteLogic hideApocalypseCloud]` @ 0x100199660
pub fn hide_apocalypse_cloud(world: &mut World, this: Entity) {
    let cloud = world.obj::<SpriteLogic>(this).apocalypse_cloud_logic;
    let Some(cloud) = cloud else {
        return;
    };

    let cloud_display_object = logic::display_object(world, cloud);
    display::remove_from_display_group(world, cloud_display_object);
}

/// `-[SpriteLogic showApocalypseCloud]` @ 0x1001996a4
pub fn show_apocalypse_cloud(world: &mut World, this: Entity) {
    let cloud = world.obj::<SpriteLogic>(this).apocalypse_cloud_logic;
    let Some(cloud) = cloud else {
        return;
    };

    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    let cloud_display_object = logic::display_object(world, cloud);
    display::object_mut(world, cloud_display_object).pos = pos;

    display::insert_after(world, display_object, Some(cloud_display_object));
}

/// `-[SpriteLogic cureDisease]` @ 0x100199768
pub fn cure_disease(world: &mut World, this: Entity) {
    world.obj_mut::<SpriteLogic>(this).apocalypse_disease_type = 0;

    let cloud = world.obj::<SpriteLogic>(this).apocalypse_cloud_logic;
    if let Some(cloud) = cloud {
        apocalypse_cloud_logic::deactivate(world, cloud);

        // `[cloud release]`: only this sprite's reference ends; the cloud tears itself down
        // when its deactivation completes.
        world.obj_mut::<SpriteLogic>(this).apocalypse_cloud_logic = None;
    }
}

/// `-[SpriteLogic infectWithDisease:]` @ 0x1001997c8
pub fn infect_with_disease(world: &mut World, this: Entity, infect_with_disease: i32) {
    let cloud = world.obj::<SpriteLogic>(this).apocalypse_cloud_logic;

    if cloud.is_none() && infect_with_disease != 0 {
        world.obj_mut::<SpriteLogic>(this).apocalypse_disease_type = infect_with_disease;

        let style_id = world.obj::<SpriteLogic>(this).apocalypse_cloud_style_id.clone();
        let style_id = style_id.unwrap_or_default();
        let created = apocalypse_cloud_logic::class_apocalypse_cloud_with_sprite(world, this, infect_with_disease, &style_id);
        world.obj_mut::<SpriteLogic>(this).apocalypse_cloud_logic = created;
        return;
    }

    let Some(cloud) = cloud else {
        return;
    };

    if infect_with_disease != 0 {
        world.obj_mut::<SpriteLogic>(this).apocalypse_disease_type = infect_with_disease;
        apocalypse_cloud_logic::set_apocalypse_disease_type(world, cloud, infect_with_disease);
        return;
    }

    cure_disease(world, this);
}
