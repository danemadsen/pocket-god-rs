//! `SkinPackManager` (superclass `NSObject`).

#![allow(unused_variables, unused_imports)]

use crate::prelude::*;

pub mod skin_pack_state;
pub use skin_pack_state::*;
pub mod skin_pack_store;
pub use skin_pack_store::*;

/// `SkinPackManager` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct SkinPackManager {
    /// `mSkinPackManagerListener`: `@"<SkinPackManagerListener>"`
    pub skin_pack_manager_listener: Option<Entity>,

    /// `mSequence`: `@"BCSequence"`
    pub sequence: Option<Entity>,

    /// `mSkinPackScreenMode`: `i`
    pub skin_pack_screen_mode: i32,

    /// `mbSkinPackScreenActive`: `B`
    pub skin_pack_screen_active: bool,

    /// `mbError`: `B`
    pub error: bool,

    /// `mbLoading`: `B`
    pub loading: bool,

    /// `mbLoadingProductData`: `B`
    pub loading_product_data: bool,

    /// `mbPurchasingSkinPack`: `B`
    pub purchasing_skin_pack: bool,

    /// `mbAllSkinPacksArePurchased`: `B`
    pub all_skin_packs_are_purchased: bool,

    /// `mpActiveSkinPackDef`: `^{BCSkinPackDef=@@@@@@@@B@ii^^^{BCSkinObjectDef}}`
    pub active_skin_pack_def: Option<usize>,

    /// `mProductArray`: `@"NSArray"`
    pub product_array: Vec<Entity>,

    /// `mSkinOptionStateDictionary`: `@"NSMutableDictionary"` (option id → `"%d"` of its applied value index)
    pub skin_option_state_dictionary: HashMap<String, String>,

    /// `mSkinOptionChannelDictionary`: `@"NSMutableDictionary"` (atlas channel → id of its owner)
    pub skin_option_channel_dictionary: HashMap<String, String>,

    /// `mLogicArray`: `@"NSMutableArray"`
    pub logic_array: Vec<Entity>,

    /// `mSkinPackUiLogicArray`: `@"NSMutableArray"`
    pub skin_pack_ui_logic_array: Vec<Entity>,

    /// `mSkinPackListUiLogicArray`: `@"NSMutableArray"`
    pub skin_pack_list_ui_logic_array: Vec<Entity>,

    /// `mSkinPackCustomizeUiLogicArray`: `@"NSMutableArray"`
    pub skin_pack_customize_ui_logic_array: Vec<Entity>,

    /// `mSkinPackPurchaseUiLogicArray`: `@"NSMutableArray"`
    pub skin_pack_purchase_ui_logic_array: Vec<Entity>,

    /// `mSkinPackSuccessUiLogicArray`: `@"NSMutableArray"`
    pub skin_pack_success_ui_logic_array: Vec<Entity>,

    /// `mSkinPackFailureUiLogicArray`: `@"NSMutableArray"`
    pub skin_pack_failure_ui_logic_array: Vec<Entity>,

    /// `mSkinPackErrorUiLogicArray`: `@"NSMutableArray"`
    pub skin_pack_error_ui_logic_array: Vec<Entity>,

    /// `mSkinPackInfoUiLogicArray`: `@"NSMutableArray"`
    pub skin_pack_info_ui_logic_array: Vec<Entity>,

    /// `mSkinPackCloseButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_close_button_logic: Option<Entity>,

    /// `mSkinPackInfoButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_info_button_logic: Option<Entity>,

    /// `mSkinPackNextButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_next_button_logic: Option<Entity>,

    /// `mSkinPackPrevButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_prev_button_logic: Option<Entity>,

    /// `mSkinPackListItem0ButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_list_item0_button_logic: Option<Entity>,

    /// `mSkinPackListItem1ButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_list_item1_button_logic: Option<Entity>,

    /// `mSkinPackListItem2ButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_list_item2_button_logic: Option<Entity>,

    /// `mSkinPackListItem3ButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_list_item3_button_logic: Option<Entity>,

    /// `mSkinPackListItem4ButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_list_item4_button_logic: Option<Entity>,

    /// `mSkinPackCustomizeItemLogic`: `@"MultiModelLogic"`
    pub skin_pack_customize_item_logic: Option<Entity>,

    /// `mSkinPackCustomizeBuyButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_customize_buy_button_logic: Option<Entity>,

    /// `mSkinPackCustomizeOption0NextButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_customize_option0_next_button_logic: Option<Entity>,

    /// `mSkinPackCustomizeOption0PrevButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_customize_option0_prev_button_logic: Option<Entity>,

    /// `mSkinPackCustomizeOption1NextButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_customize_option1_next_button_logic: Option<Entity>,

    /// `mSkinPackCustomizeOption1PrevButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_customize_option1_prev_button_logic: Option<Entity>,

    /// `mSkinPackCustomizeOption2NextButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_customize_option2_next_button_logic: Option<Entity>,

    /// `mSkinPackCustomizeOption2PrevButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_customize_option2_prev_button_logic: Option<Entity>,

    /// `mSkinPackCustomizeOption3NextButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_customize_option3_next_button_logic: Option<Entity>,

    /// `mSkinPackCustomizeOption3PrevButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_customize_option3_prev_button_logic: Option<Entity>,

    /// `mSkinPackCustomizeNextPackButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_customize_next_pack_button_logic: Option<Entity>,

    /// `mSkinPackCustomizePrevPackButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_customize_prev_pack_button_logic: Option<Entity>,

    /// `mSkinPackCustomizeBackButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_customize_back_button_logic: Option<Entity>,

    /// `mSkinPackPurchaseBuyButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_purchase_buy_button_logic: Option<Entity>,

    /// `mSkinPackPurchaseCancelButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_purchase_cancel_button_logic: Option<Entity>,

    /// `mSkinPackSuccessOkButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_success_ok_button_logic: Option<Entity>,

    /// `mSkinPackFailureCancelButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_failure_cancel_button_logic: Option<Entity>,

    /// `mSkinPackErrorCancelButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_error_cancel_button_logic: Option<Entity>,

    /// `mSkinPackInfoCloseButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_info_close_button_logic: Option<Entity>,
}

