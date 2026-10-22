//! `SkinPackManager`: the "Custom Pack" store screens and panels, the StoreKit side (product
//! request, purchases, transactions), price strings, button handling and `setLogic:`.
//!
//! This port has no store (`docs/ARCHITECTURE.md`): every pack reads as bought, so
//! `getProductData` never asks for products and the buy buttons stay hidden. The StoreKit
//! callbacks keep the original's logic for the store-less answers described at each call site.

use pg_engine::display::group::get_display_object;
use pg_engine::prefs::{self, PURCHASED_SECRET};

use crate::prelude::*;
use super::*;
use super::skin_pack_state::{
    option_state, pack_current_object, pack_key_chain, pack_purchased, set_logic_behavior, set_sequence, statics_mut,
    FREE_PRODUCT_ID, NULL_SKIN_PACK_DEF,
};

/// `mSkinPackScreenMode` of the list panel.
const SCREEN_MODE_LIST: i32 = 0;

/// `mSkinPackScreenMode` of the customize panel.
const SCREEN_MODE_CUSTOMIZE: i32 = 1;

/// `mSkinPackScreenMode` of the purchase panel.
const SCREEN_MODE_PURCHASE: i32 = 2;

/// `mSkinPackScreenMode` of the purchase success panel.
const SCREEN_MODE_SUCCESS: i32 = 3;

/// `mSkinPackScreenMode` of the purchase failure panel.
const SCREEN_MODE_FAILURE: i32 = 4;

/// `mSkinPackScreenMode` of the info panel.
const SCREEN_MODE_INFO: i32 = 5;

/// `mSkinPackScreenMode` of the error panels.
const SCREEN_MODE_ERROR: i32 = 6;

/// Rows of the list panel.
const LIST_ROW_COUNT: i32 = 5;

/// `updateSkinPackDefList:` keeps the list offset in 0…19.
const LAST_LIST_OFFSET: i32 = 19;

/// `@"FREE!"`: the price of the free pack.
const FREE_PRICE_TEXT: &str = "FREE!";

/// `SKPaymentTransactionStatePurchased`.
const TRANSACTION_STATE_PURCHASED: i64 = 1;

/// `SKPaymentTransactionStateFailed`.
const TRANSACTION_STATE_FAILED: i64 = 2;

/// `SKPaymentTransactionStateRestored`.
const TRANSACTION_STATE_RESTORED: i64 = 3;

/// An `SKProduct` of `mProductArray`: what the store screens read of it. There is no store, so
/// nothing creates one and `mProductArray` stays empty.
#[derive(Component, Clone, Debug, Default)]
pub struct StoreProduct {
    /// `productIdentifier`
    pub product_identifier: String,

    /// `[[product price] stringValue]`
    pub price: String,
}

/// An `SKPaymentTransaction`: what the transaction handlers read of it.
#[derive(Clone, Debug, Default)]
pub struct PaymentTransaction {
    /// `transactionState` (`SKPaymentTransactionState`).
    pub transaction_state: i64,

    /// `[[transaction payment] productIdentifier]`
    pub product_identifier: String,
}

/// `[[[BCLibrary instance] getDisplayGroupById:group] getDisplayObject:object]`.
fn display_object_in(world: &World, display_group: Option<Entity>, object_id: &str) -> Option<Entity> {
    let display_group = display_group?;
    get_display_object(world, display_group, object_id)
}

/// `[logic displayObject]` (nil for a nil logic).
fn logic_display_object(world: &World, logic: Option<Entity>) -> Option<Entity> {
    let logic = logic?;
    Some(pg_engine::logic::display_object(world, logic))
}

/// `-[BCDisplayObject show]` (nothing for a nil object).
fn show(world: &mut World, display_object: Option<Entity>) {
    if let Some(display_object) = display_object {
        pg_engine::display::object_mut(world, display_object).hidden = false;
    }
}

/// `-[BCDisplayObject hide]` (nothing for a nil object).
fn hide(world: &mut World, display_object: Option<Entity>) {
    if let Some(display_object) = display_object {
        pg_engine::display::object_mut(world, display_object).hidden = true;
    }
}

/// `[displayObject setIsSelectable:selectable]` (nothing for a nil object).
fn set_is_selectable(world: &mut World, display_object: Option<Entity>, selectable: bool) {
    if let Some(display_object) = display_object {
        pg_engine::display::object_mut(world, display_object).is_selectable = selectable;
    }
}

/// `[text setText:value]` (nothing for a nil object).
fn set_text(world: &mut World, text: Option<Entity>, value: &str) {
    if let Some(text) = text {
        pg_engine::display::text::set_text(world, text, Some(value));
    }
}

/// `[mProductArray objectAtIndex:0]`. The original raises for an empty but non-nil array; here
/// the array is empty exactly when the original's is nil, which answers nil.
fn first_product(world: &World, this: Entity) -> Option<Entity> {
    let manager = world.obj::<SkinPackManager>(this);
    manager.product_array.first().copied()
}

/// `[[[mProductArray objectAtIndex:0] price] stringValue]` (nil without a product).
fn first_product_price(world: &World, this: Entity) -> Option<String> {
    let product = first_product(world, this)?;
    let price = world.obj::<StoreProduct>(product).price.clone();
    Some(price)
}

