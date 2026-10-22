//! `BonusPackManager`: methods in this part of the class.

use super::*;
use crate::controller;

use pg_engine::display;
use pg_engine::display::text;
use pg_engine::library;
use pg_engine::prefs;
use pg_engine::sequence;

/// `-[BonusPackManager id]` (`0x1003cb650`).
const CLASS_ID: &str = "BonusPackManager";

/// Bonus packs count as owned from the first launch (`docs/ARCHITECTURE.md`,
/// `docs/re/managers-platform.md` §8.4): the pack reads as unlocked, as if its keychain item
/// held the unlock secret. Everything that depends on the flag is ported unchanged, so a
/// different policy only needs this flag.
const BONUS_PACKS_OWNED_FROM_FIRST_LAUNCH: bool = true;

/// `mBonusPackScreenMode` values: the panel the screen shows.
const SCREEN_MODE_LIST: i32 = 0;
const SCREEN_MODE_DETAIL: i32 = 1;
const SCREEN_MODE_UNLOCK: i32 = 2;
const SCREEN_MODE_INFO: i32 = 3;
const SCREEN_MODE_NO_KEY_CHAIN: i32 = 4;

/// `onShowKeyboard:…maxEditCharCount:` of the code entry.
const MAX_CODE_CHAR_COUNT: i32 = 30;

const OPEN_BONUS_PACK_SCREEN_SEQUENCE: &str = "OpenBonusPackScreen";
const OPEN_BONUS_PACK_SCREEN_FIRST_TIME_SEQUENCE: &str = "OpenBonusPackScreenFirstTime";
const CLOSE_BONUS_PACK_SCREEN_SEQUENCE: &str = "CloseBonusPackScreen";
const OPEN_BONUS_PACK_UNLOCK_DIALOG_SEQUENCE: &str = "OpenBonusPackUnlockDialog";
const CLOSE_BONUS_PACK_UNLOCK_DIALOG_SEQUENCE: &str = "CloseBonusPackUnlockDialog";
const OPEN_BONUS_PACK_SCREEN_DETAIL_SEQUENCE: &str = "OpenBonusPackScreenDetail";
const OPEN_BONUS_PACK_SCREEN_LIST_SEQUENCE: &str = "OpenBonusPackScreenList";
const OPEN_BONUS_PACK_SCREEN_INFO_SEQUENCE: &str = "OpenBonusPackScreenInfo";
const CLOSE_BONUS_PACK_SCREEN_INFO_SEQUENCE: &str = "CloseBonusPackScreenInfo";
const OPEN_BONUS_PACK_SCREEN_UNLOCK_SEQUENCE: &str = "OpenBonusPackScreenUnlock";
const CLOSE_BONUS_PACK_SCREEN_UNLOCK_SEQUENCE: &str = "CloseBonusPackScreenUnlock";
const OPEN_BONUS_PACK_SCREEN_NO_KEY_CHAIN_SEQUENCE: &str = "OpenBonusPackScreenNoKeyChain";
const OPEN_BONUS_PACK_SCREEN_ERROR_HIDE_SEQUENCE: &str = "OpenBonusPackScreenErrorHide";

const BONUS_PACK_LIST_PANEL: &str = "BonusPackListPanel";
const BONUS_PACK_DETAIL_PANEL: &str = "BonusPackDetailPanel";
const BONUS_PACK_UNLOCK_PANEL: &str = "BonusPackUnlockPanel";
const BONUS_PACK_ERROR_PANEL: &str = "BonusPackErrorPanel";
const BONUS_PACK_UNLOCK_DIALOG: &str = "BonusPackUnlockDialog";

/// The preview behaviour that ends the jet-pack animation.
const BONUS_PACK_STOP_IDLE_BEHAVIOR: &str = "BonusPackStopIdle";

/// `BonusPackDef` (`{@@@@@@ii@B}`, 0x48 bytes): the fields the binary initialises. The keychain
/// item (+0x38) and the unlocked flag (+0x40) change at run time and live in
/// [`BonusPackManagerStatics`].
#[derive(Debug)]
pub struct BonusPackDef {
    /// +0x00, also the keychain key.
    pub id: &'static str,

    /// +0x08
    pub name: &'static str,

    /// +0x10: the unlock code, compared with `caseInsensitiveCompare:`.
    pub code: &'static str,

    /// +0x18: the page the Details button opens.
    pub url: Option<&'static str>,

    /// +0x20: the behaviour of the preview logics.
    pub preview_behavior: &'static str,

    /// +0x28
    pub detail_text: &'static str,

    /// +0x30: the challenge type whose score unlocks the pack (4, Ooga Jump).
    pub unlock_challenge_type: i32,

    /// +0x34
    pub unlock_score: i32,
}

/// Index of the Comic Pre-Order pack (the def at 0x100602c88) in [`BONUS_PACK_DEFS`].
pub const COMIC_PRE_ORDER_BONUS_PACK_DEF: usize = 0;

