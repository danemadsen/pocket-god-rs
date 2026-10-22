//! `StoryManager`: methods in this part of the class.
//!
//! Story playback: `playStoryDef:` starts `mStorySequence` on `PlayStoryDef`, whose
//! `StoryDefLoop` plays one element after the other through the `PlayStory…ElementDef`
//! sequences and the handlers below.

use pg_engine::library::Library;

use crate::prelude::*;

use super::*;

/// `mStoryScreenMode` while a story plays.
const STORY_SCREEN_MODE_PLAYING: i32 = 7;

/// `0x43b40000`: the wrap width of the incompatible-state message.
const BAD_STATE_TEXT_WRAP_WIDTH: f32 = 360.0;

/// `0x43ce4000`: the wrap width of the caption overlay.
const CAPTION_TEXT_WRAP_WIDTH: f32 = 412.5;

/// `0x42d00000`: the wrap width of the word balloon.
const DIALOGUE_TEXT_WRAP_WIDTH: f32 = 104.0;

/// A speaker at or right of this x gets the balloon that opens to the left.
const WORD_BALLOON_LEFT_MIN_X: f32 = 240.0;

/// The listener (`mStoryManagerListener`, always the view controller).
fn listener(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<StoryManager>(this).story_manager_listener
}

/// `(*mpActiveStoryDef->elements)[mPlayingStoryElementIndex]`; nil when there is no such
/// element (the original reads past the vector).
fn playing_element_def(world: &World, this: Entity) -> Option<usize> {
    let manager = world.obj::<StoryManager>(this);
    let active_story_def = manager.active_story_def?;
    let index = manager.playing_story_element_index;

    let store = story_def_store(world);
    let count = store.element_count(active_story_def);
    let in_range = index >= 0 && index < count;
    if !in_range {
        return None;
    }
    Some(store.element_id(active_story_def, index))
}

/// The playing element itself (a copy).
fn playing_element(world: &World, this: Entity) -> Option<StoryElementDef> {
    let element_def = playing_element_def(world, this)?;
    let element = story_def_store(world).element_def(element_def);
    Some(element.clone())
}

/// `[sequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:id]]`.
fn set_sequence(world: &mut World, sequence: Option<Entity>, id: &str) {
    let Some(sequence) = sequence else {
        return;
    };
    let def = pg_engine::library::sequence_def(world, id);
    pg_engine::sequence::set_sequence_def(world, sequence, def);
}

/// `[logic setBehavior:behavior]` (nothing for a nil logic).
fn set_logic_behavior(world: &mut World, logic: Option<Entity>, behavior: &str) {
    let Some(logic) = logic else {
        return;
    };
    pg_engine::logic::set_behavior(world, logic, Some(behavior));
}

/// `[[BCLibrary instance] getDynamicTextById:id]`.
fn dynamic_text(world: &World, id: &str) -> Option<Entity> {
    world.resource::<Library>().dynamic_text(id)
}

/// `[listener onStoryAddPygmy:name]`: the pygmy playing `name` (the view controller finds or
/// adds one; nil when the island is full).
fn story_pygmy(world: &mut World, this: Entity, name: Option<&str>) -> Option<Entity> {
    let listener = listener(world, this)?;
    // The parser and the editor always give a character a name.
    let name = name.unwrap_or_default();
    crate::controller::on_story_add_pygmy(world, listener, name)
}

/// `-[StoryManager onGameFrame:]` @ 0x100049478
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let (word_balloon_open, word_balloon_logic) = {
        let manager = world.obj::<StoryManager>(this);
        (manager.word_balloon_open, manager.story_overlay_word_balloon_logic)
    };
    if !word_balloon_open {
        return;
    }

    // The balloon follows its speaker, looked up (or added) by name every frame.
    let Some(element) = playing_element(world, this) else {
        return;
    };
    let pygmy = story_pygmy(world, this, element.name.as_deref());

    // PORT: a nil speaker or balloon makes the original read through NULL; nothing moves here.
    let (Some(pygmy), Some(word_balloon_logic)) = (pygmy, word_balloon_logic) else {
        return;
    };
    let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
    let pygmy_pos = pg_engine::display::object(world, pygmy_display_object).pos;

    let balloon_display_object = pg_engine::logic::display_object(world, word_balloon_logic);
    let mut balloon = pg_engine::display::object_mut(world, balloon_display_object);
    balloon.pos.x = pygmy_pos.x;
    balloon.pos.y = pygmy_pos.y;
}

