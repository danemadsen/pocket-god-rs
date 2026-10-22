//! `PocketGodViewController`: the bridge to the manager singletons. It opens the skin pack,
//! bonus pack, story, news, idol and challenge screens and hears when they close; it lends the
//! world to story mode (environment, pygmies, earthquake, rainbow, celebration); and it passes
//! sequence episode events on to the `IdolManager` (`docs/re/controller.md` §4.9–§4.11).

use crate::prelude::*;
use super::*;

use pg_engine::library::Library;
use pg_engine::{display, prefs, sequence};

use crate::logic::ocean_creatures::shark_logic;
use crate::logic::sky_powers::rainbow_logic;
use crate::managers::{bonus_pack_manager, challenge_manager, idol_manager, news_manager};
use crate::managers::{skin_pack_manager, story_manager};
use crate::pygmy;

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerManagersBridge {
    /// `mChallengeManager`: `@"ChallengeManager"`
    pub challenge_manager: Option<Entity>,

    /// `mSkinPackManager`: `@"SkinPackManager"`
    pub skin_pack_manager: Option<Entity>,

    /// `mBonusPackManager`: `@"BonusPackManager"`
    pub bonus_pack_manager: Option<Entity>,

    /// `mStoryManager`: `@"StoryManager"`
    pub story_manager: Option<Entity>,

    /// `mDebugScreen`: `@"DebugScreen"`
    pub debug_screen: Option<Entity>,

    /// `mNewsManager`: `@"NewsManager"`
    pub news_manager: Option<Entity>,

    /// `mIdolManager`: `@"IdolManager"`
    pub idol_manager: Option<Entity>,

    /// `mGameUiManager`: `@"GameUiManager"`
    pub game_ui_manager: Option<Entity>,

    /// `mMapScreen`: `@"MapScreen"`
    pub map_screen: Option<Entity>,

    /// `mbSkinPackScreenOpen`: `B`
    pub skin_pack_screen_open: bool,

    /// `mbBonusPackScreenOpen`: `B`
    pub bonus_pack_screen_open: bool,

    /// `mbMapScreenOpen`: `B`
    pub map_screen_open: bool,

    /// `mbStoryScreenOpen`: `B`
    pub story_screen_open: bool,

    /// `mbStoryActive`: `B`
    pub story_active: bool,

    /// `mStorySaveLabelMode`: `i`
    pub story_save_label_mode: i32,

    /// `mbNewsTeaserOpen`: `B`
    pub news_teaser_open: bool,

    /// `mbNewsScreenOpen`: `B`
    pub news_screen_open: bool,

    /// `mbIdolScreenOpen`: `B`
    pub idol_screen_open: bool,

    /// `mbIdolAwardScreenOpen`: `B`
    pub idol_award_screen_open: bool,

    /// `mbIdolBattleResultsScreenOpen`: `B`
    pub idol_battle_results_screen_open: bool,

    /// `mbChallengeScreenOpen`: `B`
    pub challenge_screen_open: bool,

    /// `mbChallengeDialogOpen`: `B`
    pub challenge_dialog_open: bool,

    /// `mbChallengeActive`: `B`
    pub challenge_active: bool,
}


// `mIslandLocationState` values.
const LOCATION_SAND_ISLAND: i32 = 0;
const LOCATION_ROCK_ISLAND: i32 = 1;
const LOCATION_GRAVEYARD_ISLAND: i32 = 3;
const LOCATION_ICE_ISLAND: i32 = 4;
const LOCATION_APE_ISLAND: i32 = 5;
const LOCATION_MISFIT_ISLAND: i32 = 6;
const LOCATION_GOD_ISLAND: i32 = 7;
const LOCATION_FAIL_BOAT: i32 = 8;
const LOCATION_APOCALYPSE_ISLAND: i32 = 10;

/// The story's ape island location codes (`BCStoryEnvironmentElementDef` location 4 and 5): ape
/// island with `mApeIslandState` 1 or 2.
const STORY_LOCATION_APE_ISLAND_1: i32 = 4;
const STORY_LOCATION_APE_ISLAND_2: i32 = 5;

