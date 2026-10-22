//! `DebugScreen` (superclass `NSObject`).

#![allow(unused_variables, unused_imports)]

use crate::prelude::*;

pub mod debug_screen;
pub use debug_screen::*;

/// `DebugScreen` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct DebugScreen {
    /// `mDebugScreenListener`: `@"<DebugScreenListener>"`
    pub debug_screen_listener: Option<Entity>,

    /// `mSequence`: `@"BCSequence"`
    pub sequence: Option<Entity>,

    /// `mbDebugScreenActive`: `B`
    pub debug_screen_active: bool,

    /// `mbExitApp`: `B`
    pub exit_app: bool,

    /// `mDebugItemMenuIndex`: `i`
    pub debug_item_menu_index: i32,

    /// `mDebugCloseButtonLogic`: `@"BCButtonLogic"`
    pub debug_close_button_logic: Option<Entity>,

    /// `mDebugNextButtonLogic`: `@"BCButtonLogic"`
    pub debug_next_button_logic: Option<Entity>,

    /// `mDebugPrevButtonLogic`: `@"BCButtonLogic"`
    pub debug_prev_button_logic: Option<Entity>,

    /// `mDebugItemButtonLogicArray`: `@"NSMutableArray"`
    pub debug_item_button_logic_array: Vec<Entity>,
}

pub const DEBUG_SCREEN: Class = Class {
    name: "DebugScreen",
    superclass: None,
    sequence_item,
    id,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    ..Class::NS_OBJECT
};

/// Sequence-item handlers implemented by `DebugScreen` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "waitForDataUpdate" => seq_wait_for_data_update(world, this, call),
        "exitApp" => seq_exit_app(world, this, call),
        "updateDebugItems" => seq_update_debug_items(world, this, call),
        "loadLocationTextureAtlas" => seq_load_location_texture_atlas(world, this, call),
        _ => return (Class::NS_OBJECT.sequence_item)(world, this, call),
    };
    Some(result)
}