/// `-[StoryManager updateBadStateScreen]` @ 0x10004bd20
pub fn update_bad_state_screen(world: &mut World, this: Entity) {
    let screen = pg_engine::library::display_group(world, "StoryBadStateScreen");
    let error_text = match screen {
        Some(screen) => pg_engine::display::group::get_display_object(world, screen, "ErrorText"),
        None => None,
    };
    if let Some(error_text) = error_text {
        pg_engine::display::text::set_wrap_width(world, error_text, BAD_STATE_TEXT_WRAP_WIDTH);
    }

    let Some(element_def) = playing_element_def(world, this) else {
        return;
    };
    let element_type = story_def_store(world).element_def(element_def).element_type;
    let state = match element_type {
        STORY_EVENT_ELEMENT => get_story_event_state_def(world, this, Some(element_def)),
        STORY_CHARACTER_ELEMENT => get_story_character_state_def(world, this, Some(element_def)),
        _ => return,
    };
    let Some(state) = state else {
        return;
    };

    let environment = world.obj::<StoryManager>(this).active_story_environment_element_def;
    let compatibility = is_story_state_def(world, this, Some(state), environment);
    let environment_state = match compatibility {
        3 => get_story_environment_time_of_day_state_def(world, this, environment),
        2 => get_story_environment_weather_state_def(world, this, environment),
        1 => get_story_environment_location_state_def(world, this, environment),
        _ => return,
    };

    let state_name = story_state_def(world, state).display_name;
    let environment_name = match environment_state {
        Some(environment_state) => story_state_def(world, environment_state).display_name,
        None => None,
    };
    let state_name = ns_description(state_name.as_deref());
    let environment_name = ns_description(environment_name.as_deref());
    let message = match compatibility {
        3 => format!("The {state_name} action is not compatible with {environment_name} time. Please select a different action."),
        2 => format!("The {state_name} action is not compatible with {environment_name} weather. Please select a different action."),
        _ => format!(
            "The {state_name} action is not compatible with the {environment_name} location. Please select a different action."
        ),
    };

    if let Some(error_text) = error_text {
        pg_engine::display::text::set_text(world, error_text, Some(&message));
    }
}

/// `-[StoryManager updateBadStateScreen:::]` @ 0x10004c124
pub fn seq_update_bad_state_screen(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_bad_state_screen(world, this);
    ItemResult::Done
}

/// `-[StoryManager playStoryDef:::]` @ 0x10004c178
pub fn seq_play_story_def(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let (active_story_def, title_logic) = {
        let mut manager = world.obj_mut::<StoryManager>(this);
        manager.playing_story_element_index = 0;
        (manager.active_story_def, manager.story_overlay_title_logic)
    };

    let title = match active_story_def {
        Some(active_story_def) => story_def_store(world).story_def(active_story_def).title.clone(),
        None => None,
    };
    let title_text = dynamic_text(world, "StoryTitle");
    if let Some(title_text) = title_text {
        pg_engine::display::text::set_text(world, title_text, title.as_deref());
    }

    set_logic_behavior(world, title_logic, "StoryTitleInit");
    ItemResult::Done
}

/// `-[StoryManager playNextStoryElementDef:::]` @ 0x10004c218
pub fn seq_play_next_story_element_def(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let (active_story_def, playing_index, story_sequence, fade_out_logic) = {
        let manager = world.obj::<StoryManager>(this);
        (
            manager.active_story_def,
            manager.playing_story_element_index,
            manager.story_sequence,
            manager.story_overlay_fade_out_logic,
        )
    };
    let Some(active_story_def) = active_story_def else {
        return ItemResult::Done;
    };

    let count = story_def_store(world).element_count(active_story_def);
    if playing_index == count {
        set_logic_behavior(world, fade_out_logic, "StoryFadeOutInit");
        return ItemResult::Done;
    }

    let Some(element_def) = playing_element_def(world, this) else {
        return ItemResult::Done;
    };
    let element_type = story_def_store(world).element_def(element_def).element_type;
    let sequence_id = match element_type {
        STORY_ENVIRONMENT_ELEMENT => "PlayStoryEnvironmentElementDef",
        STORY_CAPTION_ELEMENT => "PlayStoryCaptionElementDef",
        STORY_CHARACTER_ELEMENT => {
            let state = get_story_character_state_def(world, this, Some(element_def));
            let environment = world.obj::<StoryManager>(this).active_story_environment_element_def;
            let compatibility = is_story_state_def(world, this, state, environment);
            if compatibility == 0 {
                "PlayStoryCharacterElementDef"
            } else {
                "PlayStoryIncompatibleElementDef"
            }
        }
        STORY_EVENT_ELEMENT => {
            let state = get_story_event_state_def(world, this, Some(element_def));
            let environment = world.obj::<StoryManager>(this).active_story_environment_element_def;
            let compatibility = is_story_state_def(world, this, state, environment);
            if compatibility == 0 {
                "PlayStoryEventElementDef"
            } else {
                "PlayStoryIncompatibleElementDef"
            }
        }
        _ => return ItemResult::Done,
    };

    set_sequence(world, story_sequence, sequence_id);
    ItemResult::Done
}