/// Number of bonus pack defs.
pub const BONUS_PACK_DEF_COUNT: usize = 1;

/// The bonus pack defs: the single `BonusPackDef` at 0x100602c88.
pub static BONUS_PACK_DEFS: [BonusPackDef; BONUS_PACK_DEF_COUNT] = [BonusPackDef {
    id: "ComicPreOrder",
    name: "Pocket God Comic Pre-Order Bonus",
    code: "journey to uranus",
    url: Some("http://www.boltcreative.com/"),
    preview_behavior: "BonusPackComicPreOrderPreview",
    detail_text: "Want your Ooga Jump score to sky rocket?\nGet the OOGA JUMP JET PACK BONUS! We\nare recruiting YOU to help get Pocket God comics \nin the comic stores and in return we are giving \nthe jet pack FREE to everyone who participates.\nClick DETAILS below.\n\nBut that's not all! Everyone who\npartcipates in the comic promotion is\nautomatically entered in a drawing for a\n$1000 Apple Gift Certificate!\n\nIf you choose not to participate, you\nmay also win the jet pack bonus by achieving\nthe 'God' score on Ooga Jump of 6000 Oogles.",
    unlock_challenge_type: 4,
    unlock_score: 6000,
}];

/// `PTR_PTR_10037d930`: the def of each list row. Only row 0 holds one; the next four slots
/// of the original table hold 0, 0, 0x28 and a code pointer.
const BONUS_PACK_LIST: [usize; 1] = [COMIC_PRE_ORDER_BONUS_PACK_DEF];

/// The run-time fields of the static `BonusPackDef`s.
#[derive(Resource, Default)]
pub struct BonusPackManagerStatics {
    /// Each def's keychain item (+0x38; `DAT_100602cc0` for the Comic Pre-Order pack).
    pub key_chains: [Option<Entity>; BONUS_PACK_DEF_COUNT],

    /// Each def's unlocked flag (+0x40; `DAT_100602cc8`).
    pub unlocked: [bool; BONUS_PACK_DEF_COUNT],
}

/// The ivar a button logic of `setLogic:` is stored in.
type LogicSlot = fn(&mut BonusPackManager) -> &mut Option<Entity>;

/// The UI array a button logic of `setLogic:` joins.
type UiArraySlot = fn(&mut BonusPackManager) -> &mut Vec<Entity>;

/// The ivar and the UI array of each `BCButtonLogic` that `setLogic:` creates.
fn button_logic_slot(logic_id: &str) -> Option<(LogicSlot, UiArraySlot)> {
    let slot: (LogicSlot, UiArraySlot) = match logic_id {
        "BonusPackCloseButtonLogic" => (
            |manager| &mut manager.bonus_pack_close_button_logic,
            |manager| &mut manager.bonus_pack_ui_logic_array,
        ),
        "BonusPackInfoButtonLogic" => (
            |manager| &mut manager.bonus_pack_info_button_logic,
            |manager| &mut manager.bonus_pack_ui_logic_array,
        ),
        "BonusPackListPrevButtonLogic" => (
            |manager| &mut manager.bonus_pack_list_prev_button_logic,
            |manager| &mut manager.bonus_pack_list_ui_logic_array,
        ),
        "BonusPackListNextButtonLogic" => (
            |manager| &mut manager.bonus_pack_list_next_button_logic,
            |manager| &mut manager.bonus_pack_list_ui_logic_array,
        ),
        "BonusPackListItem0ButtonLogic" => (
            |manager| &mut manager.bonus_pack_list_item0_button_logic,
            |manager| &mut manager.bonus_pack_list_ui_logic_array,
        ),
        "BonusPackListItem1ButtonLogic" => (
            |manager| &mut manager.bonus_pack_list_item1_button_logic,
            |manager| &mut manager.bonus_pack_list_ui_logic_array,
        ),
        "BonusPackListItem2ButtonLogic" => (
            |manager| &mut manager.bonus_pack_list_item2_button_logic,
            |manager| &mut manager.bonus_pack_list_ui_logic_array,
        ),
        "BonusPackListItem3ButtonLogic" => (
            |manager| &mut manager.bonus_pack_list_item3_button_logic,
            |manager| &mut manager.bonus_pack_list_ui_logic_array,
        ),
        "BonusPackListItem4ButtonLogic" => (
            |manager| &mut manager.bonus_pack_list_item4_button_logic,
            |manager| &mut manager.bonus_pack_list_ui_logic_array,
        ),
        "BonusPackDetailUnlockButtonLogic" => (
            |manager| &mut manager.bonus_pack_detail_unlock_button_logic,
            |manager| &mut manager.bonus_pack_detail_ui_logic_array,
        ),
        "BonusPackDetailDetailButtonLogic" => (
            |manager| &mut manager.bonus_pack_detail_detail_button_logic,
            |manager| &mut manager.bonus_pack_detail_ui_logic_array,
        ),
        "BonusPackDetailBackButtonLogic" => (
            |manager| &mut manager.bonus_pack_detail_back_button_logic,
            |manager| &mut manager.bonus_pack_detail_ui_logic_array,
        ),
        "BonusPackUnlockCancelButtonLogic" => (
            |manager| &mut manager.bonus_pack_unlock_cancel_button_logic,
            |manager| &mut manager.bonus_pack_unlock_ui_logic_array,
        ),
        "BonusPackInfoCloseButtonLogic" => (
            |manager| &mut manager.bonus_pack_info_close_button_logic,
            |manager| &mut manager.bonus_pack_info_ui_logic_array,
        ),
        "BonusPackErrorCloseButtonLogic" => (
            |manager| &mut manager.bonus_pack_error_close_button_logic,
            |manager| &mut manager.bonus_pack_error_ui_logic_array,
        ),
        "BonusPackUnlockDialogCloseButtonLogic" => (
            |manager| &mut manager.bonus_pack_unlock_dialog_close_button_logic,
            |manager| &mut manager.bonus_pack_unlock_dialog_ui_logic_array,
        ),
        _ => return None,
    };
    Some(slot)
}

