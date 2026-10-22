//! `TestLogic` (superclass `BCLogic`).

use crate::prelude::*;

/// `TestLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct TestLogic {
}

pub const TEST_LOGIC: Class = Class {
    name: "TestLogic",
    superclass: Some(&pg_engine::logic::BC_LOGIC),
    ..pg_engine::logic::BC_LOGIC
};

/// `[[TestLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&TEST_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[TestLogic initWithDisplayObject:]` @ 0x100113c60
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(TestLogic::default());
    pg_engine::logic::init_with_display_object(world, this, display_object);
}