/// `-[StoryManager storyDefComplete:::]` @ 0x10004c454
pub fn seq_story_def_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    on_play_story_def_complete(world, this, true);
    ItemResult::Done
}

/// `-[StoryManager storyStall:::]` @ 0x10004c47c
pub fn seq_story_stall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        if let Some(element) = playing_element(world, this) {
            world.obj_mut::<StoryManager>(this).story_stall_time = element.delay;
        }
    }

    let mut manager = world.obj_mut::<StoryManager>(this);
    if manager.story_stall_time <= 0.0 {
        return ItemResult::Done;
    }
    manager.story_stall_time = manager.story_stall_time - call.delta_time;
    ItemResult::Pending
}

/// `-[StoryManager storyElementDefComplete:::]` @ 0x10004c510
pub fn seq_story_element_def_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut manager = world.obj_mut::<StoryManager>(this);
    manager.playing_story_element_index = manager.playing_story_element_index + 1;
    ItemResult::Done
}

/// `-[StoryManager playStoryEnvironmentElementDef:::]` @ 0x10004c52c
pub fn seq_play_story_environment_element_def(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let element_def = playing_element_def(world, this);
    world.obj_mut::<StoryManager>(this).active_story_environment_element_def = element_def;

    if let Some(listener) = listener(world, this) {
        crate::controller::on_story_environment_change(world, listener, element_def);
    }
    ItemResult::Done
}

/// `-[StoryManager storyEnvironmentElementDefComplete:::]` @ 0x10004c58c
pub fn seq_story_environment_element_def_complete(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    ItemResult::Done
}

/// `-[StoryManager playStoryCaptionElementDef:::]` @ 0x10004c594
pub fn seq_play_story_caption_element_def(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let caption = match playing_element(world, this) {
        Some(element) => element.caption,
        None => None,
    };
    let Some(caption) = caption else {
        return ItemResult::Done;
    };
    if caption.is_empty() {
        return ItemResult::Done;
    }

    world.obj_mut::<StoryManager>(this).caption_open = true;

    let caption_text = dynamic_text(world, "CaptionLine1");
    if let Some(caption_text) = caption_text {
        pg_engine::display::text::set_text(world, caption_text, Some(""));
        pg_engine::display::text::set_wrap_width(world, caption_text, CAPTION_TEXT_WRAP_WIDTH);
        pg_engine::display::text::set_text(world, caption_text, Some(&caption));
    }

    let caption_logic = world.obj::<StoryManager>(this).story_overlay_caption_logic;
    set_logic_behavior(world, caption_logic, "StoryCaptionOpen");
    ItemResult::Done
}

/// `-[StoryManager storyCaptionElementDefComplete:::]` @ 0x10004c6b4
pub fn seq_story_caption_element_def_complete(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let (caption_open, caption_logic) = {
        let manager = world.obj::<StoryManager>(this);
        (manager.caption_open, manager.story_overlay_caption_logic)
    };
    if !caption_open {
        return ItemResult::Done;
    }

    // Close once, then wait for `CaptionCloseComplete` (onMultiModelEvent:eventId:).
    if call.first_time {
        set_logic_behavior(world, caption_logic, "StoryCaptionClose");
    }
    ItemResult::Pending
}