pub const SKIN_PACK_MANAGER: Class = Class {
    name: "SkinPackManager",
    superclass: None,
    sequence_item,
    id,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    ..Class::NS_OBJECT
};

/// Sequence-item handlers implemented by `SkinPackManager` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "getProductData" => seq_get_product_data(world, this, call),
        "purchaseSkinPack" => seq_purchase_skin_pack(world, this, call),
        "updateListPanel" => seq_update_list_panel(world, this, call),
        "updateCustomizePanel" => seq_update_customize_panel(world, this, call),
        "updateInfoPanel" => seq_update_info_panel(world, this, call),
        "updateNoInternetPanel" => seq_update_no_internet_panel(world, this, call),
        "updateNoKeyChainPanel" => seq_update_no_key_chain_panel(world, this, call),
        "updateNotThreeOhPanel" => seq_update_not_three_oh_panel(world, this, call),
        "updatePurchasePanel" => seq_update_purchase_panel(world, this, call),
        "updatePurchaseSuccessPanel" => seq_update_purchase_success_panel(world, this, call),
        "updatePurchaseFailurePanel" => seq_update_purchase_failure_panel(world, this, call),
        "purchaseSkinPackPanel" => seq_purchase_skin_pack_panel(world, this, call),
        "setSkinPackPanel" => seq_set_skin_pack_panel(world, this, call),
        "loadLocationTextureAtlas" => seq_load_location_texture_atlas(world, this, call),
        _ => return (Class::NS_OBJECT.sequence_item)(world, this, call),
    };
    Some(result)
}