// The story's weather codes (`setWeatherState:` values).
const STORY_WEATHER_STORM: i32 = 1;
const WEATHER_STATE_CLEAR: i32 = 0;
const WEATHER_STATE_STORM: i32 = 1;

// The story's time-of-day codes.
const STORY_TIME_OF_DAY_NIGHT: i32 = 1;
const STORY_TIME_OF_DAY_SUNSET: i32 = 2;

/// The challenge games an episode alert waits for (`isGameActive:`, in the original's order):
/// Ooga Jump, Bait Master, laser shark, Pain Drain, The Runs, Konkey Dong, Moron Pests, No Pee,
/// Dance Dance Execution.
const EPISODE_ALERT_BLOCKING_GAMES: [i32; 9] = [4, 5, 6, 10, 11, 12, 13, 15, 14];

/// `@"CloseMenu"`: the main sequence closes the menu when a manager screen opens.
const CLOSE_MENU_SEQUENCE: &str = "CloseMenu";

/// `@"StartGameAfterIdolScreen"`.
const START_GAME_AFTER_IDOL_SCREEN_SEQUENCE: &str = "StartGameAfterIdolScreen";

/// `@"EarthquakeStart"` / `@"EarthquakeStop"`: the scripts of `mEarthquakeSequence`.
const EARTHQUAKE_START_SEQUENCE: &str = "EarthquakeStart";
const EARTHQUAKE_STOP_SEQUENCE: &str = "EarthquakeStop";

/// System preferences: whether the skin pack and bonus pack info were shown.
const INFO_SHOWN_KEY: &str = "InfoShown";
const BONUS_INFO_SHOWN_KEY: &str = "BonusInfoShown";

/// Option preference: the pygmy name label mode (story mode forces 1, names shown).
const DISPLAY_NAMES_KEY: &str = "DisplayNames";

/// The winch loop that stops when a story starts.
const WINCH_TURN_SOUND: &str = "WinchTurn";

/// `FUN_100038d68(@"TextureAtlas0", @"raw", &DAT_100751f38)`: the main atlas and its slot.
const MAIN_ATLAS_IMAGE: &str = "TextureAtlas0";
const MAIN_ATLAS_ID: &str = "atlas0";

/// A story adds pygmies only while fewer than this many live.
const STORY_MAX_LIVE_PYGMIES: i32 = 6;

/// `-[PocketGodViewController openSkinPackManager]` @ 0x1000778b8
///
/// Every pygmy stops dancing; the first opening shows the store's info (system `InfoShown`).
pub fn open_skin_pack_manager(world: &mut World, this: Entity) {
    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        pygmy::stop_dancing_now(world, pygmy);
    }

    world.obj_mut::<ControllerManagersBridge>(this).skin_pack_screen_open = true;
    set_controller_sequence(world, this, CLOSE_MENU_SEQUENCE);

    let info_shown = system_int_preference(world, this, INFO_SHOWN_KEY);
    let first_time = info_shown == 0;
    if first_time {
        set_system_int_preference(world, this, INFO_SHOWN_KEY, 1);
    }

    let skin_pack_manager = world.obj::<ControllerManagersBridge>(this).skin_pack_manager;
    if let Some(skin_pack_manager) = skin_pack_manager {
        skin_pack_manager::open_skin_pack_screen(world, skin_pack_manager, first_time);
    }
}

/// `-[PocketGodViewController openBonusPackManager]` @ 0x100077abc
///
/// The first opening shows the bonus info (system `BonusInfoShown`).
pub fn open_bonus_pack_manager(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).bonus_pack_screen_open = true;
    set_controller_sequence(world, this, CLOSE_MENU_SEQUENCE);

    let bonus_info_shown = system_int_preference(world, this, BONUS_INFO_SHOWN_KEY);
    let first_time = bonus_info_shown == 0;
    if first_time {
        set_system_int_preference(world, this, BONUS_INFO_SHOWN_KEY, 1);
    }

    let bonus_pack_manager = world.obj::<ControllerManagersBridge>(this).bonus_pack_manager;
    if let Some(bonus_pack_manager) = bonus_pack_manager {
        bonus_pack_manager::open_bonus_pack_screen(world, bonus_pack_manager, first_time);
    }
}