/// `-[StoryManager playStoryCharacterElementDef:::]` @ 0x10004c714
pub fn seq_play_story_character_element_def(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let Some(element) = playing_element(world, this) else {
        return ItemResult::Done;
    };
    let Some(pygmy) = story_pygmy(world, this, element.name.as_deref()) else {
        return ItemResult::Done;
    };

    let ready = crate::pygmy::is_ready_for_story_mode(world, pygmy);
    if !ready {
        return ItemResult::Pending;
    }

    let has_dialog = match element.dialog.as_deref() {
        Some(dialog) => !dialog.is_empty(),
        None => false,
    };
    let idle = ns_string_equal(element.state.as_deref(), Some(IDLE_STATE));
    if !has_dialog && idle {
        return ItemResult::Done;
    }

    crate::pygmy::set_story_mode_character_state(world, pygmy, element.state.as_deref());
    if !has_dialog {
        return ItemResult::Done;
    }

    world.obj_mut::<StoryManager>(this).word_balloon_open = true;

    let dialogue_text = dynamic_text(world, "DialogueLine1");
    if let Some(dialogue_text) = dialogue_text {
        pg_engine::display::text::set_text(world, dialogue_text, Some(""));
        pg_engine::display::text::set_wrap_width(world, dialogue_text, DIALOGUE_TEXT_WRAP_WIDTH);
        pg_engine::display::text::set_text(world, dialogue_text, element.dialog.as_deref());
    }

    let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
    let pygmy_x = pg_engine::display::object(world, pygmy_display_object).pos.x;
    let opens_left = WORD_BALLOON_LEFT_MIN_X <= pygmy_x;
    let word_balloon_right = !opens_left;
    let word_balloon_logic = {
        let mut manager = world.obj_mut::<StoryManager>(this);
        manager.word_balloon_right = word_balloon_right;
        manager.story_overlay_word_balloon_logic
    };
    let behavior = if word_balloon_right {
        "StoryWordBalloonOpenRight"
    } else {
        "StoryWordBalloonOpenLeft"
    };
    set_logic_behavior(world, word_balloon_logic, behavior);
    ItemResult::Done
}

/// `-[StoryManager storyCharacterElementDefComplete:::]` @ 0x10004c93c
pub fn seq_story_character_element_def_complete(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let (word_balloon_open, word_balloon_right, word_balloon_logic) = {
        let manager = world.obj::<StoryManager>(this);
        (manager.word_balloon_open, manager.word_balloon_right, manager.story_overlay_word_balloon_logic)
    };

    if !word_balloon_open {
        let Some(element) = playing_element(world, this) else {
            return ItemResult::Done;
        };
        let idle = ns_string_equal(element.state.as_deref(), Some(IDLE_STATE));
        if !idle {
            if let Some(pygmy) = story_pygmy(world, this, element.name.as_deref()) {
                crate::pygmy::resume_from_story_mode_character_state(world, pygmy);
            }
        }
        return ItemResult::Done;
    }

    // Close once, then wait for `WordBalloonCloseComplete` (onMultiModelEvent:eventId:).
    if call.first_time {
        let behavior = if word_balloon_right {
            "StoryWordBalloonCloseRight"
        } else {
            "StoryWordBalloonCloseLeft"
        };
        set_logic_behavior(world, word_balloon_logic, behavior);
    }
    ItemResult::Pending
}

/// `-[StoryManager playStoryEventElementDef:::]` @ 0x10004ca30
pub fn seq_play_story_event_element_def(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let story_sequence = world.obj::<StoryManager>(this).story_sequence;
    let state = match playing_element(world, this) {
        Some(element) => element.state,
        None => None,
    };
    let state = ns_description(state.as_deref());
    let sequence_id = format!("StoryModeEvent{state}");
    set_sequence(world, story_sequence, &sequence_id);
    ItemResult::Replaced
}

/// `-[StoryManager startCelebration:::]` @ 0x10004cb00
pub fn seq_start_celebration(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if let Some(listener) = listener(world, this) {
        crate::controller::on_story_mode_start_celebration(world, listener);
    }
    ItemResult::Done
}

/// `-[StoryManager stopCelebration:::]` @ 0x10004cb30
pub fn seq_stop_celebration(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if let Some(listener) = listener(world, this) {
        crate::controller::on_story_mode_stop_celebration(world, listener);
    }
    ItemResult::Done
}

/// `-[StoryManager startEarthquake:::]` @ 0x10004cb60
pub fn seq_start_earthquake(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if let Some(listener) = listener(world, this) {
        crate::controller::on_story_mode_start_earthquake(world, listener);
    }
    ItemResult::Done
}

/// `-[StoryManager stopEarthquake:::]` @ 0x10004cb90
pub fn seq_stop_earthquake(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if let Some(listener) = listener(world, this) {
        crate::controller::on_story_mode_stop_earthquake(world, listener);
    }
    ItemResult::Done
}

/// `-[StoryManager sharkSnap:::]` @ 0x10004cbc0
pub fn seq_shark_snap(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if let Some(listener) = listener(world, this) {
        crate::controller::on_story_mode_shark_snap(world, listener);
    }
    ItemResult::Done
}

/// `-[StoryManager activateRainbow:::]` @ 0x10004cbf0
pub fn seq_activate_rainbow(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if let Some(listener) = listener(world, this) {
        crate::controller::on_story_mode_activate_rainbow(world, listener);
    }
    ItemResult::Done
}