/// The ivar of each `MultiModelLogic` preview that `setLogic:` creates (no listener, no UI
/// array).
fn preview_logic_slot(logic_id: &str) -> Option<LogicSlot> {
    let slot: LogicSlot = match logic_id {
        "BonusPackDetailPreviewItemLogic" => |manager| &mut manager.bonus_pack_detail_preview_item_logic,
        "BonusPackUnlockDialogPreviewItemLogic" => |manager| &mut manager.bonus_pack_unlock_dialog_preview_item_logic,
        _ => return None,
    };
    Some(slot)
}

/// The bonus pack defs' run-time fields, mutably (created on first use).
fn statics_mut(world: &mut World) -> Mut<'_, BonusPackManagerStatics> {
    world.get_resource_or_insert_with(BonusPackManagerStatics::default)
}

/// `def->unlocked` (+0x40).
fn is_def_unlocked(world: &World, def: usize) -> bool {
    match world.get_resource::<BonusPackManagerStatics>() {
        Some(statics) => statics.unlocked[def],
        None => false,
    }
}

/// `def->keyChain` (+0x38).
fn def_key_chain(world: &World, def: usize) -> Option<Entity> {
    let statics = world.get_resource::<BonusPackManagerStatics>()?;
    statics.key_chains[def]
}

/// `mBonusPackManagerListener` (the view controller: `loadGame` sets it right after `init`).
fn listener(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<BonusPackManager>(this).bonus_pack_manager_listener
}

/// `[mSequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:id]]`.
fn set_sequence(world: &mut World, this: Entity, id: &str) {
    let manager_sequence = world.obj::<BonusPackManager>(this).sequence;
    let sequence_def = library::sequence_def(world, id);
    if let Some(manager_sequence) = manager_sequence {
        sequence::set_sequence_def(world, manager_sequence, sequence_def);
    }
}

/// `[group getDisplayObject:id]` (nil for a nil group).
fn group_object(world: &World, display_group: Option<Entity>, id: &str) -> Option<Entity> {
    let display_group = display_group?;
    display::group::get_display_object(world, display_group, id)
}

/// `[object show]`.
fn show(world: &mut World, object: Option<Entity>) {
    if let Some(object) = object {
        display::object_mut(world, object).hidden = false;
    }
}

/// `[object hide]`.
fn hide(world: &mut World, object: Option<Entity>) {
    if let Some(object) = object {
        display::object_mut(world, object).hidden = true;
    }
}

/// `[object setIsSelectable:selectable]`.
fn set_selectable(world: &mut World, object: Option<Entity>, selectable: bool) {
    if let Some(object) = object {
        display::object_mut(world, object).is_selectable = selectable;
    }
}

/// `[object setText:value]`.
fn set_text(world: &mut World, object: Option<Entity>, value: &str) {
    if let Some(object) = object {
        text::set_text(world, object, Some(value));
    }
}

/// `[logic setBehavior:behavior]` (nothing for a nil logic).
fn set_behavior(world: &mut World, logic: Option<Entity>, behavior: &str) {
    if let Some(logic) = logic {
        pg_engine::logic::set_behavior(world, logic, Some(behavior));
    }
}

/// `[[BCButtonLogic alloc] initWithDisplayObject:]` with this object as its listener.
fn spawn_button_logic(world: &mut World, this: Entity, display_object: Entity) -> Entity {
    let button = pg_engine::logic::button::spawn(world, display_object);
    pg_engine::logic::button::set_button_logic_listener(world, button, Some(this));
    button
}

