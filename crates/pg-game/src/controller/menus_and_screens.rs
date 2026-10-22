//! `PocketGodViewController`: methods in this part of the class.

use pg_engine::display;
use pg_engine::display::group;
use pg_engine::display::text;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::object::ObjFn;
use pg_engine::prefs;
use pg_engine::sequence;
use pg_engine::sound;

use crate::persona::persona::Persona;
use crate::persona::tribe_persona::TribePersona;
use crate::prelude::*;
use super::*;

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerMenusAndScreens {
    /// `mSandIslandMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub sand_island_menu_display_group: Option<Entity>,

    /// `mRockIslandMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub rock_island_menu_display_group: Option<Entity>,

    /// `mGraveyardIslandMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub graveyard_island_menu_display_group: Option<Entity>,

    /// `mIceIslandMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub ice_island_menu_display_group: Option<Entity>,

    /// `mApeIslandMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub ape_island_menu_display_group: Option<Entity>,

    /// `mMisfitIslandMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub misfit_island_menu_display_group: Option<Entity>,

    /// `mGodIslandMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub god_island_menu_display_group: Option<Entity>,

    /// `mFailBoatMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub fail_boat_menu_display_group: Option<Entity>,

    /// `mApocalypseIslandMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub apocalypse_island_menu_display_group: Option<Entity>,

    /// `mTimeTempleMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub time_temple_menu_display_group: Option<Entity>,

    /// `mLocustTempleMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub locust_temple_menu_display_group: Option<Entity>,

    /// `mDanceTempleMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub dance_temple_menu_display_group: Option<Entity>,

    /// `mPlagueTempleMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub plague_temple_menu_display_group: Option<Entity>,

    /// `mPurificationTempleMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub purification_temple_menu_display_group: Option<Entity>,

    /// `mClosetTempleMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub closet_temple_menu_display_group: Option<Entity>,

    /// `mXenuTempleMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub xenu_temple_menu_display_group: Option<Entity>,

    /// `mUnderwaterMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub underwater_menu_display_group: Option<Entity>,

    /// `mUnderwaterCoralReefMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub underwater_coral_reef_menu_display_group: Option<Entity>,

    /// `mGlobalIslandMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub global_island_menu_display_group: Option<Entity>,

    /// `mGodPowerMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub god_power_menu_display_group: Option<Entity>,

    /// `mCustomizeMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub customize_menu_display_group: Option<Entity>,

    /// `mCommunityMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub community_menu_display_group: Option<Entity>,

    /// `mSettingsMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub settings_menu_display_group: Option<Entity>,

    /// `mMenuLabelsDisplayGroup`: `@"BCDisplayGroup"`
    pub menu_labels_display_group: Option<Entity>,

    /// `mCategoryMenuDisplayMarker`: `@"BCDisplayMarker"`
    pub category_menu_display_marker: Option<Entity>,

    /// `mCategoryMenuState`: `i`
    pub category_menu_state: i32,

    /// `mMessageDisplayObject`: `@"BCDisplayObject"`
    pub message_display_object: Option<Entity>,

    /// `mMessageSequence`: `@"BCSequence"`
    pub message_sequence: Option<Entity>,

    /// `mbMenuOpen`: `B`
    pub menu_open: bool,

    /// `mTargetMenuButtonZRot`: `f`
    pub target_menu_button_z_rot: f32,

    /// `mTargetMenuYPos`: `f`
    pub target_menu_y_pos: f32,

    /// `mNewPygmyButtonLogic`: `@"BCButtonLogic"`
    pub new_pygmy_button_logic: Option<Entity>,

    /// `mMenuButtonLogic`: `@"BCButtonLogic"`
    pub menu_button_logic: Option<Entity>,

    /// `mIslandLocationButtonLogic`: `@"BCButtonLogic"`
    pub island_location_button_logic: Option<Entity>,

    /// `mIslandMenuButtonLogic`: `@"BCButtonLogic"`
    pub island_menu_button_logic: Option<Entity>,

    /// `mGlobalIslandMenuButtonLogic`: `@"BCButtonLogic"`
    pub global_island_menu_button_logic: Option<Entity>,

    /// `mGodPowerMenuButtonLogic`: `@"BCButtonLogic"`
    pub god_power_menu_button_logic: Option<Entity>,

    /// `mCustomizeMenuButtonLogic`: `@"BCButtonLogic"`
    pub customize_menu_button_logic: Option<Entity>,

    /// `mCommunityMenuButtonLogic`: `@"BCButtonLogic"`
    pub community_menu_button_logic: Option<Entity>,

    /// `mSettingsMenuButtonLogic`: `@"BCButtonLogic"`
    pub settings_menu_button_logic: Option<Entity>,

    /// `mHelpButtonLogic`: `@"BCButtonLogic"`
    pub help_button_logic: Option<Entity>,

    /// `mDebugButtonLogic`: `@"BCButtonLogic"`
    pub debug_button_logic: Option<Entity>,

    /// `mIdolSelectionButtonLogic`: `@"BCButtonLogic"`
    pub idol_selection_button_logic: Option<Entity>,

    /// `mCrackHoleButtonLogic`: `@"BCButtonLogic"`
    pub crack_hole_button_logic: Option<Entity>,

    /// `mTheRunsButtonLogic`: `@"BCButtonLogic"`
    pub the_runs_button_logic: Option<Entity>,

    /// `mCharlieButtonLogic`: `@"BCButtonLogic"`
    pub charlie_button_logic: Option<Entity>,

    /// `mDrumButtonLogic`: `@"BCButtonLogic"`
    pub drum_button_logic: Option<Entity>,

    /// `mTemperatureButtonLogic`: `@"BCButtonLogic"`
    pub temperature_button_logic: Option<Entity>,

    /// `mHurricaneButtonLogic`: `@"BCButtonLogic"`
    pub hurricane_button_logic: Option<Entity>,

    /// `mRainbowButtonLogic`: `@"BCButtonLogic"`
    pub rainbow_button_logic: Option<Entity>,

    /// `mSkinPackButtonLogic`: `@"BCButtonLogic"`
    pub skin_pack_button_logic: Option<Entity>,

    /// `mBonusPackButtonLogic`: `@"BCButtonLogic"`
    pub bonus_pack_button_logic: Option<Entity>,

    /// `mStoryButtonLogic`: `@"BCButtonLogic"`
    pub story_button_logic: Option<Entity>,

    /// `mStatsButtonLogic`: `@"BCButtonLogic"`
    pub stats_button_logic: Option<Entity>,

    /// `mIdolsChallengeButtonLogic`: `@"BCButtonLogic"`
    pub idols_challenge_button_logic: Option<Entity>,

    /// `mNewsButtonLogic`: `@"BCButtonLogic"`
    pub news_button_logic: Option<Entity>,

    /// `mChallengeButtonLogic`: `@"BCButtonLogic"`
    pub challenge_button_logic: Option<Entity>,

    /// `mStartScreenButtonLogic`: `@"BCButtonLogic"`
    pub start_screen_button_logic: Option<Entity>,

    /// `mMuteButtonLogic`: `@"BCButtonLogic"`
    pub mute_button_logic: Option<Entity>,

    /// `mVibrateButtonLogic`: `@"BCButtonLogic"`
    pub vibrate_button_logic: Option<Entity>,

    /// `mGravityButtonLogic`: `@"BCButtonLogic"`
    pub gravity_button_logic: Option<Entity>,

    /// `mNamesButtonLogic`: `@"BCButtonLogic"`
    pub names_button_logic: Option<Entity>,

    /// `mLabelsButtonLogic`: `@"BCButtonLogic"`
    pub labels_button_logic: Option<Entity>,

    /// `mHelpPageIndex`: `i`
    pub help_page_index: i32,

    /// `mbHelpScreenOpen`: `B`
    pub help_screen_open: bool,

    /// `mHelpCloseButtonLogic`: `@"BCButtonLogic"`
    pub help_close_button_logic: Option<Entity>,

    /// `mHelpNextButtonLogic`: `@"BCButtonLogic"`
    pub help_next_button_logic: Option<Entity>,

    /// `mHelpPrevButtonLogic`: `@"BCButtonLogic"`
    pub help_prev_button_logic: Option<Entity>,

    /// `mbDebugScreenOpen`: `B`
    pub debug_screen_open: bool,

    /// `mbWhatsNewScreenOpen`: `B`
    pub whats_new_screen_open: bool,

    /// `mWhatsNewScreenGoButtonLogic`: `@"BCButtonLogic"`
    pub whats_new_screen_go_button_logic: Option<Entity>,

    /// `mbStatsScreenOpen`: `B`
    pub stats_screen_open: bool,

    /// `mLavaButtonLogic`: `@"BCButtonLogic"`
    pub lava_button_logic: Option<Entity>,
}

