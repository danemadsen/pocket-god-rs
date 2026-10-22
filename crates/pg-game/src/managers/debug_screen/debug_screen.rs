//! `DebugScreen`: methods in this part of the class.

use std::sync::OnceLock;

use pg_engine::display::{self, group, text};
use pg_engine::library;
use pg_engine::sequence::{self, BcSequence};

use crate::controller;
use crate::logic_table;
use crate::prelude::*;
use super::*;

/// `@"DebugScreen"` (`0x100386350`): the object's id and the screen's display group.
const DEBUG_SCREEN_ID: &str = "DebugScreen";

/// `@"OpenDebugScreen"` (`0x100386370`).
const OPEN_DEBUG_SCREEN_SEQUENCE: &str = "OpenDebugScreen";

/// `@"CloseDebugScreen"` (`0x100386390`).
const CLOSE_DEBUG_SCREEN_SEQUENCE: &str = "CloseDebugScreen";

/// `@"UpdateDebugItems"` (`0x100386450`).
const UPDATE_DEBUG_ITEMS_SEQUENCE: &str = "UpdateDebugItems";

// The page arrows of the `DebugScreen` group.
const NEXT_PAGE: &str = "NextPage";
const PREV_PAGE: &str = "PrevPage";

/// Item rows per page (`0xf`).
const DEBUG_ITEMS_PER_PAGE: i32 = 15;

/// The 15 item buttons, `getLogic:0x15b` … `getLogic:0x169` in `mDebugItemButtonLogicArray`
/// order.
const DEBUG_ITEM_BUTTON_LOGICS: [usize; 15] = [
    logic_table::BLACK_BOX0,
    logic_table::BLACK_BOX1,
    logic_table::BLACK_BOX2,
    logic_table::BLACK_BOX3,
    logic_table::BLACK_BOX4,
    logic_table::BLACK_BOX5,
    logic_table::BLACK_BOX6,
    logic_table::BLACK_BOX7,
    logic_table::BLACK_BOX8,
    logic_table::BLACK_BOX9,
    logic_table::BLACK_BOX10,
    logic_table::BLACK_BOX11,
    logic_table::BLACK_BOX12,
    logic_table::BLACK_BOX13,
    logic_table::BLACK_BOX14,
];

/// `-[DebugScreen init]` @ 0x100008638
pub fn init(world: &mut World, this: Entity) {
    world.entity_mut(this).insert(DebugScreen::default());

    world.get_resource_or_insert_with(Singletons::default).debug_screen = Some(this);

    let debug_sequence = sequence::spawn(world, None, Some(this));
    let close_button_logic = library::logic(world, logic_table::DEBUG_SCREEN_CLOSE_BUTTON);
    let next_button_logic = library::logic(world, logic_table::NEXT_PAGE);
    let prev_button_logic = library::logic(world, logic_table::PREV_PAGE);

    let mut item_button_logics = Vec::with_capacity(5);
    for slot in DEBUG_ITEM_BUTTON_LOGICS {
        let item_button_logic = library::logic(world, slot);
        item_button_logics.push(item_button_logic);
    }

    let mut debug_screen = world.obj_mut::<DebugScreen>(this);
    debug_screen.sequence = Some(debug_sequence);
    debug_screen.debug_close_button_logic = Some(close_button_logic);
    debug_screen.debug_next_button_logic = Some(next_button_logic);
    debug_screen.debug_prev_button_logic = Some(prev_button_logic);
    debug_screen.debug_item_button_logic_array = item_button_logics;
}

/// `-[DebugScreen id]` @ 0x100008a40
pub fn id(_world: &World, _this: Entity) -> Option<String> {
    Some(DEBUG_SCREEN_ID.to_owned())
}

/// `-[DebugScreen setDebugScreenListener:]` @ 0x100008a4c
pub fn set_debug_screen_listener(world: &mut World, this: Entity, debug_screen_listener: Entity) {
    world.obj_mut::<DebugScreen>(this).debug_screen_listener = Some(debug_screen_listener);
}