/// `[mBonusPackManagerListener onHideKeyboard]`.
fn notify_hide_keyboard(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    if let Some(listener) = listener {
        controller::on_hide_keyboard(world, listener);
    }
}

/// `if (mbKeyboardOpen) { mbKeyboardOpen = NO; [listener onHideKeyboard]; }`.
fn hide_open_keyboard(world: &mut World, this: Entity) {
    let keyboard_open = world.obj::<BonusPackManager>(this).keyboard_open;
    if !keyboard_open {
        return;
    }
    world.obj_mut::<BonusPackManager>(this).keyboard_open = false;
    notify_hide_keyboard(world, this);
}

/// `[mBonusPackManagerListener onShowKeyboard:self editingText:code cancelEditDisplayObject:nil
/// maxEditCharCount:30 preventEmpty:YES]`.
fn notify_show_keyboard(world: &mut World, this: Entity, code: Option<Entity>) {
    let listener = listener(world, this);
    let Some(listener) = listener else {
        return;
    };

    controller::on_show_keyboard(world, listener, this, code, None, MAX_CODE_CHAR_COUNT, true);
}

/// `-[BonusPackManager init]` @ 0x1000caaac
pub fn init(world: &mut World, this: Entity) {
    world.entity_mut(this).insert(BonusPackManager::default());

    let mut singletons = world.get_resource_or_insert_with(Singletons::default);
    singletons.bonus_pack_manager = Some(this);

    // The logic array and the seven UI arrays start empty (`initWithCapacity:5`).
    let manager_sequence = sequence::spawn(world, None, Some(this));
    world.obj_mut::<BonusPackManager>(this).sequence = Some(manager_sequence);

    init_bonus_pack_defs(world, this);
}

/// `-[BonusPackManager id]` @ 0x1000cac90
pub fn id(_world: &World, _this: Entity) -> Option<String> {
    Some(CLASS_ID.to_owned())
}

/// `-[BonusPackManager setBonusPackManagerListener:]` @ 0x1000cac9c
pub fn set_bonus_pack_manager_listener(world: &mut World, this: Entity, bonus_pack_manager_listener: Entity) {
    world.obj_mut::<BonusPackManager>(this).bonus_pack_manager_listener = Some(bonus_pack_manager_listener);
}

/// `-[BonusPackManager isBonusPackUnlocked:]` @ 0x1000cacf8
pub fn is_bonus_pack_unlocked(world: &mut World, _this: Entity, bonus_pack_id: &str) -> bool {
    let def = &BONUS_PACK_DEFS[COMIC_PRE_ORDER_BONUS_PACK_DEF];
    let is_comic_pre_order = def.id == bonus_pack_id;
    let unlocked = is_def_unlocked(world, COMIC_PRE_ORDER_BONUS_PACK_DEF);
    is_comic_pre_order && unlocked
}

/// `-[BonusPackManager initBonusPackDefs]` @ 0x1000cad40
pub fn init_bonus_pack_defs(world: &mut World, _this: Entity) {
    let def = &BONUS_PACK_DEFS[COMIC_PRE_ORDER_BONUS_PACK_DEF];
    statics_mut(world).unlocked[COMIC_PRE_ORDER_BONUS_PACK_DEF] = false;

    let key_chain = prefs::key_chain_with_key_using_app_name(world, prefs::KEY_CHAIN_APP_NAME, def.id);
    statics_mut(world).key_chains[COMIC_PRE_ORDER_BONUS_PACK_DEF] = Some(key_chain);

    // PORT: the pack is owned from the first launch (see BONUS_PACKS_OWNED_FROM_FIRST_LAUNCH).
    let secret = prefs::key_chain_secret(world, key_chain);
    let unlocked = secret == prefs::PURCHASED_SECRET || BONUS_PACKS_OWNED_FROM_FIRST_LAUNCH;
    if unlocked {
        statics_mut(world).unlocked[COMIC_PRE_ORDER_BONUS_PACK_DEF] = true;
    }
}

/// `-[BonusPackManager resetKeyChainsForAllBonusPackDefs]` @ 0x1000cade8
pub fn reset_key_chains_for_all_bonus_pack_defs(world: &mut World, _this: Entity) {
    let key_chain = def_key_chain(world, COMIC_PRE_ORDER_BONUS_PACK_DEF);
    if let Some(key_chain) = key_chain {
        prefs::key_chain_reset(world, key_chain);
    }
}

/// `-[BonusPackManager releaseKeyChainsForAllBonusPackDefs]` @ 0x1000cae04
pub fn release_key_chains_for_all_bonus_pack_defs(world: &mut World, _this: Entity) {
    // `BCKeyChain`'s object cache keeps the item itself.
    statics_mut(world).key_chains[COMIC_PRE_ORDER_BONUS_PACK_DEF] = None;
}

