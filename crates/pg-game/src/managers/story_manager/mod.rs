//! `StoryManager` (superclass `NSObject`).

#![allow(unused_variables, unused_imports)]

use crate::prelude::*;

pub mod story_data;
pub use story_data::*;
pub mod story_playback;
pub use story_playback::*;
pub mod story_screens_1;
pub use story_screens_1::*;
pub mod story_screens_2;
pub use story_screens_2::*;

/// `StoryManager` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct StoryManager {
    /// `mStoryManagerListener`: `@"<StoryManagerListener>"`
    pub story_manager_listener: Option<Entity>,

    /// `mSequence`: `@"BCSequence"`
    pub sequence: Option<Entity>,

    /// `mStorySequence`: `@"BCSequence"`
    pub story_sequence: Option<Entity>,

    /// `mMissingStoryIdArray`: `@"NSMutableArray"` (of `NSString` story ids; never filled)
    pub missing_story_id_array: Vec<String>,

    /// `mParseStoryXmlArray`: `@"NSMutableArray"` (of `NSString` story file names)
    pub parse_story_xml_array: Vec<String>,

    /// `mbStoryScreenActive`: `B`
    pub story_screen_active: bool,

    /// `mStoryScreenMode`: `i`
    pub story_screen_mode: i32,

    /// `mLastStoryScreenMode`: `i`
    pub last_story_screen_mode: i32,

    /// `mStoryOptionMode`: `i`
    pub story_option_mode: i32,

    /// `mbAddMenuOpen`: `B`
    pub add_menu_open: bool,

    /// `mbOptionMenuOpen`: `B`
    pub option_menu_open: bool,

    /// `mbKeyboardOpen`: `B`
    pub keyboard_open: bool,

    /// `mMainStoryIndex`: `i`
    pub main_story_index: i32,

    /// `mpMainStoryDefVector`: `^{vector<BCStoryDef *, std::__1::allocator<BCStoryDef *> >=^^{BCStoryDef}^^{BCStoryDef}{__compressed_pair<BCStoryDef **, std::__1::allocator<BCStoryDef *> >=^^{BCStoryDef}}}`
    pub main_story_def_vector: Vec<usize>,

    /// `mMyListStoryIndex`: `i`
    pub my_list_story_index: i32,

    /// `mpMyListStoryDefVector`: `^{vector<BCStoryDef *, std::__1::allocator<BCStoryDef *> >=^^{BCStoryDef}^^{BCStoryDef}{__compressed_pair<BCStoryDef **, std::__1::allocator<BCStoryDef *> >=^^{BCStoryDef}}}`
    pub my_list_story_def_vector: Vec<usize>,

    /// `mbCaptionOpen`: `B`
    pub caption_open: bool,

    /// `mbWordBalloonOpen`: `B`
    pub word_balloon_open: bool,

    /// `mbWordBalloonRight`: `B`
    pub word_balloon_right: bool,

    /// `mStoryStallTime`: `f`
    pub story_stall_time: f32,

    /// `mPlayingStoryElementIndex`: `i`
    pub playing_story_element_index: i32,

    /// `mpActiveStoryEnvironmentElementDef`: `^{BCStoryEnvironmentElementDef=^^?ifiii}`
    pub active_story_environment_element_def: Option<usize>,

    /// `mActiveStoryElementIndex`: `i`
    pub active_story_element_index: i32,

    /// `mpActiveStoryDef`: `^{BCStoryDef=^^?@@i^{vector<BCStoryElementDef *, std::__1::allocator<BCStoryElementDef *> >}}`
    pub active_story_def: Option<usize>,

    /// `mpTemplateStoryDef`: `^{BCStoryDef=^^?@@i^{vector<BCStoryElementDef *, std::__1::allocator<BCStoryElementDef *> >}}`
    pub template_story_def: Option<usize>,

    /// `mStoryOptionMenuOptionIndex`: `i`
    pub story_option_menu_option_index: i32,

    /// `mOptionButtonCount`: `i`
    pub option_button_count: i32,

    /// `mbWideOptionMenu`: `B`
    pub wide_option_menu: bool,

    /// `mStoryEditTextType`: `i`
    pub story_edit_text_type: i32,

    /// `mpLastParsedStoryDef`: `^{BCStoryDef=^^?@@i^{vector<BCStoryElementDef *, std::__1::allocator<BCStoryElementDef *> >}}`
    pub last_parsed_story_def: Option<usize>,

    /// `mpParsedStoryDef`: `^{BCStoryDef=^^?@@i^{vector<BCStoryElementDef *, std::__1::allocator<BCStoryElementDef *> >}}`
    pub parsed_story_def: Option<usize>,

    /// `mbParseStoryTitle`: `B`
    pub parse_story_title: bool,

    /// `mpParsedStoryEnvironmentElementDef`: `^{BCStoryEnvironmentElementDef=^^?ifiii}`
    pub parsed_story_environment_element_def: Option<usize>,

    /// `mbParseStoryCaption`: `B`
    pub parse_story_caption: bool,

    /// `mpParsedStoryCaptionElementDef`: `^{BCStoryCaptionElementDef=^^?if@}`
    pub parsed_story_caption_element_def: Option<usize>,

    /// `mpParsedStoryCharacterElementDef`: `^{BCStoryCharacterElementDef=^^?if@@@}`
    pub parsed_story_character_element_def: Option<usize>,

    /// `mpParsedStoryEventElementDef`: `^{BCStoryEventElementDef=^^?if@}`
    pub parsed_story_event_element_def: Option<usize>,

    /// `mbParseStoryCharacterName`: `B`
    pub parse_story_character_name: bool,

    /// `mbParseStoryCharacterDialog`: `B`
    pub parse_story_character_dialog: bool,

    /// `mLogicArray`: `@"NSMutableArray"`
    pub logic_array: Vec<Entity>,

    /// `mStoryUiLogicArray`: `@"NSMutableArray"`
    pub story_ui_logic_array: Vec<Entity>,

    /// `mStoryWatchUiLogicArray`: `@"NSMutableArray"`
    pub story_watch_ui_logic_array: Vec<Entity>,

    /// `mStoryMyListUiLogicArray`: `@"NSMutableArray"`
    pub story_my_list_ui_logic_array: Vec<Entity>,

    /// `mStoryCreateUiLogicArray`: `@"NSMutableArray"`
    pub story_create_ui_logic_array: Vec<Entity>,

    /// `mStoryCreateTitleUiLogicArray`: `@"NSMutableArray"`
    pub story_create_title_ui_logic_array: Vec<Entity>,

    /// `mStoryCreateEnvironmentUiLogicArray`: `@"NSMutableArray"`
    pub story_create_environment_ui_logic_array: Vec<Entity>,

    /// `mStoryCreateCaptionUiLogicArray`: `@"NSMutableArray"`
    pub story_create_caption_ui_logic_array: Vec<Entity>,

    /// `mStoryCreateCharacterUiLogicArray`: `@"NSMutableArray"`
    pub story_create_character_ui_logic_array: Vec<Entity>,

    /// `mStoryCreateEventUiLogicArray`: `@"NSMutableArray"`
    pub story_create_event_ui_logic_array: Vec<Entity>,

    /// `mStoryConfirmUiLogicArray`: `@"NSMutableArray"`
    pub story_confirm_ui_logic_array: Vec<Entity>,

    /// `mStoryAddMenuUiLogicArray`: `@"NSMutableArray"`
    pub story_add_menu_ui_logic_array: Vec<Entity>,

    /// `mStoryOverlayUiLogicArray`: `@"NSMutableArray"`
    pub story_overlay_ui_logic_array: Vec<Entity>,

    /// `mStoryOptionMenuUiLogicArray`: `@"NSMutableArray"`
    pub story_option_menu_ui_logic_array: Vec<Entity>,

    /// `mStoryOptionButtonLogicArray`: `@"NSMutableArray"`
    pub story_option_button_logic_array: Vec<Entity>,

    /// `mStoryWideOptionMenuUiLogicArray`: `@"NSMutableArray"`
    pub story_wide_option_menu_ui_logic_array: Vec<Entity>,

    /// `mStoryWideOptionButtonLogicArray`: `@"NSMutableArray"`
    pub story_wide_option_button_logic_array: Vec<Entity>,

    /// `mStoryCloseButtonLogic`: `@"BCButtonLogic"`
    pub story_close_button_logic: Option<Entity>,

    /// `mStoryTabWatchButtonLogic`: `@"BCButtonLogic"`
    pub story_tab_watch_button_logic: Option<Entity>,

    /// `mStoryTabMyListButtonLogic`: `@"BCButtonLogic"`
    pub story_tab_my_list_button_logic: Option<Entity>,

    /// `mStoryTabCreateButtonLogic`: `@"BCButtonLogic"`
    pub story_tab_create_button_logic: Option<Entity>,

    /// `mStoryWatchListItem0ButtonLogic`: `@"BCButtonLogic"`
    pub story_watch_list_item0_button_logic: Option<Entity>,

    /// `mStoryWatchListItem1ButtonLogic`: `@"BCButtonLogic"`
    pub story_watch_list_item1_button_logic: Option<Entity>,

    /// `mStoryWatchListItem2ButtonLogic`: `@"BCButtonLogic"`
    pub story_watch_list_item2_button_logic: Option<Entity>,

    /// `mStoryWatchListItem3ButtonLogic`: `@"BCButtonLogic"`
    pub story_watch_list_item3_button_logic: Option<Entity>,

    /// `mStoryWatchListItem4ButtonLogic`: `@"BCButtonLogic"`
    pub story_watch_list_item4_button_logic: Option<Entity>,

    /// `mStoryWatchListItem5ButtonLogic`: `@"BCButtonLogic"`
    pub story_watch_list_item5_button_logic: Option<Entity>,

    /// `mStoryWatchEditStory0ButtonLogic`: `@"BCButtonLogic"`
    pub story_watch_edit_story0_button_logic: Option<Entity>,

    /// `mStoryWatchEditStory1ButtonLogic`: `@"BCButtonLogic"`
    pub story_watch_edit_story1_button_logic: Option<Entity>,

    /// `mStoryWatchEditStory2ButtonLogic`: `@"BCButtonLogic"`
    pub story_watch_edit_story2_button_logic: Option<Entity>,

    /// `mStoryWatchEditStory3ButtonLogic`: `@"BCButtonLogic"`
    pub story_watch_edit_story3_button_logic: Option<Entity>,

    /// `mStoryWatchEditStory4ButtonLogic`: `@"BCButtonLogic"`
    pub story_watch_edit_story4_button_logic: Option<Entity>,

    /// `mStoryWatchEditStory5ButtonLogic`: `@"BCButtonLogic"`
    pub story_watch_edit_story5_button_logic: Option<Entity>,

    /// `mStoryWatchPrevStoryButtonLogic`: `@"BCButtonLogic"`
    pub story_watch_prev_story_button_logic: Option<Entity>,

    /// `mStoryWatchNextStoryButtonLogic`: `@"BCButtonLogic"`
    pub story_watch_next_story_button_logic: Option<Entity>,

    /// `mStoryMyListItem0ButtonLogic`: `@"BCButtonLogic"`
    pub story_my_list_item0_button_logic: Option<Entity>,

    /// `mStoryMyListItem1ButtonLogic`: `@"BCButtonLogic"`
    pub story_my_list_item1_button_logic: Option<Entity>,

    /// `mStoryMyListItem2ButtonLogic`: `@"BCButtonLogic"`
    pub story_my_list_item2_button_logic: Option<Entity>,

    /// `mStoryMyListItem3ButtonLogic`: `@"BCButtonLogic"`
    pub story_my_list_item3_button_logic: Option<Entity>,

    /// `mStoryMyListItem4ButtonLogic`: `@"BCButtonLogic"`
    pub story_my_list_item4_button_logic: Option<Entity>,

    /// `mStoryMyListItem5ButtonLogic`: `@"BCButtonLogic"`
    pub story_my_list_item5_button_logic: Option<Entity>,

    /// `mStoryMyListPrevStoryButtonLogic`: `@"BCButtonLogic"`
    pub story_my_list_prev_story_button_logic: Option<Entity>,

    /// `mStoryMyListNextStoryButtonLogic`: `@"BCButtonLogic"`
    pub story_my_list_next_story_button_logic: Option<Entity>,

    /// `mStoryMyListEditStory0ButtonLogic`: `@"BCButtonLogic"`
    pub story_my_list_edit_story0_button_logic: Option<Entity>,

    /// `mStoryMyListEditStory1ButtonLogic`: `@"BCButtonLogic"`
    pub story_my_list_edit_story1_button_logic: Option<Entity>,

    /// `mStoryMyListEditStory2ButtonLogic`: `@"BCButtonLogic"`
    pub story_my_list_edit_story2_button_logic: Option<Entity>,

    /// `mStoryMyListEditStory3ButtonLogic`: `@"BCButtonLogic"`
    pub story_my_list_edit_story3_button_logic: Option<Entity>,

    /// `mStoryMyListEditStory4ButtonLogic`: `@"BCButtonLogic"`
    pub story_my_list_edit_story4_button_logic: Option<Entity>,

    /// `mStoryMyListEditStory5ButtonLogic`: `@"BCButtonLogic"`
    pub story_my_list_edit_story5_button_logic: Option<Entity>,

    /// `mStoryCreatePreviewStoryButtonLogic`: `@"BCButtonLogic"`
    pub story_create_preview_story_button_logic: Option<Entity>,

    /// `mStoryCreateClearStoryButtonLogic`: `@"BCButtonLogic"`
    pub story_create_clear_story_button_logic: Option<Entity>,

    /// `mStoryCreateNextNodeButtonLogic`: `@"BCButtonLogic"`
    pub story_create_next_node_button_logic: Option<Entity>,

    /// `mStoryCreatePrevNodeButtonLogic`: `@"BCButtonLogic"`
    pub story_create_prev_node_button_logic: Option<Entity>,

    /// `mStoryCreateMoveUpButtonLogic`: `@"BCButtonLogic"`
    pub story_create_move_up_button_logic: Option<Entity>,

    /// `mStoryCreateMoveDownButtonLogic`: `@"BCButtonLogic"`
    pub story_create_move_down_button_logic: Option<Entity>,

    /// `mStoryCreateAddMenuButtonLogic`: `@"BCButtonLogic"`
    pub story_create_add_menu_button_logic: Option<Entity>,

    /// `mStoryCreateTitleEditTitleButtonLogic`: `@"BCButtonLogic"`
    pub story_create_title_edit_title_button_logic: Option<Entity>,

    /// `mStoryCreateTitleCancelEditTitleButtonLogic`: `@"BCButtonLogic"`
    pub story_create_title_cancel_edit_title_button_logic: Option<Entity>,

    /// `mStoryCreateEnvironmentDelayPrevButtonLogic`: `@"BCButtonLogic"`
    pub story_create_environment_delay_prev_button_logic: Option<Entity>,

    /// `mStoryCreateEnvironmentDelayNextButtonLogic`: `@"BCButtonLogic"`
    pub story_create_environment_delay_next_button_logic: Option<Entity>,

    /// `mStoryCreateEnvironmentTimeOfDayButtonLogic`: `@"BCButtonLogic"`
    pub story_create_environment_time_of_day_button_logic: Option<Entity>,

    /// `mStoryCreateEnvironmentWeatherButtonLogic`: `@"BCButtonLogic"`
    pub story_create_environment_weather_button_logic: Option<Entity>,

    /// `mStoryCreateEnvironmentLocationButtonLogic`: `@"BCButtonLogic"`
    pub story_create_environment_location_button_logic: Option<Entity>,

    /// `mStoryCreateCaptionDelayPrevButtonLogic`: `@"BCButtonLogic"`
    pub story_create_caption_delay_prev_button_logic: Option<Entity>,

    /// `mStoryCreateCaptionDelayNextButtonLogic`: `@"BCButtonLogic"`
    pub story_create_caption_delay_next_button_logic: Option<Entity>,

    /// `mStoryCreateCaptionEditCaptionButtonLogic`: `@"BCButtonLogic"`
    pub story_create_caption_edit_caption_button_logic: Option<Entity>,

    /// `mStoryCreateCaptionCancelEditCaptionButtonLogic`: `@"BCButtonLogic"`
    pub story_create_caption_cancel_edit_caption_button_logic: Option<Entity>,

    /// `mStoryCreateCharacterDelayPrevButtonLogic`: `@"BCButtonLogic"`
    pub story_create_character_delay_prev_button_logic: Option<Entity>,

    /// `mStoryCreateCharacterDelayNextButtonLogic`: `@"BCButtonLogic"`
    pub story_create_character_delay_next_button_logic: Option<Entity>,

    /// `mStoryCreateCharacterStateButtonLogic`: `@"BCButtonLogic"`
    pub story_create_character_state_button_logic: Option<Entity>,

    /// `mStoryCreateCharacterEditNameButtonLogic`: `@"BCButtonLogic"`
    pub story_create_character_edit_name_button_logic: Option<Entity>,

    /// `mStoryCreateCharacterCancelEditNameButtonLogic`: `@"BCButtonLogic"`
    pub story_create_character_cancel_edit_name_button_logic: Option<Entity>,

    /// `mStoryCreateCharacterEditDialogButtonLogic`: `@"BCButtonLogic"`
    pub story_create_character_edit_dialog_button_logic: Option<Entity>,

    /// `mStoryCreateCharacterCancelEditDialogButtonLogic`: `@"BCButtonLogic"`
    pub story_create_character_cancel_edit_dialog_button_logic: Option<Entity>,

    /// `mStoryCreateEventDelayPrevButtonLogic`: `@"BCButtonLogic"`
    pub story_create_event_delay_prev_button_logic: Option<Entity>,

    /// `mStoryCreateEventDelayNextButtonLogic`: `@"BCButtonLogic"`
    pub story_create_event_delay_next_button_logic: Option<Entity>,

    /// `mStoryCreateEventStateButtonLogic`: `@"BCButtonLogic"`
    pub story_create_event_state_button_logic: Option<Entity>,

    /// `mStoryConfirmOkButtonLogic`: `@"BCButtonLogic"`
    pub story_confirm_ok_button_logic: Option<Entity>,

    /// `mStoryConfirmCancelButtonLogic`: `@"BCButtonLogic"`
    pub story_confirm_cancel_button_logic: Option<Entity>,

    /// `mStoryAddMenuAddEnvironmentButtonLogic`: `@"BCButtonLogic"`
    pub story_add_menu_add_environment_button_logic: Option<Entity>,

    /// `mStoryAddMenuAddCaptionButtonLogic`: `@"BCButtonLogic"`
    pub story_add_menu_add_caption_button_logic: Option<Entity>,

    /// `mStoryAddMenuAddCharacterButtonLogic`: `@"BCButtonLogic"`
    pub story_add_menu_add_character_button_logic: Option<Entity>,

    /// `mStoryAddMenuAddEventButtonLogic`: `@"BCButtonLogic"`
    pub story_add_menu_add_event_button_logic: Option<Entity>,

    /// `mStoryAddMenuDeleteNodeButtonLogic`: `@"BCButtonLogic"`
    pub story_add_menu_delete_node_button_logic: Option<Entity>,

    /// `mStoryAddMenuCloseButtonLogic`: `@"BCButtonLogic"`
    pub story_add_menu_close_button_logic: Option<Entity>,

    /// `mStoryAddMenuBackgroundButtonLogic`: `@"BCButtonLogic"`
    pub story_add_menu_background_button_logic: Option<Entity>,

    /// `mStoryOptionMenuCancelButtonLogic`: `@"BCButtonLogic"`
    pub story_option_menu_cancel_button_logic: Option<Entity>,

    /// `mStoryOptionMenuPageNextButtonLogic`: `@"BCButtonLogic"`
    pub story_option_menu_page_next_button_logic: Option<Entity>,

    /// `mStoryOptionMenuPagePrevButtonLogic`: `@"BCButtonLogic"`
    pub story_option_menu_page_prev_button_logic: Option<Entity>,

    /// `mStoryWideOptionMenuCancelButtonLogic`: `@"BCButtonLogic"`
    pub story_wide_option_menu_cancel_button_logic: Option<Entity>,

    /// `mStoryWideOptionMenuPageNextButtonLogic`: `@"BCButtonLogic"`
    pub story_wide_option_menu_page_next_button_logic: Option<Entity>,

    /// `mStoryWideOptionMenuPagePrevButtonLogic`: `@"BCButtonLogic"`
    pub story_wide_option_menu_page_prev_button_logic: Option<Entity>,

    /// `mStoryOverlayCloseButtonLogic`: `@"BCButtonLogic"`
    pub story_overlay_close_button_logic: Option<Entity>,

    /// `mStoryOverlayWordBalloonLogic`: `@"MultiModelLogic"`
    pub story_overlay_word_balloon_logic: Option<Entity>,

    /// `mStoryOverlayCaptionLogic`: `@"MultiModelLogic"`
    pub story_overlay_caption_logic: Option<Entity>,

    /// `mStoryOverlayTitleLogic`: `@"MultiModelLogic"`
    pub story_overlay_title_logic: Option<Entity>,

    /// `mStoryOverlayFadeOutLogic`: `@"MultiModelLogic"`
    pub story_overlay_fade_out_logic: Option<Entity>,
}