/// The table index of `mpActiveSkinPackDef`, which the panels dereference without a check.
fn active_pack_index(world: &World, this: Entity) -> usize {
    let active_skin_pack_def = world.obj::<SkinPackManager>(this).active_skin_pack_def;
    active_skin_pack_def.expect(NULL_SKIN_PACK_DEF)
}

/// `DAT_1007d1e78`.
fn list_offset(world: &mut World) -> i32 {
    statics_mut(world).list_offset
}

/// `-[SkinPackManager isInternetReachable]` @ 0x10005cd90
pub fn is_internet_reachable(_world: &mut World, _this: Entity) -> bool {
    // PORT: `[[Reachability reachabilityWithHostName:@"www.apple.com"] currentReachabilityStatus]
    // != NotReachable`. The port has no reachability check and answers YES
    // (`docs/re/managers-platform.md` §18.1, row 18).
    true
}

/// `-[SkinPackManager throwError:]` @ 0x10005ddb0 (it only logs the message).
pub fn throw_error(_world: &mut World, _this: Entity, _message: &str) {}

/// `-[SkinPackManager requestProductData]` @ 0x10005ddf8
pub fn request_product_data(world: &mut World, this: Entity) {
    // PORT: the original starts an `SKProductsRequest` for the 23 products on sale, with this
    // object as its delegate. There is no store, so the request is answered at once, without
    // products.
    let products = Vec::new();
    products_request(world, this, products);
}

/// `-[SkinPackManager productsRequest:didReceiveResponse:]` @ 0x10005dfa0, with the response's
/// products.
pub fn products_request(world: &mut World, this: Entity, products: Vec<Entity>) {
    let mut manager = world.obj_mut::<SkinPackManager>(this);
    manager.product_array = products;
    if manager.product_array.is_empty() {
        manager.error = true;
    }
    manager.loading_product_data = false;
}

/// `-[SkinPackManager getProductData:::]` @ 0x10005e0ec
pub fn seq_get_product_data(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let all_skin_packs_are_purchased = world.obj::<SkinPackManager>(this).all_skin_packs_are_purchased;
        if all_skin_packs_are_purchased {
            return ItemResult::Done;
        }
        let internet_reachable = is_internet_reachable(world, this);
        if !internet_reachable {
            return ItemResult::Done;
        }

        {
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.error = false;
            manager.loading = true;
            manager.loading_product_data = true;
        }
        request_product_data(world, this);
    }

    let loading_product_data = world.obj::<SkinPackManager>(this).loading_product_data;
    if loading_product_data {
        return ItemResult::Pending;
    }

    world.obj_mut::<SkinPackManager>(this).loading = false;
    let error = world.obj::<SkinPackManager>(this).error;
    if error {
        throw_error(world, this, "Loading Product Data Fail");
        set_sequence(world, this, "OpenSkinPackScreenNoInternet");
    }
    ItemResult::Done
}

/// `-[SkinPackManager purchaseSkinPack]` @ 0x10005e218
pub fn purchase_skin_pack(world: &mut World, this: Entity) {
    let products = world.obj::<SkinPackManager>(this).product_array.clone();
    for product in products {
        let product_identifier = world.obj::<StoreProduct>(product).product_identifier.clone();
        let pack_index = active_pack_index(world, this);
        let active_product_id = &skin_pack_defs()[pack_index].product_id;
        if product_identifier != *active_product_id {
            continue;
        }

        // PORT: `[[SKPaymentQueue defaultQueue] addPayment:[SKPayment paymentWithProduct:product]]`.
        // There is no store and every purchase is free (`docs/ARCHITECTURE.md`): the payment
        // queue reports the transaction as purchased at once.
        let transaction = PaymentTransaction {
            transaction_state: TRANSACTION_STATE_PURCHASED,
            product_identifier,
        };
        let transactions = [transaction];
        payment_queue(world, this, &transactions);
    }
}

/// `-[SkinPackManager failedTransaction:]` @ 0x10005e3cc
pub fn failed_transaction(world: &mut World, this: Entity, _transaction: &PaymentTransaction) {
    // `[[SKPaymentQueue defaultQueue] finishTransaction:transaction]`: there is no queue.
    let mut manager = world.obj_mut::<SkinPackManager>(this);
    manager.purchasing_skin_pack = false;
    manager.error = true;
}

/// `-[SkinPackManager restoreTransaction:]` @ 0x10005e4e8
pub fn restore_transaction(world: &mut World, this: Entity, transaction: &PaymentTransaction) {
    mark_skin_pack_purchased(world, this, &transaction.product_identifier);

    // `[[SKPaymentQueue defaultQueue] finishTransaction:transaction]`: there is no queue.
    world.obj_mut::<SkinPackManager>(this).purchasing_skin_pack = false;
}

/// `-[SkinPackManager completeTransaction:]` @ 0x10005e604
pub fn complete_transaction(world: &mut World, this: Entity, transaction: &PaymentTransaction) {
    mark_skin_pack_purchased(world, this, &transaction.product_identifier);

    // `[[SKPaymentQueue defaultQueue] finishTransaction:transaction]`: there is no queue.
    world.obj_mut::<SkinPackManager>(this).purchasing_skin_pack = false;
}