/// `-[BonusPackManager dealloc]` @ 0x1000cae3c
pub fn dealloc(world: &mut World, this: Entity) {
    release_key_chains_for_all_bonus_pack_defs(world, this);

    // The arrays are the logics' only owners, but their display objects still point at them,
    // so the logics are kept alive.
    let manager_sequence = world.obj::<BonusPackManager>(this).sequence;
    if let Some(manager_sequence) = manager_sequence {
        sequence::stop(world, manager_sequence);
        world.despawn(manager_sequence);
    }

    world.despawn(this);
}

/// `-[BonusPackManager openBonusPackScreen:]` @ 0x1000caf84
pub fn open_bonus_pack_screen(world: &mut World, this: Entity, first_time: bool) {
    world.obj_mut::<BonusPackManager>(this).bonus_pack_screen_active = true;

    let sequence_id = if first_time {
        OPEN_BONUS_PACK_SCREEN_FIRST_TIME_SEQUENCE
    } else {
        OPEN_BONUS_PACK_SCREEN_SEQUENCE
    };
    set_sequence(world, this, sequence_id);
}

/// `-[BonusPackManager closeBonusPackScreen]` @ 0x1000cb010
pub fn close_bonus_pack_screen(world: &mut World, this: Entity) {
    world.obj_mut::<BonusPackManager>(this).bonus_pack_screen_active = false;

    let preview = world.obj::<BonusPackManager>(this).bonus_pack_detail_preview_item_logic;
    set_behavior(world, preview, BONUS_PACK_STOP_IDLE_BEHAVIOR);
    set_sequence(world, this, CLOSE_BONUS_PACK_SCREEN_SEQUENCE);

    let listener = listener(world, this);
    if let Some(listener) = listener {
        controller::on_bonus_pack_screen_closed(world, listener);
    }
}

/// `-[BonusPackManager checkBonusPackUnlock:score:]` @ 0x1000cb0c8
pub fn check_bonus_pack_unlock(world: &mut World, this: Entity, challenge_type: i32, score: i32) -> bool {
    let def = &BONUS_PACK_DEFS[COMIC_PRE_ORDER_BONUS_PACK_DEF];
    let is_unlock_type = def.unlock_challenge_type == challenge_type;
    let locked = !is_def_unlocked(world, COMIC_PRE_ORDER_BONUS_PACK_DEF);
    let unlocks = is_unlock_type && locked && def.unlock_score <= score;
    if !unlocks {
        return false;
    }

    world.obj_mut::<BonusPackManager>(this).unlock_bonus_pack_def = Some(COMIC_PRE_ORDER_BONUS_PACK_DEF);
    statics_mut(world).unlocked[COMIC_PRE_ORDER_BONUS_PACK_DEF] = true;

    let key_chain = def_key_chain(world, COMIC_PRE_ORDER_BONUS_PACK_DEF);
    if let Some(key_chain) = key_chain {
        prefs::key_chain_set_secret(world, key_chain, prefs::PURCHASED_SECRET);
    }

    set_sequence(world, this, OPEN_BONUS_PACK_UNLOCK_DIALOG_SEQUENCE);
    world.obj_mut::<BonusPackManager>(this).bonus_pack_unlock_dialog_active = true;

    let listener = listener(world, this);
    if let Some(listener) = listener {
        controller::on_bonus_pack_unlock_dialog_opened(world, listener);
    }
    true
}

/// `-[BonusPackManager activateBonusPackDef:]` @ 0x1000cb1c4
pub fn activate_bonus_pack_def(world: &mut World, this: Entity, bonus_pack_def: Option<usize>) {
    world.obj_mut::<BonusPackManager>(this).active_bonus_pack_def = bonus_pack_def;
    set_sequence(world, this, OPEN_BONUS_PACK_SCREEN_DETAIL_SEQUENCE);
}

/// `-[BonusPackManager updateListPanel]` @ 0x1000cb238
pub fn update_list_panel(world: &mut World, this: Entity) {
    world.obj_mut::<BonusPackManager>(this).bonus_pack_screen_mode = SCREEN_MODE_LIST;

    let panel = library::display_group(world, BONUS_PACK_LIST_PANEL);
    let next_button = group_object(world, panel, "BonusPackNextButton");
    let prev_button = group_object(world, panel, "BonusPackPrevButton");
    set_selectable(world, prev_button, false);
    hide(world, prev_button);
    set_selectable(world, next_button, false);
    hide(world, next_button);

    for row in 0..5 {
        let pack_name_id = format!("PackName{row}");
        let pack_name = group_object(world, panel, &pack_name_id);
        let unlocked_id = format!("Unlocked{row}");
        let unlocked = group_object(world, panel, &unlocked_id);
        let blackbox_id = format!("Blackbox{row}");
        let blackbox = group_object(world, panel, &blackbox_id);
        let view_button_id = format!("ViewButton{row}");
        let view_button = group_object(world, panel, &view_button_id);

        if row >= BONUS_PACK_LIST.len() {
            hide(world, pack_name);
            set_selectable(world, blackbox, false);
            hide(world, blackbox);
            hide(world, view_button);
            hide(world, unlocked);
            continue;
        }

        let def = BONUS_PACK_LIST[row];
        set_text(world, pack_name, BONUS_PACK_DEFS[def].name);
        show(world, pack_name);
        set_selectable(world, blackbox, true);
        show(world, blackbox);
        show(world, view_button);

        let def_unlocked = is_def_unlocked(world, def);
        if def_unlocked {
            show(world, unlocked);
        } else {
            hide(world, unlocked);
        }
    }
}

