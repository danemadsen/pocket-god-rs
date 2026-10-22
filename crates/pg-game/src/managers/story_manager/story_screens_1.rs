//! `StoryManager`: methods in this part of the class.
//!
//! The story screen (`docs/re/managers-platform.md` §6.4): the Watch, My List and Create panels,
//! the element editor with its option menus, the add menu, the confirm dialogs, the delays, text
//! entry and the button handling. The story objects and state lists it edits are those of
//! `story_data` ([`StoryDefStore`], the `STORY_*_ARRAY` lists).

use pg_engine::display::{self, group, text};
use pg_engine::library;
use pg_engine::logic;
use pg_engine::sequence;
use pg_engine::view;

use crate::controller;
use crate::managers::idol_manager;
use crate::prelude::*;

use super::*;

/// `mStoryScreenMode` of the Watch panel.
const WATCH_SCREEN_MODE: i32 = 0;

/// `mStoryScreenMode` of the My List panel.
const MY_LIST_SCREEN_MODE: i32 = 1;

/// `mStoryScreenMode` of the Create panel.
const CREATE_SCREEN_MODE: i32 = 2;

/// `mStoryScreenMode` of the "Remove Story From Your List?" dialog.
const CLEAR_CONFIRM_SCREEN_MODE: i32 = 3;

/// `mStoryScreenMode` of the "Edit Story?" dialog.
const EDIT_MAIN_STORY_CONFIRM_SCREEN_MODE: i32 = 4;

/// `mStoryOptionMode`: the list the option menu shows.
const LOCATION_OPTION_MODE: i32 = 0;
const WEATHER_OPTION_MODE: i32 = 1;
const TIME_OF_DAY_OPTION_MODE: i32 = 2;
const CHARACTER_NAME_OPTION_MODE: i32 = 3;
const CHARACTER_STATE_OPTION_MODE: i32 = 4;
const EVENT_OPTION_MODE: i32 = 5;

/// `mStoryEditTextType`: the field the keyboard edits.
const TITLE_EDIT_TEXT_TYPE: i32 = 0;
const CAPTION_EDIT_TEXT_TYPE: i32 = 1;
const NAME_EDIT_TEXT_TYPE: i32 = 2;
const DIALOG_EDIT_TEXT_TYPE: i32 = 3;

/// The rows of the Watch and My List panels (`StoryTitle0` … `StoryTitle5`), also their page size.
const STORY_LIST_ROW_COUNT: i32 = 6;

/// `mOptionButtonCount` of `StoryOptionMenu`.
const OPTION_MENU_ROW_COUNT: i32 = 15;

/// `mOptionButtonCount` of `StoryWideOptionMenu`.
const WIDE_OPTION_MENU_ROW_COUNT: i32 = 10;

/// The wrap width of the caption text (`0x439071c7`).
const CAPTION_WRAP_WIDTH: f32 = 288.888_89;

/// The wrap width of the dialogue text (`0x438ae38e`).
const DIALOGUE_WRAP_WIDTH: f32 = 277.777_77;

/// The text object and the three markers of one row of the Watch or My List panel.
struct StoryListRow {
    title: Option<Entity>,
    arrow: Option<Entity>,
    black_box: Option<Entity>,
    edit_button: Option<Entity>,
}

/// `-[StoryManager viewStoryElementDef:]` @ 0x100049094
pub fn view_story_element_def(world: &mut World, this: Entity, element_index: i32) {
    delay_trigger_episode_event(world, "CreateStory");

    let story_def = active_story_def(world, this);
    let element_count = story_def_store(world).element_count(story_def);
    let shown_index = element_index.min(element_count).max(0);
    world.obj_mut::<StoryManager>(this).active_story_element_index = shown_index;

    let sequence_id = if shown_index == 0 {
        "OpenStoryCreateTitlePanel"
    } else {
        let element_type = story_element(world, story_def, shown_index - 1).element_type;
        match element_type {
            STORY_ENVIRONMENT_ELEMENT => "OpenStoryCreateEnvironmentPanel",
            STORY_CAPTION_ELEMENT => "OpenStoryCreateCaptionPanel",
            STORY_CHARACTER_ELEMENT => "OpenStoryCreateCharacterPanel",
            STORY_EVENT_ELEMENT => "OpenStoryCreateEventPanel",
            _ => return,
        }
    };
    set_sequence_def(world, this, sequence_id);
}

/// `-[StoryManager openStoryScreen]` @ 0x100049298
pub fn open_story_screen(world: &mut World, this: Entity) {
    {
        let mut manager = world.obj_mut::<StoryManager>(this);
        manager.main_story_index = 0;
        manager.my_list_story_index = 0;
        manager.story_screen_active = true;
    }

    view::add_game_frame_listener(world, this);
    delay_trigger_episode_event(world, "OpenTheStoryScreen");
    set_sequence_def(world, this, "OpenStoryScreen");
}

/// `-[StoryManager closeStoryScreen]` @ 0x10004937c
pub fn close_story_screen(world: &mut World, this: Entity) {
    {
        let mut manager = world.obj_mut::<StoryManager>(this);
        manager.option_menu_open = false;
        manager.story_screen_active = false;
    }

    check_for_unmodified_story_def(world, this);
    listener_on_hide_keyboard(world, this);
    view::remove_game_frame_listener(world, this);
    set_sequence_def(world, this, "CloseStoryScreen");

    let listener = world.obj::<StoryManager>(this).story_manager_listener;
    if let Some(listener) = listener {
        controller::on_story_screen_closed(world, listener);
    }
}

/// `-[StoryManager updateWatchPanel]` @ 0x100049548
pub fn update_watch_panel(world: &mut World, this: Entity) {
    set_story_screen_mode(world, this, WATCH_SCREEN_MODE);

    let manager = world.obj::<StoryManager>(this);
    let story_index = manager.main_story_index;
    let story_defs = manager.main_story_def_vector.clone();
    update_story_list_panel(world, "StoryWatchPanel", story_index, &story_defs);
}

/// `-[StoryManager updateMyListPanel]` @ 0x100049e70
pub fn update_my_list_panel(world: &mut World, this: Entity) {
    set_story_screen_mode(world, this, MY_LIST_SCREEN_MODE);

    let manager = world.obj::<StoryManager>(this);
    let story_index = manager.my_list_story_index;
    let story_defs = manager.my_list_story_def_vector.clone();
    update_story_list_panel(world, "StoryMyListPanel", story_index, &story_defs);
}