/// The bought pack of `completeTransaction:` and `restoreTransaction:`: its keychain secret
/// becomes `@"BabaBooey"` and it counts as purchased.
fn mark_skin_pack_purchased(world: &mut World, this: Entity, product_identifier: &str) {
    let pack_index = get_skin_pack_def_by_product_identifier(world, this, product_identifier);
    let Some(pack_index) = pack_index else {
        return;
    };

    let key_chain = pack_key_chain(world, pack_index);
    if let Some(key_chain) = key_chain {
        prefs::key_chain_set_secret(world, key_chain, PURCHASED_SECRET);
    }
    statics_mut(world).packs[pack_index].purchased = true;
}

/// `-[SkinPackManager paymentQueue:updatedTransactions:]` @ 0x10005e720, with the updated
/// transactions.
pub fn payment_queue(world: &mut World, this: Entity, updated_transactions: &[PaymentTransaction]) {
    for transaction in updated_transactions {
        match transaction.transaction_state {
            TRANSACTION_STATE_RESTORED => restore_transaction(world, this, transaction),
            TRANSACTION_STATE_FAILED => failed_transaction(world, this, transaction),
            TRANSACTION_STATE_PURCHASED => complete_transaction(world, this, transaction),
            _ => {}
        }
    }
}

/// `-[SkinPackManager purchaseSkinPack:::]` @ 0x10005e8a4
pub fn seq_purchase_skin_pack(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.error = false;
            manager.loading = true;
            manager.purchasing_skin_pack = true;
        }
        purchase_skin_pack(world, this);
    }

    let purchasing_skin_pack = world.obj::<SkinPackManager>(this).purchasing_skin_pack;
    if purchasing_skin_pack {
        return ItemResult::Pending;
    }

    world.obj_mut::<SkinPackManager>(this).loading = false;
    let error = world.obj::<SkinPackManager>(this).error;
    if error {
        throw_error(world, this, "Purchasing Skin Pack Fail");
        set_sequence(world, this, "OpenSkinPackScreenFailure");
    }
    ItemResult::Done
}

/// `-[SkinPackManager openSkinPackScreen:]` @ 0x10005e9a8
pub fn open_skin_pack_screen(world: &mut World, this: Entity, first_time: bool) {
    world.obj_mut::<SkinPackManager>(this).skin_pack_screen_active = true;

    let sequence_id = if first_time {
        "OpenSkinPackScreenFirstTime"
    } else {
        "OpenSkinPackScreen"
    };
    set_sequence(world, this, sequence_id);
}

/// `-[SkinPackManager closeSkinPackScreen]` @ 0x10005ea34
pub fn close_skin_pack_screen(world: &mut World, this: Entity) {
    world.obj_mut::<SkinPackManager>(this).skin_pack_screen_active = false;

    for pack_index in 0..SKIN_PACK_COUNT {
        clear_unpurchased_skin_options_from_skin_pack_def(world, this, Some(pack_index));
    }

    let customize_item = world.obj::<SkinPackManager>(this).skin_pack_customize_item_logic;
    set_logic_behavior(world, customize_item, "SkinPackStopIdle");
    set_sequence(world, this, "CloseSkinPackScreen");

    let listener = world.obj::<SkinPackManager>(this).skin_pack_manager_listener;
    if let Some(listener) = listener {
        crate::controller::on_skin_pack_screen_closed(world, listener);
    }
}

/// `-[SkinPackManager getPriceString:]` @ 0x10005eb28
pub fn get_price_string(_world: &mut World, _this: Entity, price: Option<&str>) -> &'static str {
    match price {
        Some("0.59") => "}0.59",
        Some("0.79") => "0,79 ~",
        Some("115") => "|115",
        Some("1.1") => "1.10Fr",
        Some("1.19") => "$1.19",
        Some("1.29") => "$1.29",
        Some("10") => "$10.00",
        Some("6") => "6.00Kr",
        Some("7") => "7.00Kr",
        _ => "$0.99",
    }
}