/// `-[PocketGodViewController openStoryManager]` @ 0x100077bc0
pub fn open_story_manager(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).story_screen_open = true;
    set_controller_sequence(world, this, CLOSE_MENU_SEQUENCE);

    let story_manager = world.obj::<ControllerManagersBridge>(this).story_manager;
    if let Some(story_manager) = story_manager {
        story_manager::open_story_screen(world, story_manager);
    }
}

/// `-[PocketGodViewController openChallengeManager]` @ 0x100077c58
///
/// A running challenge keeps the screen shut (`cantOpenChallengeScreen` says so).
pub fn open_challenge_manager(world: &mut World, this: Entity) {
    let challenge_active = world.obj::<ControllerManagersBridge>(this).challenge_active;
    if challenge_active {
        let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
        if let Some(challenge_manager) = challenge_manager {
            challenge_manager::cant_open_challenge_screen(world, challenge_manager);
        }
        return;
    }

    world.obj_mut::<ControllerManagersBridge>(this).challenge_screen_open = true;
    set_controller_sequence(world, this, CLOSE_MENU_SEQUENCE);

    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    if let Some(challenge_manager) = challenge_manager {
        challenge_manager::open_challenge_screen(world, challenge_manager);
    }
}

/// `-[PocketGodViewController onBonusPackScreenClosed]` @ 0x100077d38
pub fn on_bonus_pack_screen_closed(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).bonus_pack_screen_open = false;
}

/// `-[PocketGodViewController onBonusPackUnlockDialogOpened]` @ 0x100077d48
pub fn on_bonus_pack_unlock_dialog_opened(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).bonus_pack_screen_open = true;
}

/// `-[PocketGodViewController onBonusPackUnlockDialogClosed]` @ 0x100077d5c
pub fn on_bonus_pack_unlock_dialog_closed(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).bonus_pack_screen_open = false;
}

/// `-[PocketGodViewController onSkinPackScreenClosed]` @ 0x100077d6c
pub fn on_skin_pack_screen_closed(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).skin_pack_screen_open = false;
    load_monster_texture_atlas(world, this);
}

/// `-[PocketGodViewController onStoryScreenClosed]` @ 0x100077d88
pub fn on_story_screen_closed(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).story_screen_open = false;
}

/// `-[PocketGodViewController onStoryModeStartCelebration]` @ 0x100077d98
pub fn on_story_mode_start_celebration(world: &mut World, this: Entity) {
    start_celebration(world, this);
}

/// `-[PocketGodViewController onStoryModeStopCelebration]` @ 0x100077da8
pub fn on_story_mode_stop_celebration(world: &mut World, this: Entity) {
    stop_celebration(world, this);
}

/// `-[PocketGodViewController onStoryModeStartEarthquake]` @ 0x100077db8
///
/// A new `mEarthquakeSequence` runs `EarthquakeStart` (one still running is overwritten, not
/// released, as in the original) and every pygmy quakes.
pub fn on_story_mode_start_earthquake(world: &mut World, this: Entity) {
    let earthquake_def = pg_engine::library::sequence_def(world, EARTHQUAKE_START_SEQUENCE);
    let earthquake_sequence = sequence::spawn(world, earthquake_def, Some(this));
    world.obj_mut::<ControllerGodPowers>(this).earthquake_sequence = Some(earthquake_sequence);

    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        pygmy::start_story_earthquake(world, pygmy);
    }
}

/// `-[PocketGodViewController onStoryModeStopEarthquake]` @ 0x100077f3c
///
/// `mEarthquakeSequence` plays `EarthquakeStop` (whose `stopEarthquake:::` ends it) and every
/// pygmy stops quaking.
pub fn on_story_mode_stop_earthquake(world: &mut World, this: Entity) {
    let earthquake_sequence = world.obj::<ControllerGodPowers>(this).earthquake_sequence;
    let earthquake_stop_def = pg_engine::library::sequence_def(world, EARTHQUAKE_STOP_SEQUENCE);
    if let Some(earthquake_sequence) = earthquake_sequence {
        sequence::set_sequence_def(world, earthquake_sequence, earthquake_stop_def);
    }

    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        pygmy::stop_story_earthquake(world, pygmy);
    }
}