/// Selectors the controller sends itself with
/// `performSelectorOnMainThread:withObject:self waitUntilDone:NO`, in the order they were sent.
/// Each runs later, on the next pass of the main run loop, once the current call (and the game
/// loop tick that made it) has returned: see [`fire_main_thread_calls`].
#[derive(Resource, Default, Debug)]
pub struct ControllerMainThreadCalls {
    pub calls: Vec<(Entity, ObjFn)>,
}

/// `[self performSelectorOnMainThread:selector withObject:self waitUntilDone:NO]`: queues
/// `selector` for the next pass of the main run loop.
pub fn perform_selector_on_main_thread(world: &mut World, this: Entity, selector: ObjFn) {
    let mut main_thread_calls = world.get_resource_or_insert_with(ControllerMainThreadCalls::default);
    main_thread_calls.calls.push((this, selector));
}

/// The next pass of the main run loop: runs the calls queued so far, in order. Calls queued
/// while they run wait for the following pass. The app drives this after every game loop tick.
pub fn fire_main_thread_calls(world: &mut World) {
    let calls = match world.get_resource_mut::<ControllerMainThreadCalls>() {
        Some(mut main_thread_calls) => std::mem::take(&mut main_thread_calls.calls),
        None => return,
    };

    for (receiver, selector) in calls {
        selector(world, receiver);
    }
}

/// `[mSequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:id]]`.
fn set_controller_sequence_def(world: &mut World, this: Entity, id: &str) {
    let controller_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
    let def = library::sequence_def(world, id);
    if let Some(controller_sequence) = controller_sequence {
        sequence::set_sequence_def(world, controller_sequence, def);
    }
}

/// `[mOptionPreferences getIntPreference:key]` (0 while there is no document).
fn option_int_preference(world: &World, this: Entity, key: &str) -> i32 {
    let option_preferences = world.obj::<ControllerBootAndFrame>(this).option_preferences;
    match option_preferences {
        Some(option_preferences) => prefs::get_int_preference(world, option_preferences, key),
        None => 0,
    }
}

/// `[mOptionPreferences setPreference:key asInt:value]`.
fn set_option_int_preference(world: &mut World, this: Entity, key: &str, value: i32) {
    let option_preferences = world.obj::<ControllerBootAndFrame>(this).option_preferences;
    if let Some(option_preferences) = option_preferences {
        prefs::set_preference_as_int(world, option_preferences, key, value);
    }
}

/// `[button setBehavior:behavior]` on a button logic that may be nil.
fn set_button_behavior(world: &mut World, button: Option<Entity>, behavior: &str) {
    if let Some(button) = button {
        logic::set_behavior(world, button, Some(behavior));
    }
}

/// The on/off settings of `onButtonPress:touch:`: flips the option `key` between 0 and 1 and
/// shows the new state on the pressed button.
fn toggle_option(world: &mut World, this: Entity, button: Entity, key: &str, on_behavior: &str, off_behavior: &str) {
    let value = option_int_preference(world, this, key);
    if value == 0 {
        set_option_int_preference(world, this, key, 1);
        logic::set_behavior(world, button, Some(on_behavior));
    } else {
        set_option_int_preference(world, this, key, 0);
        logic::set_behavior(world, button, Some(off_behavior));
    }
}

/// `[mNewsManager closeNewsScreen]`.
fn close_news_manager_screen(world: &mut World, this: Entity) {
    let news_manager = world.obj::<ControllerManagersBridge>(this).news_manager;
    if let Some(news_manager) = news_manager {
        crate::managers::news_manager::close_news_screen(world, news_manager);
    }
}

/// `[mIdolManager delayTriggerEpisodeEvent:event]`.
fn delay_trigger_episode_event(world: &mut World, this: Entity, event: &str) {
    let idol_manager = world.obj::<ControllerManagersBridge>(this).idol_manager;
    let Some(idol_manager) = idol_manager else {
        return;
    };

    // PORT: `IdolManager::delay_trigger_episode_event` still takes the event as an `Entity`
    // (it is an `NSString`); the event is dropped until it takes the id.
    let _ = (idol_manager, event);
}

/// `[[button displayObject] hide]` or `show`, then `setIsSelectable:` to match (the HUD buttons
/// of `hideUi` / `showUi`).
fn set_hud_button_shown(world: &mut World, button: Option<Entity>, shown: bool) {
    let Some(button) = button else {
        return;
    };
    let button_display_object = logic::display_object(world, button);
    let mut object = display::object_mut(world, button_display_object);
    object.hidden = !shown;
    object.is_selectable = shown;
}

/// The start of a stats page text edit: `mActiveCancelEditDisplayObject = [cancelButton
/// displayObject]`, made selectable and shown.
fn show_cancel_edit_display_object(world: &mut World, this: Entity, cancel_button: Option<Entity>) {
    let cancel_edit_display_object = cancel_button.map(|button| logic::display_object(world, button));
    world.obj_mut::<ControllerAppPlatform>(this).active_cancel_edit_display_object = cancel_edit_display_object;
    if let Some(cancel_edit_display_object) = cancel_edit_display_object {
        display::object_mut(world, cancel_edit_display_object).is_selectable = true;
    }

    let active_cancel_edit_display_object = world.obj::<ControllerAppPlatform>(this).active_cancel_edit_display_object;
    if let Some(active_cancel_edit_display_object) = active_cancel_edit_display_object {
        display::object_mut(world, active_cancel_edit_display_object).hidden = false;
    }
}

/// `[mActiveCancelEditDisplayObject setIsSelectable:NO]`, then `hide`.
fn hide_cancel_edit_display_object(world: &mut World, this: Entity) {
    let active_cancel_edit_display_object = world.obj::<ControllerAppPlatform>(this).active_cancel_edit_display_object;
    if let Some(active_cancel_edit_display_object) = active_cancel_edit_display_object {
        let mut object = display::object_mut(world, active_cancel_edit_display_object);
        object.is_selectable = false;
        object.hidden = true;
    }
}

/// `mEditingText = [[[BCLibrary instance] getDisplayGroupById:group_id] getDisplayObject:text_id]`.
fn set_editing_text(world: &mut World, this: Entity, group_id: &str, text_id: &str) {
    let stats_page = library::display_group(world, group_id);
    let editing_text = stats_page.and_then(|stats_page| group::get_display_object(world, stats_page, text_id));
    world.obj_mut::<ControllerAppPlatform>(this).editing_text = editing_text;
}

/// `[mEditingText setText:text]` (the text a cancelled edit restores).
fn restore_editing_text(world: &mut World, this: Entity, restored_text: Option<String>) {
    let editing_text = world.obj::<ControllerAppPlatform>(this).editing_text;
    if let Some(editing_text) = editing_text {
        text::set_text(world, editing_text, restored_text.as_deref());
    }
}

/// `[persona name]`: the persona's `mName` (nil for a nil persona).
fn persona_name(world: &World, persona: Option<Entity>) -> Option<String> {
    let persona = persona?;
    world.obj::<Persona>(persona).name.clone()
}

