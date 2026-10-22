//! `StoryManager`: methods in this part of the class.

use crate::prelude::*;

use super::*;

/// The `mStory…UiLogicArray` a button logic of `setLogic:` joins.
#[derive(Clone, Copy, Debug)]
enum StoryUiArray {
    Story,
    Watch,
    MyList,
    Create,
    CreateTitle,
    CreateEnvironment,
    CreateCaption,
    CreateCharacter,
    CreateEvent,
    Confirm,
    AddMenu,
    Overlay,
    OptionMenu,
    WideOptionMenu,
}

fn ui_array_mut(manager: &mut StoryManager, array: StoryUiArray) -> &mut Vec<Entity> {
    match array {
        StoryUiArray::Story => &mut manager.story_ui_logic_array,
        StoryUiArray::Watch => &mut manager.story_watch_ui_logic_array,
        StoryUiArray::MyList => &mut manager.story_my_list_ui_logic_array,
        StoryUiArray::Create => &mut manager.story_create_ui_logic_array,
        StoryUiArray::CreateTitle => &mut manager.story_create_title_ui_logic_array,
        StoryUiArray::CreateEnvironment => &mut manager.story_create_environment_ui_logic_array,
        StoryUiArray::CreateCaption => &mut manager.story_create_caption_ui_logic_array,
        StoryUiArray::CreateCharacter => &mut manager.story_create_character_ui_logic_array,
        StoryUiArray::CreateEvent => &mut manager.story_create_event_ui_logic_array,
        StoryUiArray::Confirm => &mut manager.story_confirm_ui_logic_array,
        StoryUiArray::AddMenu => &mut manager.story_add_menu_ui_logic_array,
        StoryUiArray::Overlay => &mut manager.story_overlay_ui_logic_array,
        StoryUiArray::OptionMenu => &mut manager.story_option_menu_ui_logic_array,
        StoryUiArray::WideOptionMenu => &mut manager.story_wide_option_menu_ui_logic_array,
    }
}

/// The ivar a logic of `setLogic:` is stored in.
type LogicSlot = fn(&mut StoryManager) -> &mut Option<Entity>;