/// `-[PocketGodViewController onStoryModeSharkSnap]` @ 0x1000780a0
pub fn on_story_mode_shark_snap(world: &mut World, this: Entity) {
    let shark = world.obj::<ControllerWorldRegistry>(this).shark_logic;
    if let Some(shark) = shark {
        shark_logic::story_mode_shark_snap(world, shark);
    }
}

/// `-[PocketGodViewController onStoryModeActivateRainbow]` @ 0x1000780bc
pub fn on_story_mode_activate_rainbow(world: &mut World, this: Entity) {
    let rainbow = world.obj::<ControllerGodPowers>(this).rainbow_logic;
    if let Some(rainbow) = rainbow {
        rainbow_logic::story_mode_activate_rainbow(world, rainbow);
    }
}

/// `-[PocketGodViewController onStoryModeActivateMyRainbow]` @ 0x1000780d8
pub fn on_story_mode_activate_my_rainbow(world: &mut World, this: Entity) {
    let rainbow = world.obj::<ControllerGodPowers>(this).rainbow_logic;
    if let Some(rainbow) = rainbow {
        rainbow_logic::story_mode_activate_my_rainbow(world, rainbow);
    }
}

/// `-[PocketGodViewController onStoryModeDeactivateRainbow]` @ 0x1000780f4
pub fn on_story_mode_deactivate_rainbow(world: &mut World, this: Entity) {
    let rainbow = world.obj::<ControllerGodPowers>(this).rainbow_logic;
    if let Some(rainbow) = rainbow {
        rainbow_logic::story_mode_deactivate_rainbow(world, rainbow);
    }
}

/// `-[PocketGodViewController removeAllPygmiesFromStoryMode]` @ 0x100078110
pub fn remove_all_pygmies_from_story_mode(world: &mut World, this: Entity) {
    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        pygmy::remove_from_story_mode(world, pygmy);
    }
}

/// `-[PocketGodViewController onStoryStart]` @ 0x10007821c
///
/// The island is lent to the story: sleeping sounds pause, the pygmies and the rainbow leave
/// story mode, names are shown (the option is saved for `onStoryComplete`), the island logics
/// leave and the HUD hides.
pub fn on_story_start(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).story_active = true;
    pause_sleeping_sounds(world, this);
    pg_engine::sound::stop_sound(world, WINCH_TURN_SOUND);
    remove_all_pygmies_from_story_mode(world, this);
    clear_story_mode_rainbow(world, this);

    let display_names = option_int_preference(world, this, DISPLAY_NAMES_KEY);
    world.obj_mut::<ControllerManagersBridge>(this).story_save_label_mode = display_names;
    set_option_int_preference(world, this, DISPLAY_NAMES_KEY, 1);

    disable_island_logics(world, this);
    hide_ui(world, this);
}

/// `-[PocketGodViewController onStoryComplete]` @ 0x100078344
pub fn on_story_complete(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).story_active = false;
    resume_sleeping_sounds(world, this);
    remove_all_pygmies_from_story_mode(world, this);
    clear_story_mode_rainbow(world, this);

    let story_save_label_mode = world.obj::<ControllerManagersBridge>(this).story_save_label_mode;
    set_option_int_preference(world, this, DISPLAY_NAMES_KEY, story_save_label_mode);

    enable_island_logics(world, this);
    show_ui(world, this);
}

/// `-[PocketGodViewController onStoryCancel]` @ 0x100078400
pub fn on_story_cancel(world: &mut World, this: Entity) {
    on_story_complete(world, this);
}

/// `-[PocketGodViewController isStoryActive]` @ 0x100078410
pub fn is_story_active(world: &mut World, this: Entity) -> bool {
    world.obj::<ControllerManagersBridge>(this).story_active
}

/// `-[PocketGodViewController onDebugScreenClosed]` @ 0x100078420
pub fn on_debug_screen_closed(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerMenusAndScreens>(this).debug_screen_open = false;
}