/// The body `updateWatchPanel` and `updateMyListPanel` share: the rows show the titles of
/// `story_defs[story_index…]` up to the end of the list; `PrevStory` and `NextStory` appear when
/// there is something to page to.
fn update_story_list_panel(world: &mut World, group_id: &str, story_index: i32, story_defs: &[usize]) {
    let panel = library::display_group(world, group_id);
    let mut rows = Vec::new();
    for row in 0..STORY_LIST_ROW_COUNT {
        let title_id = format!("StoryTitle{row}");
        let arrow_id = format!("Arrow{row}");
        let black_box_id = format!("BlackBox{row}");
        let edit_button_id = format!("EditStory{row}Button");
        let story_list_row = StoryListRow {
            title: group_object(world, panel, &title_id),
            arrow: group_object(world, panel, &arrow_id),
            black_box: group_object(world, panel, &black_box_id),
            edit_button: group_object(world, panel, &edit_button_id),
        };
        rows.push(story_list_row);
    }
    let prev_story = group_object(world, panel, "PrevStory");
    let next_story = group_object(world, panel, "NextStory");

    hide(world, prev_story);
    set_selectable(world, prev_story, false);
    hide(world, next_story);
    set_selectable(world, next_story, false);

    if story_index > 0 {
        show(world, prev_story);
        set_selectable(world, prev_story, true);
    }

    // `(unsigned long)index < count - 1`, tested only for a non-empty list.
    let story_count = story_defs.len() as u64;
    if story_count != 0 {
        let first_shown = story_index as i64 as u64;
        if first_shown < story_count - 1 {
            show(world, next_story);
            set_selectable(world, next_story, true);
        }
    }

    for row in &rows {
        set_text(world, row.title, Some(""));
        hide(world, row.arrow);
        hide(world, row.black_box);
        set_selectable(world, row.black_box, false);
        hide(world, row.edit_button);
        set_selectable(world, row.edit_button, false);
    }

    for (row_offset, row) in rows.iter().enumerate() {
        let position = story_index as usize + row_offset;
        let Some(story_def) = story_defs.get(position).copied() else {
            break;
        };
        let title = story_def_store(world).story_def(story_def).title.clone();
        set_text(world, row.title, title.as_deref());
        show(world, row.arrow);
        show(world, row.black_box);
        set_selectable(world, row.black_box, true);
        show(world, row.edit_button);
        set_selectable(world, row.edit_button, true);
    }
}

/// `-[StoryManager updateCreatePanel]` @ 0x10004a79c
pub fn update_create_panel(world: &mut World, this: Entity) {
    set_story_screen_mode(world, this, CREATE_SCREEN_MODE);

    let step_number = panel_object(world, "StoryCreatePanel", "StepNumber");
    let manager = world.obj::<StoryManager>(this);
    let next_node_button = manager.story_create_next_node_button_logic;
    let prev_node_button = manager.story_create_prev_node_button_logic;
    let delete_node_button = manager.story_add_menu_delete_node_button_logic;
    let move_up_button = manager.story_create_move_up_button_logic;
    let move_down_button = manager.story_create_move_down_button_logic;
    let element_index = manager.active_story_element_index;

    let next_node = button_display_object(world, next_node_button);
    hide(world, next_node);
    set_selectable(world, next_node, false);
    let prev_node = button_display_object(world, prev_node_button);
    hide(world, prev_node);
    set_selectable(world, prev_node, false);
    if element_index > 0 {
        show(world, prev_node);
        set_selectable(world, prev_node, true);
    }

    let story_def = active_story_def(world, this);
    let element_count = story_def_store(world).element_count(story_def);
    if element_index < element_count {
        show(world, next_node);
        set_selectable(world, next_node, true);
    }

    let delete_node = button_display_object(world, delete_node_button);
    hide(world, delete_node);
    set_selectable(world, delete_node, false);
    if element_index > 1 {
        show(world, delete_node);
        set_selectable(world, delete_node, true);
    }

    let move_up = button_display_object(world, move_up_button);
    hide(world, move_up);
    set_selectable(world, move_up, false);
    let move_down = button_display_object(world, move_down_button);
    hide(world, move_down);
    set_selectable(world, move_down, false);

    let step_text = if element_index == 0 {
        String::new()
    } else {
        if element_index > 1 {
            if element_index < element_count {
                show(world, move_up);
                set_selectable(world, move_up, true);
            }
            if element_index > 2 {
                show(world, move_down);
                set_selectable(world, move_down, true);
            }
        }

        let element_type = story_element(world, story_def, element_index - 1).element_type;
        let element_kind = match element_type {
            STORY_ENVIRONMENT_ELEMENT => "ENVIRONMENT",
            STORY_CAPTION_ELEMENT => "CAPTION",
            STORY_CHARACTER_ELEMENT => "CHARACTER",
            STORY_EVENT_ELEMENT => "EVENT",
            _ => return,
        };
        format!("STEP {element_index} of {element_count} ({element_kind})")
    };
    set_text(world, step_number, Some(&step_text));
}

/// `-[StoryManager updateCreateTitlePanel]` @ 0x10004ae64
pub fn update_create_title_panel(world: &mut World, this: Entity) {
    let story_title = panel_object(world, "StoryMainNodePanel", "StoryTitle");
    let story_def = active_story_def(world, this);
    let title = story_def_store(world).story_def(story_def).title.clone();
    set_text(world, story_title, title.as_deref());
}

/// `-[StoryManager updateCreateEnvironmentPanel]` @ 0x10004aee4
pub fn update_create_environment_panel(world: &mut World, this: Entity) {
    let element_def = active_element_def(world, this);
    let panel = library::display_group(world, "StoryEnvironmentNodePanel");
    let delay_seconds = group_object(world, panel, "DelaySeconds");
    let time = group_object(world, panel, "Time");
    let weather = group_object(world, panel, "Weather");
    let island = group_object(world, panel, "Island");

    let delay = story_def_store(world).element_def(element_def).delay;
    let delay_text = delay_seconds_text(delay);
    set_text(world, delay_seconds, Some(&delay_text));

    let location_state = get_story_environment_location_state_def(world, this, Some(element_def));
    let location_name = state_display_name(world, location_state);
    set_text(world, island, location_name.as_deref());

    let weather_state = get_story_environment_weather_state_def(world, this, Some(element_def));
    let weather_name = state_display_name(world, weather_state);
    set_text(world, weather, weather_name.as_deref());

    let time_of_day_state = get_story_environment_time_of_day_state_def(world, this, Some(element_def));
    let time_of_day_name = state_display_name(world, time_of_day_state);
    set_text(world, time, time_of_day_name.as_deref());
}

/// `-[StoryManager updateCreateCaptionPanel]` @ 0x10004b0a0
pub fn update_create_caption_panel(world: &mut World, this: Entity) {
    let element_def = active_element_def(world, this);
    let panel = library::display_group(world, "StoryCaptionNodePanel");
    let delay_seconds = group_object(world, panel, "DelaySeconds");
    let caption_text = group_object(world, panel, "CaptionText");

    let element = story_def_store(world).element_def(element_def).clone();
    let delay_text = delay_seconds_text(element.delay);
    set_text(world, delay_seconds, Some(&delay_text));

    let caption = string_at_0x10(&element);
    set_text(world, caption_text, caption.as_deref());
    if let Some(caption_text) = caption_text {
        text::set_wrap_width(world, caption_text, CAPTION_WRAP_WIDTH);
        text::set_line_count(world, caption_text, 4);
    }
}

/// `-[StoryManager updateCreateCharacterPanel]` @ 0x10004b1e4
pub fn update_create_character_panel(world: &mut World, this: Entity) {
    let element_def = active_element_def(world, this);
    let panel = library::display_group(world, "StoryCharacterNodePanel");
    let delay_seconds = group_object(world, panel, "DelaySeconds");
    let pygmy_name = group_object(world, panel, "PygmyName");
    let pygmy_state = group_object(world, panel, "PygmyState");
    let dialogue = group_object(world, panel, "Dialogue");

    let element = story_def_store(world).element_def(element_def).clone();
    let delay_text = delay_seconds_text(element.delay);
    set_text(world, delay_seconds, Some(&delay_text));

    let name = string_at_0x10(&element);
    set_text(world, pygmy_name, name.as_deref());
    set_text(world, dialogue, element.dialog.as_deref());
    if let Some(dialogue) = dialogue {
        text::set_wrap_width(world, dialogue, DIALOGUE_WRAP_WIDTH);
        text::set_line_count(world, dialogue, 2);
    }

    let state = get_story_character_state_def(world, this, Some(element_def));
    let state_name = state_display_name(world, state);
    set_text(world, pygmy_state, state_name.as_deref());
}