/// `[persona description]`: the persona's `mDescription` (nil for a nil persona).
fn persona_description(world: &World, persona: Option<Entity>) -> Option<String> {
    let persona = persona?;
    world.obj::<Persona>(persona).description.clone()
}

/// `[mMessageDisplayObject removeFromDisplayGroup]`, `release` and `mMessageDisplayObject = nil`.
fn release_message_display_object(world: &mut World, this: Entity) {
    let message_display_object = world.obj::<ControllerMenusAndScreens>(this).message_display_object;
    let Some(message_display_object) = message_display_object else {
        return;
    };

    display::remove_from_display_group(world, message_display_object);

    // The controller held the last reference: the message model is deallocated.
    world.despawn(message_display_object);
    world.obj_mut::<ControllerMenusAndScreens>(this).message_display_object = None;
}

/// The island category menu of a location (the inner switch of `setCategoryMenuState:`).
fn island_menu_display_group(menus: &ControllerMenusAndScreens, island_location_state: i32) -> Option<Entity> {
    match island_location_state {
        0 => menus.sand_island_menu_display_group,
        1 => menus.rock_island_menu_display_group,
        2 => menus.underwater_menu_display_group,
        3 => menus.graveyard_island_menu_display_group,
        4 => menus.ice_island_menu_display_group,
        5 => menus.ape_island_menu_display_group,
        6 => menus.misfit_island_menu_display_group,
        7 => menus.god_island_menu_display_group,
        8 => menus.fail_boat_menu_display_group,
        9 => menus.underwater_coral_reef_menu_display_group,
        10 => menus.apocalypse_island_menu_display_group,
        11 => menus.time_temple_menu_display_group,
        12 => menus.locust_temple_menu_display_group,
        13 => menus.dance_temple_menu_display_group,
        14 => menus.plague_temple_menu_display_group,
        15 => menus.purification_temple_menu_display_group,
        16 => menus.closet_temple_menu_display_group,
        17 => menus.xenu_temple_menu_display_group,
        _ => None,
    }
}

/// `-[PocketGodViewController updateHelpPage:]` @ 0x1000761b4
pub fn update_help_page(world: &mut World, this: Entity, page_index: i32) {
    let mut help_page_index = if page_index >= 0 { page_index } else { 68 };
    if 68 < help_page_index {
        help_page_index = 0;
    }
    world.obj_mut::<ControllerMenusAndScreens>(this).help_page_index = help_page_index;

    let help_page_sequence = match help_page_index {
        0 => String::from("HelpPageStart"),
        1..=68 => format!("HelpPage{:02}", help_page_index),
        _ => return,
    };
    set_controller_sequence_def(world, this, &help_page_sequence);
}

/// `-[PocketGodViewController updateHelpPage:::]` @ 0x10007737c
pub fn seq_update_help_page(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let help_page_index = world.obj::<ControllerMenusAndScreens>(this).help_page_index;
    update_help_page(world, this, help_page_index);
    ItemResult::Done
}

/// `-[PocketGodViewController hideUi]` @ 0x1000773ac
pub fn hide_ui(world: &mut World, this: Entity) {
    let new_pygmy_button_logic = world.obj::<ControllerMenusAndScreens>(this).new_pygmy_button_logic;
    set_hud_button_shown(world, new_pygmy_button_logic, false);

    let menu_button_logic = world.obj::<ControllerMenusAndScreens>(this).menu_button_logic;
    set_hud_button_shown(world, menu_button_logic, false);

    let menu_open = world.obj::<ControllerMenusAndScreens>(this).menu_open;
    if menu_open {
        world.obj_mut::<ControllerMenusAndScreens>(this).menu_open = false;
        set_category_menu_state(world, this, 0);
        set_controller_sequence_def(world, this, "CloseMenu");
    }
}

/// `-[PocketGodViewController showUi]` @ 0x1000774f4
pub fn show_ui(world: &mut World, this: Entity) {
    let new_pygmy_button_logic = world.obj::<ControllerMenusAndScreens>(this).new_pygmy_button_logic;
    set_hud_button_shown(world, new_pygmy_button_logic, true);

    let menu_button_logic = world.obj::<ControllerMenusAndScreens>(this).menu_button_logic;
    set_hud_button_shown(world, menu_button_logic, true);
}

/// `-[PocketGodViewController closeScreens]` @ 0x1000775a8
pub fn close_screens(world: &mut World, this: Entity) {
    let map_screen_open = world.obj::<ControllerManagersBridge>(this).map_screen_open;
    if map_screen_open {
        let map_screen = world.obj::<ControllerManagersBridge>(this).map_screen;
        if let Some(map_screen) = map_screen {
            crate::managers::map_screen::close_map_screen(world, map_screen);
        }
    }

    let stats_screen_open = world.obj::<ControllerMenusAndScreens>(this).stats_screen_open;
    if stats_screen_open {
        close_stats_screen(world, this);
    }

    let challenge_screen_open = world.obj::<ControllerManagersBridge>(this).challenge_screen_open;
    if challenge_screen_open {
        let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
        if let Some(challenge_manager) = challenge_manager {
            crate::managers::challenge_manager::close_challenge_screen(world, challenge_manager);
        }
    }

    let skin_pack_screen_open = world.obj::<ControllerManagersBridge>(this).skin_pack_screen_open;
    if skin_pack_screen_open {
        let skin_pack_manager = world.obj::<ControllerManagersBridge>(this).skin_pack_manager;
        if let Some(skin_pack_manager) = skin_pack_manager {
            crate::managers::skin_pack_manager::close_skin_pack_screen(world, skin_pack_manager);
        }
    }

    let bonus_pack_screen_open = world.obj::<ControllerManagersBridge>(this).bonus_pack_screen_open;
    if bonus_pack_screen_open {
        let bonus_pack_manager = world.obj::<ControllerManagersBridge>(this).bonus_pack_manager;
        if let Some(bonus_pack_manager) = bonus_pack_manager {
            crate::managers::bonus_pack_manager::close_bonus_pack_screen(world, bonus_pack_manager);
        }
    }

    let story_screen_open = world.obj::<ControllerManagersBridge>(this).story_screen_open;
    if story_screen_open {
        let story_manager = world.obj::<ControllerManagersBridge>(this).story_manager;
        if let Some(story_manager) = story_manager {
            crate::managers::story_manager::close_story_screen(world, story_manager);
        }
    }

    let news_screen_open = world.obj::<ControllerManagersBridge>(this).news_screen_open;
    if news_screen_open {
        close_news_manager_screen(world, this);
    }

    let idol_screen_open = world.obj::<ControllerManagersBridge>(this).idol_screen_open;
    if idol_screen_open {
        let idol_manager = world.obj::<ControllerManagersBridge>(this).idol_manager;
        if let Some(idol_manager) = idol_manager {
            crate::managers::idol_manager::close_idol_screen(world, idol_manager);
        }
    }

    let help_screen_open = world.obj::<ControllerMenusAndScreens>(this).help_screen_open;
    if help_screen_open {
        world.obj_mut::<ControllerMenusAndScreens>(this).help_screen_open = false;
        load_monster_texture_atlas(world, this);
        set_controller_sequence_def(world, this, "CloseHelp");
    }

    let debug_screen_open = world.obj::<ControllerMenusAndScreens>(this).debug_screen_open;
    if debug_screen_open {
        world.obj_mut::<ControllerMenusAndScreens>(this).debug_screen_open = false;
        let debug_screen = world.obj::<ControllerManagersBridge>(this).debug_screen;
        if let Some(debug_screen) = debug_screen {
            crate::managers::debug_screen::close_debug_screen(world, debug_screen);
        }
    }
}

/// `-[PocketGodViewController isDialogActive]` @ 0x100078430
pub fn is_dialog_active(world: &mut World, this: Entity) -> bool {
    let bridge = world.obj::<ControllerManagersBridge>(this);
    if bridge.challenge_dialog_open {
        return true;
    }
    bridge.idol_award_screen_open
}