/// `-[DebugScreen dealloc]` @ 0x100008aa8
///
/// Only the controller's own `dealloc` releases the debug screen, so this never runs in the game.
pub fn dealloc(world: &mut World, this: Entity) {
    let debug_sequence = world.obj::<DebugScreen>(this).sequence;

    // `[mSequence release]` without `stop`: a running sequence lives on in the view's
    // game-frame listener array, which retains it; an idle one is freed.
    if let Some(debug_sequence) = debug_sequence {
        let running = world.obj::<BcSequence>(debug_sequence).is_running;
        if !running {
            world.despawn(debug_sequence);
        }
    }

    world.despawn(this);
}

/// `-[DebugScreen openDebugScreen]` @ 0x100008b38
pub fn open_debug_screen(world: &mut World, this: Entity) {
    world.obj_mut::<DebugScreen>(this).debug_screen_active = true;
    set_sequence_def(world, this, OPEN_DEBUG_SCREEN_SEQUENCE);
}

/// `-[DebugScreen closeDebugScreen]` @ 0x100008bb0
pub fn close_debug_screen(world: &mut World, this: Entity) {
    world.obj_mut::<DebugScreen>(this).debug_screen_active = false;
    set_sequence_def(world, this, CLOSE_DEBUG_SCREEN_SEQUENCE);

    let listener = world.obj::<DebugScreen>(this).debug_screen_listener;
    if let Some(listener) = listener {
        controller::on_debug_screen_closed(world, listener);
    }
}

/// `-[DebugScreen countDebugItems]` @ 0x100008c44
pub fn count_debug_items(_world: &mut World, _this: Entity) -> i32 {
    debug_item_names().len() as i32
}

/// `-[DebugScreen updateDebugItems]` @ 0x100008c74
pub fn update_debug_items(world: &mut World, this: Entity) {
    let debug_display_group = library::display_group(world, DEBUG_SCREEN_ID);
    let debug_item_count = count_debug_items(world, this);
    let next_page = display_group_object(world, debug_display_group, NEXT_PAGE);
    let prev_page = display_group_object(world, debug_display_group, PREV_PAGE);

    set_selectable_and_shown(world, next_page, false);
    set_selectable_and_shown(world, prev_page, false);

    let mut debug_item_menu_index = world.obj::<DebugScreen>(this).debug_item_menu_index;
    if 0 < debug_item_menu_index {
        set_selectable_and_shown(world, prev_page, true);
        debug_item_menu_index = world.obj::<DebugScreen>(this).debug_item_menu_index;
    }
    if debug_item_menu_index + DEBUG_ITEMS_PER_PAGE < debug_item_count {
        set_selectable_and_shown(world, next_page, true);
    }

    let mut row = 0;
    loop {
        let option_button_id = format!("Option{row}Button");
        let option_button = display_group_object(world, debug_display_group, &option_button_id);
        let black_box_id = format!("BlackBox{row}");
        let black_box = display_group_object(world, debug_display_group, &black_box_id);
        let Some(option_button) = option_button else {
            break;
        };

        let menu_index = world.obj::<DebugScreen>(this).debug_item_menu_index;
        let item_index = row + menu_index;
        if item_index < debug_item_count {
            let item_name = debug_item_names()[item_index as usize].as_str();
            text::set_text(world, option_button, Some(item_name));
            set_selectable_and_shown(world, black_box, true);
        } else {
            text::set_text(world, option_button, Some(""));
            set_selectable_and_shown(world, black_box, false);
        }
        row += 1;
    }
}

/// `-[DebugScreen selectDebugItemIndex:]` @ 0x100008f2c
///
/// Empty in the original: no debug item does anything.
pub fn select_debug_item_index(_world: &mut World, _this: Entity, _select_debug_item_index: i32) {}

/// `-[DebugScreen updateDebugItemPage:]` @ 0x100008f30
pub fn update_debug_item_page(world: &mut World, this: Entity, update_debug_item_page: i32) {
    let debug_item_count = count_debug_items(world, this);

    {
        let mut debug_screen = world.obj_mut::<DebugScreen>(this);
        let mut menu_index = debug_screen.debug_item_menu_index + update_debug_item_page * DEBUG_ITEMS_PER_PAGE;
        if debug_item_count <= menu_index {
            menu_index = (debug_item_count / DEBUG_ITEMS_PER_PAGE) * DEBUG_ITEMS_PER_PAGE;
        }
        debug_screen.debug_item_menu_index = menu_index;
        if debug_screen.debug_item_menu_index < 0 {
            debug_screen.debug_item_menu_index = 0;
        }
    }

    set_sequence_def(world, this, UPDATE_DEBUG_ITEMS_SEQUENCE);
}