/// `-[PocketGodViewController onStoryEnvironmentChange:]` @ 0x100078460
///
/// Applies a story environment element (an id in the story manager's `StoryDefStore`): travels
/// to its location (ape island also picks its state), then sets the weather and the time of day,
/// each only when it differs.
pub fn on_story_environment_change(world: &mut World, this: Entity, story_environment_element_def: Option<usize>) {
    // The story manager always passes its playing element; the original reads through the
    // pointer without a check.
    let element_def = story_environment_element_def.expect("onStoryEnvironmentChange: a story environment element");

    let location = story_manager::story_def_store(world).element_def(element_def).location;
    let old_island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    match location {
        STORY_LOCATION_APE_ISLAND_1 | STORY_LOCATION_APE_ISLAND_2 => {
            if old_island_location_state != LOCATION_APE_ISLAND {
                set_island_location_state(world, this, LOCATION_APE_ISLAND, false);
            }

            let story_ape_island_state = if location == STORY_LOCATION_APE_ISLAND_1 { 1 } else { 2 };
            let ape_island_state = world.obj::<ControllerIslandFeatures>(this).ape_island_state;
            if ape_island_state != story_ape_island_state {
                set_ape_island_state(world, this, story_ape_island_state, false);
            }
        }
        _ => {
            let new_island_location_state = story_island_location_state(location);
            if old_island_location_state != new_island_location_state {
                set_island_location_state(world, this, new_island_location_state, false);
            }
        }
    }

    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if old_island_location_state != island_location_state {
        remove_all_pygmies_from_story_mode(world, this);
    }

    let weather = story_manager::story_def_store(world).element_def(element_def).weather;
    let weather_state = world.obj::<ControllerSkyWeather>(this).weather_state;
    if weather == STORY_WEATHER_STORM {
        if weather_state != WEATHER_STATE_STORM {
            set_weather_state(world, this, WEATHER_STATE_STORM, false);
        }
    } else if weather_state != WEATHER_STATE_CLEAR {
        set_weather_state(world, this, WEATHER_STATE_CLEAR, false);
    }

    let time_of_day = story_manager::story_def_store(world).element_def(element_def).time_of_day;
    let (night_active, sunset_active) = {
        let sky_weather = world.obj::<ControllerSkyWeather>(this);
        (sky_weather.night_active, sky_weather.sunset_active)
    };
    if time_of_day == STORY_TIME_OF_DAY_NIGHT {
        if !night_active {
            start_night(world, this);
        }
    } else if time_of_day == STORY_TIME_OF_DAY_SUNSET {
        if !sunset_active {
            start_sunset(world, this);
        }
    } else if sunset_active || night_active {
        start_day(world, this);
    }
}

/// `-[PocketGodViewController onStoryAddPygmy:]` @ 0x100078840
///
/// The pygmy playing the story character `name`: the one with that name label, else a new
/// story-mode pygmy while fewer than six live, else nil.
pub fn on_story_add_pygmy(world: &mut World, this: Entity, name: &str) -> Option<Entity> {
    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        let pygmy_name = pygmy::pygmy_name_label(world, pygmy);
        if pygmy_name.as_deref() == Some(name) {
            return Some(pygmy);
        }
    }

    let live_pygmy_count = world.obj::<ControllerPygmies>(this).live_pygmy_count;
    if live_pygmy_count >= STORY_MAX_LIVE_PYGMIES {
        return None;
    }

    let new_pygmy = create_pygmy_logic(world, this);
    if let Some(new_pygmy) = new_pygmy {
        pygmy::set_pygmy_name_label(world, new_pygmy, Some(name));
        pygmy::set_story_mode(world, new_pygmy);
    }
    new_pygmy
}

/// `-[PocketGodViewController onNewsScreenClosed]` @ 0x1000789ec
pub fn on_news_screen_closed(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).news_screen_open = false;
}

/// `-[PocketGodViewController onNewsTeaserClosed]` @ 0x1000789fc
pub fn on_news_teaser_closed(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).news_teaser_open = false;
}

