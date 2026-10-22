//! `MapScreen` (superclass `NSObject`).

#![allow(unused_variables, unused_imports)]

use crate::prelude::*;

pub mod map_screen;
pub use map_screen::*;

/// `MapScreen` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct MapScreen {
    /// `mMapScreenListener`: `@"<MapScreenListener>"`
    pub map_screen_listener: Option<Entity>,

    /// `mSequence`: `@"BCSequence"`
    pub sequence: Option<Entity>,

    /// `mbMapScreenActive`: `B`
    pub map_screen_active: bool,

    /// `mbCancelLocationStateChange`: `B`
    pub cancel_location_state_change: bool,

    /// `mIslandLocationState`: `i`
    pub island_location_state: i32,

    /// `mMapScreenCloseButtonLogic`: `@"BCButtonLogic"`
    pub map_screen_close_button_logic: Option<Entity>,

    /// `mMapScreenAdBlockerLogic`: `@"AdBlockerLogic"`
    pub map_screen_ad_blocker_logic: Option<Entity>,

    /// `mMapScreenSandIslandButtonLogic`: `@"BCButtonLogic"`
    pub map_screen_sand_island_button_logic: Option<Entity>,

    /// `mMapScreenRockIslandButtonLogic`: `@"BCButtonLogic"`
    pub map_screen_rock_island_button_logic: Option<Entity>,

    /// `mMapScreenGraveyardIslandButtonLogic`: `@"BCButtonLogic"`
    pub map_screen_graveyard_island_button_logic: Option<Entity>,

    /// `mMapScreenIceIslandButtonLogic`: `@"BCButtonLogic"`
    pub map_screen_ice_island_button_logic: Option<Entity>,

    /// `mMapScreenApeIslandButtonLogic`: `@"BCButtonLogic"`
    pub map_screen_ape_island_button_logic: Option<Entity>,

    /// `mMapScreenMisfitIslandButtonLogic`: `@"BCButtonLogic"`
    pub map_screen_misfit_island_button_logic: Option<Entity>,

    /// `mMapScreenGodIslandButtonLogic`: `@"BCButtonLogic"`
    pub map_screen_god_island_button_logic: Option<Entity>,

    /// `mMapScreenFailBoatButtonLogic`: `@"BCButtonLogic"`
    pub map_screen_fail_boat_button_logic: Option<Entity>,

    /// `mMapScreenApocalypseIslandButtonLogic`: `@"BCButtonLogic"`
    pub map_screen_apocalypse_island_button_logic: Option<Entity>,

    /// `mMapScreenUnderwaterButtonLogic`: `@"BCButtonLogic"`
    pub map_screen_underwater_button_logic: Option<Entity>,

    /// `mMapScreenUnderwaterCoralReefButtonLogic`: `@"BCButtonLogic"`
    pub map_screen_underwater_coral_reef_button_logic: Option<Entity>,
}

pub const MAP_SCREEN: Class = Class {
    name: "MapScreen",
    superclass: None,
    sequence_item,
    id,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    ..Class::NS_OBJECT
};

/// Sequence-item handlers implemented by `MapScreen` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "onMapCloseComplete" => seq_on_map_close_complete(world, this, call),
        _ => return (Class::NS_OBJECT.sequence_item)(world, this, call),
    };
    Some(result)
}
