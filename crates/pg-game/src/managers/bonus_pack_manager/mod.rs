//! `BonusPackManager` (superclass `NSObject`).

#![allow(unused_variables, unused_imports)]

use crate::prelude::*;

pub mod bonus_pack;
pub use bonus_pack::*;

/// `BonusPackManager` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct BonusPackManager {
    /// `mBonusPackManagerListener`: `@"<BonusPackManagerListener>"`
    pub bonus_pack_manager_listener: Option<Entity>,

    /// `mSequence`: `@"BCSequence"`
    pub sequence: Option<Entity>,

    /// `mBonusPackScreenMode`: `i`
    pub bonus_pack_screen_mode: i32,

    /// `mbKeyboardOpen`: `B`
    pub keyboard_open: bool,

    /// `mbBonusPackScreenActive`: `B`
    pub bonus_pack_screen_active: bool,

    /// `mbBonusPackUnlockDialogActive`: `B`
    pub bonus_pack_unlock_dialog_active: bool,

    /// `mpActiveBonusPackDef`: `^{BonusPackDef=@@@@@@ii@B}`
    pub active_bonus_pack_def: Option<usize>,

    /// `mpUnlockBonusPackDef`: `^{BonusPackDef=@@@@@@ii@B}`
    pub unlock_bonus_pack_def: Option<usize>,

    /// `mLogicArray`: `@"NSMutableArray"`
    pub logic_array: Vec<Entity>,

    /// `mBonusPackUiLogicArray`: `@"NSMutableArray"`
    pub bonus_pack_ui_logic_array: Vec<Entity>,

    /// `mBonusPackListUiLogicArray`: `@"NSMutableArray"`
    pub bonus_pack_list_ui_logic_array: Vec<Entity>,

    /// `mBonusPackDetailUiLogicArray`: `@"NSMutableArray"`
    pub bonus_pack_detail_ui_logic_array: Vec<Entity>,

    /// `mBonusPackUnlockUiLogicArray`: `@"NSMutableArray"`
    pub bonus_pack_unlock_ui_logic_array: Vec<Entity>,

    /// `mBonusPackInfoUiLogicArray`: `@"NSMutableArray"`
    pub bonus_pack_info_ui_logic_array: Vec<Entity>,

    /// `mBonusPackErrorUiLogicArray`: `@"NSMutableArray"`
    pub bonus_pack_error_ui_logic_array: Vec<Entity>,

    /// `mBonusPackUnlockDialogUiLogicArray`: `@"NSMutableArray"`
    pub bonus_pack_unlock_dialog_ui_logic_array: Vec<Entity>,

    /// `mBonusPackCloseButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_close_button_logic: Option<Entity>,

    /// `mBonusPackInfoButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_info_button_logic: Option<Entity>,

    /// `mBonusPackListPrevButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_list_prev_button_logic: Option<Entity>,

    /// `mBonusPackListNextButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_list_next_button_logic: Option<Entity>,

    /// `mBonusPackListItem0ButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_list_item0_button_logic: Option<Entity>,

    /// `mBonusPackListItem1ButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_list_item1_button_logic: Option<Entity>,

    /// `mBonusPackListItem2ButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_list_item2_button_logic: Option<Entity>,

    /// `mBonusPackListItem3ButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_list_item3_button_logic: Option<Entity>,

    /// `mBonusPackListItem4ButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_list_item4_button_logic: Option<Entity>,

    /// `mBonusPackDetailPreviewItemLogic`: `@"MultiModelLogic"`
    pub bonus_pack_detail_preview_item_logic: Option<Entity>,

    /// `mBonusPackDetailUnlockButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_detail_unlock_button_logic: Option<Entity>,

    /// `mBonusPackDetailDetailButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_detail_detail_button_logic: Option<Entity>,

    /// `mBonusPackDetailBackButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_detail_back_button_logic: Option<Entity>,

    /// `mBonusPackUnlockCancelButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_unlock_cancel_button_logic: Option<Entity>,

    /// `mBonusPackInfoCloseButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_info_close_button_logic: Option<Entity>,

    /// `mBonusPackErrorCloseButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_error_close_button_logic: Option<Entity>,

    /// `mBonusPackUnlockDialogPreviewItemLogic`: `@"MultiModelLogic"`
    pub bonus_pack_unlock_dialog_preview_item_logic: Option<Entity>,

    /// `mBonusPackUnlockDialogCloseButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_unlock_dialog_close_button_logic: Option<Entity>,
}

pub const BONUS_PACK_MANAGER: Class = Class {
    name: "BonusPackManager",
    superclass: None,
    sequence_item,
    id,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    ..Class::NS_OBJECT
};

/// Sequence-item handlers implemented by `BonusPackManager` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "updateListPanel" => seq_update_list_panel(world, this, call),
        "updateDetailPanel" => seq_update_detail_panel(world, this, call),
        "updateUnlockPanel" => seq_update_unlock_panel(world, this, call),
        "updateInfoPanel" => seq_update_info_panel(world, this, call),
        "updateNoKeyChainPanel" => seq_update_no_key_chain_panel(world, this, call),
        "updateUnlockDialog" => seq_update_unlock_dialog(world, this, call),
        "loadLocationTextureAtlas" => seq_load_location_texture_atlas(world, this, call),
        _ => return (Class::NS_OBJECT.sequence_item)(world, this, call),
    };
    Some(result)
}