/// `-[PocketGodViewController onNewsTeaserLaunchNewsScreen]` @ 0x100078a0c
pub fn on_news_teaser_launch_news_screen(world: &mut World, this: Entity) {
    {
        let mut bridge = world.obj_mut::<ControllerManagersBridge>(this);
        bridge.news_teaser_open = false;
        bridge.news_screen_open = true;
    }
    set_controller_sequence(world, this, CLOSE_MENU_SEQUENCE);

    let news_manager = world.obj::<ControllerManagersBridge>(this).news_manager;
    if let Some(news_manager) = news_manager {
        news_manager::open_news_screen(world, news_manager);
    }
}

/// `-[PocketGodViewController onNewsLaunchStoreScreen]` @ 0x100078ab0
pub fn on_news_launch_store_screen(world: &mut World, this: Entity) {
    close_news_flags(world, this);
    open_skin_pack_manager(world, this);
}

/// `-[PocketGodViewController onNewsLaunchChallengeScreen]` @ 0x100078ad8
pub fn on_news_launch_challenge_screen(world: &mut World, this: Entity) {
    close_news_flags(world, this);
    open_challenge_manager(world, this);
}

/// `-[PocketGodViewController onNewsLaunchStoryScreen]` @ 0x100078b00
pub fn on_news_launch_story_screen(world: &mut World, this: Entity) {
    close_news_flags(world, this);
    open_story_manager(world, this);
}

/// `-[PocketGodViewController onIdolScreenClosed]` @ 0x100078b28
///
/// After the battle results the main atlas comes back and the island returns in place of the
/// arena; a game that waited for the idol start screen begins.
pub fn on_idol_screen_closed(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).idol_screen_open = false;

    let idol_battle_results_screen_open = world.obj::<ControllerManagersBridge>(this).idol_battle_results_screen_open;
    if idol_battle_results_screen_open {
        world.obj_mut::<ControllerManagersBridge>(this).idol_battle_results_screen_open = false;

        let main_atlas = world.resource::<Library>().atlas_id(MAIN_ATLAS_ID);
        let Some(main_atlas) = main_atlas else {
            panic!("onIdolScreenClosed: there is no atlas slot {MAIN_ATLAS_ID}");
        };
        pg_engine::texture::load_atlas(world, main_atlas, MAIN_ATLAS_IMAGE);

        let battle_of_the_gods_display_group = world.obj::<ControllerMinigames>(this).battle_of_the_gods_display_group;
        if let Some(battle_of_the_gods_display_group) = battle_of_the_gods_display_group {
            display::remove_from_display_group(world, battle_of_the_gods_display_group);
        }

        let (island_display_group, island_root_display_marker) = {
            let locations = world.obj::<ControllerIslandLocations>(this);
            (locations.island_display_group, locations.island_root_display_marker)
        };
        if let Some(island_display_group) = island_display_group {
            display::move_to_display_group(world, island_display_group, island_root_display_marker);
        }
    }

    load_monster_texture_atlas(world, this);

    let add_pygmy_after_idol_start_screen = world.obj::<ControllerBootAndFrame>(this).add_pygmy_after_idol_start_screen;
    if add_pygmy_after_idol_start_screen {
        world.obj_mut::<ControllerBootAndFrame>(this).add_pygmy_after_idol_start_screen = false;
        set_controller_sequence(world, this, START_GAME_AFTER_IDOL_SCREEN_SEQUENCE);
    }
}

/// `-[PocketGodViewController onIdolScreenLaunchStoreScreen]` @ 0x100078c48
pub fn on_idol_screen_launch_store_screen(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).idol_screen_open = false;
    open_skin_pack_manager(world, this);
}

/// `-[PocketGodViewController onIdolAwardScreenOpened]` @ 0x100078c64
pub fn on_idol_award_screen_opened(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).idol_award_screen_open = true;
}

/// `-[PocketGodViewController onIdolAwardScreenClosed]` @ 0x100078c78
pub fn on_idol_award_screen_closed(world: &mut World, this: Entity) {
    let mut bridge = world.obj_mut::<ControllerManagersBridge>(this);
    bridge.idol_screen_open = false;
    bridge.idol_award_screen_open = false;
}