/// `-[SkinPackManager updateListPanel]` @ 0x10005ec9c
pub fn update_list_panel(world: &mut World, this: Entity) {
    world.obj_mut::<SkinPackManager>(this).skin_pack_screen_mode = SCREEN_MODE_LIST;

    let panel = pg_engine::library::display_group(world, "CustomPackListPanel");
    let next_button = display_object_in(world, panel, "SkinPackNextButton");
    let prev_button = display_object_in(world, panel, "SkinPackPrevButton");

    let first_shown = list_offset(world);
    if first_shown == 0 {
        set_is_selectable(world, prev_button, false);
        hide(world, prev_button);
    } else {
        set_is_selectable(world, prev_button, true);
        show(world, prev_button);
    }

    let first_shown = list_offset(world);
    if first_shown + LIST_ROW_COUNT < SKIN_PACK_COUNT as i32 {
        set_is_selectable(world, next_button, true);
        show(world, next_button);
    } else {
        set_is_selectable(world, next_button, false);
        hide(world, next_button);
    }

    let first_shown = list_offset(world);
    let product = first_product(world, this);
    for row in 0..LIST_ROW_COUNT {
        let pack_name = display_object_in(world, panel, &format!("PackName{row}"));
        let price_text = display_object_in(world, panel, &format!("PriceText{row}"));
        let buy_status = display_object_in(world, panel, &format!("BuyStatus{row}"));
        let purchased_mark = display_object_in(world, panel, &format!("Purchased{row}"));
        let blackbox = display_object_in(world, panel, &format!("Blackbox{row}"));
        let view_button = display_object_in(world, panel, &format!("ViewButton{row}"));

        let pack_index = first_shown + row;
        if pack_index >= SKIN_PACK_COUNT as i32 {
            hide(world, pack_name);
            set_is_selectable(world, blackbox, false);
            hide(world, blackbox);
            hide(world, view_button);
            hide(world, price_text);
            hide(world, buy_status);
            hide(world, purchased_mark);
            continue;
        }

        let pack_index = pack_index as usize;
        let pack = &skin_pack_defs()[pack_index];
        set_text(world, pack_name, &pack.name);
        show(world, pack_name);
        set_is_selectable(world, blackbox, true);
        show(world, blackbox);
        show(world, view_button);

        let purchased = pack_purchased(world, pack_index);
        if pack.product_id == FREE_PRODUCT_ID {
            show(world, price_text);
            set_text(world, price_text, FREE_PRICE_TEXT);
            show(world, buy_status);
            hide(world, purchased_mark);
        } else if product.is_none() || purchased {
            hide(world, price_text);
            set_text(world, price_text, "");
            hide(world, buy_status);
            if purchased {
                show(world, purchased_mark);
            } else {
                hide(world, purchased_mark);
            }
        } else {
            show(world, price_text);
            let price = first_product_price(world, this);
            let price_string = get_price_string(world, this, price.as_deref());
            set_text(world, price_text, price_string);
            show(world, buy_status);
            hide(world, purchased_mark);
        }
    }
}

/// `-[SkinPackManager updateCustomizePanel]` @ 0x10005f1bc
pub fn update_customize_panel(world: &mut World, this: Entity) {
    world.obj_mut::<SkinPackManager>(this).skin_pack_screen_mode = SCREEN_MODE_CUSTOMIZE;

    let panel = pg_engine::library::display_group(world, "CustomPackPanel");
    let price_text = display_object_in(world, panel, "PriceText");
    let cents = display_object_in(world, panel, "99cents");
    let pack_name = display_object_in(world, panel, "PackName");
    let option_0_text = display_object_in(world, panel, "PackOption0Text");
    let option_1_text = display_object_in(world, panel, "PackOption1Text");
    let option_2_text = display_object_in(world, panel, "PackOption2Text");
    let option_3_text = display_object_in(world, panel, "PackOption3Text");

    let manager = world.obj::<SkinPackManager>(this).clone();
    let next_pack_button = logic_display_object(world, manager.skin_pack_customize_next_pack_button_logic);
    let prev_pack_button = logic_display_object(world, manager.skin_pack_customize_prev_pack_button_logic);
    let buy_button = logic_display_object(world, manager.skin_pack_customize_buy_button_logic);
    let option_0_next = logic_display_object(world, manager.skin_pack_customize_option0_next_button_logic);
    let option_0_prev = logic_display_object(world, manager.skin_pack_customize_option0_prev_button_logic);
    let option_1_next = logic_display_object(world, manager.skin_pack_customize_option1_next_button_logic);
    let option_1_prev = logic_display_object(world, manager.skin_pack_customize_option1_prev_button_logic);
    let option_2_next = logic_display_object(world, manager.skin_pack_customize_option2_next_button_logic);
    let option_2_prev = logic_display_object(world, manager.skin_pack_customize_option2_prev_button_logic);
    let option_3_next = logic_display_object(world, manager.skin_pack_customize_option3_next_button_logic);
    let option_3_prev = logic_display_object(world, manager.skin_pack_customize_option3_prev_button_logic);
    let product = first_product(world, this);

    let pack_index = active_pack_index(world, this);
    let pack = &skin_pack_defs()[pack_index];
    if pack.product_id == FREE_PRODUCT_ID {
        show(world, price_text);
        show(world, cents);
        set_text(world, price_text, FREE_PRICE_TEXT);
    } else if product.is_none() {
        hide(world, price_text);
        hide(world, cents);
    } else {
        show(world, price_text);
        show(world, cents);
        let price = first_product_price(world, this);
        let price_string = get_price_string(world, this, price.as_deref());
        set_text(world, price_text, price_string);
    }

    set_text(world, pack_name, &pack.name);
    hide(world, next_pack_button);
    set_is_selectable(world, next_pack_button, false);
    hide(world, prev_pack_button);
    set_is_selectable(world, prev_pack_button, false);
    set_text(world, option_0_text, "");
    set_text(world, option_1_text, "");
    set_text(world, option_2_text, "");
    set_text(world, option_3_text, "");

    let option_buttons = [
        option_0_next,
        option_0_prev,
        option_1_next,
        option_1_prev,
        option_2_next,
        option_2_prev,
        option_3_next,
        option_3_prev,
    ];
    for option_button in option_buttons {
        hide(world, option_button);
        set_is_selectable(world, option_button, false);
    }

    let pack_index = active_pack_index(world, this);
    let pack = &skin_pack_defs()[pack_index];
    if pack.objects.len() > 1 {
        show(world, option_0_next);
        set_is_selectable(world, option_0_next, true);
        show(world, option_0_prev);
        set_is_selectable(world, option_0_prev, true);
    }

    let pack_index = active_pack_index(world, this);
    let pack = &skin_pack_defs()[pack_index];
    let object_index = pack_current_object(world, pack_index) as usize;
    let object = &pack.objects[object_index];
    set_text(world, option_0_text, &object.id);

    for (option_index, option) in object.options.iter().enumerate() {
        let (next_button, prev_button, value_text) = match option_index {
            0 => (option_1_next, option_1_prev, option_1_text),
            1 => (option_2_next, option_2_prev, option_2_text),
            2 => (option_3_next, option_3_prev, option_3_text),
            _ => (None, None, None),
        };

        if option.values.len() > 1 {
            show(world, next_button);
            set_is_selectable(world, next_button, true);
            show(world, prev_button);
            set_is_selectable(world, prev_button, true);
        }

        if !option.values.is_empty() {
            let current_value = option_state(world, pack_index, object_index, option_index).current_value;
            let value = &option.values[current_value as usize];
            set_text(world, value_text, &value.id);
        }
    }

    let pack_index = active_pack_index(world, this);
    let purchased = pack_purchased(world, pack_index);
    if purchased {
        hide(world, buy_button);
    } else {
        show(world, buy_button);
    }
    set_is_selectable(world, buy_button, !purchased);
}

