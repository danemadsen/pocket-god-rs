//! `IceTreeLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;

/// `IceTreeLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct IceTreeLogic {
    /// `mIceTreeLogicListener`: `@"<IceTreeLogicListener>"`
    pub ice_tree_logic_listener: Option<Entity>,
}

pub const ICE_TREE_LOGIC: Class = Class {
    name: "IceTreeLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `[[IceTreeLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&ICE_TREE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[IceTreeLogic initWithDisplayObject:]` @ 0x1000c4bd8
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(IceTreeLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::stop_game_frame(world, this);

    let own_display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, own_display_object).is_selectable = false;
}

/// `-[IceTreeLogic setIceTreeLogicListener:]` @ 0x1000c4c60
pub fn set_ice_tree_logic_listener(world: &mut World, this: Entity, ice_tree_logic_listener: Entity) {
    world.obj_mut::<IceTreeLogic>(this).ice_tree_logic_listener = Some(ice_tree_logic_listener);
}

/// `-[IceTreeLogic enterIsland:]` @ 0x1000c4cbc
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    pg_engine::logic::set_behavior(world, this, Some("IceTreeEnterIsland"));
}

/// `-[IceTreeLogic exitIsland]` @ 0x1000c4cd4
pub fn exit_island(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("IceTreeExitIsland"));
}

/// `-[IceTreeLogic onGameFrame:]` @ 0x1000c4d50
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}
