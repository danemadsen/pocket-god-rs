//! `NoPeeMapScreenLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::nopee::no_pee_logic;
use pg_engine::display;
use pg_engine::logic;

/// `NoPeeMapScreenLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct NoPeeMapScreenLogic {
    /// `mNoPeeMapScreenLogicListener`: `@"<NoPeeMapScreenLogicListener>"`
    pub no_pee_map_screen_logic_listener: Option<Entity>,
}

pub const NO_PEE_MAP_SCREEN_LOGIC: Class = Class {
    name: "NoPeeMapScreenLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    game_frame_order: 60,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The view window's fixed x on the map.
const MAP_SCREEN_X_POS: f32 = 64.0;

/// `[[NoPeeMapScreenLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&NO_PEE_MAP_SCREEN_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[NoPeeMapScreenLogic initWithDisplayObject:]` @ 0x10001ad40
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(NoPeeMapScreenLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);
}

/// `-[NoPeeMapScreenLogic setNoPeeMapScreenLogicListener:]` @ 0x10001ad7c
pub fn set_no_pee_map_screen_logic_listener(world: &mut World, this: Entity, no_pee_map_screen_logic_listener: Entity) {
    world.obj_mut::<NoPeeMapScreenLogic>(this).no_pee_map_screen_logic_listener = Some(no_pee_map_screen_logic_listener);
}

/// `-[NoPeeMapScreenLogic onGameFrame:]` @ 0x10001ade0
///
/// Follows the camera's depth on the map (the world's y 0…1000 in 50 points).
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let display_object = logic::display_object(world, this);
    let listener = world.obj::<NoPeeMapScreenLogic>(this).no_pee_map_screen_logic_listener;
    let no_pee = match listener {
        Some(listener) => no_pee_logic::no_pee_logic(world, listener),
        None => None,
    };
    let group_pos = no_pee_logic::world_group_pos(world, no_pee);

    let mut display = display::object_mut(world, display_object);
    display.pos.x = MAP_SCREEN_X_POS;
    display.pos.y = (group_pos.y * -50.0) / 1000.0 + 10.0;
}