pub const STORY_MANAGER: Class = Class {
    name: "StoryManager",
    superclass: None,
    game_frame_order: 50,
    sequence_item,
    id,
    on_game_frame,
    on_multi_model_event,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    ..Class::NS_OBJECT
};

/// Sequence-item handlers implemented by `StoryManager` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "updateWatchPanel" => seq_update_watch_panel(world, this, call),
        "updateMyListPanel" => seq_update_my_list_panel(world, this, call),
        "updateCreatePanel" => seq_update_create_panel(world, this, call),
        "updateActiveCreateElementPanel" => seq_update_active_create_element_panel(world, this, call),
        "updateCreateTitlePanel" => seq_update_create_title_panel(world, this, call),
        "updateCreateEnvironmentPanel" => seq_update_create_environment_panel(world, this, call),
        "updateCreateCaptionPanel" => seq_update_create_caption_panel(world, this, call),
        "updateCreateCharacterPanel" => seq_update_create_character_panel(world, this, call),
        "updateCreateEventPanel" => seq_update_create_event_panel(world, this, call),
        "updateClearConfirm" => seq_update_clear_confirm(world, this, call),
        "updateEditMainStoryConfirm" => seq_update_edit_main_story_confirm(world, this, call),
        "updateAddMenu" => seq_update_add_menu(world, this, call),
        "updateOptionMenu" => seq_update_option_menu(world, this, call),
        "updateBadStateScreen" => seq_update_bad_state_screen(world, this, call),
        "loadLocationTextureAtlas" => seq_load_location_texture_atlas(world, this, call),
        "playStoryDef" => seq_play_story_def(world, this, call),
        "playNextStoryElementDef" => seq_play_next_story_element_def(world, this, call),
        "storyDefComplete" => seq_story_def_complete(world, this, call),
        "storyStall" => seq_story_stall(world, this, call),
        "storyElementDefComplete" => seq_story_element_def_complete(world, this, call),
        "playStoryEnvironmentElementDef" => seq_play_story_environment_element_def(world, this, call),
        "storyEnvironmentElementDefComplete" => seq_story_environment_element_def_complete(world, this, call),
        "playStoryCaptionElementDef" => seq_play_story_caption_element_def(world, this, call),
        "storyCaptionElementDefComplete" => seq_story_caption_element_def_complete(world, this, call),
        "playStoryCharacterElementDef" => seq_play_story_character_element_def(world, this, call),
        "storyCharacterElementDefComplete" => seq_story_character_element_def_complete(world, this, call),
        "playStoryEventElementDef" => seq_play_story_event_element_def(world, this, call),
        "startCelebration" => seq_start_celebration(world, this, call),
        "stopCelebration" => seq_stop_celebration(world, this, call),
        "startEarthquake" => seq_start_earthquake(world, this, call),
        "stopEarthquake" => seq_stop_earthquake(world, this, call),
        "sharkSnap" => seq_shark_snap(world, this, call),
        "activateRainbow" => seq_activate_rainbow(world, this, call),
        "activateMyRainbow" => seq_activate_my_rainbow(world, this, call),
        "deactivateRainbow" => seq_deactivate_rainbow(world, this, call),
        _ => return (Class::NS_OBJECT.sequence_item)(world, this, call),
    };
    Some(result)
}