/// `-[StoryManager updateCreateEventPanel]` @ 0x10004b3a4
pub fn update_create_event_panel(world: &mut World, this: Entity) {
    let element_def = active_element_def(world, this);
    let panel = library::display_group(world, "StoryEventNodePanel");
    let delay_seconds = group_object(world, panel, "DelaySeconds");
    let event_state = group_object(world, panel, "EventState");

    let delay = story_def_store(world).element_def(element_def).delay;
    let delay_text = delay_seconds_text(delay);
    set_text(world, delay_seconds, Some(&delay_text));

    let state = get_story_event_state_def(world, this, Some(element_def));
    let state_name = state_display_name(world, state);
    set_text(world, event_state, state_name.as_deref());
}

/// `-[StoryManager updateClearConfirm]` @ 0x10004b4d8
pub fn update_clear_confirm(world: &mut World, this: Entity) {
    set_story_screen_mode(world, this, CLEAR_CONFIRM_SCREEN_MODE);
    update_confirm_panel(world, "Remove Story From Your List?");
}

/// `-[StoryManager updateEditMainStoryConfirm]` @ 0x10004b5b4
pub fn update_edit_main_story_confirm(world: &mut World, this: Entity) {
    set_story_screen_mode(world, this, EDIT_MAIN_STORY_CONFIRM_SCREEN_MODE);
    update_confirm_panel(world, "Edit Story?\nThe edited version will be located in your story list.");
}

/// The body both confirm dialogs share: the message, and a selectable `ButtonCancel`.
fn update_confirm_panel(world: &mut World, message_text: &str) {
    let panel = library::display_group(world, "StoryConfirmPanel");
    let message = group_object(world, panel, "Message");
    set_text(world, message, Some(message_text));

    let button_cancel = group_object(world, panel, "ButtonCancel");
    set_selectable(world, button_cancel, true);
    show(world, button_cancel);
}

/// `-[StoryManager updateAddMenu]` @ 0x10004b690
pub fn update_add_menu(_world: &mut World, _this: Entity) {}

/// `-[StoryManager getStoryOptionMenuStateArray]` @ 0x10004b694
pub fn get_story_option_menu_state_array(world: &mut World, this: Entity) -> Option<usize> {
    let option_mode = world.obj::<StoryManager>(this).story_option_mode;
    match option_mode {
        LOCATION_OPTION_MODE => Some(STORY_LOCATION_STATE_ARRAY),
        WEATHER_OPTION_MODE => Some(STORY_WEATHER_STATE_ARRAY),
        TIME_OF_DAY_OPTION_MODE => Some(STORY_TIME_OF_DAY_STATE_ARRAY),
        CHARACTER_NAME_OPTION_MODE => create_story_character_name_array(world, this),
        CHARACTER_STATE_OPTION_MODE => Some(STORY_CHARACTER_STATE_ARRAY),
        EVENT_OPTION_MODE => Some(STORY_EVENT_STATE_ARRAY),
        _ => None,
    }
}

/// `-[StoryManager getStoryOptionMenuActiveIndex]` @ 0x10004b72c
pub fn get_story_option_menu_active_index(world: &mut World, this: Entity) -> i32 {
    let option_mode = world.obj::<StoryManager>(this).story_option_mode;
    match option_mode {
        LOCATION_OPTION_MODE => {
            let element_def = active_element_def(world, this);
            story_def_store(world).element_def(element_def).location
        }
        WEATHER_OPTION_MODE => {
            let element_def = active_element_def(world, this);
            story_def_store(world).element_def(element_def).weather
        }
        TIME_OF_DAY_OPTION_MODE => {
            let element_def = active_element_def(world, this);
            story_def_store(world).element_def(element_def).time_of_day
        }
        CHARACTER_NAME_OPTION_MODE => {
            let names = create_story_character_name_array(world, this);
            let element_def = active_element_def(world, this);
            let element = story_def_store(world).element_def(element_def).clone();
            let name = string_at_0x10(&element);
            // `[entry.displayName isEqualToString:name]` up to the list's NULL, at most 100.
            for index in 0..100 {
                let Some(entry) = story_state_array_entry(world, names, index) else {
                    return -1;
                };
                let display_name = story_state_def(world, entry).display_name;
                if is_equal_to_string(display_name.as_deref(), name.as_deref()) {
                    return index;
                }
            }
            -1
        }
        CHARACTER_STATE_OPTION_MODE => {
            let element_def = active_element_def(world, this);
            let state = story_def_store(world).element_def(element_def).state.clone();
            state_index_by_id(world, STORY_CHARACTER_STATE_ARRAY, state.as_deref())
        }
        EVENT_OPTION_MODE => {
            let element_def = active_element_def(world, this);
            let element = story_def_store(world).element_def(element_def).clone();
            let state = string_at_0x10(&element);
            state_index_by_id(world, STORY_EVENT_STATE_ARRAY, state.as_deref())
        }
        _ => -1,
    }
}

/// The index of the entry of a static state list whose id `isEqualToString:` `id`, −1 when the
/// list's NULL comes first.
fn state_index_by_id(world: &World, array: usize, id: Option<&str>) -> i32 {
    let mut index = 0;
    while let Some(entry) = story_state_array_entry(world, Some(array), index) {
        let entry_id = story_state_def(world, entry).id;
        if is_equal_to_string(entry_id.as_deref(), id) {
            return index;
        }
        index += 1;
    }
    -1
}

/// `-[StoryManager updateOptionMenu]` @ 0x10004b994
pub fn update_option_menu(world: &mut World, this: Entity) {
    let wide_option_menu = world.obj::<StoryManager>(this).wide_option_menu;
    let group_id = if wide_option_menu { "StoryWideOptionMenu" } else { "StoryOptionMenu" };
    let menu = library::display_group(world, group_id);

    let state_array = get_story_option_menu_state_array(world, this);
    let state_count = count_story_state_array(world, this, state_array);
    let active_index = get_story_option_menu_active_index(world, this);

    let next_page = group_object(world, menu, "NextPage");
    let prev_page = group_object(world, menu, "PrevPage");
    set_selectable(world, next_page, false);
    hide(world, next_page);
    set_selectable(world, prev_page, false);
    hide(world, prev_page);

    let manager = world.obj::<StoryManager>(this);
    let option_index = manager.story_option_menu_option_index;
    let option_button_count = manager.option_button_count;
    if option_index > 0 {
        set_selectable(world, prev_page, true);
        show(world, prev_page);
    }
    if option_button_count + option_index < state_count {
        set_selectable(world, next_page, true);
        show(world, next_page);
    }

    let selection_box = group_object(world, menu, "SelectionBox");
    hide(world, selection_box);

    let mut row = 0;
    loop {
        let option_button_id = format!("Option{row}Button");
        let option_button = group_object(world, menu, &option_button_id);
        let black_box_id = format!("BlackBox{row}");
        let black_box = group_object(world, menu, &black_box_id);
        if option_button.is_none() {
            return;
        }

        let state_index = row + option_index;
        if state_index < state_count {
            let entry = story_state_array_entry(world, state_array, state_index);
            let display_name = state_display_name(world, entry);
            set_text(world, option_button, display_name.as_deref());
            set_selectable(world, black_box, true);
            show(world, black_box);
            if active_index - option_index == row {
                move_selection_box(world, selection_box, black_box);
                show(world, selection_box);
            }
        } else {
            set_text(world, option_button, Some(""));
            set_selectable(world, black_box, false);
            hide(world, black_box);
        }
        row += 1;
    }
}

