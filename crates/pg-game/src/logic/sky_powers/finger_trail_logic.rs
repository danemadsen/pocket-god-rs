//! `FingerTrailLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::logic;

/// `FingerTrailLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct FingerTrailLogic {
    /// `mTargetXPos`: `f`
    pub target_x_pos: f32,

    /// `mTargetYPos`: `f`
    pub target_y_pos: f32,
}

pub const FINGER_TRAIL_LOGIC: Class = Class {
    name: "FingerTrailLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The share of the remaining distance to the target covered each frame.
const TRAIL_EASE: f32 = 0.5;

/// `[[FingerTrailLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&FINGER_TRAIL_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[FingerTrailLogic initWithDisplayObject:]` @ 0x100116fec
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(FingerTrailLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);
}

/// `-[FingerTrailLogic onGameFrame:]` @ 0x100117028
///
/// Eases the display object toward the target once per frame (not scaled by `dt`); the
/// superclass's physics does not run.
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;

    let trail = world.obj::<FingerTrailLogic>(this);
    let target_x_pos = trail.target_x_pos;
    let target_y_pos = trail.target_y_pos;

    let mut display = display::object_mut(world, display_object);
    display.pos.x = display.pos.x + (target_x_pos - display.pos.x) * TRAIL_EASE;
    display.pos.y = display.pos.y + (target_y_pos - display.pos.y) * TRAIL_EASE;
}

/// `-[FingerTrailLogic setTargetXPos:yPos:]` @ 0x1001170f8
pub fn set_target_x_pos(world: &mut World, this: Entity, target_x_pos: f32, y_pos: f32) {
    let mut trail = world.obj_mut::<FingerTrailLogic>(this);
    trail.target_x_pos = target_x_pos;
    trail.target_y_pos = y_pos;
}