/// `-[SkinPackManager updateInfoPanel]` @ 0x10005f960
pub fn update_info_panel(world: &mut World, this: Entity) {
    world.obj_mut::<SkinPackManager>(this).skin_pack_screen_mode = SCREEN_MODE_INFO;
}

/// `-[SkinPackManager updatePurchasePanel]` @ 0x10005f974
pub fn update_purchase_panel(world: &mut World, this: Entity) {
    world.obj_mut::<SkinPackManager>(this).skin_pack_screen_mode = SCREEN_MODE_PURCHASE;

    let panel = pg_engine::library::display_group(world, "CustomPackBuyPanel");
    let purchase_header = display_object_in(world, panel, "PurchaseHeader");
    let purchase_0 = display_object_in(world, panel, "Purchase0");
    let purchase_1 = display_object_in(world, panel, "Purchase1");
    let purchase_2 = display_object_in(world, panel, "Purchase2");

    let pack_index = active_pack_index(world, this);
    let pack = &skin_pack_defs()[pack_index];
    set_text(world, purchase_header, &pack.buy_text[0]);
    set_text(world, purchase_0, &pack.buy_text[1]);
    set_text(world, purchase_1, &pack.buy_text[2]);
    set_text(world, purchase_2, &pack.buy_text[3]);

    let price_text = display_object_in(world, panel, "PriceText");
    let price = first_product_price(world, this);
    let price_string = get_price_string(world, this, price.as_deref());
    set_text(world, price_text, price_string);
}

/// `-[SkinPackManager updatePurchaseSuccessPanel]` @ 0x10005fb3c
pub fn update_purchase_success_panel(world: &mut World, this: Entity) {
    world.obj_mut::<SkinPackManager>(this).skin_pack_screen_mode = SCREEN_MODE_SUCCESS;

    let panel = pg_engine::library::display_group(world, "CustomPackBuySuccessPanel");
    let purchase_success = display_object_in(world, panel, "PurchaseSuccess1");
    let pack_index = active_pack_index(world, this);
    let pack = &skin_pack_defs()[pack_index];
    set_text(world, purchase_success, &pack.success_text);
}

/// `-[SkinPackManager updatePurchaseFailurePanel]` @ 0x10005fbcc
pub fn update_purchase_failure_panel(world: &mut World, this: Entity) {
    world.obj_mut::<SkinPackManager>(this).skin_pack_screen_mode = SCREEN_MODE_FAILURE;
}

/// The three lines of `CustomPackErrorPanel` (`ErrorHeader`, `Error1`, `Error2`).
fn set_error_panel_text(world: &mut World, this: Entity, header: &str, line_1: &str, line_2: &str) {
    world.obj_mut::<SkinPackManager>(this).skin_pack_screen_mode = SCREEN_MODE_ERROR;

    let panel = pg_engine::library::display_group(world, "CustomPackErrorPanel");
    let error_header = display_object_in(world, panel, "ErrorHeader");
    let error_1 = display_object_in(world, panel, "Error1");
    let error_2 = display_object_in(world, panel, "Error2");
    set_text(world, error_header, header);
    set_text(world, error_1, line_1);
    set_text(world, error_2, line_2);
}