/// `selectionPos.x = boxPos.x; selectionPos.y = boxPos.y` (the `pos` pointers of
/// `updateOptionMenu`).
fn move_selection_box(world: &mut World, selection_box: Option<Entity>, black_box: Option<Entity>) {
    let Some(selection_box) = selection_box else {
        return;
    };
    let box_pos = match black_box {
        Some(black_box) => display::object(world, black_box).pos,
        None => Vec4::ZERO,
    };
    let mut selection = display::object_mut(world, selection_box);
    selection.pos.x = box_pos.x;
    selection.pos.y = box_pos.y;
}

/// `-[StoryManager updateWatchPanel:::]` @ 0x10004bf44
pub fn seq_update_watch_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_watch_panel(world, this);
    ItemResult::Done
}

/// `-[StoryManager updateMyListPanel:::]` @ 0x10004bf68
pub fn seq_update_my_list_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_my_list_panel(world, this);
    ItemResult::Done
}

/// `-[StoryManager updateCreatePanel:::]` @ 0x10004bf8c
pub fn seq_update_create_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_create_panel(world, this);
    ItemResult::Done
}

/// `-[StoryManager updateActiveCreateElementPanel:::]` @ 0x10004bfb0
pub fn seq_update_active_create_element_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let element_index = world.obj::<StoryManager>(this).active_story_element_index;
    view_story_element_def(world, this, element_index);
    ItemResult::Done
}

/// `-[StoryManager updateCreateTitlePanel:::]` @ 0x10004bfe0
pub fn seq_update_create_title_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_create_title_panel(world, this);
    ItemResult::Done
}

/// `-[StoryManager updateCreateEnvironmentPanel:::]` @ 0x10004c004
pub fn seq_update_create_environment_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_create_environment_panel(world, this);
    ItemResult::Done
}

/// `-[StoryManager updateCreateCaptionPanel:::]` @ 0x10004c028
pub fn seq_update_create_caption_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_create_caption_panel(world, this);
    ItemResult::Done
}

/// `-[StoryManager updateCreateCharacterPanel:::]` @ 0x10004c04c
pub fn seq_update_create_character_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_create_character_panel(world, this);
    ItemResult::Done
}

/// `-[StoryManager updateCreateEventPanel:::]` @ 0x10004c070
pub fn seq_update_create_event_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_create_event_panel(world, this);
    ItemResult::Done
}

/// `-[StoryManager updateClearConfirm:::]` @ 0x10004c094
pub fn seq_update_clear_confirm(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_clear_confirm(world, this);
    ItemResult::Done
}

/// `-[StoryManager updateEditMainStoryConfirm:::]` @ 0x10004c0b8
pub fn seq_update_edit_main_story_confirm(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_edit_main_story_confirm(world, this);
    ItemResult::Done
}

/// `-[StoryManager updateAddMenu:::]` @ 0x10004c0dc
pub fn seq_update_add_menu(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_add_menu(world, this);
    ItemResult::Done
}

/// `-[StoryManager updateOptionMenu:::]` @ 0x10004c100
pub fn seq_update_option_menu(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_option_menu(world, this);
    ItemResult::Done
}

/// `-[StoryManager loadLocationTextureAtlas:::]` @ 0x10004c148
pub fn seq_load_location_texture_atlas(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<StoryManager>(this).story_manager_listener;
    if let Some(listener) = listener {
        controller::on_load_location_texture_atlas(world, listener);
    }
    ItemResult::Done
}

/// `-[StoryManager updateStoryCreateEnvironmentDelay:]` @ 0x10004cc80
pub fn update_story_create_environment_delay(world: &mut World, this: Entity, direction: i32) {
    step_active_element_delay(world, this, direction);
    set_sequence_def(world, this, "UpdateStoryCreateEnvironmentValue");
}

/// `-[StoryManager updateStoryCreateCaptionDelay:]` @ 0x10004cd70
pub fn update_story_create_caption_delay(world: &mut World, this: Entity, direction: i32) {
    step_active_element_delay(world, this, direction);
    set_sequence_def(world, this, "UpdateStoryCreateCaptionValue");
}

/// `-[StoryManager updateStoryCreateCharacterDelay:]` @ 0x10004ce60
pub fn update_story_create_character_delay(world: &mut World, this: Entity, direction: i32) {
    step_active_element_delay(world, this, direction);
    set_sequence_def(world, this, "UpdateStoryCreateCharacterValue");
}

/// `-[StoryManager updateStoryCreateEventDelay:]` @ 0x10004d344
pub fn update_story_create_event_delay(world: &mut World, this: Entity, direction: i32) {
    step_active_element_delay(world, this, direction);
    set_sequence_def(world, this, "UpdateStoryCreateEventValue");
}

/// The step the four `updateStoryCreate…Delay:` methods share: below one second (or leaving one
/// second downwards) the delay moves by half a second and stops at 0, otherwise by a whole
/// second with no upper limit. Then the story is saved.
fn step_active_element_delay(world: &mut World, this: Entity, direction: i32) {
    let story_def = active_story_def(world, this);
    let element_def = active_element_def(world, this);
    let delay = story_def_store(world).element_def(element_def).delay;

    let new_delay = if delay < 1.0 || (delay == 1.0 && direction < 0) {
        let half_step_delay = direction as f32 * 0.5 + delay;
        if half_step_delay < 0.0 { 0.0 } else { half_step_delay }
    } else {
        direction as f32 + delay
    };
    story_def_store_mut(world).element_def_mut(element_def).delay = new_delay;

    save_story_to_file(world, this, Some(story_def));
}

/// `-[StoryManager updateStoryOptionMenuPage:]` @ 0x10004d51c
pub fn update_story_option_menu_page(world: &mut World, this: Entity, direction: i32) {
    let state_array = get_story_option_menu_state_array(world, this);
    let state_count = count_story_state_array(world, this, state_array);

    let wide_option_menu = {
        let mut manager = world.obj_mut::<StoryManager>(this);
        let option_button_count = manager.option_button_count;
        manager.story_option_menu_option_index += option_button_count * direction;
        if state_count <= manager.story_option_menu_option_index {
            let page_count = if option_button_count != 0 { state_count / option_button_count } else { 0 };
            manager.story_option_menu_option_index = page_count * option_button_count;
        }
        if manager.story_option_menu_option_index < 0 {
            manager.story_option_menu_option_index = 0;
        }
        manager.wide_option_menu
    };

    let sequence_id = if wide_option_menu { "OpenStoryWideOptionMenu" } else { "OpenStoryOptionMenu" };
    set_sequence_def(world, this, sequence_id);
}

/// `-[StoryManager onKeyboardTextEntered:]` @ 0x10004d614
pub fn on_keyboard_text_entered(world: &mut World, this: Entity, entered_text: Option<&str>) {
    let edit_text_type = {
        let mut manager = world.obj_mut::<StoryManager>(this);
        manager.keyboard_open = false;
        manager.story_edit_text_type
    };

    let entered_text = entered_text.map(str::to_owned);
    match edit_text_type {
        TITLE_EDIT_TEXT_TYPE => {
            let story_def = active_story_def(world, this);
            story_def_store_mut(world).story_def_mut(story_def).title = entered_text;
        }
        CAPTION_EDIT_TEXT_TYPE | NAME_EDIT_TEXT_TYPE => {
            let element_def = active_element_def(world, this);
            let mut store = story_def_store_mut(world);
            set_string_at_0x10(store.element_def_mut(element_def), entered_text);
        }
        DIALOG_EDIT_TEXT_TYPE => {
            let element_def = active_element_def(world, this);
            let mut store = story_def_store_mut(world);
            let element = store.element_def_mut(element_def);
            // PORT: +0x18 is a character's dialog. The keyboard opens on a character, but the
            // add menu can show another element while it is open; there the original writes
            // past or over other members, which is dropped here.
            if element.class == StoryElementClass::Character {
                element.dialog = entered_text;
            }
        }
        _ => return,
    }

    let story_def = world.obj::<StoryManager>(this).active_story_def;
    save_story_to_file(world, this, story_def);
}