/// `-[BonusPackManager updateDetailPanel]` @ 0x1000cb4f8
pub fn update_detail_panel(world: &mut World, this: Entity) {
    world.obj_mut::<BonusPackManager>(this).bonus_pack_screen_mode = SCREEN_MODE_DETAIL;

    let panel = library::display_group(world, BONUS_PACK_DETAIL_PANEL);
    let pack_name = group_object(world, panel, "PackName");
    let detail_text = group_object(world, panel, "DetailText");
    let unlock_button = group_object(world, panel, "UnlockButton");

    // A nil def (list rows 1–4, never selectable) would crash the original here.
    let active_def = world.obj::<BonusPackManager>(this).active_bonus_pack_def;
    let Some(active_def) = active_def else {
        return;
    };

    let def_unlocked = is_def_unlocked(world, active_def);
    if def_unlocked {
        set_selectable(world, unlock_button, false);
        hide(world, unlock_button);
    } else {
        set_selectable(world, unlock_button, true);
        show(world, unlock_button);
    }

    let def = &BONUS_PACK_DEFS[active_def];
    set_text(world, pack_name, def.name);
    set_text(world, detail_text, def.detail_text);

    let preview = world.obj::<BonusPackManager>(this).bonus_pack_detail_preview_item_logic;
    set_behavior(world, preview, def.preview_behavior);
}

/// `-[BonusPackManager updateUnlockPanel]` @ 0x1000cb664
pub fn update_unlock_panel(world: &mut World, this: Entity) {
    world.obj_mut::<BonusPackManager>(this).bonus_pack_screen_mode = SCREEN_MODE_UNLOCK;

    let panel = library::display_group(world, BONUS_PACK_UNLOCK_PANEL);
    let message = group_object(world, panel, "Message");
    let code = group_object(world, panel, "Code");
    set_text(world, message, "Enter Code");
    set_text(world, code, "");

    world.obj_mut::<BonusPackManager>(this).keyboard_open = true;
    notify_show_keyboard(world, this, code);
}

/// `-[BonusPackManager updateInfoPanel]` @ 0x1000cb774
pub fn update_info_panel(world: &mut World, this: Entity) {
    world.obj_mut::<BonusPackManager>(this).bonus_pack_screen_mode = SCREEN_MODE_INFO;
}

/// `-[BonusPackManager updateNoKeyChainPanel]` @ 0x1000cb788
pub fn update_no_key_chain_panel(world: &mut World, this: Entity) {
    world.obj_mut::<BonusPackManager>(this).bonus_pack_screen_mode = SCREEN_MODE_NO_KEY_CHAIN;

    let panel = library::display_group(world, BONUS_PACK_ERROR_PANEL);
    let error_header = group_object(world, panel, "ErrorHeader");
    let error1 = group_object(world, panel, "Error1");
    let error2 = group_object(world, panel, "Error2");
    set_text(world, error_header, "Something is wrong with your keychain services!");
    set_text(world, error1, "To unlock Bonus Packs you must have a keychain.");
    set_text(world, error2, "");
}

/// `-[BonusPackManager updateUnlockDialog]` @ 0x1000cb880
pub fn update_unlock_dialog(world: &mut World, this: Entity) {
    let dialog = library::display_group(world, BONUS_PACK_UNLOCK_DIALOG);
    let bonus_pack_text = group_object(world, dialog, "BonusPackText");

    // Only `checkBonusPackUnlock:score:` opens the dialog, after setting the def.
    let unlock_def = world.obj::<BonusPackManager>(this).unlock_bonus_pack_def;
    let Some(unlock_def) = unlock_def else {
        return;
    };

    let def = &BONUS_PACK_DEFS[unlock_def];
    let message = format!("You unlocked the {}\nBonus Pack!", def.name);
    set_text(world, bonus_pack_text, &message);

    let preview = world.obj::<BonusPackManager>(this).bonus_pack_unlock_dialog_preview_item_logic;
    set_behavior(world, preview, def.preview_behavior);
}

/// `-[BonusPackManager updateListPanel:::]` @ 0x1000cb964
pub fn seq_update_list_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_list_panel(world, this);
    ItemResult::Done
}