/// `-[StoryManager activateMyRainbow:::]` @ 0x10004cc20
pub fn seq_activate_my_rainbow(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if let Some(listener) = listener(world, this) {
        crate::controller::on_story_mode_activate_my_rainbow(world, listener);
    }
    ItemResult::Done
}

/// `-[StoryManager deactivateRainbow:::]` @ 0x10004cc50
pub fn seq_deactivate_rainbow(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if let Some(listener) = listener(world, this) {
        crate::controller::on_story_mode_deactivate_rainbow(world, listener);
    }
    ItemResult::Done
}

/// `-[StoryManager onMultiModelEvent:eventId:]` @ 0x10004d434
pub fn on_multi_model_event(world: &mut World, this: Entity, _logic: Entity, event_id: &str) {
    if event_id == "FadeOutComplete" {
        let story_sequence = world.obj::<StoryManager>(this).story_sequence;
        set_sequence(world, story_sequence, "StoryDefComplete");
    }
    if event_id == "CaptionCloseComplete" {
        world.obj_mut::<StoryManager>(this).caption_open = false;
    }
    if event_id == "WordBalloonCloseComplete" {
        world.obj_mut::<StoryManager>(this).word_balloon_open = false;
    }
}

/// `-[StoryManager playStoryDef:]` @ 0x10004d78c
pub fn play_story_def(world: &mut World, this: Entity, story_def: Option<usize>) {
    {
        let mut manager = world.obj_mut::<StoryManager>(this);
        manager.last_story_screen_mode = manager.story_screen_mode;
        manager.story_screen_mode = STORY_SCREEN_MODE_PLAYING;
    }

    if let Some(idol_manager) = crate::managers::idol_manager::instance(world) {
        crate::managers::idol_manager::delay_trigger_episode_event(world, idol_manager, "WatchStory");
    }

    let (fade_out_logic, story_sequence, sequence) = {
        let mut manager = world.obj_mut::<StoryManager>(this);
        manager.active_story_def = story_def;
        (manager.story_overlay_fade_out_logic, manager.story_sequence, manager.sequence)
    };
    set_logic_behavior(world, fade_out_logic, "StoryFadeOutClear");
    set_sequence(world, story_sequence, "PlayStoryDef");
    set_sequence(world, sequence, "StartStory");

    if let Some(listener) = listener(world, this) {
        crate::controller::on_story_start(world, listener);
    }
}

/// `-[StoryManager onPlayStoryDefComplete:]` @ 0x10004d8e4
pub fn on_play_story_def_complete(world: &mut World, this: Entity, play_story_def_complete: bool) {
    let completed = play_story_def_complete;

    let (caption_open, caption_logic) = {
        let manager = world.obj::<StoryManager>(this);
        (manager.caption_open, manager.story_overlay_caption_logic)
    };
    if caption_open {
        world.obj_mut::<StoryManager>(this).caption_open = false;
        set_logic_behavior(world, caption_logic, "StoryCaptionClose");
    }

    let (word_balloon_open, word_balloon_right, word_balloon_logic) = {
        let manager = world.obj::<StoryManager>(this);
        (manager.word_balloon_open, manager.word_balloon_right, manager.story_overlay_word_balloon_logic)
    };
    if word_balloon_open {
        world.obj_mut::<StoryManager>(this).word_balloon_open = false;
        let behavior = if word_balloon_right {
            "StoryWordBalloonCloseRight"
        } else {
            "StoryWordBalloonCloseLeft"
        };
        set_logic_behavior(world, word_balloon_logic, behavior);
    }

    let (last_story_screen_mode, sequence, story_sequence) = {
        let manager = world.obj::<StoryManager>(this);
        (manager.last_story_screen_mode, manager.sequence, manager.story_sequence)
    };
    let return_sequence_id = match (last_story_screen_mode, completed) {
        (2, true) => Some("StoryCompleteFromCreatePanel"),
        (2, false) => Some("CancelStoryFromCreatePanel"),
        (1, true) => Some("StoryCompleteFromMyListPanel"),
        (1, false) => Some("CancelStoryFromMyListPanel"),
        (0, true) => Some("StoryCompleteFromWatchPanel"),
        (0, false) => Some("CancelStoryFromWatchPanel"),
        _ => None,
    };
    if let Some(return_sequence_id) = return_sequence_id {
        set_sequence(world, sequence, return_sequence_id);
    }

    // Original bug, kept: a finished story also reports `onStoryCancel`; `onStoryComplete` is
    // never sent.
    if let Some(listener) = listener(world, this) {
        crate::controller::on_story_cancel(world, listener);
    }
    if let Some(story_sequence) = story_sequence {
        pg_engine::sequence::stop(world, story_sequence);
    }
}