/// The ivar and the UI array of each `BCButtonLogic` that `setLogic:` names on its own.
fn button_logic_slot(logic_id: &str) -> Option<(LogicSlot, StoryUiArray)> {
    use StoryUiArray::*;

    let slot: (LogicSlot, StoryUiArray) = match logic_id {
        "StoryCloseButtonLogic" => (|manager| &mut manager.story_close_button_logic, Story),
        "StoryTabWatchButtonLogic" => (|manager| &mut manager.story_tab_watch_button_logic, Story),
        "StoryTabMyListButtonLogic" => (|manager| &mut manager.story_tab_my_list_button_logic, Story),
        "StoryTabCreateButtonLogic" => (|manager| &mut manager.story_tab_create_button_logic, Story),
        "StoryWatchListItem0ButtonLogic" => (|manager| &mut manager.story_watch_list_item0_button_logic, Watch),
        "StoryWatchListItem1ButtonLogic" => (|manager| &mut manager.story_watch_list_item1_button_logic, Watch),
        "StoryWatchListItem2ButtonLogic" => (|manager| &mut manager.story_watch_list_item2_button_logic, Watch),
        "StoryWatchListItem3ButtonLogic" => (|manager| &mut manager.story_watch_list_item3_button_logic, Watch),
        "StoryWatchListItem4ButtonLogic" => (|manager| &mut manager.story_watch_list_item4_button_logic, Watch),
        "StoryWatchListItem5ButtonLogic" => (|manager| &mut manager.story_watch_list_item5_button_logic, Watch),
        "StoryWatchPrevStoryButtonLogic" => (|manager| &mut manager.story_watch_prev_story_button_logic, Watch),
        "StoryWatchNextStoryButtonLogic" => (|manager| &mut manager.story_watch_next_story_button_logic, Watch),
        "StoryWatchEditStory0ButtonLogic" => (|manager| &mut manager.story_watch_edit_story0_button_logic, Watch),
        "StoryWatchEditStory1ButtonLogic" => (|manager| &mut manager.story_watch_edit_story1_button_logic, Watch),
        "StoryWatchEditStory2ButtonLogic" => (|manager| &mut manager.story_watch_edit_story2_button_logic, Watch),
        "StoryWatchEditStory3ButtonLogic" => (|manager| &mut manager.story_watch_edit_story3_button_logic, Watch),
        "StoryWatchEditStory4ButtonLogic" => (|manager| &mut manager.story_watch_edit_story4_button_logic, Watch),
        "StoryWatchEditStory5ButtonLogic" => (|manager| &mut manager.story_watch_edit_story5_button_logic, Watch),
        "StoryMyListItem0ButtonLogic" => (|manager| &mut manager.story_my_list_item0_button_logic, MyList),
        "StoryMyListItem1ButtonLogic" => (|manager| &mut manager.story_my_list_item1_button_logic, MyList),
        "StoryMyListItem2ButtonLogic" => (|manager| &mut manager.story_my_list_item2_button_logic, MyList),
        "StoryMyListItem3ButtonLogic" => (|manager| &mut manager.story_my_list_item3_button_logic, MyList),
        "StoryMyListItem4ButtonLogic" => (|manager| &mut manager.story_my_list_item4_button_logic, MyList),
        "StoryMyListItem5ButtonLogic" => (|manager| &mut manager.story_my_list_item5_button_logic, MyList),
        "StoryMyListPrevStoryButtonLogic" => (|manager| &mut manager.story_my_list_prev_story_button_logic, MyList),
        "StoryMyListNextStoryButtonLogic" => (|manager| &mut manager.story_my_list_next_story_button_logic, MyList),
        "StoryMyListEditStory0ButtonLogic" => (|manager| &mut manager.story_my_list_edit_story0_button_logic, MyList),
        "StoryMyListEditStory1ButtonLogic" => (|manager| &mut manager.story_my_list_edit_story1_button_logic, MyList),
        "StoryMyListEditStory2ButtonLogic" => (|manager| &mut manager.story_my_list_edit_story2_button_logic, MyList),
        "StoryMyListEditStory3ButtonLogic" => (|manager| &mut manager.story_my_list_edit_story3_button_logic, MyList),
        "StoryMyListEditStory4ButtonLogic" => (|manager| &mut manager.story_my_list_edit_story4_button_logic, MyList),
        "StoryMyListEditStory5ButtonLogic" => (|manager| &mut manager.story_my_list_edit_story5_button_logic, MyList),
        "StoryCreatePreviewStoryButtonLogic" => (|manager| &mut manager.story_create_preview_story_button_logic, Create),
        "StoryCreateClearStoryButtonLogic" => (|manager| &mut manager.story_create_clear_story_button_logic, Create),
        "StoryCreateNextNodeButtonLogic" => (|manager| &mut manager.story_create_next_node_button_logic, Create),
        "StoryCreatePrevNodeButtonLogic" => (|manager| &mut manager.story_create_prev_node_button_logic, Create),
        "StoryCreateMoveUpButtonLogic" => (|manager| &mut manager.story_create_move_up_button_logic, Create),
        "StoryCreateMoveDownButtonLogic" => (|manager| &mut manager.story_create_move_down_button_logic, Create),
        "StoryCreateAddMenuButtonLogic" => (|manager| &mut manager.story_create_add_menu_button_logic, Create),
        "StoryCreateTitleEditTitleButtonLogic" => (|manager| &mut manager.story_create_title_edit_title_button_logic, CreateTitle),
        "StoryCreateTitleCancelEditTitleButtonLogic" => {
            (|manager| &mut manager.story_create_title_cancel_edit_title_button_logic, CreateTitle)
        }
        "StoryCreateEnvironmentDelayPrevButtonLogic" => {
            (|manager| &mut manager.story_create_environment_delay_prev_button_logic, CreateEnvironment)
        }
        "StoryCreateEnvironmentDelayNextButtonLogic" => {
            (|manager| &mut manager.story_create_environment_delay_next_button_logic, CreateEnvironment)
        }
        "StoryCreateEnvironmentTimeOfDayButtonLogic" => {
            (|manager| &mut manager.story_create_environment_time_of_day_button_logic, CreateEnvironment)
        }
        "StoryCreateEnvironmentWeatherButtonLogic" => {
            (|manager| &mut manager.story_create_environment_weather_button_logic, CreateEnvironment)
        }
        "StoryCreateEnvironmentLocationButtonLogic" => {
            (|manager| &mut manager.story_create_environment_location_button_logic, CreateEnvironment)
        }
        "StoryCreateCaptionDelayPrevButtonLogic" => {
            (|manager| &mut manager.story_create_caption_delay_prev_button_logic, CreateCaption)
        }
        "StoryCreateCaptionDelayNextButtonLogic" => {
            (|manager| &mut manager.story_create_caption_delay_next_button_logic, CreateCaption)
        }
        "StoryCreateCaptionEditCaptionButtonLogic" => {
            (|manager| &mut manager.story_create_caption_edit_caption_button_logic, CreateCaption)
        }
        "StoryCreateCaptionCancelEditCaptionButtonLogic" => {
            (|manager| &mut manager.story_create_caption_cancel_edit_caption_button_logic, CreateCaption)
        }
        "StoryCreateCharacterDelayPrevButtonLogic" => {
            (|manager| &mut manager.story_create_character_delay_prev_button_logic, CreateCharacter)
        }
        "StoryCreateCharacterDelayNextButtonLogic" => {
            (|manager| &mut manager.story_create_character_delay_next_button_logic, CreateCharacter)
        }
        "StoryCreateCharacterStateButtonLogic" => {
            (|manager| &mut manager.story_create_character_state_button_logic, CreateCharacter)
        }
        "StoryCreateCharacterEditNameButtonLogic" => {
            (|manager| &mut manager.story_create_character_edit_name_button_logic, CreateCharacter)
        }
        "StoryCreateCharacterCancelEditNameButtonLogic" => {
            (|manager| &mut manager.story_create_character_cancel_edit_name_button_logic, CreateCharacter)
        }
        "StoryCreateCharacterEditDialogButtonLogic" => {
            (|manager| &mut manager.story_create_character_edit_dialog_button_logic, CreateCharacter)
        }
        "StoryCreateCharacterCancelEditDialogButtonLogic" => {
            (|manager| &mut manager.story_create_character_cancel_edit_dialog_button_logic, CreateCharacter)
        }
        "StoryCreateEventDelayPrevButtonLogic" => (|manager| &mut manager.story_create_event_delay_prev_button_logic, CreateEvent),
        "StoryCreateEventDelayNextButtonLogic" => (|manager| &mut manager.story_create_event_delay_next_button_logic, CreateEvent),
        "StoryCreateEventStateButtonLogic" => (|manager| &mut manager.story_create_event_state_button_logic, CreateEvent),
        "StoryConfirmOkButtonLogic" => (|manager| &mut manager.story_confirm_ok_button_logic, Confirm),
        "StoryConfirmCancelButtonLogic" => (|manager| &mut manager.story_confirm_cancel_button_logic, Confirm),
        "StoryAddMenuAddEnvironmentButtonLogic" => (|manager| &mut manager.story_add_menu_add_environment_button_logic, AddMenu),
        "StoryAddMenuAddCaptionButtonLogic" => (|manager| &mut manager.story_add_menu_add_caption_button_logic, AddMenu),
        "StoryAddMenuAddCharacterButtonLogic" => (|manager| &mut manager.story_add_menu_add_character_button_logic, AddMenu),
        "StoryAddMenuAddEventButtonLogic" => (|manager| &mut manager.story_add_menu_add_event_button_logic, AddMenu),
        "StoryAddMenuDeleteNodeButtonLogic" => (|manager| &mut manager.story_add_menu_delete_node_button_logic, AddMenu),
        "StoryAddMenuCloseButtonLogic" => (|manager| &mut manager.story_add_menu_close_button_logic, AddMenu),
        "StoryAddMenuBackgroundButtonLogic" => (|manager| &mut manager.story_add_menu_background_button_logic, AddMenu),
        "StoryOverlayCloseButtonLogic" => (|manager| &mut manager.story_overlay_close_button_logic, Overlay),
        "StoryOptionMenuCancelButtonLogic" => (|manager| &mut manager.story_option_menu_cancel_button_logic, OptionMenu),
        "StoryOptionMenuPageNextButtonLogic" => (|manager| &mut manager.story_option_menu_page_next_button_logic, OptionMenu),
        "StoryOptionMenuPagePrevButtonLogic" => (|manager| &mut manager.story_option_menu_page_prev_button_logic, OptionMenu),
        "StoryWideOptionMenuCancelButtonLogic" => {
            (|manager| &mut manager.story_wide_option_menu_cancel_button_logic, WideOptionMenu)
        }
        "StoryWideOptionMenuPageNextButtonLogic" => {
            (|manager| &mut manager.story_wide_option_menu_page_next_button_logic, WideOptionMenu)
        }
        "StoryWideOptionMenuPagePrevButtonLogic" => {
            (|manager| &mut manager.story_wide_option_menu_page_prev_button_logic, WideOptionMenu)
        }
        _ => return None,
    };
    Some(slot)
}