/// `-[BonusPackManager updateDetailPanel:::]` @ 0x1000cb988
pub fn seq_update_detail_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_detail_panel(world, this);
    ItemResult::Done
}

/// `-[BonusPackManager updateUnlockPanel:::]` @ 0x1000cb9ac
pub fn seq_update_unlock_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_unlock_panel(world, this);
    ItemResult::Done
}

/// `-[BonusPackManager updateInfoPanel:::]` @ 0x1000cb9d0
pub fn seq_update_info_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_info_panel(world, this);
    ItemResult::Done
}

/// `-[BonusPackManager updateNoKeyChainPanel:::]` @ 0x1000cb9f4
pub fn seq_update_no_key_chain_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_no_key_chain_panel(world, this);
    ItemResult::Done
}

/// `-[BonusPackManager updateUnlockDialog:::]` @ 0x1000cba18
pub fn seq_update_unlock_dialog(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_unlock_dialog(world, this);
    ItemResult::Done
}

/// `-[BonusPackManager loadLocationTextureAtlas:::]` @ 0x1000cba3c
pub fn seq_load_location_texture_atlas(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener(world, this);
    if let Some(listener) = listener {
        controller::on_load_location_texture_atlas(world, listener);
    }
    ItemResult::Done
}

/// `-[BonusPackManager onKeyboardTextEntered:]` @ 0x1000cba6c
pub fn on_keyboard_text_entered(world: &mut World, this: Entity, entered_text: Option<&str>) {
    let active_def = world.obj::<BonusPackManager>(this).active_bonus_pack_def;

    // `[nil caseInsensitiveCompare:]` is NSOrderedSame, so nil text counts as the right code.
    // A nil def only exists for list rows 1–4, which are never selectable.
    let correct_code = match (entered_text, active_def) {
        (Some(entered_text), Some(active_def)) => entered_text.eq_ignore_ascii_case(BONUS_PACK_DEFS[active_def].code),
        (None, _) => true,
        (Some(_), None) => false,
    };

    if !correct_code {
        let panel = library::display_group(world, BONUS_PACK_UNLOCK_PANEL);
        let message = group_object(world, panel, "Message");
        let code = group_object(world, panel, "Code");
        set_text(world, message, "Incorrect Code");

        world.obj_mut::<BonusPackManager>(this).keyboard_open = true;
        notify_show_keyboard(world, this, code);
        return;
    }

    world.obj_mut::<BonusPackManager>(this).keyboard_open = false;
    if let Some(active_def) = active_def {
        statics_mut(world).unlocked[active_def] = true;

        let key_chain = def_key_chain(world, active_def);
        if let Some(key_chain) = key_chain {
            prefs::key_chain_set_secret(world, key_chain, prefs::PURCHASED_SECRET);
        }
    }
    set_sequence(world, this, CLOSE_BONUS_PACK_SCREEN_UNLOCK_SEQUENCE);
}