/// `-[SkinPackManager updateNoInternetPanel]` @ 0x10005fbe0
pub fn update_no_internet_panel(world: &mut World, this: Entity) {
    set_error_panel_text(
        world,
        this,
        "You aren't connected to the internet!",
        "To make in-app purchases, you must have",
        "an active internet connection.",
    );
}

/// `-[SkinPackManager updateNoKeyChainPanel]` @ 0x10005fcd8
pub fn update_no_key_chain_panel(world: &mut World, this: Entity) {
    set_error_panel_text(
        world,
        this,
        "Something is wrong with your keychain services!",
        "To make in-app purchases, you must have",
        "a keychain for us to store your purchases.",
    );
}

/// `-[SkinPackManager updateNotThreeOhPanel]` @ 0x10005fdd0
pub fn update_not_three_oh_panel(world: &mut World, this: Entity) {
    set_error_panel_text(
        world,
        this,
        "NO 3.0 OS",
        "You need the latest 3.0 firmware to purchase Skin Packs",
        "",
    );
}

/// `-[SkinPackManager updateListPanel:::]` @ 0x10005fec8
pub fn seq_update_list_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_list_panel(world, this);
    ItemResult::Done
}

/// `-[SkinPackManager updateCustomizePanel:::]` @ 0x10005feec
pub fn seq_update_customize_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_customize_panel(world, this);
    ItemResult::Done
}

/// `-[SkinPackManager updateInfoPanel:::]` @ 0x10005ff10
pub fn seq_update_info_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_info_panel(world, this);
    ItemResult::Done
}

/// `-[SkinPackManager updateNoInternetPanel:::]` @ 0x10005ff34
pub fn seq_update_no_internet_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_no_internet_panel(world, this);
    ItemResult::Done
}

/// `-[SkinPackManager updateNoKeyChainPanel:::]` @ 0x10005ff58
pub fn seq_update_no_key_chain_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_no_key_chain_panel(world, this);
    ItemResult::Done
}

/// `-[SkinPackManager updateNotThreeOhPanel:::]` @ 0x10005ff7c
pub fn seq_update_not_three_oh_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_not_three_oh_panel(world, this);
    ItemResult::Done
}

/// `-[SkinPackManager updatePurchasePanel:::]` @ 0x10005ffa0
pub fn seq_update_purchase_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_purchase_panel(world, this);
    ItemResult::Done
}

/// `-[SkinPackManager updatePurchaseSuccessPanel:::]` @ 0x10005ffc4
pub fn seq_update_purchase_success_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_purchase_success_panel(world, this);
    ItemResult::Done
}

/// `-[SkinPackManager updatePurchaseFailurePanel:::]` @ 0x10005ffe8
pub fn seq_update_purchase_failure_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_purchase_failure_panel(world, this);
    ItemResult::Done
}

/// `-[SkinPackManager purchaseSkinPackPanel:::]` @ 0x10006000c
pub fn seq_purchase_skin_pack_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    purchase_skin_pack(world, this);
    ItemResult::Done
}

/// `-[SkinPackManager setSkinPackPanel:::]` @ 0x100060030
pub fn seq_set_skin_pack_panel(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    ItemResult::Done
}

/// `-[SkinPackManager loadLocationTextureAtlas:::]` @ 0x100060038
pub fn seq_load_location_texture_atlas(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<SkinPackManager>(this).skin_pack_manager_listener;
    if let Some(listener) = listener {
        crate::controller::on_load_location_texture_atlas(world, listener);
    }
    ItemResult::Done
}

/// `-[SkinPackManager updateSkinPackDefList:]` @ 0x100060738
pub fn update_skin_pack_def_list(world: &mut World, this: Entity, first_shown: i32) {
    let below_last = first_shown.min(LAST_LIST_OFFSET);
    let clamped = below_last.max(0);

    let current_offset = list_offset(world);
    if clamped == current_offset {
        return;
    }
    statics_mut(world).list_offset = clamped;

    set_sequence(world, this, "UpdateSkinPackList");
}

/// The object buttons (`goto` target of option 0 next and prev): moves to another object of the
/// active pack, then re-applies its first option's current value silently.
fn step_skin_object(world: &mut World, this: Entity, step: i32) {
    let active_skin_pack_def = world.obj::<SkinPackManager>(this).active_skin_pack_def;
    let pack_index = active_skin_pack_def.expect(NULL_SKIN_PACK_DEF);
    let object_index = pack_current_object(world, pack_index) + step;
    update_skin_pack_def_skin_object_index_silent(world, this, active_skin_pack_def, object_index, false);

    let active_skin_pack_def = world.obj::<SkinPackManager>(this).active_skin_pack_def;
    let value_index = skin_option_value_index(world, this, 0);
    update_skin_pack_def_skin_option_index_skin_option_value_index_silent_init(
        world,
        this,
        active_skin_pack_def,
        0,
        value_index,
        true,
        false,
    );
}