/// `-[PocketGodViewController onButtonPress:touch:]` @ 0x100079090
pub fn on_button_press(world: &mut World, this: Entity, button: Entity, _touch: Entity) {
    let pressed = Some(button);
    let menus = world.obj::<ControllerMenusAndScreens>(this).clone();
    let features = world.obj::<ControllerIslandFeatures>(this).clone();
    let weather_button_logic = world.obj::<ControllerSkyWeather>(this).weather_button_logic;
    let stats = world.obj::<ControllerStatsAndAchievements>(this).clone();

    if pressed == menus.new_pygmy_button_logic {
        let charlie_state = option_int_preference(world, this, "CharlieState");
        let mut max_pygmy_count = 6;
        if charlie_state == 1 {
            max_pygmy_count = 7;
        }
        let tom_pygmy_alive = is_tom_pygmy_alive(world, this);
        let live_pygmy_count = world.obj::<ControllerPygmies>(this).live_pygmy_count;
        if i32::from(tom_pygmy_alive) + max_pygmy_count <= live_pygmy_count {
            return;
        }
        set_controller_sequence_def(world, this, "AddPygmy");
        return;
    }

    if pressed == menus.menu_button_logic {
        let menu_open = world.obj::<ControllerMenusAndScreens>(this).menu_open;
        if !menu_open {
            world.obj_mut::<ControllerMenusAndScreens>(this).menu_open = true;
            set_controller_sequence_def(world, this, "OpenMenu");
        } else {
            world.obj_mut::<ControllerMenusAndScreens>(this).menu_open = false;
            set_category_menu_state(world, this, 0);
            set_controller_sequence_def(world, this, "CloseMenu");
        }
        return;
    }

    if pressed == menus.island_location_button_logic {
        cancel_all_touches(world, this);
        close_news_manager_screen(world, this);
        world.obj_mut::<ControllerManagersBridge>(this).map_screen_open = true;
        let map_screen = world.obj::<ControllerManagersBridge>(this).map_screen;
        if let Some(map_screen) = map_screen {
            crate::managers::map_screen::open_map_screen(world, map_screen);
        }
        return;
    }

    if pressed == menus.island_menu_button_logic {
        set_category_menu_state(world, this, 1);
        return;
    }

    if pressed == menus.global_island_menu_button_logic {
        set_category_menu_state(world, this, 2);
        return;
    }

    if pressed == menus.god_power_menu_button_logic {
        set_category_menu_state(world, this, 3);
        return;
    }

    if pressed == menus.customize_menu_button_logic {
        set_category_menu_state(world, this, 4);
        return;
    }

    if pressed == menus.community_menu_button_logic {
        set_category_menu_state(world, this, 5);
        return;
    }

    if pressed == menus.settings_menu_button_logic {
        set_category_menu_state(world, this, 6);
        return;
    }

    if pressed == menus.help_button_logic {
        cancel_all_touches(world, this);
        close_news_manager_screen(world, this);
        world.obj_mut::<ControllerMenusAndScreens>(this).help_screen_open = true;
        set_controller_sequence_def(world, this, "OpenHelp");
        return;
    }

    if pressed == menus.debug_button_logic {
        cancel_all_touches(world, this);
        close_news_manager_screen(world, this);
        world.obj_mut::<ControllerMenusAndScreens>(this).debug_screen_open = true;
        set_controller_sequence_def(world, this, "CloseMenu");
        let debug_screen = world.obj::<ControllerManagersBridge>(this).debug_screen;
        if let Some(debug_screen) = debug_screen {
            crate::managers::debug_screen::open_debug_screen(world, debug_screen);
        }
        return;
    }

    if pressed == menus.help_close_button_logic {
        world.obj_mut::<ControllerMenusAndScreens>(this).help_screen_open = false;
        load_monster_texture_atlas(world, this);
        set_controller_sequence_def(world, this, "CloseHelp");
        return;
    }

    if pressed == menus.help_next_button_logic {
        let help_page_index = world.obj::<ControllerMenusAndScreens>(this).help_page_index;
        update_help_page(world, this, help_page_index + 1);
        return;
    }

    if pressed == menus.help_prev_button_logic {
        let help_page_index = world.obj::<ControllerMenusAndScreens>(this).help_page_index;
        update_help_page(world, this, help_page_index - 1);
        return;
    }

    if pressed == features.sand_boulder_button_logic {
        set_sand_island_state(world, this, 1, false);
        return;
    }

    if pressed == features.sand_fish_button_logic {
        set_sand_island_state(world, this, 2, false);
        return;
    }

    if pressed == features.sand_ant_hill_button_logic {
        set_sand_island_state(world, this, 3, false);
        return;
    }

    if pressed == features.sand_water_wings_button_logic {
        set_sand_island_state(world, this, 4, false);
        return;
    }

    if pressed == features.rock_fish_button_logic {
        set_rock_island_state(world, this, 1, false);
        return;
    }

    if pressed == features.rock_wood_button_logic {
        set_rock_island_feature_state(world, this, 1, false);
        return;
    }

    if pressed == features.rock_egg_button_logic {
        set_rock_island_feature_state(world, this, 2, false);
        return;
    }

    if pressed == features.rock_out_house_button_logic {
        set_rock_island_out_house_state(world, this, 1, false);
        return;
    }

    if pressed == features.rock_t_rex_button_logic {
        set_rock_island_t_rex_state(world, this, 1, false);
        return;
    }

    if pressed == features.rock_spear_button_logic {
        set_rock_island_spear_state(world, this, 1, false);
        return;
    }

    if pressed == features.graveyard_zombie_grave_button_logic {
        set_graveyard_island_zombie_grave_state(world, this, 1, false);
        return;
    }

    if pressed == features.graveyard_ghost_button_logic {
        set_graveyard_island_ghost_state(world, this, 1, false);
        return;
    }

    if pressed == features.graveyard_spider_web_button_logic {
        set_graveyard_island_spider_web_state(world, this, 1, false);
        return;
    }

    if pressed == features.graveyard_spider_button_logic {
        set_graveyard_island_spider_state(world, this, 1, false);
        return;
    }

    if pressed == features.graveyard_spear_button_logic {
        set_graveyard_island_spear_state(world, this, 1, false);
        return;
    }

    if pressed == features.graveyard_water_wings_button_logic {
        set_graveyard_island_state(world, this, 1, false);
        return;
    }

    if pressed == features.ice_fish_button_logic {
        set_ice_island_state(world, this, 1, false);
        return;
    }

    if pressed == features.ice_water_wings_button_logic {
        set_ice_island_state(world, this, 2, false);
        return;
    }

    if pressed == features.ice_ice_monster_button_logic {
        set_ice_island_ice_monster_state(world, this, 1, false);
        return;
    }

    if pressed == features.ice_igloo_button_logic {
        set_ice_island_igloo_state(world, this, 1, false);
        return;
    }

    if pressed == features.ice_ice_hole_button_logic {
        set_ice_island_ice_hole_state(world, this, 1, false);
        return;
    }

    if pressed == features.ice_ice_squid_button_logic {
        set_ice_island_ice_squid_state(world, this, 1, false);
        return;
    }

    if pressed == features.ape_ape_altar_button_logic {
        set_ape_island_state(world, this, 1, false);
        return;
    }

    if pressed == features.ape_ape_button_logic {
        set_ape_island_ape_state(world, this, 1, false);
        return;
    }

    if pressed == features.ape_banana_tree_button_logic {
        set_ape_island_state(world, this, 2, false);
        return;
    }

    if pressed == features.ape_monkey_button_logic {
        set_ape_island_monkey_state(world, this, 1, false);
        return;
    }

    if pressed == features.ape_konkey_dong_button_logic {
        toggle_option(world, this, button, "KonkeyDongState", "ToggleKonkeyDongOn", "ToggleKonkeyDongOff");
        return;
    }

    if pressed == features.misfit_moron_pests_button_logic {
        toggle_option(world, this, button, "MoronPestsState", "ToggleMoronPestsOn", "ToggleMoronPestsOff");
        return;
    }

    if pressed == features.misfit_moron_snake_button_logic {
        set_misfit_island_state(world, this, 1, false);
        return;
    }

    if pressed == features.misfit_water_wings_button_logic {
        set_misfit_island_state(world, this, 2, false);
        return;
    }

    if pressed == features.idols_button_logic {
        set_god_island_idols_state(world, this, 1, false);
        return;
    }

    if pressed == features.idol_water_wings_button_logic {
        set_god_island_state(world, this, 1, false);
        return;
    }

    if pressed == features.apocalypse_water_wings_button_logic {
        set_apocalypse_island_state(world, this, 1, false);
        return;
    }

    if pressed == features.underwater_piranha_button_logic {
        set_underwater_state(world, this, 1, false);
        return;
    }

    if pressed == features.underwater_laser_shark_button_logic {
        set_underwater_state(world, this, 2, false);
        return;
    }

    if pressed == features.underwater_tile_drain_button_logic {
        set_underwater_state(world, this, 3, false);
        return;
    }

    if pressed == features.underwater_coral_reef_jellyfish_button_logic {
        set_underwater_coral_reef_state(world, this, 1, false);
        return;
    }

    if pressed == weather_button_logic {
        let weather_hazard = option_int_preference(world, this, "WeatherHazard");
        let mut weather_state = 0;
        if weather_hazard < 1 {
            weather_state = weather_hazard + 1;
        }
        set_weather_state(world, this, weather_state, false);
        return;
    }

    if pressed == features.shark_button_logic {
        set_shark_state(world, this, 1, false);
        return;
    }

    if pressed == features.bird_button_logic {
        set_bird_state(world, this, 1, false);
        return;
    }

    if pressed == menus.crack_hole_button_logic {
        toggle_option(world, this, button, "CrackHoleState", "ToggleCrackHoleOn", "ToggleCrackHoleOff");
        return;
    }

    if pressed == menus.idol_selection_button_logic {
        cancel_all_touches(world, this);
        close_news_manager_screen(world, this);
        world.obj_mut::<ControllerManagersBridge>(this).idol_screen_open = true;
        set_controller_sequence_def(world, this, "CloseMenu");
        let idol_manager = world.obj::<ControllerManagersBridge>(this).idol_manager;
        if let Some(idol_manager) = idol_manager {
            crate::managers::idol_manager::open_idol_selection_screen(world, idol_manager);
        }
        return;
    }

    if pressed == menus.the_runs_button_logic {
        toggle_option(world, this, button, "TheRunsState", "ToggleTheRunsOn", "ToggleTheRunsOff");
        return;
    }

    if pressed == menus.charlie_button_logic {
        let charlie_state = option_int_preference(world, this, "CharlieState");
        if charlie_state == 0 {
            set_option_int_preference(world, this, "CharlieState", 1);
            logic::set_behavior(world, button, Some("ToggleCharlieOn"));
            let personas = world.obj::<ControllerStatsAndAchievements>(this);
            let tribe_persona = personas.tribe_persona;
            let charlie_pygmy_persona = personas.charlie_pygmy_persona;
            if let (Some(tribe_persona), Some(charlie_pygmy_persona)) = (tribe_persona, charlie_pygmy_persona) {
                crate::persona::tribe_persona::activate_guest_pygmy_persona(world, tribe_persona, charlie_pygmy_persona);
            }
        } else {
            set_option_int_preference(world, this, "CharlieState", 0);
            logic::set_behavior(world, button, Some("ToggleCharlieOff"));
            let personas = world.obj::<ControllerStatsAndAchievements>(this);
            let tribe_persona = personas.tribe_persona;
            let charlie_pygmy_persona = personas.charlie_pygmy_persona;
            if let (Some(tribe_persona), Some(charlie_pygmy_persona)) = (tribe_persona, charlie_pygmy_persona) {
                crate::persona::tribe_persona::deactivate_guest_pygmy_persona(world, tribe_persona, charlie_pygmy_persona);
            }
        }
        return;
    }

    if pressed == menus.drum_button_logic {
        toggle_option(world, this, button, "DrumHazard", "ToggleDrumOn", "ToggleDrumOff");
        return;
    }

    if pressed == menus.temperature_button_logic {
        let temperature_state = option_int_preference(world, this, "TemperatureState");
        if temperature_state == 0 {
            set_option_int_preference(world, this, "TemperatureState", 1);
            logic::set_behavior(world, button, Some("ToggleTemperatureOn"));
        } else {
            set_sun_state(world, this, 0);
            set_option_int_preference(world, this, "TemperatureState", 0);
            logic::set_behavior(world, button, Some("ToggleTemperatureOff"));
        }
        return;
    }

    if pressed == menus.hurricane_button_logic {
        toggle_option(world, this, button, "HurricaneState", "ToggleHurricaneOn", "ToggleHurricaneOff");
        return;
    }

    if pressed == menus.rainbow_button_logic {
        toggle_option(world, this, button, "RainbowState", "ToggleRainbowOn", "ToggleRainbowOff");
        return;
    }

    if pressed == menus.skin_pack_button_logic {
        cancel_all_touches(world, this);
        close_news_manager_screen(world, this);
        open_skin_pack_manager(world, this);
        return;
    }

    if pressed == menus.bonus_pack_button_logic {
        cancel_all_touches(world, this);
        open_bonus_pack_manager(world, this);
        return;
    }

    if pressed == menus.story_button_logic {
        cancel_all_touches(world, this);
        close_news_manager_screen(world, this);
        open_story_manager(world, this);
        return;
    }

    if pressed == menus.stats_button_logic {
        cancel_all_touches(world, this);
        close_news_manager_screen(world, this);
        world.obj_mut::<ControllerMenusAndScreens>(this).stats_screen_open = true;
        world.obj_mut::<ControllerStatsAndAchievements>(this).active_stats_screen = 1;
        update_tribe_stats(world, this);
        set_controller_sequence_def(world, this, "OpenStatsTribe");
        delay_trigger_episode_event(world, this, "OpenTribeScreen");
        return;
    }

    if pressed == menus.idols_challenge_button_logic {
        cancel_all_touches(world, this);
        close_news_manager_screen(world, this);
        world.obj_mut::<ControllerManagersBridge>(this).idol_screen_open = true;
        set_controller_sequence_def(world, this, "CloseMenu");
        let idol_manager = world.obj::<ControllerManagersBridge>(this).idol_manager;
        if let Some(idol_manager) = idol_manager {
            crate::managers::idol_manager::open_idol_list_screen(world, idol_manager);
        }
        return;
    }

    if pressed == menus.news_button_logic {
        cancel_all_touches(world, this);
        close_news_manager_screen(world, this);
        world.obj_mut::<ControllerManagersBridge>(this).news_screen_open = true;
        set_controller_sequence_def(world, this, "CloseMenu");
        let news_manager = world.obj::<ControllerManagersBridge>(this).news_manager;
        if let Some(news_manager) = news_manager {
            crate::managers::news_manager::open_news_screen(world, news_manager);
        }
        return;
    }

    if pressed == menus.challenge_button_logic {
        cancel_all_touches(world, this);
        close_news_manager_screen(world, this);
        open_challenge_manager(world, this);
        return;
    }

    if pressed == menus.start_screen_button_logic {
        cancel_all_touches(world, this);
        close_news_manager_screen(world, this);
        world.obj_mut::<ControllerManagersBridge>(this).idol_screen_open = true;
        set_controller_sequence_def(world, this, "CloseMenu");
        let idol_manager = world.obj::<ControllerManagersBridge>(this).idol_manager;
        if let Some(idol_manager) = idol_manager {
            crate::managers::idol_manager::open_idol_start_screen(world, idol_manager);
        }
        return;
    }

    if pressed == menus.mute_button_logic {
        let mute = option_int_preference(world, this, "Mute");
        if mute == 0 {
            sound::mute(world, true);
            set_option_int_preference(world, this, "Mute", 1);
            logic::set_behavior(world, button, Some("ToggleMuteOn"));
            sound::stop_all_sounds(world);
        } else {
            sound::mute(world, false);
            set_option_int_preference(world, this, "Mute", 0);
            logic::set_behavior(world, button, Some("ToggleMuteOff"));
            play_background_loop(world, this, None);
        }
        return;
    }

    if pressed == menus.vibrate_button_logic {
        let vibrate = option_int_preference(world, this, "Vibrate");
        if vibrate == 0 {
            sound::mute_vibrate(world, true);
            set_option_int_preference(world, this, "Vibrate", 1);
            logic::set_behavior(world, button, Some("ToggleVibrateOn"));
        } else {
            sound::mute_vibrate(world, false);
            set_option_int_preference(world, this, "Vibrate", 0);
            logic::set_behavior(world, button, Some("ToggleVibrateOff"));
        }
        return;
    }

    if pressed == menus.gravity_button_logic {
        toggle_option(world, this, button, "GravityHazard", "ToggleGravityOn", "ToggleGravityOff");
        return;
    }

    if pressed == menus.names_button_logic {
        toggle_option(world, this, button, "DisplayNames", "ToggleNamesOn", "ToggleNamesOff");
        return;
    }

    if pressed == menus.labels_button_logic {
        let button_labels = option_int_preference(world, this, "ButtonLabels");
        if button_labels == 0 {
            set_option_int_preference(world, this, "ButtonLabels", 1);
            logic::set_behavior(world, button, Some("ToggleLabelsOn"));
            let current = world.obj::<ControllerMenusAndScreens>(this);
            let category_menu_display_marker = current.category_menu_display_marker;
            let menu_labels_display_group = current.menu_labels_display_group;
            if let Some(category_menu_display_marker) = category_menu_display_marker {
                display::insert_after(world, category_menu_display_marker, menu_labels_display_group);
            }
        } else {
            set_option_int_preference(world, this, "ButtonLabels", 0);
            logic::set_behavior(world, button, Some("ToggleLabelsOff"));
            let menu_labels_display_group = world.obj::<ControllerMenusAndScreens>(this).menu_labels_display_group;
            if let Some(menu_labels_display_group) = menu_labels_display_group {
                display::remove_from_display_group(world, menu_labels_display_group);
            }
        }
        return;
    }

    if pressed == menus.whats_new_screen_go_button_logic {
        world.obj_mut::<ControllerMenusAndScreens>(this).whats_new_screen_open = false;
        set_controller_sequence_def(world, this, "CloseWhatsNewScreen");
        return;
    }

    if pressed == stats.stats_tribe_close_button_logic {
        close_stats_screen(world, this);
        return;
    }

    if pressed == stats.stats_tribe_edit_name_button_logic {
        if world.obj::<ControllerAppPlatform>(this).keyboard_open {
            return;
        }
        {
            let mut platform = world.obj_mut::<ControllerAppPlatform>(this);
            platform.keyboard_open = true;
            platform.max_edit_char_count = 16;
        }
        set_editing_text(world, this, "StatsPageTribe", "TribeName");
        let cancel_button = world.obj::<ControllerStatsAndAchievements>(this).stats_tribe_cancel_edit_name_button_logic;
        show_cancel_edit_display_object(world, this, cancel_button);
        perform_selector_on_main_thread(world, this, show_keyboard);
        set_controller_sequence_def(world, this, "TribeStatsEditName");
        return;
    }

    if pressed == stats.stats_tribe_cancel_edit_name_button_logic {
        if !world.obj::<ControllerAppPlatform>(this).keyboard_open {
            return;
        }
        world.obj_mut::<ControllerAppPlatform>(this).keyboard_open = false;
        let tribe_persona = world.obj::<ControllerStatsAndAchievements>(this).tribe_persona;
        let tribe_name = persona_name(world, tribe_persona);
        restore_editing_text(world, this, tribe_name);
        hide_cancel_edit_display_object(world, this);
        perform_selector_on_main_thread(world, this, hide_keyboard);
        set_controller_sequence_def(world, this, "TribeStatsCancelEditName");
        return;
    }

    if pressed == stats.stats_pygmy_close_button_logic {
        world.obj_mut::<ControllerStatsAndAchievements>(this).active_stats_screen = 1;
        if world.obj::<ControllerAppPlatform>(this).keyboard_open {
            world.obj_mut::<ControllerAppPlatform>(this).keyboard_open = false;
            hide_cancel_edit_display_object(world, this);
            perform_selector_on_main_thread(world, this, hide_keyboard);
        }
        update_tribe_stats(world, this);
        set_controller_sequence_def(world, this, "CloseStatsPygmy");
        return;
    }

    if pressed == stats.stats_pygmy_next_button_logic {
        if world.obj::<ControllerAppPlatform>(this).keyboard_open {
            return;
        }
        let pygmy_stats_type = world.obj::<ControllerStatsAndAchievements>(this).pygmy_stats_type;
        let mut next_pygmy_stats_type = 0;
        if pygmy_stats_type < 1 {
            next_pygmy_stats_type = pygmy_stats_type + 1;
        }
        world.obj_mut::<ControllerStatsAndAchievements>(this).pygmy_stats_type = next_pygmy_stats_type;
        update_pygmy_stats(world, this);
        set_controller_sequence_def(world, this, "NextStatsPygmy");
        return;
    }

    if pressed == stats.stats_pygmy_edit_name_button_logic {
        if world.obj::<ControllerAppPlatform>(this).keyboard_open {
            return;
        }
        {
            let mut platform = world.obj_mut::<ControllerAppPlatform>(this);
            platform.keyboard_open = true;
            platform.max_edit_char_count = 10;
        }
        delay_trigger_episode_event(world, this, "ChangePygmyName");
        set_editing_text(world, this, "StatsPagePygmy", "PygmyName");
        let cancel_button = world.obj::<ControllerStatsAndAchievements>(this).stats_pygmy_cancel_edit_name_button_logic;
        show_cancel_edit_display_object(world, this, cancel_button);
        perform_selector_on_main_thread(world, this, show_keyboard);
        set_controller_sequence_def(world, this, "PygmyStatsEditName");
        return;
    }

    if pressed == stats.stats_pygmy_edit_description_button_logic {
        if world.obj::<ControllerAppPlatform>(this).keyboard_open {
            return;
        }
        {
            let mut platform = world.obj_mut::<ControllerAppPlatform>(this);
            platform.keyboard_open = true;
            platform.max_edit_char_count = 30;
        }
        set_editing_text(world, this, "StatsPagePygmy", "PygmyDescription");
        let cancel_button = world.obj::<ControllerStatsAndAchievements>(this).stats_pygmy_cancel_edit_description_button_logic;
        show_cancel_edit_display_object(world, this, cancel_button);
        perform_selector_on_main_thread(world, this, show_keyboard);
        set_controller_sequence_def(world, this, "PygmyStatsEditDescription");
        return;
    }

    if pressed == stats.stats_pygmy_cancel_edit_name_button_logic {
        if !world.obj::<ControllerAppPlatform>(this).keyboard_open {
            return;
        }
        world.obj_mut::<ControllerAppPlatform>(this).keyboard_open = false;
        let active_pygmy_persona = world.obj::<ControllerStatsAndAchievements>(this).active_pygmy_persona;
        let pygmy_name = persona_name(world, active_pygmy_persona);
        restore_editing_text(world, this, pygmy_name);
        hide_cancel_edit_display_object(world, this);
        perform_selector_on_main_thread(world, this, hide_keyboard);
        set_controller_sequence_def(world, this, "PygmyStatsCancelEditName");
        return;
    }

    if pressed == stats.stats_pygmy_cancel_edit_description_button_logic {
        if !world.obj::<ControllerAppPlatform>(this).keyboard_open {
            return;
        }
        world.obj_mut::<ControllerAppPlatform>(this).keyboard_open = false;
        let active_pygmy_persona = world.obj::<ControllerStatsAndAchievements>(this).active_pygmy_persona;
        let pygmy_description = persona_description(world, active_pygmy_persona);
        restore_editing_text(world, this, pygmy_description);
        hide_cancel_edit_display_object(world, this);
        perform_selector_on_main_thread(world, this, hide_keyboard);
        set_controller_sequence_def(world, this, "PygmyStatsCancelEditDescription");
        return;
    }

    if pressed == stats.stats_god_close_button_logic {
        world.obj_mut::<ControllerStatsAndAchievements>(this).active_stats_screen = 1;
        if world.obj::<ControllerAppPlatform>(this).keyboard_open {
            world.obj_mut::<ControllerAppPlatform>(this).keyboard_open = false;
            hide_cancel_edit_display_object(world, this);
            perform_selector_on_main_thread(world, this, hide_keyboard);
        }
        update_tribe_stats(world, this);
        set_controller_sequence_def(world, this, "CloseStatsGod");
        return;
    }

    if pressed == stats.stats_god_next_god_type_button_logic {
        if world.obj::<ControllerAppPlatform>(this).keyboard_open {
            return;
        }
        let god_persona = world.obj::<ControllerStatsAndAchievements>(this).god_persona;
        let god_type = match god_persona {
            Some(god_persona) => crate::persona::god_persona::god_type(world, god_persona),
            None => 0,
        };
        let mut next_god_type = 0;
        if god_type < 6 {
            next_god_type = god_type + 1;
        }
        let god_persona = world.obj::<ControllerStatsAndAchievements>(this).god_persona;
        if let Some(god_persona) = god_persona {
            crate::persona::god_persona::set_god_type(world, god_persona, next_god_type);
        }
        delay_trigger_episode_event(world, this, "ChangeGodType");
        update_god_stats(world, this);
        set_controller_sequence_def(world, this, "NextGodType");
        return;
    }

    if pressed == stats.stats_god_edit_name_button_logic {
        if world.obj::<ControllerAppPlatform>(this).keyboard_open {
            return;
        }
        {
            let mut platform = world.obj_mut::<ControllerAppPlatform>(this);
            platform.keyboard_open = true;
            platform.max_edit_char_count = 10;
        }
        set_editing_text(world, this, "StatsPageGod", "GodName");
        let cancel_button = world.obj::<ControllerStatsAndAchievements>(this).stats_god_cancel_edit_name_button_logic;
        show_cancel_edit_display_object(world, this, cancel_button);
        perform_selector_on_main_thread(world, this, show_keyboard);
        set_controller_sequence_def(world, this, "GodStatsEditName");
        return;
    }

    if pressed == stats.stats_god_edit_description_button_logic {
        if world.obj::<ControllerAppPlatform>(this).keyboard_open {
            return;
        }
        {
            let mut platform = world.obj_mut::<ControllerAppPlatform>(this);
            platform.keyboard_open = true;
            platform.max_edit_char_count = 30;
        }
        set_editing_text(world, this, "StatsPageGod", "GodDescription");
        let cancel_button = world.obj::<ControllerStatsAndAchievements>(this).stats_god_cancel_edit_description_button_logic;
        show_cancel_edit_display_object(world, this, cancel_button);
        perform_selector_on_main_thread(world, this, show_keyboard);
        set_controller_sequence_def(world, this, "GodStatsEditDescription");
        return;
    }

    if pressed == stats.stats_god_cancel_edit_name_button_logic {
        if !world.obj::<ControllerAppPlatform>(this).keyboard_open {
            return;
        }
        world.obj_mut::<ControllerAppPlatform>(this).keyboard_open = false;
        let god_persona = world.obj::<ControllerStatsAndAchievements>(this).god_persona;
        let god_name = persona_name(world, god_persona);
        restore_editing_text(world, this, god_name);
        hide_cancel_edit_display_object(world, this);
        perform_selector_on_main_thread(world, this, hide_keyboard);
        set_controller_sequence_def(world, this, "GodStatsCancelEditName");
        return;
    }

    if pressed == stats.stats_god_cancel_edit_description_button_logic {
        if !world.obj::<ControllerAppPlatform>(this).keyboard_open {
            return;
        }
        world.obj_mut::<ControllerAppPlatform>(this).keyboard_open = false;
        let god_persona = world.obj::<ControllerStatsAndAchievements>(this).god_persona;
        let god_description = persona_description(world, god_persona);
        restore_editing_text(world, this, god_description);
        hide_cancel_edit_display_object(world, this);
        perform_selector_on_main_thread(world, this, hide_keyboard);
        set_controller_sequence_def(world, this, "GodStatsCancelEditDescription");
        return;
    }

    if pressed == stats.god_stats_button_logic {
        if world.obj::<ControllerAppPlatform>(this).keyboard_open {
            return;
        }
        world.obj_mut::<ControllerStatsAndAchievements>(this).active_stats_screen = 3;
        update_god_stats(world, this);
        set_controller_sequence_def(world, this, "OpenStatsGod");
        return;
    }

    let pygmy_stats_button_logic_array = &stats.pygmy_stats_button_logic_array;
    if !pygmy_stats_button_logic_array.contains(&button) {
        return;
    }
    if world.obj::<ControllerAppPlatform>(this).keyboard_open {
        return;
    }
    world.obj_mut::<ControllerStatsAndAchievements>(this).active_stats_screen = 2;

    let tribe_persona = world.obj::<ControllerStatsAndAchievements>(this).tribe_persona;
    let button_index = pygmy_stats_button_logic_array.iter().position(|pygmy_stats_button| *pygmy_stats_button == button);
    let active_pygmy_persona = match (tribe_persona, button_index) {
        (Some(tribe_persona), Some(button_index)) => {
            let pygmy_persona_array = &world.obj::<TribePersona>(tribe_persona).pygmy_persona_array;
            Some(pygmy_persona_array[button_index])
        }
        _ => None,
    };
    world.obj_mut::<ControllerStatsAndAchievements>(this).active_pygmy_persona = active_pygmy_persona;
    update_pygmy_stats(world, this);
    set_controller_sequence_def(world, this, "OpenStatsPygmy");
}

