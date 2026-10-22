//! `DiscoLightsLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::logic;

/// `DiscoLightsLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct DiscoLightsLogic {
    /// `mDiscoLightsLogicListener`: `@"<DiscoLightsLogicListener>"`
    pub disco_lights_logic_listener: Option<Entity>,
}

pub const DISCO_LIGHTS_LOGIC: Class = Class {
    name: "DiscoLightsLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `[[DiscoLightsLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&DISCO_LIGHTS_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[DiscoLightsLogic initWithDisplayObject:]` @ 0x1001a745c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(DiscoLightsLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::stop_game_frame(world, this);
}

/// `-[DiscoLightsLogic setDiscoLightsLogicListener:]` @ 0x1001a74c0
pub fn set_disco_lights_logic_listener(world: &mut World, this: Entity, disco_lights_logic_listener: Entity) {
    world.obj_mut::<DiscoLightsLogic>(this).disco_lights_logic_listener = Some(disco_lights_logic_listener);
}

/// `-[DiscoLightsLogic enterIsland:]` @ 0x1001a751c
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    logic::set_behavior(world, this, Some("DiscoLightsEnterIsland"));
}

/// `-[DiscoLightsLogic exitIsland:]` @ 0x1001a7534
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    logic::set_behavior(world, this, Some("DiscoLightsExitIsland"));
}