/// `-[BonusPackManager onButtonPress:touch:]` @ 0x1000cbc14
pub fn on_button_press(world: &mut World, this: Entity, button: Entity, _touch: Entity) {
    let manager = world.obj::<BonusPackManager>(this).clone();
    let pressed = Some(button);

    if manager.bonus_pack_close_button_logic == pressed {
        hide_open_keyboard(world, this);
        close_bonus_pack_screen(world, this);
        return;
    }

    if manager.bonus_pack_info_button_logic == pressed {
        hide_open_keyboard(world, this);
        let screen_mode = world.obj::<BonusPackManager>(this).bonus_pack_screen_mode;
        let sequence_id = if screen_mode == SCREEN_MODE_INFO {
            CLOSE_BONUS_PACK_SCREEN_INFO_SEQUENCE
        } else {
            OPEN_BONUS_PACK_SCREEN_INFO_SEQUENCE
        };
        set_sequence(world, this, sequence_id);
        return;
    }

    if manager.bonus_pack_info_close_button_logic == pressed {
        set_sequence(world, this, CLOSE_BONUS_PACK_SCREEN_INFO_SEQUENCE);
        return;
    }

    // The list's previous and next buttons only log ("BONUS PACK PREV:" / "BONUS PACK NEXT:").
    if manager.bonus_pack_list_prev_button_logic == pressed || manager.bonus_pack_list_next_button_logic == pressed {
        return;
    }

    if manager.bonus_pack_list_item0_button_logic == pressed {
        activate_bonus_pack_def(world, this, Some(COMIC_PRE_ORDER_BONUS_PACK_DEF));
        return;
    }

    let other_list_items = [
        manager.bonus_pack_list_item1_button_logic,
        manager.bonus_pack_list_item2_button_logic,
        manager.bonus_pack_list_item3_button_logic,
        manager.bonus_pack_list_item4_button_logic,
    ];
    if other_list_items.contains(&pressed) {
        // PORT: rows 1–4 pass slots 1–4 of the list table (0, 0, 0x28 and a code pointer), none
        // of them a def; the rows are never selectable. Nil stands in for all four.
        activate_bonus_pack_def(world, this, None);
        return;
    }

    if manager.bonus_pack_detail_unlock_button_logic == pressed {
        let key_chain = match manager.active_bonus_pack_def {
            Some(active_def) => def_key_chain(world, active_def),
            None => None,
        };
        let has_error = match key_chain {
            Some(key_chain) => prefs::key_chain_has_error(world, key_chain),
            None => false,
        };

        // The keychain error path also logs "NO KEYCHAIN:".
        let sequence_id = if has_error {
            OPEN_BONUS_PACK_SCREEN_NO_KEY_CHAIN_SEQUENCE
        } else {
            OPEN_BONUS_PACK_SCREEN_UNLOCK_SEQUENCE
        };
        set_sequence(world, this, sequence_id);
        return;
    }

    if manager.bonus_pack_detail_detail_button_logic == pressed {
        let url = match manager.active_bonus_pack_def {
            Some(active_def) => BONUS_PACK_DEFS[active_def].url,
            None => None,
        };
        let Some(url) = url else {
            return;
        };

        let listener = listener(world, this);
        if let Some(listener) = listener {
            controller::open_url(world, listener, url);
        }
        return;
    }

    if manager.bonus_pack_detail_back_button_logic == pressed {
        set_sequence(world, this, OPEN_BONUS_PACK_SCREEN_LIST_SEQUENCE);
        return;
    }

    if manager.bonus_pack_unlock_cancel_button_logic == pressed {
        world.obj_mut::<BonusPackManager>(this).keyboard_open = false;
        notify_hide_keyboard(world, this);
        set_sequence(world, this, CLOSE_BONUS_PACK_SCREEN_UNLOCK_SEQUENCE);
        return;
    }

    if manager.bonus_pack_error_close_button_logic == pressed {
        set_sequence(world, this, OPEN_BONUS_PACK_SCREEN_ERROR_HIDE_SEQUENCE);
        return;
    }

    if manager.bonus_pack_unlock_dialog_close_button_logic != pressed {
        return;
    }

    world.obj_mut::<BonusPackManager>(this).bonus_pack_unlock_dialog_active = false;
    let preview = world.obj::<BonusPackManager>(this).bonus_pack_unlock_dialog_preview_item_logic;
    set_behavior(world, preview, BONUS_PACK_STOP_IDLE_BEHAVIOR);
    set_sequence(world, this, CLOSE_BONUS_PACK_UNLOCK_DIALOG_SEQUENCE);

    let listener = listener(world, this);
    if let Some(listener) = listener {
        controller::on_bonus_pack_unlock_dialog_closed(world, listener);
    }
}

/// `-[BonusPackManager onButtonRelease:touch:]` @ 0x1000cc1b8
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[BonusPackManager onButtonReleaseOutside:touch:]` @ 0x1000cc1bc
pub fn on_button_release_outside(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[BonusPackManager setLogic:]` @ 0x1000cc1c0: the boot script's `setLogic` items for this
/// manager (`call` carries the `BCSequenceItemDef`: `displayGroupId`, `logicId`,
/// `displayObjectId`).
pub fn set_logic(world: &mut World, this: Entity, call: &SequenceCall) {
    let display_group_id = call.param("displayGroupId");
    let logic_id = call.param("logicId");
    let display_object_id = call.param("displayObjectId");

    let display_group = match display_group_id {
        Some(display_group_id) => library::display_group(world, display_group_id),
        None => None,
    };
    let display_object = match display_object_id {
        Some(display_object_id) => group_object(world, display_group, display_object_id),
        None => None,
    };

    // A nil id matches nothing.
    let Some(logic_id) = logic_id else {
        return;
    };

    if let Some(slot) = preview_logic_slot(logic_id) {
        // PORT: the original also creates the logic on a nil display object; the engine's
        // logics need one, so nothing is created. Every `setLogic` item of the boot script
        // names an existing object.
        let Some(display_object) = display_object else {
            return;
        };

        let preview = pg_engine::logic::multi_model::spawn(world, display_object);
        let mut manager = world.obj_mut::<BonusPackManager>(this);
        *slot(&mut manager) = Some(preview);
        manager.logic_array.push(preview);
        return;
    }

    let Some((slot, ui_array)) = button_logic_slot(logic_id) else {
        return;
    };
    let Some(display_object) = display_object else {
        return;
    };

    let button = spawn_button_logic(world, this, display_object);
    let mut manager = world.obj_mut::<BonusPackManager>(this);
    *slot(&mut manager) = Some(button);
    ui_array(&mut manager).push(button);
    manager.logic_array.push(button);
}

/// `+[BonusPackManager instance]` @ 0x1000caaa0
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().bonus_pack_manager
}