/// `-[PocketGodViewController onIdolAwardScreenOpenChallenges]` @ 0x100078c94
pub fn on_idol_award_screen_open_challenges(world: &mut World, this: Entity) {
    cancel_all_touches(world, this);
    close_news_screen(world, this);
    {
        let mut bridge = world.obj_mut::<ControllerManagersBridge>(this);
        bridge.idol_screen_open = true;
        bridge.idol_award_screen_open = false;
    }
    set_controller_sequence(world, this, CLOSE_MENU_SEQUENCE);

    let idol_manager = world.obj::<ControllerManagersBridge>(this).idol_manager;
    if let Some(idol_manager) = idol_manager {
        idol_manager::open_idol_list_screen(world, idol_manager);
    }
}

/// `-[PocketGodViewController onEpisodeAlertOpenChallengeDetail:]` @ 0x100078d78
///
/// Opens the idol detail screen of the episode (a node index of the episode table) unless a
/// screen, a story, a timed challenge game, No Pee or a dialog is active.
pub fn on_episode_alert_open_challenge_detail(world: &mut World, this: Entity, episode_node: Option<usize>) {
    if is_screen_open(world, this) {
        return;
    }
    if is_story_active(world, this) {
        return;
    }
    for challenge_type in EPISODE_ALERT_BLOCKING_GAMES {
        if is_challenge_game_active(world, this, challenge_type) {
            return;
        }
    }
    if is_no_pee_active(world, this) {
        return;
    }
    if is_dialog_active(world, this) {
        return;
    }

    cancel_all_touches(world, this);
    close_news_screen(world, this);
    {
        let mut bridge = world.obj_mut::<ControllerManagersBridge>(this);
        bridge.idol_screen_open = true;
        bridge.idol_award_screen_open = false;
    }
    set_controller_sequence(world, this, CLOSE_MENU_SEQUENCE);

    let idol_manager = world.obj::<ControllerManagersBridge>(this).idol_manager;
    if let Some(idol_manager) = idol_manager {
        idol_manager::open_idol_detail_screen(world, idol_manager, episode_node);
    }
}

/// `-[PocketGodViewController onBattleAlertSelect]` @ 0x100078f90
///
/// The original only logs `ONBATTLEALERTSELECT`.
pub fn on_battle_alert_select(_world: &mut World, _this: Entity) {}

/// `-[PocketGodViewController onChallengeScreenClosed]` @ 0x100078ffc
pub fn on_challenge_screen_closed(world: &mut World, this: Entity) {
    let mut bridge = world.obj_mut::<ControllerManagersBridge>(this);
    bridge.challenge_screen_open = false;
    bridge.challenge_dialog_open = false;
}

/// `-[PocketGodViewController onChallengeDialogOpen]` @ 0x100079018
pub fn on_challenge_dialog_open(world: &mut World, this: Entity) {
    cancel_all_touches(world, this);

    let mut bridge = world.obj_mut::<ControllerManagersBridge>(this);
    bridge.challenge_screen_open = true;
    bridge.challenge_dialog_open = true;
}

/// `-[PocketGodViewController onStartChallenge:]` @ 0x100079060
pub fn on_start_challenge(world: &mut World, this: Entity, _challenge_type: i32) {
    let mut bridge = world.obj_mut::<ControllerManagersBridge>(this);
    bridge.challenge_screen_open = false;
    bridge.challenge_active = true;
}

/// `-[PocketGodViewController onEndChallenge:]` @ 0x100079080
pub fn on_end_challenge(world: &mut World, this: Entity, _challenge_type: i32) {
    world.obj_mut::<ControllerManagersBridge>(this).challenge_active = false;
}