/// The value buttons of options 1–3: steps through the values of option `skin_option_index`
/// until one can be applied (values of packs that are not owned are skipped).
fn step_skin_option_value(world: &mut World, this: Entity, skin_option_index: i32, step: i32) {
    let mut value_index = skin_option_value_index(world, this, skin_option_index);
    loop {
        value_index += step;
        let active_skin_pack_def = world.obj::<SkinPackManager>(this).active_skin_pack_def;
        let updated = update_skin_pack_def_skin_option_index_skin_option_value_index_silent_init(
            world,
            this,
            active_skin_pack_def,
            skin_option_index,
            value_index,
            false,
            false,
        );
        if updated {
            return;
        }
    }
}

/// `-[SkinPackManager onButtonPress:touch:]` @ 0x1000607d8
pub fn on_button_press(world: &mut World, this: Entity, button: Entity, _touch: Entity) {
    let manager = world.obj::<SkinPackManager>(this).clone();
    let pressed = Some(button);

    if pressed == manager.skin_pack_close_button_logic {
        close_skin_pack_screen(world, this);
        return;
    }
    if manager.loading {
        return;
    }

    if pressed == manager.skin_pack_info_button_logic {
        let sequence_id = if manager.skin_pack_screen_mode == SCREEN_MODE_INFO {
            "CloseSkinPackScreenInfo"
        } else {
            "OpenSkinPackScreenInfo"
        };
        set_sequence(world, this, sequence_id);
        return;
    }

    if pressed == manager.skin_pack_next_button_logic {
        let first_shown = list_offset(world);
        update_skin_pack_def_list(world, this, first_shown + LIST_ROW_COUNT);
        return;
    }
    if pressed == manager.skin_pack_prev_button_logic {
        let first_shown = list_offset(world);
        update_skin_pack_def_list(world, this, first_shown - LIST_ROW_COUNT);
        return;
    }

    let list_items = [
        manager.skin_pack_list_item0_button_logic,
        manager.skin_pack_list_item1_button_logic,
        manager.skin_pack_list_item2_button_logic,
        manager.skin_pack_list_item3_button_logic,
        manager.skin_pack_list_item4_button_logic,
    ];
    let row = list_items.iter().position(|list_item| *list_item == pressed);
    if let Some(row) = row {
        let first_shown = list_offset(world);
        let pack_index = first_shown as usize + row;
        activate_skin_pack_def(world, this, Some(pack_index));
        return;
    }

    if pressed == manager.skin_pack_customize_buy_button_logic {
        let internet_reachable = is_internet_reachable(world, this);
        let sequence_id = if !internet_reachable {
            "OpenSkinPackScreenNoInternet"
        } else {
            let pack_index = active_pack_index(world, this);
            let key_chain = pack_key_chain(world, pack_index);
            let key_chain_error = match key_chain {
                Some(key_chain) => prefs::key_chain_has_error(world, key_chain),
                None => false,
            };
            if key_chain_error {
                "OpenSkinPackScreenNoKeyChain"
            } else {
                "OpenSkinPackScreenPurchase"
            }
        };
        set_sequence(world, this, sequence_id);
        return;
    }

    if pressed == manager.skin_pack_customize_option0_next_button_logic {
        step_skin_object(world, this, 1);
        return;
    }
    if pressed == manager.skin_pack_customize_option0_prev_button_logic {
        step_skin_object(world, this, -1);
        return;
    }
    if pressed == manager.skin_pack_customize_option1_next_button_logic {
        step_skin_option_value(world, this, 0, 1);
        return;
    }
    if pressed == manager.skin_pack_customize_option1_prev_button_logic {
        step_skin_option_value(world, this, 0, -1);
        return;
    }
    if pressed == manager.skin_pack_customize_option2_next_button_logic {
        step_skin_option_value(world, this, 1, 1);
        return;
    }
    if pressed == manager.skin_pack_customize_option2_prev_button_logic {
        step_skin_option_value(world, this, 1, -1);
        return;
    }
    if pressed == manager.skin_pack_customize_option3_next_button_logic {
        step_skin_option_value(world, this, 2, 1);
        return;
    }
    if pressed == manager.skin_pack_customize_option3_prev_button_logic {
        step_skin_option_value(world, this, 2, -1);
        return;
    }

    // The next and previous pack buttons only log.
    if pressed == manager.skin_pack_customize_next_pack_button_logic {
        return;
    }
    if pressed == manager.skin_pack_customize_prev_pack_button_logic {
        return;
    }

    if pressed == manager.skin_pack_customize_back_button_logic {
        clear_unpurchased_skin_options_from_skin_pack_def(world, this, manager.active_skin_pack_def);
        set_sequence(world, this, "OpenSkinPackScreenList");
        return;
    }
    if pressed == manager.skin_pack_purchase_buy_button_logic {
        set_sequence(world, this, "PurchaseSkinPack");
        return;
    }
    if pressed == manager.skin_pack_purchase_cancel_button_logic {
        set_sequence(world, this, "OpenSkinPackScreenPurchaseHide");
        return;
    }
    if pressed == manager.skin_pack_success_ok_button_logic {
        set_sequence(world, this, "OpenSkinPackScreenSuccessHide");
        return;
    }
    if pressed == manager.skin_pack_failure_cancel_button_logic {
        set_sequence(world, this, "OpenSkinPackScreenFailureHide");
        return;
    }
    if pressed == manager.skin_pack_error_cancel_button_logic {
        if manager.error {
            close_skin_pack_screen(world, this);
        } else {
            set_sequence(world, this, "OpenSkinPackScreenErrorHide");
        }
        return;
    }
    if pressed == manager.skin_pack_info_close_button_logic {
        set_sequence(world, this, "CloseSkinPackScreenInfo");
    }
}

