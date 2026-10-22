//! `IslandObjectLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::logic;

/// `IslandObjectLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct IslandObjectLogic {
    /// `mDisplayMarkerId`: `@"NSString"`
    pub display_marker_id: Option<String>,
}

pub const ISLAND_OBJECT_LOGIC: Class = Class {
    name: "IslandObjectLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `-[IslandObjectLogic initWithDisplayObject:displayMarkerId:]` @ 0x100125510
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity, display_marker_id: &str) {
    world.entity_mut(this).insert(IslandObjectLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::stop_game_frame(world, this);

    let display_marker_id = display_marker_id.to_owned();
    world.obj_mut::<IslandObjectLogic>(this).display_marker_id = Some(display_marker_id);

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[IslandObjectLogic enterIsland:]` @ 0x1001255a4
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    // `getDisplayMarkerById:` answers nil for a nil or unknown id; `[nil insertBefore:]` does
    // nothing.
    let display_marker_id = world.obj::<IslandObjectLogic>(this).display_marker_id.clone();
    let marker = match display_marker_id {
        Some(display_marker_id) => pg_engine::library::display_marker(world, &display_marker_id),
        None => None,
    };
    let Some(marker) = marker else {
        return;
    };

    let display_object = logic::display_object(world, this);
    display::insert_before(world, marker, Some(display_object));
}

/// `-[IslandObjectLogic exitIsland]` @ 0x100125610
pub fn exit_island(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}
