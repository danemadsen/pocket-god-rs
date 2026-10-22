//! `GlacierLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;

/// `GlacierLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct GlacierLogic {
    /// `mGlacierLogicListener`: `@"<GlacierLogicListener>"`
    pub glacier_logic_listener: Option<Entity>,
}

pub const GLACIER_LOGIC: Class = Class {
    name: "GlacierLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `[[GlacierLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&GLACIER_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[GlacierLogic initWithDisplayObject:]` @ 0x10001ab58
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(GlacierLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::stop_game_frame(world, this);

    let own_display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, own_display_object);

    pg_engine::logic::set_behavior(world, this, Some("GlacierInit"));
}

/// `-[GlacierLogic setGlacierLogicListener:]` @ 0x10001abf8
pub fn set_glacier_logic_listener(world: &mut World, this: Entity, glacier_logic_listener: Entity) {
    world.obj_mut::<GlacierLogic>(this).glacier_logic_listener = Some(glacier_logic_listener);
}

/// `-[GlacierLogic enterIsland:]` @ 0x10001ac54
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    let display_object = pg_engine::logic::display_object(world, this);
    let marker = pg_engine::library::display_marker(world, "MidOceanMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_before(world, marker, Some(display_object));
    }
}

/// `-[GlacierLogic exitIsland]` @ 0x10001acbc
pub fn exit_island(world: &mut World, this: Entity) {
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, display_object);
}