/// `-[PocketGodViewController (Game) triggerEpisodeEvent:::]` @ 0x10015a78c
///
/// `[0] event` goes to the `IdolManager`.
pub fn seq_trigger_episode_event(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let idol_manager = world.obj::<ControllerManagersBridge>(this).idol_manager;

    // Every shipped item names its event; "" stands in for a nil one (the manager takes a string).
    let event = call.param("event").unwrap_or("");
    if let Some(idol_manager) = idol_manager {
        idol_manager::trigger_episode_event(world, idol_manager, event);
    }
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) isAnyGameActive]` @ 0x100166304
pub fn is_any_game_active(world: &mut World, this: Entity) -> bool {
    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    match challenge_manager {
        Some(challenge_manager) => challenge_manager::is_any_game_active(world, challenge_manager),
        None => false,
    }
}

/// The island location state of a story location code (`onStoryEnvironmentChange:`'s jump
/// table; the ape island codes are handled by the caller): 1 Rock, 2 Graveyard, 3 Ice,
/// 6 Misfit, 7 God, 8 Fail boat, 9 Apocalypse, 10–16 the temples 11–17, anything else Sand.
fn story_island_location_state(location: i32) -> i32 {
    match location {
        1 => LOCATION_ROCK_ISLAND,
        2 => LOCATION_GRAVEYARD_ISLAND,
        3 => LOCATION_ICE_ISLAND,
        6 => LOCATION_MISFIT_ISLAND,
        7 => LOCATION_GOD_ISLAND,
        8 => LOCATION_FAIL_BOAT,
        9 => LOCATION_APOCALYPSE_ISLAND,
        10..=16 => location + 1,
        _ => LOCATION_SAND_ISLAND,
    }
}

/// `mbNewsTeaserOpen = NO; mbNewsScreenOpen = NO;`
fn close_news_flags(world: &mut World, this: Entity) {
    let mut bridge = world.obj_mut::<ControllerManagersBridge>(this);
    bridge.news_teaser_open = false;
    bridge.news_screen_open = false;
}

/// `[mNewsManager closeNewsScreen]`.
fn close_news_screen(world: &mut World, this: Entity) {
    let news_manager = world.obj::<ControllerManagersBridge>(this).news_manager;
    if let Some(news_manager) = news_manager {
        news_manager::close_news_screen(world, news_manager);
    }
}

/// `[mRainbowLogic storyModeClearRainbow]`.
fn clear_story_mode_rainbow(world: &mut World, this: Entity) {
    let rainbow = world.obj::<ControllerGodPowers>(this).rainbow_logic;
    if let Some(rainbow) = rainbow {
        rainbow_logic::story_mode_clear_rainbow(world, rainbow);
    }
}

/// `[mSequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:sequence_id]]`.
fn set_controller_sequence(world: &mut World, this: Entity, sequence_id: &str) {
    let controller_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
    let def = pg_engine::library::sequence_def(world, sequence_id);
    if let Some(controller_sequence) = controller_sequence {
        sequence::set_sequence_def(world, controller_sequence, def);
    }
}

/// `[mChallengeManager isGameActive:challenge_type]` (NO while there is no manager).
fn is_challenge_game_active(world: &mut World, this: Entity, challenge_type: i32) -> bool {
    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    match challenge_manager {
        Some(challenge_manager) => challenge_manager::is_game_active(world, challenge_manager, challenge_type),
        None => false,
    }
}

/// `[mSystemPreferences getIntPreference:key]` (0 while there is no document).
fn system_int_preference(world: &World, this: Entity, key: &str) -> i32 {
    let system_preferences = world.obj::<ControllerBootAndFrame>(this).system_preferences;
    match system_preferences {
        Some(system_preferences) => prefs::get_int_preference(world, system_preferences, key),
        None => 0,
    }
}

/// `[mSystemPreferences setPreference:key asInt:value]` (nothing while there is no document).
fn set_system_int_preference(world: &mut World, this: Entity, key: &str, value: i32) {
    let system_preferences = world.obj::<ControllerBootAndFrame>(this).system_preferences;
    if let Some(system_preferences) = system_preferences {
        prefs::set_preference_as_int(world, system_preferences, key, value);
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

/// `[mOptionPreferences setPreference:key asInt:value]` (nothing while there is no document).
fn set_option_int_preference(world: &mut World, this: Entity, key: &str, value: i32) {
    let option_preferences = world.obj::<ControllerBootAndFrame>(this).option_preferences;
    if let Some(option_preferences) = option_preferences {
        prefs::set_preference_as_int(world, option_preferences, key, value);
    }
}