/// The ivar of each `MultiModelLogic` overlay that `setLogic:` names.
fn overlay_logic_slot(logic_id: &str) -> Option<LogicSlot> {
    let slot: LogicSlot = match logic_id {
        "StoryOverlayWordBalloonLogic" => |manager| &mut manager.story_overlay_word_balloon_logic,
        "StoryOverlayCaptionLogic" => |manager| &mut manager.story_overlay_caption_logic,
        "StoryOverlayTitleLogic" => |manager| &mut manager.story_overlay_title_logic,
        "StoryOverlayFadeOutLogic" => |manager| &mut manager.story_overlay_fade_out_logic,
        _ => return None,
    };
    Some(slot)
}

/// `[NSString stringWithFormat:format, value]` for the `BlackBox%d` names of the option rows.
fn string_with_int_format(format: &str, value: i32) -> String {
    let number = value.to_string();
    format.replacen("%d", &number, 1)
}

/// The option-row buttons (`count` of them, named by the `%d` format): each joins the menu's
/// UI array, its option-button array and the logic array.
fn create_option_button_logics(world: &mut World, this: Entity, call: &SequenceCall, display_group: Option<Entity>, wide: bool) {
    let count = call.int("count");
    if count < 1 {
        return;
    }

    let name_format = call.param("displayObjectId").unwrap_or_default();
    for index in 0..count {
        let button_id = string_with_int_format(name_format, index);
        let button_object = match display_group {
            Some(group) => pg_engine::display::group::get_display_object(world, group, &button_id),
            None => None,
        };

        // PORT: the original also makes a button on a nil object (the wide menu names 15 rows
        // but has 10); nothing can press it, so none is made here. The rows that exist keep
        // their indices.
        let Some(button_object) = button_object else {
            continue;
        };

        let button = pg_engine::logic::button::spawn(world, button_object);
        pg_engine::logic::button::set_button_logic_listener(world, button, Some(this));

        let mut manager = world.obj_mut::<StoryManager>(this);
        if wide {
            manager.story_wide_option_menu_ui_logic_array.push(button);
            manager.story_wide_option_button_logic_array.push(button);
        } else {
            manager.story_option_menu_ui_logic_array.push(button);
            manager.story_option_button_logic_array.push(button);
        }
        manager.logic_array.push(button);
    }
}