/// `-[StoryManager selectStoryOptionIndex:]` @ 0x10004dda8
pub fn select_story_option_index(world: &mut World, this: Entity, option_index: i32) {
    let option_mode = world.obj::<StoryManager>(this).story_option_mode;
    let sequence_id = match option_mode {
        LOCATION_OPTION_MODE | WEATHER_OPTION_MODE | TIME_OF_DAY_OPTION_MODE => {
            let story_def = active_story_def(world, this);
            let element_def = active_element_def(world, this);
            {
                let mut store = story_def_store_mut(world);
                let element = store.element_def_mut(element_def);
                match option_mode {
                    LOCATION_OPTION_MODE => element.location = option_index,
                    WEATHER_OPTION_MODE => element.weather = option_index,
                    _ => element.time_of_day = option_index,
                }
            }
            save_story_to_file(world, this, Some(story_def));
            "UpdateStoryCreateEnvironmentValue"
        }
        CHARACTER_NAME_OPTION_MODE => {
            let names = create_story_character_name_array(world, this);
            let story_def = active_story_def(world, this);
            let element_def = active_element_def(world, this);
            let entry = story_state_array_entry(world, names, option_index);
            let name = state_display_name(world, entry);
            if is_equal_to_string(name.as_deref(), Some(NEW_NAME)) {
                if world.obj::<StoryManager>(this).keyboard_open {
                    return;
                }
                {
                    let mut manager = world.obj_mut::<StoryManager>(this);
                    manager.keyboard_open = true;
                    manager.story_edit_text_type = NAME_EDIT_TEXT_TYPE;
                }
                let pygmy_name = panel_object(world, "StoryCharacterNodePanel", "PygmyName");
                let cancel_button = world.obj::<StoryManager>(this).story_create_character_cancel_edit_name_button_logic;
                listener_on_show_keyboard(world, this, pygmy_name, cancel_button, 10, true);
                "StoryOptionMenuEditPygmyName"
            } else {
                {
                    let mut store = story_def_store_mut(world);
                    set_string_at_0x10(store.element_def_mut(element_def), name);
                }
                save_story_to_file(world, this, Some(story_def));
                "UpdateStoryCreateCharacterValue"
            }
        }
        CHARACTER_STATE_OPTION_MODE => {
            let story_def = active_story_def(world, this);
            let element_def = active_element_def(world, this);
            let entry = story_state_array_entry(world, Some(STORY_CHARACTER_STATE_ARRAY), option_index);
            let state_id = state_id(world, entry);
            story_def_store_mut(world).element_def_mut(element_def).state = state_id;
            save_story_to_file(world, this, Some(story_def));
            "UpdateStoryCreateCharacterValue"
        }
        EVENT_OPTION_MODE => {
            let story_def = active_story_def(world, this);
            let element_def = active_element_def(world, this);
            let entry = story_state_array_entry(world, Some(STORY_EVENT_STATE_ARRAY), option_index);
            let state_id = state_id(world, entry);
            {
                let mut store = story_def_store_mut(world);
                set_string_at_0x10(store.element_def_mut(element_def), state_id);
            }
            save_story_to_file(world, this, Some(story_def));
            "UpdateStoryCreateEventValue"
        }
        _ => return,
    };
    set_sequence_def(world, this, sequence_id);
}