/// `-[PocketGodViewController onButtonRelease:touch:]` @ 0x10007bc28
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[PocketGodViewController onButtonReleaseOutside:touch:]` @ 0x10007bc2c
pub fn on_button_release_outside(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[PocketGodViewController updateMenuLabelsDisplayGroup:]` @ 0x10007dae0
pub fn update_menu_labels_display_group(world: &mut World, this: Entity, menu_group: Option<Entity>) {
    let menu_bg = menu_group.and_then(|menu_group| group::get_display_object(world, menu_group, "MenuBG"));

    let mut child_index: i32 = 0;
    let mut label_number: i32 = 1;
    loop {
        let menu_labels_display_group = world.obj::<ControllerMenusAndScreens>(this).menu_labels_display_group;
        let label_id = format!("Label{}Text", label_number);
        let label = menu_labels_display_group.and_then(|labels| group::get_display_object(world, labels, &label_id));
        let Some(label) = label else {
            return;
        };

        loop {
            let child = menu_group.and_then(|menu_group| group::get_display_object_at_index(world, menu_group, child_index));
            let Some(child) = child else {
                text::set_text(world, label, Some(""));
                break;
            };

            let child_object = display::object(world, child);
            if !child_object.is_selectable {
                child_index = child_index.wrapping_add(1);
                continue;
            }

            let child_data = child_object.data.clone();
            text::set_text(world, label, child_data.as_deref());

            // `[MenuBG pos]` is only dereferenced here: a menu without one would crash.
            let menu_bg = menu_bg.expect("a category menu with buttons has a MenuBG");
            let menu_bg_x = display::object(world, menu_bg).pos.x;
            if menu_bg_x <= 240.0 {
                display::object_mut(world, label).pos.x = menu_bg_x + 60.0;
            } else {
                let text_width = text::text_width(world, label);
                let label_scale_x = display::object(world, label).scale.x;
                display::object_mut(world, label).pos.x = (menu_bg_x - text_width * label_scale_x) + -10.0;
            }
            break;
        }

        child_index = child_index.wrapping_add(1);
        label_number = label_number.wrapping_add(1);
    }
}