/// `-[StoryManager setLogic:]` @ 0x100050080
pub fn set_logic(world: &mut World, this: Entity, call: &SequenceCall) {
    let display_group_id = call.param("displayGroupId");
    let logic_id = call.param("logicId");
    let display_object_id = call.param("displayObjectId");

    let display_group = match display_group_id {
        Some(display_group_id) => pg_engine::library::display_group(world, display_group_id),
        None => None,
    };
    let display_object = match (display_group, display_object_id) {
        (Some(group), Some(object_id)) => pg_engine::display::group::get_display_object(world, group, object_id),
        _ => None,
    };

    // A nil id matches nothing.
    let Some(logic_id) = logic_id else {
        return;
    };

    if let Some((slot, ui_array)) = button_logic_slot(logic_id) {
        // PORT: the original also creates the logic on a nil object; the engine's logics need
        // one, so nothing is made. Every StoryManager `setLogic` item names an existing object.
        let Some(display_object) = display_object else {
            return;
        };

        let button = pg_engine::logic::button::spawn(world, display_object);
        *slot(&mut world.obj_mut::<StoryManager>(this)) = Some(button);
        pg_engine::logic::button::set_button_logic_listener(world, button, Some(this));

        let mut manager = world.obj_mut::<StoryManager>(this);
        ui_array_mut(&mut manager, ui_array).push(button);
        manager.logic_array.push(button);
        return;
    }

    if let Some(slot) = overlay_logic_slot(logic_id) {
        let Some(display_object) = display_object else {
            return;
        };

        let overlay = pg_engine::logic::multi_model::spawn(world, display_object);
        *slot(&mut world.obj_mut::<StoryManager>(this)) = Some(overlay);
        pg_engine::logic::multi_model::set_multi_model_logic_listener(world, overlay, Some(this));
        world.obj_mut::<StoryManager>(this).logic_array.push(overlay);
        return;
    }

    match logic_id {
        "StoryOptionMenuOptionButtonLogic" => create_option_button_logics(world, this, call, display_group, false),
        "StoryWideOptionMenuOptionButtonLogic" => create_option_button_logics(world, this, call, display_group, true),
        _ => {}
    }
}