/// `-[StoryManager onButtonPress:touch:]` @ 0x10004e218
pub fn on_button_press(world: &mut World, this: Entity, button: Entity, _touch: Entity) {
    // Only `setLogic:` writes the button ivars, so one snapshot serves every comparison.
    let buttons = world.obj::<StoryManager>(this).clone();
    let pressed = Some(button);

    // The add menu inserts a default element just after the shown one (never before the first
    // element), or deletes the shown element.
    let new_element = if pressed == buttons.story_add_menu_add_environment_button_logic {
        Some(StoryElementDef::new_environment())
    } else if pressed == buttons.story_add_menu_add_caption_button_logic {
        Some(StoryElementDef::new_caption())
    } else if pressed == buttons.story_add_menu_add_character_button_logic {
        Some(StoryElementDef::new_character())
    } else if pressed == buttons.story_add_menu_add_event_button_logic {
        Some(StoryElementDef::new_event())
    } else {
        None
    };
    if let Some(new_element) = new_element {
        let element_index = world.obj::<StoryManager>(this).active_story_element_index;
        let position = if element_index == 0 { 1 } else { element_index };
        let element_def = story_def_store_mut(world).insert_element_def(new_element);
        let story_def = active_story_def(world, this);
        insert_story_element_def(world, story_def, position, element_def);
        save_story_to_file(world, this, Some(story_def));
        view_story_element_def(world, this, position + 1);
        return;
    }

    if pressed == buttons.story_add_menu_delete_node_button_logic {
        let story_def = active_story_def(world, this);
        if story_def_store(world).element_count(story_def) < 1 {
            return;
        }
        let element_index = world.obj::<StoryManager>(this).active_story_element_index;
        let position = if element_index != 0 { element_index - 1 } else { 0 };
        erase_story_element_def(world, story_def, position);
        save_story_to_file(world, this, Some(story_def));
        view_story_element_def(world, this, position + 1);
        return;
    }

    if pressed == buttons.story_add_menu_close_button_logic {
        world.obj_mut::<StoryManager>(this).add_menu_open = false;
        set_sequence_def(world, this, "StoryAddMenuClose");
        return;
    }

    // Any other button closes an open add menu first.
    if world.obj::<StoryManager>(this).add_menu_open {
        world.obj_mut::<StoryManager>(this).add_menu_open = false;
        set_sequence_def(world, this, "StoryAddMenuClose");
    }

    if pressed == buttons.story_close_button_logic {
        close_story_screen(world, this);
        return;
    }

    if pressed == buttons.story_tab_watch_button_logic {
        leave_story_editor(world, this);
        set_sequence_def(world, this, "OpenStoryWatchPanel");
        return;
    }

    if pressed == buttons.story_tab_my_list_button_logic {
        leave_story_editor(world, this);
        set_sequence_def(world, this, "OpenStoryMyListPanel");
        return;
    }

    if pressed == buttons.story_tab_create_button_logic {
        if world.obj::<StoryManager>(this).story_screen_mode == CREATE_SCREEN_MODE {
            return;
        }
        leave_story_editor(world, this);
        world.obj_mut::<StoryManager>(this).active_story_element_index = 0;

        let story_def = story_def_store_mut(world).insert_story_def(StoryDef::new());
        world.obj_mut::<StoryManager>(this).active_story_def = Some(story_def);
        let environment = story_def_store_mut(world).insert_element_def(StoryElementDef::new_environment());
        append_story_element_def(world, story_def, environment);
        world.obj_mut::<StoryManager>(this).my_list_story_def_vector.push(story_def);
        save_story_map_xml(world, this);

        let element_index = world.obj::<StoryManager>(this).active_story_element_index;
        view_story_element_def(world, this, element_index);
        return;
    }

    let watch_rows = [
        buttons.story_watch_list_item0_button_logic,
        buttons.story_watch_list_item1_button_logic,
        buttons.story_watch_list_item2_button_logic,
        buttons.story_watch_list_item3_button_logic,
        buttons.story_watch_list_item4_button_logic,
        buttons.story_watch_list_item5_button_logic,
    ];
    if let Some(row) = watch_rows.iter().position(|row_button| *row_button == pressed) {
        let manager = world.obj::<StoryManager>(this);
        let position = manager.main_story_index as usize + row;
        let story_def = manager.main_story_def_vector[position];
        play_story_def(world, this, Some(story_def));
        return;
    }

    let watch_edit_buttons = [
        buttons.story_watch_edit_story0_button_logic,
        buttons.story_watch_edit_story1_button_logic,
        buttons.story_watch_edit_story2_button_logic,
        buttons.story_watch_edit_story3_button_logic,
        buttons.story_watch_edit_story4_button_logic,
        buttons.story_watch_edit_story5_button_logic,
    ];
    if let Some(row) = watch_edit_buttons.iter().position(|row_button| *row_button == pressed) {
        let mut manager = world.obj_mut::<StoryManager>(this);
        let position = manager.main_story_index as usize + row;
        manager.template_story_def = Some(manager.main_story_def_vector[position]);
        set_sequence_def(world, this, "OpenStoryEditMainStoryConfirmPanel");
        return;
    }

    if pressed == buttons.story_watch_prev_story_button_logic {
        let mut manager = world.obj_mut::<StoryManager>(this);
        manager.main_story_index = (manager.main_story_index - STORY_LIST_ROW_COUNT).max(0);
        set_sequence_def(world, this, "UpdateStoryWatchPanel");
        return;
    }

    if pressed == buttons.story_watch_next_story_button_logic {
        let mut manager = world.obj_mut::<StoryManager>(this);
        let story_count = manager.main_story_def_vector.len();
        manager.main_story_index = next_story_page_index(manager.main_story_index, story_count);
        set_sequence_def(world, this, "UpdateStoryWatchPanel");
        return;
    }

    let my_list_rows = [
        buttons.story_my_list_item0_button_logic,
        buttons.story_my_list_item1_button_logic,
        buttons.story_my_list_item2_button_logic,
        buttons.story_my_list_item3_button_logic,
        buttons.story_my_list_item4_button_logic,
        buttons.story_my_list_item5_button_logic,
    ];
    if let Some(row) = my_list_rows.iter().position(|row_button| *row_button == pressed) {
        let manager = world.obj::<StoryManager>(this);
        let position = manager.my_list_story_index as usize + row;
        let story_def = manager.my_list_story_def_vector[position];
        play_story_def(world, this, Some(story_def));
        return;
    }

    let my_list_edit_buttons = [
        buttons.story_my_list_edit_story0_button_logic,
        buttons.story_my_list_edit_story1_button_logic,
        buttons.story_my_list_edit_story2_button_logic,
        buttons.story_my_list_edit_story3_button_logic,
        buttons.story_my_list_edit_story4_button_logic,
        buttons.story_my_list_edit_story5_button_logic,
    ];
    if let Some(row) = my_list_edit_buttons.iter().position(|row_button| *row_button == pressed) {
        let element_index = {
            let mut manager = world.obj_mut::<StoryManager>(this);
            manager.template_story_def = None;
            manager.active_story_element_index = 0;
            let position = manager.my_list_story_index as usize + row;
            manager.active_story_def = Some(manager.my_list_story_def_vector[position]);
            manager.active_story_element_index
        };
        view_story_element_def(world, this, element_index);
        return;
    }

    if pressed == buttons.story_my_list_prev_story_button_logic {
        let mut manager = world.obj_mut::<StoryManager>(this);
        manager.my_list_story_index = (manager.my_list_story_index - STORY_LIST_ROW_COUNT).max(0);
        set_sequence_def(world, this, "UpdateStoryMyListPanel");
        return;
    }

    if pressed == buttons.story_my_list_next_story_button_logic {
        let mut manager = world.obj_mut::<StoryManager>(this);
        let story_count = manager.my_list_story_def_vector.len();
        manager.my_list_story_index = next_story_page_index(manager.my_list_story_index, story_count);
        set_sequence_def(world, this, "UpdateStoryMyListPanel");
        return;
    }

    if pressed == buttons.story_create_preview_story_button_logic {
        let story_def = world.obj::<StoryManager>(this).active_story_def;
        play_story_def(world, this, story_def);
        return;
    }

    if pressed == buttons.story_create_clear_story_button_logic {
        set_sequence_def(world, this, "OpenStoryClearConfirmPanel");
        return;
    }

    if pressed == buttons.story_create_next_node_button_logic {
        hide_keyboard(world, this);
        let element_index = world.obj::<StoryManager>(this).active_story_element_index;
        view_story_element_def(world, this, element_index + 1);
        return;
    }

    if pressed == buttons.story_create_prev_node_button_logic {
        hide_keyboard(world, this);
        let element_index = world.obj::<StoryManager>(this).active_story_element_index;
        view_story_element_def(world, this, element_index - 1);
        return;
    }

    if pressed == buttons.story_create_move_up_button_logic {
        hide_keyboard(world, this);
        let element_index = world.obj::<StoryManager>(this).active_story_element_index;
        let position = if element_index != 0 { element_index - 1 } else { 0 };
        let story_def = active_story_def(world, this);
        move_story_element_def_later(world, story_def, position);
        save_story_to_file(world, this, Some(story_def));
        view_story_element_def(world, this, position + 2);
        return;
    }

    if pressed == buttons.story_create_move_down_button_logic {
        hide_keyboard(world, this);
        let element_index = world.obj::<StoryManager>(this).active_story_element_index;
        let position = if element_index != 0 { element_index - 1 } else { 0 };
        let story_def = active_story_def(world, this);
        move_story_element_def_earlier(world, story_def, position);
        save_story_to_file(world, this, Some(story_def));
        view_story_element_def(world, this, position);
        return;
    }

    if pressed == buttons.story_create_add_menu_button_logic {
        world.obj_mut::<StoryManager>(this).add_menu_open = true;
        set_sequence_def(world, this, "OpenStoryAddMenu");
        return;
    }

    if pressed == buttons.story_create_title_edit_title_button_logic {
        {
            let mut manager = world.obj_mut::<StoryManager>(this);
            manager.keyboard_open = true;
            manager.story_edit_text_type = TITLE_EDIT_TEXT_TYPE;
        }
        let story_title = panel_object(world, "StoryMainNodePanel", "StoryTitle");
        let cancel_button = buttons.story_create_title_cancel_edit_title_button_logic;
        listener_on_show_keyboard(world, this, story_title, cancel_button, 30, true);
        set_sequence_def(world, this, "StoryCreateTitleEditTitle");
        return;
    }

    if pressed == buttons.story_create_title_cancel_edit_title_button_logic {
        hide_keyboard(world, this);
        return;
    }

    if pressed == buttons.story_create_environment_delay_next_button_logic {
        update_story_create_environment_delay(world, this, 1);
        return;
    }

    if pressed == buttons.story_create_environment_delay_prev_button_logic {
        update_story_create_environment_delay(world, this, -1);
        return;
    }

    if pressed == buttons.story_create_environment_time_of_day_button_logic {
        open_option_menu(world, this, true, TIME_OF_DAY_OPTION_MODE);
        return;
    }

    if pressed == buttons.story_create_environment_weather_button_logic {
        open_option_menu(world, this, true, WEATHER_OPTION_MODE);
        return;
    }

    if pressed == buttons.story_create_environment_location_button_logic {
        open_option_menu(world, this, true, LOCATION_OPTION_MODE);
        return;
    }

    if pressed == buttons.story_create_caption_delay_next_button_logic {
        update_story_create_caption_delay(world, this, 1);
        return;
    }

    if pressed == buttons.story_create_caption_delay_prev_button_logic {
        update_story_create_caption_delay(world, this, -1);
        return;
    }

    if pressed == buttons.story_create_caption_edit_caption_button_logic {
        {
            let mut manager = world.obj_mut::<StoryManager>(this);
            manager.keyboard_open = true;
            manager.story_edit_text_type = CAPTION_EDIT_TEXT_TYPE;
        }
        let caption_text = panel_object(world, "StoryCaptionNodePanel", "CaptionText");
        let cancel_button = buttons.story_create_caption_cancel_edit_caption_button_logic;
        listener_on_show_keyboard(world, this, caption_text, cancel_button, 150, true);
        set_sequence_def(world, this, "StoryCreateCaptionEditCaption");
        return;
    }

    if pressed == buttons.story_create_caption_cancel_edit_caption_button_logic {
        hide_keyboard(world, this);
        return;
    }

    if pressed == buttons.story_create_character_delay_next_button_logic {
        update_story_create_character_delay(world, this, 1);
        return;
    }

    if pressed == buttons.story_create_character_delay_prev_button_logic {
        update_story_create_character_delay(world, this, -1);
        return;
    }

    if pressed == buttons.story_create_character_state_button_logic {
        open_option_menu(world, this, false, CHARACTER_STATE_OPTION_MODE);
        return;
    }

    if pressed == buttons.story_create_character_edit_name_button_logic {
        open_option_menu(world, this, false, CHARACTER_NAME_OPTION_MODE);
        return;
    }

    if pressed == buttons.story_create_character_cancel_edit_name_button_logic {
        hide_keyboard(world, this);
        return;
    }

    if pressed == buttons.story_create_character_edit_dialog_button_logic {
        if world.obj::<StoryManager>(this).keyboard_open {
            return;
        }
        {
            let mut manager = world.obj_mut::<StoryManager>(this);
            manager.keyboard_open = true;
            manager.story_edit_text_type = DIALOG_EDIT_TEXT_TYPE;
        }
        let dialogue = panel_object(world, "StoryCharacterNodePanel", "Dialogue");
        let cancel_button = buttons.story_create_character_cancel_edit_dialog_button_logic;
        listener_on_show_keyboard(world, this, dialogue, cancel_button, 60, false);
        set_sequence_def(world, this, "StoryCreateCharacterEditDialog");
        return;
    }

    if pressed == buttons.story_create_character_cancel_edit_dialog_button_logic {
        hide_keyboard(world, this);
        return;
    }

    if pressed == buttons.story_create_event_delay_next_button_logic {
        update_story_create_event_delay(world, this, 1);
        return;
    }

    if pressed == buttons.story_create_event_delay_prev_button_logic {
        update_story_create_event_delay(world, this, -1);
        return;
    }

    if pressed == buttons.story_create_event_state_button_logic {
        open_option_menu(world, this, true, EVENT_OPTION_MODE);
        return;
    }

    if pressed == buttons.story_confirm_ok_button_logic {
        let screen_mode = world.obj::<StoryManager>(this).story_screen_mode;
        if screen_mode == EDIT_MAIN_STORY_CONFIRM_SCREEN_MODE {
            // The Watch panel's edit buttons set the template before this dialog opens.
            let template_story_def = world.obj::<StoryManager>(this).template_story_def;
            let template_story_def = template_story_def.expect("the edit dialog has a template story");
            let story_def = clone_story_def(world, template_story_def);
            world.obj_mut::<StoryManager>(this).active_story_def = Some(story_def);
            let has_date = story_def_store(world).story_def(story_def).date.is_some();
            if !has_date {
                let now = controller::cf_absolute_time_get_current(world);
                story_def_store_mut(world).story_def_mut(story_def).date = Some(now);
            }
            world.obj_mut::<StoryManager>(this).my_list_story_def_vector.push(story_def);
            save_story_map_xml(world, this);
            world.obj_mut::<StoryManager>(this).active_story_element_index = 0;
            view_story_element_def(world, this, 0);
            return;
        }
        if screen_mode != CLEAR_CONFIRM_SCREEN_MODE {
            return;
        }
        delete_active_story_def(world, this);
        set_sequence_def(world, this, "StoryClearConfirmOk");
        return;
    }

    if pressed == buttons.story_confirm_cancel_button_logic {
        let screen_mode = world.obj::<StoryManager>(this).story_screen_mode;
        let sequence_id = if screen_mode == EDIT_MAIN_STORY_CONFIRM_SCREEN_MODE {
            "StoryEditMainStoryConfirmCancel"
        } else if screen_mode == CLEAR_CONFIRM_SCREEN_MODE {
            "StoryClearConfirmCancel"
        } else {
            return;
        };
        set_sequence_def(world, this, sequence_id);
        return;
    }

    let option_rows = &buttons.story_option_button_logic_array;
    let wide_option_rows = &buttons.story_wide_option_button_logic_array;
    let option_row = option_rows.iter().position(|option_button| *option_button == button);
    let option_row = option_row.or_else(|| wide_option_rows.iter().position(|option_button| *option_button == button));
    if let Some(row) = option_row {
        let option_index = {
            let mut manager = world.obj_mut::<StoryManager>(this);
            manager.option_menu_open = false;
            manager.story_option_menu_option_index
        };
        select_story_option_index(world, this, option_index + row as i32);
        return;
    }

    if pressed == buttons.story_option_menu_cancel_button_logic {
        world.obj_mut::<StoryManager>(this).option_menu_open = false;
        set_sequence_def(world, this, "StoryOptionMenuClose");
        return;
    }

    if pressed == buttons.story_option_menu_page_next_button_logic {
        update_story_option_menu_page(world, this, 1);
        return;
    }

    if pressed == buttons.story_option_menu_page_prev_button_logic {
        update_story_option_menu_page(world, this, -1);
        return;
    }

    if pressed == buttons.story_wide_option_menu_cancel_button_logic {
        world.obj_mut::<StoryManager>(this).option_menu_open = false;
        set_sequence_def(world, this, "StoryWideOptionMenuClose");
        return;
    }

    if pressed == buttons.story_wide_option_menu_page_next_button_logic {
        update_story_option_menu_page(world, this, 1);
        return;
    }

    if pressed == buttons.story_wide_option_menu_page_prev_button_logic {
        update_story_option_menu_page(world, this, -1);
        return;
    }

    if pressed == buttons.story_overlay_close_button_logic {
        on_play_story_def_complete(world, this, false);
    }
}