/// `-[PocketGodViewController setCategoryMenuState:]` @ 0x10007dd04
pub fn set_category_menu_state(world: &mut World, this: Entity, category_menu_state: i32) {
    let requested_state = category_menu_state;

    let menus = world.obj::<ControllerMenusAndScreens>(this).clone();
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let closed_menu = match menus.category_menu_state {
        1 => {
            let island_menu = island_menu_display_group(&menus, island_location_state);
            Some((island_menu, menus.island_menu_button_logic, "ToggleIslandOff"))
        }
        2 => Some((menus.global_island_menu_display_group, menus.global_island_menu_button_logic, "ToggleGlobalIslandOff")),
        3 => Some((menus.god_power_menu_display_group, menus.god_power_menu_button_logic, "ToggleGodPowerOff")),
        4 => Some((menus.customize_menu_display_group, menus.customize_menu_button_logic, "ToggleCustomizeOff")),
        5 => Some((menus.community_menu_display_group, menus.community_menu_button_logic, "ToggleCommunityOff")),
        6 => Some((menus.settings_menu_display_group, menus.settings_menu_button_logic, "ToggleSettingsOff")),
        _ => None,
    };
    if let Some((closed_menu_group, closed_menu_button, off_behavior)) = closed_menu {
        if let Some(closed_menu_group) = closed_menu_group {
            display::remove_from_display_group(world, closed_menu_group);
        }
        set_button_behavior(world, closed_menu_button, off_behavior);
    }

    let current_state = world.obj::<ControllerMenusAndScreens>(this).category_menu_state;
    let mut new_state = 0;
    if current_state != requested_state {
        new_state = requested_state;
    }
    world.obj_mut::<ControllerMenusAndScreens>(this).category_menu_state = new_state;

    let menus = world.obj::<ControllerMenusAndScreens>(this).clone();
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let opened_menu = match new_state {
        1 => {
            let island_menu = island_menu_display_group(&menus, island_location_state);
            Some((island_menu, menus.island_menu_button_logic, "ToggleIslandOn"))
        }
        2 => Some((menus.global_island_menu_display_group, menus.global_island_menu_button_logic, "ToggleGlobalIslandOn")),
        3 => Some((menus.god_power_menu_display_group, menus.god_power_menu_button_logic, "ToggleGodPowerOn")),
        4 => Some((menus.customize_menu_display_group, menus.customize_menu_button_logic, "ToggleCustomizeOn")),
        5 => Some((menus.community_menu_display_group, menus.community_menu_button_logic, "ToggleCommunityOn")),
        6 => Some((menus.settings_menu_display_group, menus.settings_menu_button_logic, "ToggleSettingsOn")),
        _ => None,
    };
    let mut opened_menu_group = None;
    if let Some((menu_group, menu_button, on_behavior)) = opened_menu {
        opened_menu_group = menu_group;
        set_button_behavior(world, menu_button, on_behavior);
    }

    update_menu_labels_display_group(world, this, opened_menu_group);

    let category_menu_display_marker = world.obj::<ControllerMenusAndScreens>(this).category_menu_display_marker;
    if let Some(category_menu_display_marker) = category_menu_display_marker {
        display::insert_after(world, category_menu_display_marker, opened_menu_group);
    }

    let category_menu_open = world.obj::<ControllerMenusAndScreens>(this).category_menu_state != 0;
    if category_menu_open && option_int_preference(world, this, "ButtonLabels") == 1 {
        let current = world.obj::<ControllerMenusAndScreens>(this);
        let category_menu_display_marker = current.category_menu_display_marker;
        let menu_labels_display_group = current.menu_labels_display_group;
        if let Some(category_menu_display_marker) = category_menu_display_marker {
            display::insert_after(world, category_menu_display_marker, menu_labels_display_group);
        }
        return;
    }

    let menu_labels_display_group = world.obj::<ControllerMenusAndScreens>(this).menu_labels_display_group;
    if let Some(menu_labels_display_group) = menu_labels_display_group {
        display::remove_from_display_group(world, menu_labels_display_group);
    }
}