/// `-[SkinPackManager onButtonRelease:touch:]` @ 0x1000611cc
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[SkinPackManager onButtonReleaseOutside:touch:]` @ 0x1000611d0
pub fn on_button_release_outside(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `[[BCButtonLogic alloc] initWithDisplayObject:]` with this object as its listener.
fn spawn_button_logic(world: &mut World, this: Entity, display_object: Entity) -> Entity {
    let button = pg_engine::logic::button::spawn(world, display_object);
    pg_engine::logic::button::set_button_logic_listener(world, button, Some(this));
    button
}

/// `-[SkinPackManager setLogic:]` @ 0x1000611d4
pub fn set_logic(world: &mut World, this: Entity, call: &SequenceCall) {
    let display_group_id = call.param("displayGroupId");
    let logic_id = call.param("logicId");
    let display_object_id = call.param("displayObjectId");

    let display_group = match display_group_id {
        Some(display_group_id) => pg_engine::library::display_group(world, display_group_id),
        None => None,
    };
    let display_object = match display_object_id {
        Some(display_object_id) => display_object_in(world, display_group, display_object_id),
        None => None,
    };

    let Some(logic_id) = logic_id else {
        return;
    };

    // PORT: the original also creates the logic on a nil display object (the boot script's
    // `NextPack` and `PrevPack` are not in `CustomPackPanel`); the engine's logics need one, so
    // nothing is created. Such a button can never be pressed, and hiding its display object did
    // nothing.
    let Some(display_object) = display_object else {
        return;
    };

    match logic_id {
        "SkinPackCloseButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_close_button_logic = Some(button);
            manager.skin_pack_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackInfoButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_info_button_logic = Some(button);
            manager.skin_pack_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackNextButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_next_button_logic = Some(button);
            manager.skin_pack_list_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackPrevButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_prev_button_logic = Some(button);
            manager.skin_pack_list_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackListItem0ButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_list_item0_button_logic = Some(button);
            manager.skin_pack_list_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackListItem1ButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_list_item1_button_logic = Some(button);
            manager.skin_pack_list_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackListItem2ButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_list_item2_button_logic = Some(button);
            manager.skin_pack_list_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackListItem3ButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_list_item3_button_logic = Some(button);
            manager.skin_pack_list_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackListItem4ButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_list_item4_button_logic = Some(button);
            manager.skin_pack_list_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackCustomizeItemLogic" => {
            let customize_item = pg_engine::logic::multi_model::spawn(world, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_customize_item_logic = Some(customize_item);
            manager.logic_array.push(customize_item);
        }
        "SkinPackCustomizeBuyButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_customize_buy_button_logic = Some(button);
            manager.skin_pack_customize_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackCustomizeOption0NextButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_customize_option0_next_button_logic = Some(button);
            manager.skin_pack_customize_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackCustomizeOption0PrevButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_customize_option0_prev_button_logic = Some(button);
            manager.skin_pack_customize_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackCustomizeOption1NextButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_customize_option1_next_button_logic = Some(button);
            manager.skin_pack_customize_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackCustomizeOption1PrevButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_customize_option1_prev_button_logic = Some(button);
            manager.skin_pack_customize_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackCustomizeOption2NextButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_customize_option2_next_button_logic = Some(button);
            manager.skin_pack_customize_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackCustomizeOption2PrevButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_customize_option2_prev_button_logic = Some(button);
            manager.skin_pack_customize_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackCustomizeOption3NextButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_customize_option3_next_button_logic = Some(button);
            manager.skin_pack_customize_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackCustomizeOption3PrevButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_customize_option3_prev_button_logic = Some(button);
            manager.skin_pack_customize_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackCustomizeNextPackButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_customize_next_pack_button_logic = Some(button);
            manager.skin_pack_customize_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackCustomizePrevPackButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_customize_prev_pack_button_logic = Some(button);
            manager.skin_pack_customize_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackCustomizeBackButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_customize_back_button_logic = Some(button);
            manager.skin_pack_customize_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackPurchaseBuyButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_purchase_buy_button_logic = Some(button);
            manager.skin_pack_purchase_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackPurchaseCancelButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_purchase_cancel_button_logic = Some(button);
            manager.skin_pack_purchase_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackSuccessOkButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_success_ok_button_logic = Some(button);
            manager.skin_pack_success_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackFailureCancelButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_failure_cancel_button_logic = Some(button);
            manager.skin_pack_failure_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackErrorCancelButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_error_cancel_button_logic = Some(button);
            manager.skin_pack_error_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "SkinPackInfoCloseButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<SkinPackManager>(this);
            manager.skin_pack_info_close_button_logic = Some(button);
            manager.skin_pack_info_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        _ => {}
    }
}