/// `-[StoryManager onButtonRelease:touch:]` @ 0x100050078
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[StoryManager onButtonReleaseOutside:touch:]` @ 0x10005007c
pub fn on_button_release_outside(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// The tab buttons' way out of the editor: `mbKeyboardOpen = NO`, `checkForUnmodifiedStoryDef`,
/// then the listener's `onHideKeyboard`.
fn leave_story_editor(world: &mut World, this: Entity) {
    world.obj_mut::<StoryManager>(this).keyboard_open = false;
    check_for_unmodified_story_def(world, this);
    listener_on_hide_keyboard(world, this);
}

/// `mbKeyboardOpen = NO`, then the listener's `onHideKeyboard` (which puts the edited text back).
fn hide_keyboard(world: &mut World, this: Entity) {
    world.obj_mut::<StoryManager>(this).keyboard_open = false;
    listener_on_hide_keyboard(world, this);
}

/// Opens the option menu (15 rows) or the wide option menu (10 rows) on its first page, listing
/// `option_mode`'s values.
fn open_option_menu(world: &mut World, this: Entity, wide_option_menu: bool, option_mode: i32) {
    {
        let mut manager = world.obj_mut::<StoryManager>(this);
        manager.option_menu_open = true;
        manager.wide_option_menu = wide_option_menu;
        manager.option_button_count = if wide_option_menu { WIDE_OPTION_MENU_ROW_COUNT } else { OPTION_MENU_ROW_COUNT };
        manager.story_option_menu_option_index = 0;
        manager.story_option_mode = option_mode;
    }

    let sequence_id = if wide_option_menu { "OpenStoryWideOptionMenu" } else { "OpenStoryOptionMenu" };
    set_sequence_def(world, this, sequence_id);
}

/// The next page of a story list: `index + 6`, pulled back to the last story when it passes it
/// (`count - 1` compared unsigned, so an empty list never pulls back).
fn next_story_page_index(story_index: i32, story_count: usize) -> i32 {
    let next_index = story_index + STORY_LIST_ROW_COUNT;
    let last_index = (story_count as u64).wrapping_sub(1);
    if last_index < next_index as i64 as u64 {
        story_count as i32 - 1
    } else {
        next_index
    }
}

/// `mLastStoryScreenMode = mStoryScreenMode; mStoryScreenMode = mode`.
fn set_story_screen_mode(world: &mut World, this: Entity, mode: i32) {
    let mut manager = world.obj_mut::<StoryManager>(this);
    manager.last_story_screen_mode = manager.story_screen_mode;
    manager.story_screen_mode = mode;
}

/// `mpActiveStoryDef`, which every editor path has (the original dereferences it unchecked).
fn active_story_def(world: &World, this: Entity) -> usize {
    let active_story_def = world.obj::<StoryManager>(this).active_story_def;
    active_story_def.expect("the story editor has an active story")
}

/// The shown element: `(*mpActiveStoryDef->elements)[mActiveStoryElementIndex - 1]`.
fn active_element_def(world: &World, this: Entity) -> usize {
    let story_def = active_story_def(world, this);
    let element_index = world.obj::<StoryManager>(this).active_story_element_index;
    story_def_store(world).element_id(story_def, element_index - 1)
}

/// Element `index` of a story.
fn story_element(world: &World, story_def: usize, index: i32) -> &StoryElementDef {
    let store = story_def_store(world);
    let element_def = store.element_id(story_def, index);
    store.element_def(element_def)
}

/// The `NSString *` at +0x10, which each class names differently: a caption's text, a
/// character's name, an event's state (an environment keeps two `int`s there).
fn string_at_0x10(element: &StoryElementDef) -> Option<String> {
    match element.class {
        StoryElementClass::Caption => element.caption.clone(),
        StoryElementClass::Character => element.name.clone(),
        StoryElementClass::Event => element.state.clone(),
        StoryElementClass::Environment => None,
    }
}

/// Stores an `NSString *` at +0x10 (see [`string_at_0x10`]).
fn set_string_at_0x10(element: &mut StoryElementDef, value: Option<String>) {
    match element.class {
        StoryElementClass::Caption => element.caption = value,
        StoryElementClass::Character => element.name = value,
        StoryElementClass::Event => element.state = value,
        // PORT: the original overwrites the location and weather with the string's address
        // (only reachable when the add menu shows a new environment while the keyboard is
        // open); dropped here.
        StoryElementClass::Environment => {}
    }
}

/// `state->displayName`. PORT: a nil state (an unknown state id) is read through NULL in the
/// original; here it reads as nil, as `story_data` does for the compatibility test.
fn state_display_name(world: &World, state: Option<usize>) -> Option<String> {
    let state = state?;
    story_state_def(world, state).display_name
}

/// `state->id` (nil for a nil state, see [`state_display_name`]).
fn state_id(world: &World, state: Option<usize>) -> Option<String> {
    let state = state?;
    story_state_def(world, state).id
}

/// `[receiver isEqualToString:other]`: NO when either is nil.
fn is_equal_to_string(receiver: Option<&str>, other: Option<&str>) -> bool {
    match (receiver, other) {
        (Some(receiver), Some(other)) => receiver == other,
        _ => false,
    }
}

/// `[NSString stringWithFormat:@"%1.1f SECONDS", delay]`.
fn delay_seconds_text(delay: f32) -> String {
    let delay = f64::from(delay);
    format!("{delay:.1} SECONDS")
}

/// `[[IdolManager instance] delayTriggerEpisodeEvent:event]`.
fn delay_trigger_episode_event(world: &mut World, event: &str) {
    let idol_manager = idol_manager::instance(world);
    if let Some(idol_manager) = idol_manager {
        idol_manager::delay_trigger_episode_event(world, idol_manager, event);
    }
}

/// `[mSequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:sequence_id]]`.
fn set_sequence_def(world: &mut World, this: Entity, sequence_id: &str) {
    let manager_sequence = world.obj::<StoryManager>(this).sequence;
    let sequence_def = library::sequence_def(world, sequence_id);
    if let Some(manager_sequence) = manager_sequence {
        sequence::set_sequence_def(world, manager_sequence, sequence_def);
    }
}

/// `[mStoryManagerListener onHideKeyboard]` (the listener is the view controller).
fn listener_on_hide_keyboard(world: &mut World, this: Entity) {
    let listener = world.obj::<StoryManager>(this).story_manager_listener;
    if let Some(listener) = listener {
        controller::on_hide_keyboard(world, listener);
    }
}

/// `[mStoryManagerListener onShowKeyboard:self editingText:editing_text
/// cancelEditDisplayObject:[cancel_button displayObject] maxEditCharCount:… preventEmpty:…]`.
fn listener_on_show_keyboard(world: &mut World, this: Entity, editing_text: Option<Entity>, cancel_button: Option<Entity>, max_edit_char_count: i32, prevent_empty: bool) {
    let listener = world.obj::<StoryManager>(this).story_manager_listener;
    let cancel_edit_display_object = button_display_object(world, cancel_button);
    let Some(listener) = listener else {
        return;
    };

    controller::on_show_keyboard(world, listener, this, editing_text, cancel_edit_display_object, max_edit_char_count, prevent_empty);
}

/// `[group getDisplayObject:id]` (nil for a nil group).
fn group_object(world: &World, display_group: Option<Entity>, id: &str) -> Option<Entity> {
    let display_group = display_group?;
    group::get_display_object(world, display_group, id)
}

/// `[[[BCLibrary instance] getDisplayGroupById:group_id] getDisplayObject:id]`.
fn panel_object(world: &World, group_id: &str, id: &str) -> Option<Entity> {
    let display_group = library::display_group(world, group_id);
    group_object(world, display_group, id)
}

/// `[button displayObject]` (nil for a nil button).
fn button_display_object(world: &World, button: Option<Entity>) -> Option<Entity> {
    let button = button?;
    Some(logic::display_object(world, button))
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
fn set_text(world: &mut World, object: Option<Entity>, value: Option<&str>) {
    if let Some(object) = object {
        text::set_text(world, object, value);
    }
}