/// `-[PocketGodViewController areNameLabelsEnabled]` @ 0x100087034
pub fn are_name_labels_enabled(world: &mut World, this: Entity) -> bool {
    option_int_preference(world, this, "DisplayNames") == 1
}

/// `-[PocketGodViewController areButtonLabelsEnabled]` @ 0x100087070
pub fn are_button_labels_enabled(world: &mut World, this: Entity) -> bool {
    option_int_preference(world, this, "ButtonLabels") == 1
}

/// `-[PocketGodViewController isScreenOpen]` @ 0x1000870c4
pub fn is_screen_open(world: &mut World, this: Entity) -> bool {
    let bridge = world.obj::<ControllerManagersBridge>(this);
    let menus = world.obj::<ControllerMenusAndScreens>(this);
    bridge.map_screen_open
        || menus.stats_screen_open
        || menus.whats_new_screen_open
        || bridge.challenge_screen_open
        || bridge.skin_pack_screen_open
        || bridge.bonus_pack_screen_open
        || bridge.story_screen_open
        || bridge.news_screen_open
        || bridge.idol_screen_open
        || menus.help_screen_open
        || menus.debug_screen_open
}

/// `-[PocketGodViewController (Game) openMenu:::]` @ 0x100159e2c
pub fn seq_open_menu(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut menus = world.obj_mut::<ControllerMenusAndScreens>(this);
    menus.target_menu_button_z_rot = 0.0;
    menus.target_menu_y_pos = 0.0;
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) closeMenu:::]` @ 0x100159e54
pub fn seq_close_menu(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ControllerMenusAndScreens>(this).menu_open = false;
    set_category_menu_state(world, this, 0);

    let mut menus = world.obj_mut::<ControllerMenusAndScreens>(this);
    menus.target_menu_button_z_rot = 90.0;
    menus.target_menu_y_pos = 90.0;
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) showMessage:::]` @ 0x100159ebc
pub fn seq_show_message(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    release_message_display_object(world, this);

    let x_pos = call.float("xPos");
    let y_pos = call.float("yPos");
    let z_pos = call.float("zPos");
    let texture_def = match call.param("texture") {
        Some(texture_id) => library::texture_def_id(world, texture_id),
        None => None,
    };

    let message_display_object = display::model::spawn(world, Some("Message"), texture_def);
    world.obj_mut::<ControllerMenusAndScreens>(this).message_display_object = Some(message_display_object);
    {
        let mut message_object = display::object_mut(world, message_display_object);
        message_object.pos.x = x_pos;
        message_object.pos.y = y_pos;
        message_object.pos.z = z_pos;
        message_object.set_alpha(0.0);
    }

    let menu_display_marker = world.obj::<ControllerBootAndFrame>(this).menu_display_marker;
    if let Some(menu_display_marker) = menu_display_marker {
        display::insert_after(world, menu_display_marker, Some(message_display_object));
    }

    let message_sequence = world.obj::<ControllerMenusAndScreens>(this).message_sequence;
    let show_message = library::sequence_def(world, "ShowMessage");
    if let Some(message_sequence) = message_sequence {
        sequence::set_sequence_def(world, message_sequence, show_message);
    }
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) removeMessage:::]` @ 0x10015a0bc
pub fn seq_remove_message(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    release_message_display_object(world, this);
    ItemResult::Done
}