/// `-[DebugScreen waitForDataUpdate:::]` @ 0x100009008
pub fn seq_wait_for_data_update(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<DebugScreen>(this).debug_screen_listener;
    let challenge_preferences = match listener {
        Some(listener) => controller::challenge_preferences(world, listener),
        None => None,
    };
    let waiting_to_save = match challenge_preferences {
        Some(challenge_preferences) => pg_engine::prefs::waiting_to_save(world, challenge_preferences),
        None => false,
    };

    if waiting_to_save {
        ItemResult::Pending
    } else {
        ItemResult::Done
    }
}

/// `-[DebugScreen exitApp:::]` @ 0x100009050
///
/// Does not exit the app.
pub fn seq_exit_app(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    ItemResult::Done
}

/// `-[DebugScreen updateDebugItems:::]` @ 0x100009058
pub fn seq_update_debug_items(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_debug_items(world, this);
    ItemResult::Done
}

/// `-[DebugScreen loadLocationTextureAtlas:::]` @ 0x10000907c
pub fn seq_load_location_texture_atlas(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<DebugScreen>(this).debug_screen_listener;
    if let Some(listener) = listener {
        controller::on_load_location_texture_atlas(world, listener);
    }
    ItemResult::Done
}

/// `-[DebugScreen onButtonPress:touch:]` @ 0x1000090ac
pub fn on_button_press(world: &mut World, this: Entity, button: Entity, _touch: Entity) {
    let debug_screen = world.obj::<DebugScreen>(this).clone();
    if debug_screen.exit_app {
        return;
    }

    let pressed = Some(button);

    if debug_screen.debug_close_button_logic == pressed {
        close_debug_screen(world, this);
        return;
    }

    let item_button_logics = &debug_screen.debug_item_button_logic_array;
    let item_position = item_button_logics.iter().position(|logic| *logic == button);
    if let Some(item_position) = item_position {
        let item_index = debug_screen.debug_item_menu_index + item_position as i32;
        select_debug_item_index(world, this, item_index);
        return;
    }

    if debug_screen.debug_next_button_logic == pressed {
        update_debug_item_page(world, this, 1);
    } else if debug_screen.debug_prev_button_logic == pressed {
        update_debug_item_page(world, this, -1);
    }
}

/// `-[DebugScreen onButtonRelease:touch:]` @ 0x1000091d4
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[DebugScreen onButtonReleaseOutside:touch:]` @ 0x1000091d8
pub fn on_button_release_outside(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `+[DebugScreen instance]` @ 0x10000862c
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().debug_screen
}

/// The item names of the nil-terminated record list at `0x100730938` (the string at +8 of each
/// record; `debug-items.json`), parsed once.
fn debug_item_names() -> &'static [String] {
    static NAMES: OnceLock<Vec<String>> = OnceLock::new();
    NAMES.get_or_init(load_debug_item_names)
}

fn load_debug_item_names() -> Vec<String> {
    let debug_items = pg_assets::data::debug_items::load();
    let debug_items = debug_items.expect("assets/data/debug-items.json matches its schema");
    debug_items.items
}

/// `[mSequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:sequence_id]]`.
fn set_sequence_def(world: &mut World, this: Entity, sequence_id: &str) {
    let debug_sequence = world.obj::<DebugScreen>(this).sequence;
    let sequence_def = library::sequence_def(world, sequence_id);
    if let Some(debug_sequence) = debug_sequence {
        sequence::set_sequence_def(world, debug_sequence, sequence_def);
    }
}

/// `[group getDisplayObject:id]` on a group that may be nil.
fn display_group_object(world: &World, display_group: Option<Entity>, id: &str) -> Option<Entity> {
    let display_group = display_group?;
    group::get_display_object(world, display_group, id)
}

/// `[object setIsSelectable:shown]` followed by `[object show]` or `[object hide]` (nothing for
/// nil).
fn set_selectable_and_shown(world: &mut World, object: Option<Entity>, shown: bool) {
    let Some(object) = object else {
        return;
    };
    let mut display_object = display::object_mut(world, object);
    display_object.is_selectable = shown;
    display_object.hidden = !shown;
}
