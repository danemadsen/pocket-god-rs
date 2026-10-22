//! `LaserTargetLogic` (superclass `SpriteLogic`).
//!
//! The laser shark's crosshair, steered by tilting the device. Its listener (the
//! `ChumBucketLogic`) is never called.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::logic;
use pg_engine::view::BcView;

/// `LaserTargetLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct LaserTargetLogic {
    /// `mLaserTargetLogicListener`: `@"<LaserTargetLogicListener>"`
    pub laser_target_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,
}

pub const LASER_TARGET_LOGIC: Class = Class {
    name: "LaserTargetLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Speed of the unused ping-pong motion of `updateTargetPos1`, per frame.
const PING_PONG_SPEED: f32 = 4.0;

/// The span of the tilt range that maps onto the target's height: `−0.5 − (−0.9)` as the
/// binary stores it (`0x3ecccccc`, one ulp below `0.4`).
const TILT_X_SPAN: f32 = f32::from_bits(0x3ecc_cccc);

/// `[[LaserTargetLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&LASER_TARGET_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[LaserTargetLogic initWithDisplayObject:]` @ 0x10008f9e8
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(LaserTargetLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::logic_mut(world, this).pos_vel.x = PING_PONG_SPEED;

    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
}

/// `-[LaserTargetLogic setLaserTargetLogicListener:]` @ 0x10008fa6c
pub fn set_laser_target_logic_listener(world: &mut World, this: Entity, laser_target_logic_listener: Entity) {
    world.obj_mut::<LaserTargetLogic>(this).laser_target_logic_listener = Some(laser_target_logic_listener);
}

/// `-[LaserTargetLogic enterIsland]` @ 0x10008fac8
pub fn enter_island(world: &mut World, this: Entity) {
    let active = world.obj::<LaserTargetLogic>(this).active;
    if active {
        return;
    }
    world.obj_mut::<LaserTargetLogic>(this).active = true;

    let display_object = logic::display_object(world, this);
    let marker = pg_engine::library::display_marker(world, "TsunamiDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
    logic::set_behavior(world, this, Some("LaserTargetEnter"));
}

/// `-[LaserTargetLogic exitIsland]` @ 0x10008fb70
pub fn exit_island(world: &mut World, this: Entity) {
    let active = world.obj::<LaserTargetLogic>(this).active;
    if !active {
        return;
    }
    world.obj_mut::<LaserTargetLogic>(this).active = false;

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    logic::set_behavior(world, this, Some("LaserTargetExit"));
}

/// `-[LaserTargetLogic updateTargetPos1]` @ 0x10008fbe4
///
/// Unused: sweeps the target across the screen at height 220.
pub fn update_target_pos1(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let vel_x = logic::logic(world, this).pos_vel.x;

    let x_pos = {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = vel_x + object.pos.x;
        object.pos.y = 220.0;
        object.pos.x
    };

    if x_pos > 470.0 {
        logic::logic_mut(world, this).pos_vel.x = -PING_PONG_SPEED;
    }
    if x_pos < 10.0 {
        logic::logic_mut(world, this).pos_vel.x = PING_PONG_SPEED;
    }
}

/// `-[LaserTargetLogic updateTargetPos]` @ 0x10008fc74
///
/// Maps the raw tilt onto the screen (x from 470 to 10 as `ay` goes from −0.2 to 0.2, y from 10
/// to 310 as `ax` goes from −0.9 to −0.5) and eases the target 6 % of the way there.
pub fn update_target_pos(world: &mut World, this: Entity) {
    let raw_accel_force = world.resource::<BcView>().raw_accel_force;
    let tilt_x = if raw_accel_force.x > -0.5 { -0.5 } else { raw_accel_force.x };
    let tilt_y = if raw_accel_force.y > 0.2 { 0.2 } else { raw_accel_force.y };

    let target_x_pos = if tilt_y < -0.2 { 470.0 } else { ((tilt_y + 0.2) / -0.4) * 460.0 + 470.0 };
    let target_y_pos = if tilt_x < -0.9 { 10.0 } else { ((tilt_x + 0.9) / TILT_X_SPAN) * 300.0 + 10.0 };

    let display_object = logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.pos.x = object.pos.x + (target_x_pos - object.pos.x) * 0.06;
    object.pos.y = object.pos.y + (target_y_pos - object.pos.y) * 0.06;
}

/// `-[LaserTargetLogic onGameFrame:]` @ 0x10008fd9c
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }
    update_target_pos(world, this);
}
