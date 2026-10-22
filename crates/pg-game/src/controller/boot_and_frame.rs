//! `PocketGodViewController`: methods in this part of the class.

use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::library::{self, Library};
use pg_engine::math::trunc_i32;
use pg_engine::prefs;
use pg_engine::sequence;
use pg_engine::sound;
use pg_engine::view::{self, BcView};

use crate::logic::apocalypse_locust_closet_xenu::{locust_logic, locust_model};
use crate::logic::battle::{god_idol_logic, idol_pedestal_logic};
use crate::logic::konkeydong::kong_statue_logic::{self, KongStatueLogic};
use crate::logic::sky_powers::{rainbow_logic, rainbow_model};
use crate::logic_table as slot;
use crate::managers::{
    bonus_pack_manager, challenge_manager, debug_screen, game_server, game_ui_manager, idol_manager,
    map_screen, news_manager, pocket_god_app_delegate, skin_pack_manager, story_manager,
};
use crate::persona::god_persona::{self as god_persona_class, GOD_PERSONA};
use crate::persona::pygmy_persona::{self as pygmy_persona_class, PYGMY_PERSONA};
use crate::persona::tribe_persona::{self as tribe_persona_class, TRIBE_PERSONA};
use crate::prelude::*;

use super::*;

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerBootAndFrame {
    /// `mLibrary`: `@"BCLibrary"`
    pub library: Option<Entity>,

    /// `mAppStartTime`: `d`
    pub app_start_time: f64,

    /// `mbGameStarted`: `B`
    pub game_started: bool,

    /// `mFpsTime`: `f`
    pub fps_time: f32,

    /// `mTargetFps`: `f`
    pub target_fps: f32,

    /// `mLongestFrameTime`: `f`
    pub longest_frame_time: f32,

    /// `mSpikeCount`: `i`
    pub spike_count: i32,

    /// `mbInitComplete`: `B`
    pub init_complete: bool,

    /// `mPersonaPreferences`: `@"BCPreferences"`
    pub persona_preferences: Option<Entity>,

    /// `mOptionPreferences`: `@"BCPreferences"`
    pub option_preferences: Option<Entity>,

    /// `mSystemPreferences`: `@"BCPreferences"`
    pub system_preferences: Option<Entity>,

    /// `mChallengePreferences`: `@"BCPreferences"`
    pub challenge_preferences: Option<Entity>,

    /// `mSequence`: `@"BCSequence"`
    pub sequence: Option<Entity>,

    /// `mMenuDisplayGroup`: `@"BCDisplayGroup"`
    pub menu_display_group: Option<Entity>,

    /// `mMenuDisplayMarker`: `@"BCDisplayMarker"`
    pub menu_display_marker: Option<Entity>,

    /// `mStartTime`: `f`
    pub start_time: f32,

    /// `mTargetIslandYPosVel`: `f`
    pub target_island_y_pos_vel: f32,

    /// `mTargetIslandYPos`: `f`
    pub target_island_y_pos: f32,

    /// `mbAddPygmyAfterIdolStartScreen`: `B`
    pub add_pygmy_after_idol_start_screen: bool,
}

/// `CFAbsoluteTimeGetCurrent()`: wall-clock seconds. pg-game has no OS clock (`std::time` is
/// unavailable on the web), so [`run_deferred_calls`] advances this one by the time the platform
/// reports. Only differences between two readings matter.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct WallClock {
    pub seconds: f64,
}

/// File-scope globals and deferred calls of this part of `PocketGodViewController`.
#[derive(Resource, Default, Clone, Debug)]
pub struct BootAndFrameStatics {
    /// `DAT_1007e5090`: the clock at the last `startTime:::` (only `stopTime:::`'s log read it).
    pub script_start_time: f64,

    /// The one-shot `NSTimer` of `loadGame` (`scheduledTimerWithTimeInterval:target:mView
    /// selector:@selector(startRenderLoop)…`): seconds until it fires.
    pub start_render_loop_timer: Option<f64>,

    /// `[self performSelectorOnMainThread:@selector(openLaunchUrl) withObject:self
    /// waitUntilDone:NO]` of `hideLoadingScreen:::`: the controllers to send it to, one entry
    /// per request.
    pub open_launch_url_targets: Vec<Entity>,
}

/// `CFBundleVersion` of 1.48.2 (`[[[NSBundle mainBundle] infoDictionary]
/// objectForKey:@"CFBundleVersion"]`).
pub const BUNDLE_VERSION: &str = "7";

/// `setLegacyPreferenceFileName:` of the `Persona`, `Options` and `System` documents.
const LEGACY_PREFERENCE_FILE_NAME: &str = "Preferences.plist";

/// dt of the three warm-up `[mView gameLoop]` calls of `loadGame`. `BCView`'s `mStartTime` and
/// `mLastTime` are still zero, so the first call's dt clamps to 1/15; the other two follow within
/// microseconds, and at ~8e8 s the f32 difference of the two timestamps is exactly 0.
const WARM_UP_FRAME_TIMES: [f32; 3] = [1.0 / 15.0, 0.0, 0.0];

/// `loadGame` keeps the display link at every screen refresh when loading took at most this
/// many seconds, and every second refresh otherwise.
const SLOW_LOAD_TIME: f32 = 13.0;

/// The interval of `loadGame`'s `startRenderLoop` timer (`0x3fb99999a0000000`, `(double)0.1f`).
const START_RENDER_LOOP_DELAY: f64 = 0.1_f32 as f64;

/// `onGameFrame:`'s island y easing factor, a double (`0x10029da98`).
const ISLAND_Y_EASING: f64 = 0.3;

/// `MenuDisplayGroup`'s y when the menu is fully closed (`onGameFrame:` then takes it out of
/// the display tree).
const MENU_CLOSED_Y_POS: f32 = 90.0;

/// `EpisodeIdolSlot1` … `EpisodeIdolSlot4` (0x100569dc8, 0x100569d98, 0x100569d68, 0x100569d38):
/// the `BCSkinTextureBitsDef` arrays of the four god idols, as indices into the `bits_slots`
/// table of `textures.json`.
const EPISODE_IDOL_SLOT_1: usize = 18;
const EPISODE_IDOL_SLOT_2: usize = 17;
const EPISODE_IDOL_SLOT_3: usize = 16;
const EPISODE_IDOL_SLOT_4: usize = 15;

/// `loadGame`: the idol pedestals (logic-table slot, `setPedestalIndex:`).
const IDOL_PEDESTALS: [(usize, i32); 4] = [
    (slot::GOD_ISLAND_PEDESTAL1, 1),
    (slot::GOD_ISLAND_PEDESTAL2, 2),
    (slot::GOD_ISLAND_PEDESTAL3, 3),
    (slot::GOD_ISLAND_PEDESTAL4, 4),
];

/// `loadGame`: the god idols (logic-table slot, `setSlot:`, `setSkinTextureBitsDefArray:`).
const GOD_IDOLS: [(usize, i32, usize); 4] = [
    (slot::GOD_IDOL1, 0, EPISODE_IDOL_SLOT_1),
    (slot::GOD_IDOL2, 1, EPISODE_IDOL_SLOT_2),
    (slot::GOD_IDOL3, 2, EPISODE_IDOL_SLOT_3),
    (slot::GOD_IDOL4, 3, EPISODE_IDOL_SLOT_4),
];

/// `loadPreferences:::`: the `Options` defaults (`setDefaultPreference:asInt:`), in order.
const OPTION_DEFAULTS: &[(&str, i32)] = &[
    ("WeatherHazard", 0),
    ("ShowIdolStartScreen", 1),
    ("ShowEpisodeIcon", 1),
    ("ShowBattleHelp", 1),
    ("IslandLocation", 5),
    ("SandIslandHazard", 3),
    ("RockIslandHazard", 1),
    ("RockIslandFeature", 1),
    ("RockIslandOutHouseState", 1),
    ("RockIslandTRexState", 1),
    ("RockIslandSpearState", 1),
    ("GraveyardIslandState", 0),
    ("GraveyardIslandZombieGraveState", 1),
    ("GraveyardIslandGhostState", 1),
    ("GraveyardIslandSpiderWebState", 1),
    ("GraveyardIslandSpiderState", 1),
    ("GraveyardIslandSpearState", 1),
    ("IceIslandState", 1),
    ("IceIslandIceMonsterState", 1),
    ("IceIslandIglooState", 1),
    ("IceIslandIceHoleState", 1),
    ("IceIslandIceSquidState", 1),
    ("ApeIslandState", 2),
    ("ApeIslandApeState", 1),
    ("ApeIslandMonkeyState", 1),
    ("MisfitIslandState", 1),
    ("GodIslandState", 0),
    ("GodIslandIdolsState", 1),
    ("ApocalypseIslandState", 0),
    ("UnderwaterHazard", 2),
    ("UnderwaterCoralReefState", 1),
    ("SharkState", 1),
    ("BirdState", 1),
    ("TemperatureState", 1),
    ("HurricaneState", 1),
    ("RainbowState", 1),
    ("CrackHoleState", 1),
    ("TheRunsState", 1),
    ("CharlieState", 1),
    ("KonkeyDongState", 1),
    ("MoronPestsState", 1),
    ("DrumHazard", 1),
    ("GravityHazard", 1),
    ("Mute", 0),
    ("Vibrate", 0),
    ("DisplayNames", 1),
    ("ButtonLabels", 1),
];

/// `loadPreferences:::`: the six pygmies of the tribe (id, default name, default description).
const PYGMY_PERSONAS: [(&str, &str, &str); 6] = [
    ("Pygmy0", "OOGA", "The fearless leader."),
    ("Pygmy1", "BOOGA", "Booga is gross and smells horrible."),
    ("Pygmy2", "NOOBY", "Nooby is dazed and confused."),
    ("Pygmy3", "DOOBY", "The happy-go-lucky islander."),
    ("Pygmy4", "KLIK", "Klik likes to organize and build."),
    ("Pygmy5", "KLAK", "Klak likes to break things."),
];

/// `CFAbsoluteTimeGetCurrent()` (0 until the app wiring advances the clock).
pub fn cf_absolute_time_get_current(world: &World) -> f64 {
    match world.get_resource::<WallClock>() {
        Some(clock) => clock.seconds,
        None => 0.0,
    }
}

/// Advances [`WallClock`] by `elapsed` seconds and runs this part's deferred calls once they
/// are due: `loadGame`'s `startRenderLoop` timer and a pending `openLaunchUrl`. The app wiring
/// calls it on every platform tick, also while the render loop is stopped (before the timer
/// starts it, nothing else runs).
pub fn run_deferred_calls(world: &mut World, elapsed: f64) {
    world.get_resource_or_insert_with(WallClock::default).seconds += elapsed;

    let (start_render_loop, open_launch_url_targets) = {
        let mut statics = world.get_resource_or_insert_with(BootAndFrameStatics::default);
        let mut start_render_loop = false;
        if let Some(remaining) = statics.start_render_loop_timer {
            let remaining = remaining - elapsed;
            if remaining <= 0.0 {
                statics.start_render_loop_timer = None;
                start_render_loop = true;
            } else {
                statics.start_render_loop_timer = Some(remaining);
            }
        }
        let open_launch_url_targets = std::mem::take(&mut statics.open_launch_url_targets);
        (start_render_loop, open_launch_url_targets)
    };

    if start_render_loop {
        view::start_render_loop(world);
    }
    for controller in open_launch_url_targets {
        open_launch_url(world, controller);
    }
}

/// `-[PocketGodViewController loadGame]` @ 0x10006a030
pub fn load_game(world: &mut World, this: Entity) {
    let load_start_time = cf_absolute_time_get_current(world);

    set_start_up_values(world, this);

    // PORT: `[[BCLibrary alloc] init]` is the engine's `Library` resource, so `mLibrary` stays
    // nil (only `dealloc` reads it). The loader runs the library part of this method in its
    // order: atlas and texture defs, the atlas pixels (`WidescreenOverlayAtlas0` only when
    // `[mView isWideScreen] || [mView isTallScreen]`), sequences, struct XML, `Timelines.bin`.
    library::loader::load_game_data(world);

    create_preferences(world, this);
    create_skin_pack_manager(world, this);
    crate::library_logic::create_logic_classes(world);
    cache_world_logics(world, this);
    cache_button_logics(world, this);
    cache_idol_logics(world, this);
    create_managers(world, this);
    crate::library_logic::init_logic_listeners(world);
    cache_display_groups(world, this);
    cache_sun_reflection(world, this);
    cache_gradient_defs(world, this);
    create_logic_arrays(world, this);
    view::add_game_frame_listener(world, this);
    create_rainbow_and_locust(world, this);
    create_sequences(world, this);

    // Three warm-up frames before the render loop exists; the boot script starts in them.
    for frame_time in WARM_UP_FRAME_TIMES {
        view::game_loop(world, frame_time);
    }

    let load_end_time = cf_absolute_time_get_current(world);
    let load_time = (load_end_time - load_start_time) as f32;
    let display_link_count = if load_time > SLOW_LOAD_TIME { 2 } else { 1 };
    view::set_display_link_count(world, display_link_count);

    world.get_resource_or_insert_with(BootAndFrameStatics::default).start_render_loop_timer = Some(START_RENDER_LOOP_DELAY);

    world.obj_mut::<ControllerBootAndFrame>(this).game_started = false;
}

/// `loadGame`: the start values of the counters, screen flags and menu targets.
fn set_start_up_values(world: &mut World, this: Entity) {
    {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        god_powers.shake_rot_vel = 1.3;
        god_powers.shake_rot = 0.0;
        god_powers.volcano_hit_count = 0;
        god_powers.earthquake_count = 0;
        god_powers.draw_lightning = false;
        god_powers.draw_rainbow = false;
        god_powers.draw_locust = false;
    }
    {
        let mut locations = world.obj_mut::<ControllerIslandLocations>(this);
        locations.island_min_x_pos = 50.0;
        locations.island_max_x_pos = 390.0;
    }
    {
        let mut pygmies = world.obj_mut::<ControllerPygmies>(this);
        pygmies.pygmy_count = 0;
        pygmies.live_pygmy_count = 0;
        pygmies.sleeping_count = 0;
    }
    {
        let mut menus = world.obj_mut::<ControllerMenusAndScreens>(this);
        menus.stats_screen_open = false;
        menus.help_screen_open = false;
        menus.debug_screen_open = false;
        menus.target_menu_button_z_rot = 90.0;
        menus.target_menu_y_pos = 90.0;
    }
    {
        let mut sky_weather = world.obj_mut::<ControllerSkyWeather>(this);
        sky_weather.weather_state = 0;
        sky_weather.cloud_gradient_fade = 0.01;
    }
}

/// `loadGame`: the four `BCPreferences` documents. Loading completes asynchronously; the boot
/// script waits for it (`waitForPreferencesLoadComplete:::`).
fn create_preferences(world: &mut World, this: Entity) {
    let persona_preferences = prefs::spawn(world, "Persona", false, false);
    world.obj_mut::<ControllerBootAndFrame>(this).persona_preferences = Some(persona_preferences);
    prefs::set_legacy_preference_file_name(world, persona_preferences, Some(LEGACY_PREFERENCE_FILE_NAME));
    prefs::load_preferences(world, persona_preferences);

    let option_preferences = prefs::spawn(world, "Options", false, false);
    world.obj_mut::<ControllerBootAndFrame>(this).option_preferences = Some(option_preferences);
    prefs::set_legacy_preference_file_name(world, option_preferences, Some(LEGACY_PREFERENCE_FILE_NAME));
    prefs::load_preferences(world, option_preferences);

    let system_preferences = prefs::spawn(world, "System", false, false);
    world.obj_mut::<ControllerBootAndFrame>(this).system_preferences = Some(system_preferences);
    prefs::set_legacy_preference_file_name(world, system_preferences, Some(LEGACY_PREFERENCE_FILE_NAME));
    prefs::load_preferences(world, system_preferences);

    let challenge_preferences = prefs::spawn(world, "Challenge", true, true);
    world.obj_mut::<ControllerBootAndFrame>(this).challenge_preferences = Some(challenge_preferences);
    prefs::load_preferences(world, challenge_preferences);
}

/// `loadGame`: `mSkinPackManager`, with the shark skin applied at once.
fn create_skin_pack_manager(world: &mut World, this: Entity) {
    let manager = world.spawn(Isa(&skin_pack_manager::SKIN_PACK_MANAGER)).id();
    skin_pack_manager::init(world, manager);
    world.obj_mut::<ControllerManagersBridge>(this).skin_pack_manager = Some(manager);
    skin_pack_manager::set_skin_pack_manager_listener(world, manager, this);

    skin_pack_manager::refresh_skin_pack_option_skin_object_id_skin_option_id_use_first_purchased(
        world,
        manager,
        "Shark",
        "Shark",
        "Shark Skins",
        true,
    );
}

/// `loadGame`: the temple and apocalypse objects of the logic table.
fn cache_world_logics(world: &mut World, this: Entity) {
    let logics = world.resource::<Library>().logic_table.clone();
    let logic = |slot_index: usize| Some(logics[slot_index]);

    {
        let mut registry = world.obj_mut::<ControllerWorldRegistry>(this);
        registry.temple_door_logic = logic(slot::GOD_ISLAND_TEMPLE);
        registry.apocalypse_temple_door_logic = logic(slot::TEMPLE_DOOR);
        registry.apocalypse_portal_temple_door_logic = logic(slot::TEMPLE_PORTAL);
        registry.apocalypse_calendar_logic = logic(slot::CLOCK_WHEEL_MULTI_MODEL);
        registry.apocalypse_room_fader_logic = logic(slot::APOCALYPSE_ROOM_FADER);
        registry.apocalypse_gem_pedestal_logic_array = vec![
            logics[slot::APOCALYPSE_ISLAND_PEDESTAL1],
            logics[slot::APOCALYPSE_ISLAND_PEDESTAL2],
            logics[slot::APOCALYPSE_ISLAND_PEDESTAL3],
            logics[slot::APOCALYPSE_ISLAND_PEDESTAL4],
            logics[slot::PEDESTAL5],
            logics[slot::PEDESTAL6],
            logics[slot::PEDESTAL7],
        ];
        registry.apocalypse_time_temple_lever_logic = logic(slot::LEVER);
        registry.apocalypse_time_temple_platform_logic = logic(slot::TIME_TEMPLE_PLATFORM);
        registry.time_temple_door_logic_array = vec![logics[slot::TIME_TEMPLE_DOOR1], logics[slot::TIME_TEMPLE_DOOR2]];
        registry.locust_temple_door_logic_array = vec![
            logics[slot::LOCUST_TEMPLE_DOOR1],
            logics[slot::LOCUST_TEMPLE_DOOR2],
            logics[slot::DOOR3],
        ];
        registry.dance_temple_door_logic_array = vec![logics[slot::DANCE_TEMPLE_DOOR1], logics[slot::DANCE_TEMPLE_DOOR2]];
        registry.locust_queen_logic = logic(slot::QUEEN);
        registry.go_go_cage_logic = logic(slot::DISCO_CAGE);
        registry.disco_ball_logic = logic(slot::DISCO_BALL);
        registry.disco_lights_logic = logic(slot::DISCO_LIGHTS);
        registry.dance_balloon_bundle_logic = logic(slot::BALLOON_BUNDLE);
        registry.dance_balloon_logic = logic(slot::DANCE_BALLOON);
        registry.dance_dance_execution_logic = logic(slot::DANCE_DANCE_EXECUTION);
        registry.plague_temple_door_logic_array = vec![logics[slot::PLAGUE_TEMPLE_DOOR1], logics[slot::PLAGUE_TEMPLE_DOOR2]];
        registry.apocalypse_plague_temple_fountain_logic = logic(slot::PLAGUE_FOUNTAIN);
        registry.purification_temple_door_logic_array = vec![logics[slot::PURIFICATION_TEMPLE_DOOR1]];
        registry.apocalypse_purification_temple_fountain_logic = logic(slot::PURIFICATION_FOUNTAIN);
        registry.closet_temple_door_logic_array = vec![logics[slot::CLOSET_TEMPLE_DOOR1]];
        registry.closet_temple_thetan_meter_logic = logic(slot::THETAN_METER);
        registry.closet_temple_couch_logic = logic(slot::COUCH);
        registry.closet_temple_closet_door_logic = logic(slot::SLIDING_DOOR);
        registry.xenu_temple_door_logic_array = vec![logics[slot::XENU_TEMPLE_DOOR1]];
        registry.xenu_temple_logic = logic(slot::XENU_TEMPLE_BACKGROUND);
        registry.xenu_logic = logic(slot::XENU);
    }

    world.obj_mut::<ControllerApocalypse>(this).apocalypse_gem_logic_array = Vec::new();
}

/// `loadGame`: the menu and island-menu buttons of the logic table (the ivar writes grouped by
/// the component that holds them). The debug button is taken out of `MenuDisplayGroup` instead
/// of being kept, and `OpenFeintButton` (#624) is not cached.
fn cache_button_logics(world: &mut World, this: Entity) {
    let logics = world.resource::<Library>().logic_table.clone();
    let logic = |slot_index: usize| Some(logics[slot_index]);

    {
        let mut menus = world.obj_mut::<ControllerMenusAndScreens>(this);
        menus.island_location_button_logic = logic(slot::ISLAND_LOCATION_BUTTON);
        menus.island_menu_button_logic = logic(slot::ISLAND_MENU_BUTTON);
        menus.global_island_menu_button_logic = logic(slot::GLOBAL_ISLAND_MENU_BUTTON);
        menus.god_power_menu_button_logic = logic(slot::GOD_POWER_MENU_BUTTON);
        menus.customize_menu_button_logic = logic(slot::CUSTOMIZE_MENU_BUTTON);
        menus.community_menu_button_logic = logic(slot::COMMUNITY_MENU_BUTTON);
        menus.settings_menu_button_logic = logic(slot::SETTINGS_MENU_BUTTON);
        menus.help_button_logic = logic(slot::MENU_HELP_BUTTON);
    }

    let debug_button = logics[slot::DEBUG_BUTTON];
    let debug_button_object = pg_engine::logic::display_object(world, debug_button);
    display::remove_from_display_group(world, debug_button_object);

    {
        let mut features = world.obj_mut::<ControllerIslandFeatures>(this);
        features.sand_boulder_button_logic = logic(slot::BOULDER_BUTTON);
        features.sand_fish_button_logic = logic(slot::SAND_ISLAND_MENU_FISH_BUTTON);
        features.sand_ant_hill_button_logic = logic(slot::ANT_HILL_BUTTON);
        features.sand_water_wings_button_logic = logic(slot::SAND_ISLAND_MENU_WATER_WINGS_BUTTON);
        features.rock_fish_button_logic = logic(slot::ROCK_ISLAND_MENU_FISH_BUTTON);
        features.rock_wood_button_logic = logic(slot::WOOD_BUTTON);
        features.rock_egg_button_logic = logic(slot::EGG_BUTTON);
        features.rock_out_house_button_logic = logic(slot::OUT_HOUSE_BUTTON);
        features.rock_t_rex_button_logic = logic(slot::T_REX_BUTTON);
        features.rock_spear_button_logic = logic(slot::ROCK_ISLAND_MENU_SPEAR_BUTTON);
        features.graveyard_zombie_grave_button_logic = logic(slot::ZOMBIE_GRAVE_BUTTON);
        features.graveyard_ghost_button_logic = logic(slot::GHOST_BUTTON);
        features.graveyard_spider_web_button_logic = logic(slot::SPIDER_WEB_BUTTON);
        features.graveyard_spider_button_logic = logic(slot::SPIDER_BUTTON);
        features.graveyard_spear_button_logic = logic(slot::GRAVEYARD_ISLAND_MENU_SPEAR_BUTTON);
        features.graveyard_water_wings_button_logic = logic(slot::GRAVEYARD_ISLAND_MENU_WATER_WINGS_BUTTON);
        features.ice_fish_button_logic = logic(slot::ICE_ISLAND_MENU_FISH_BUTTON);
        features.ice_water_wings_button_logic = logic(slot::ICE_ISLAND_MENU_WATER_WINGS_BUTTON);
        features.ice_ice_monster_button_logic = logic(slot::ICE_MONSTER_BUTTON);
        features.ice_igloo_button_logic = logic(slot::IGLOO_BUTTON);
        features.ice_ice_hole_button_logic = logic(slot::ICE_HOLE_BUTTON);
        features.ice_ice_squid_button_logic = logic(slot::ICE_SQUID_BUTTON);
        features.ape_ape_altar_button_logic = logic(slot::APE_ALTAR_BUTTON);
        features.ape_ape_button_logic = logic(slot::APE_APE_BUTTON);
        features.ape_banana_tree_button_logic = logic(slot::APE_BANANA_TREE_BUTTON);
        features.ape_monkey_button_logic = logic(slot::APE_MONKEY_BUTTON);
        features.ape_konkey_dong_button_logic = logic(slot::APE_KONKEY_DONG_BUTTON);
        features.misfit_moron_snake_button_logic = logic(slot::MORON_SNAKE_BUTTON);
        features.misfit_water_wings_button_logic = logic(slot::MISFIT_ISLAND_MENU_WATER_WINGS_BUTTON);
        features.misfit_moron_pests_button_logic = logic(slot::MISFIT_MORON_PESTS_BUTTON);
        features.idols_button_logic = logic(slot::IDOLS_BUTTON);
        features.idol_water_wings_button_logic = logic(slot::GOD_ISLAND_MENU_WATER_WINGS_BUTTON);
        features.apocalypse_water_wings_button_logic = logic(slot::APOCALYPSE_ISLAND_MENU_WATER_WINGS_BUTTON);
        features.underwater_piranha_button_logic = logic(slot::PIRANHA_BUTTON);
        features.underwater_laser_shark_button_logic = logic(slot::LASER_SHARK_BUTTON);
        features.underwater_tile_drain_button_logic = logic(slot::TILE_DRAIN_BUTTON);
        features.underwater_coral_reef_jellyfish_button_logic = logic(slot::JELLYFISH_BUTTON);
        features.shark_button_logic = logic(slot::SHARK_BUTTON);
        features.bird_button_logic = logic(slot::BIRD_BUTTON);
    }

    world.obj_mut::<ControllerSkyWeather>(this).weather_button_logic = logic(slot::WEATHER_BUTTON);

    {
        let mut menus = world.obj_mut::<ControllerMenusAndScreens>(this);
        menus.idol_selection_button_logic = logic(slot::IDOL_SELECTION_BUTTON);
        menus.crack_hole_button_logic = logic(slot::CRACK_HOLE_BUTTON);
        menus.the_runs_button_logic = logic(slot::THE_RUNS_BUTTON);
        menus.charlie_button_logic = logic(slot::CHARLIE_BUTTON);
        menus.drum_button_logic = logic(slot::DRUM_BUTTON);
        menus.temperature_button_logic = logic(slot::TEMPERATURE_BUTTON);
        menus.hurricane_button_logic = logic(slot::HURRICANE_BUTTON);
        menus.rainbow_button_logic = logic(slot::RAINBOW_BUTTON);
        menus.skin_pack_button_logic = logic(slot::CUSTOMIZE_MENU_SKIN_PACK_BUTTON);
        menus.bonus_pack_button_logic = logic(slot::BONUS_PACK_BUTTON);
        menus.story_button_logic = logic(slot::STORY_BUTTON);
        menus.stats_button_logic = logic(slot::STATS_BUTTON);
        menus.idols_challenge_button_logic = logic(slot::IDOLS_CHALLENGE_BUTTON);
        menus.news_button_logic = logic(slot::NEWS_BUTTON);
        menus.challenge_button_logic = logic(slot::CHALLENGE_BUTTON);
        menus.start_screen_button_logic = logic(slot::START_SCREEN_BUTTON);
        menus.mute_button_logic = logic(slot::MUTE_BUTTON);
        menus.vibrate_button_logic = logic(slot::VIBRATE_BUTTON);
        menus.gravity_button_logic = logic(slot::GRAVITY_BUTTON);
        menus.names_button_logic = logic(slot::NAMES_BUTTON);
        menus.labels_button_logic = logic(slot::LABELS_BUTTON);
    }
}

/// `loadGame`: the idol pedestals and god idols (numbered as they are added), then the
/// remaining single objects of the logic table.
fn cache_idol_logics(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerWorldRegistry>(this).idol_pedestal_logic_array = Vec::with_capacity(5);
    for (slot_index, pedestal_index) in IDOL_PEDESTALS {
        let pedestal = library::logic(world, slot_index);
        idol_pedestal_logic::set_pedestal_index(world, pedestal, pedestal_index);
        world.obj_mut::<ControllerWorldRegistry>(this).idol_pedestal_logic_array.push(pedestal);
    }

    world.obj_mut::<ControllerWorldRegistry>(this).god_idol_logic_array = Vec::with_capacity(5);
    for (slot_index, idol_slot, bits_slot) in GOD_IDOLS {
        let idol = library::logic(world, slot_index);
        god_idol_logic::set_slot(world, idol, idol_slot);
        god_idol_logic::set_skin_texture_bits_def_array(world, idol, Some(bits_slot));
        world.obj_mut::<ControllerWorldRegistry>(this).god_idol_logic_array.push(idol);
    }

    let logics = world.resource::<Library>().logic_table.clone();
    let mut registry = world.obj_mut::<ControllerWorldRegistry>(this);
    registry.moron_snake_logic = Some(logics[slot::MORON_SNAKE]);
    registry.misfit_ocean_rock_logic = Some(logics[slot::MISFIT_OCEAN_ROCK]);
    registry.water_wings_logic = Some(logics[slot::WATER_WINGS]);
    registry.no_pee_logic = Some(logics[slot::NO_PEE_DISPLAY_GROUP]);
    registry.underwater_temple_logic = Some(logics[slot::UNDERWATER_TEMPLE]);
}

/// `loadGame`: the managers, each with the controller as its listener.
fn create_managers(world: &mut World, this: Entity) {
    let server = world.spawn(Isa(&game_server::GAME_SERVER)).id();
    game_server::init(world, server);
    world.obj_mut::<ControllerStatsAndAchievements>(this).game_server = Some(server);
    game_server::set_game_server_listener(world, server, this);
    game_server::set_game_server_leaderboard_listener(world, server, this);
    game_server::set_game_server_achievement_listener(world, server, this);

    let challenges = world.spawn(Isa(&challenge_manager::CHALLENGE_MANAGER)).id();
    challenge_manager::init(world, challenges);
    world.obj_mut::<ControllerManagersBridge>(this).challenge_manager = Some(challenges);
    challenge_manager::set_challenge_manager_listener(world, challenges, this);

    let bonus_packs = world.spawn(Isa(&bonus_pack_manager::BONUS_PACK_MANAGER)).id();
    bonus_pack_manager::init(world, bonus_packs);
    world.obj_mut::<ControllerManagersBridge>(this).bonus_pack_manager = Some(bonus_packs);
    bonus_pack_manager::set_bonus_pack_manager_listener(world, bonus_packs, this);

    let stories = world.spawn(Isa(&story_manager::STORY_MANAGER)).id();
    story_manager::init(world, stories);
    world.obj_mut::<ControllerManagersBridge>(this).story_manager = Some(stories);
    story_manager::set_story_manager_listener(world, stories, this);
    story_manager::load_story_defs(world, stories);

    let debug = world.spawn(Isa(&debug_screen::DEBUG_SCREEN)).id();
    debug_screen::init(world, debug);
    world.obj_mut::<ControllerManagersBridge>(this).debug_screen = Some(debug);
    debug_screen::set_debug_screen_listener(world, debug, this);

    let news = world.spawn(Isa(&news_manager::NEWS_MANAGER)).id();
    news_manager::init(world, news);
    world.obj_mut::<ControllerManagersBridge>(this).news_manager = Some(news);
    news_manager::set_news_manager_listener(world, news, this);
    news_manager::load_news_defs(world, news);

    let idols = world.spawn(Isa(&idol_manager::IDOL_MANAGER)).id();
    idol_manager::init(world, idols);
    world.obj_mut::<ControllerManagersBridge>(this).idol_manager = Some(idols);
    idol_manager::set_idol_manager_listener(world, idols, this);

    let game_ui = world.spawn(Isa(&game_ui_manager::GAME_UI_MANAGER)).id();
    game_ui_manager::init(world, game_ui);
    world.obj_mut::<ControllerManagersBridge>(this).game_ui_manager = Some(game_ui);
    game_ui_manager::set_game_ui_manager_listener(world, game_ui, this);

    let map = world.spawn(Isa(&map_screen::MAP_SCREEN)).id();
    map_screen::init(world, map);
    world.obj_mut::<ControllerManagersBridge>(this).map_screen = Some(map);
    map_screen::set_map_screen_listener(world, map, this);
}

/// `loadGame`: the display groups and markers the controller keeps (`getDisplayGroupById:`,
/// `getDisplayMarkerById:`; the ivar writes grouped by the component that holds them).
fn cache_display_groups(world: &mut World, this: Entity) {
    world.resource_scope(|world, library: Mut<Library>| {
        {
            let mut locations = world.obj_mut::<ControllerIslandLocations>(this);
            locations.island_display_group = library.display_group("IslandDisplayGroup");
            locations.underwater_display_group = library.display_group("UnderwaterDisplayGroup");
            locations.underwater_ruins_display_group = library.display_group("UnderwaterRuinsDisplayGroup");
            locations.underwater_coral_reef_display_group = library.display_group("UnderwaterCoralReefDisplayGroup");
        }
        {
            let mut minigames = world.obj_mut::<ControllerMinigames>(this);
            minigames.doodle_display_group = library.display_group("DoodleDisplayGroup");
            minigames.pain_drain_display_group = library.display_group("PainDrainDisplayGroup");
            minigames.the_runs_display_group = library.display_group("TheRunsDisplayGroup");
            minigames.konkey_dong_display_group = library.display_group("KonkeyDongDisplayGroup");
            minigames.moron_pests_display_group = library.display_group("MoronPestsDisplayGroup");
            minigames.no_pee_display_group = library.display_group("NoPeeDisplayGroup");
            minigames.battle_of_the_gods_display_group = library.display_group("BattleArenaDisplayGroup");
        }

        world.obj_mut::<ControllerBootAndFrame>(this).menu_display_group = library.display_group("MenuDisplayGroup");

        {
            let mut menus = world.obj_mut::<ControllerMenusAndScreens>(this);
            menus.sand_island_menu_display_group = library.display_group("SandIslandMenuDisplayGroup");
            menus.rock_island_menu_display_group = library.display_group("RockIslandMenuDisplayGroup");
            menus.graveyard_island_menu_display_group = library.display_group("GraveyardIslandMenuDisplayGroup");
            menus.ice_island_menu_display_group = library.display_group("IceIslandMenuDisplayGroup");
            menus.ape_island_menu_display_group = library.display_group("ApeIslandMenuDisplayGroup");
            menus.misfit_island_menu_display_group = library.display_group("MisfitIslandMenuDisplayGroup");
            menus.god_island_menu_display_group = library.display_group("GodIslandMenuDisplayGroup");
            menus.fail_boat_menu_display_group = library.display_group("FailBoatMenuDisplayGroup");
            menus.apocalypse_island_menu_display_group = library.display_group("ApocalypseIslandMenuDisplayGroup");
            menus.time_temple_menu_display_group = library.display_group("TimeTempleMenuDisplayGroup");
            menus.locust_temple_menu_display_group = library.display_group("LocustTempleMenuDisplayGroup");
            menus.dance_temple_menu_display_group = library.display_group("DanceTempleMenuDisplayGroup");
            menus.plague_temple_menu_display_group = library.display_group("PlagueTempleMenuDisplayGroup");
            menus.purification_temple_menu_display_group = library.display_group("PurificationTempleMenuDisplayGroup");
            menus.closet_temple_menu_display_group = library.display_group("ClosetTempleMenuDisplayGroup");
            menus.xenu_temple_menu_display_group = library.display_group("XenuTempleMenuDisplayGroup");
            menus.underwater_menu_display_group = library.display_group("UnderwaterMenuDisplayGroup");
            menus.underwater_coral_reef_menu_display_group = library.display_group("UnderwaterCoralReefMenuDisplayGroup");
            menus.global_island_menu_display_group = library.display_group("GlobalIslandMenuDisplayGroup");
            menus.god_power_menu_display_group = library.display_group("GodPowerMenuDisplayGroup");
            menus.customize_menu_display_group = library.display_group("CustomizeMenuDisplayGroup");
            menus.community_menu_display_group = library.display_group("CommunityMenuDisplayGroup");
            menus.settings_menu_display_group = library.display_group("SettingsMenuDisplayGroup");
            menus.menu_labels_display_group = library.display_group("MenuLabelsDisplayGroup");
        }
        {
            let mut sky_weather = world.obj_mut::<ControllerSkyWeather>(this);
            sky_weather.cloud_front_display_marker = library.display_marker("CloudFrontDisplayMarker");
            sky_weather.cloud_behind_display_marker = library.display_marker("CloudBehindDisplayMarker");
        }

        world.obj_mut::<ControllerGodPowers>(this).firework_display_marker = library.display_marker("FireworksDisplayMarker");
        world.obj_mut::<ControllerWorldRegistry>(this).small_object_display_marker = library.display_marker("SmallObjectsMarker");
        world.obj_mut::<ControllerBootAndFrame>(this).menu_display_marker = library.display_marker("MenuDisplayMarker");
        world.obj_mut::<ControllerMenusAndScreens>(this).category_menu_display_marker = library.display_marker("CategoryMenuDisplayMarker");

        {
            let mut locations = world.obj_mut::<ControllerIslandLocations>(this);
            locations.island_root_display_marker = library.display_marker("IslandRootDisplayMarker");
            locations.underwater_display_marker = library.display_marker("UnderwaterDisplayMarker");
            locations.underwater_realm_display_marker = library.display_marker("UnderwaterRealmDisplayMarker");
        }
        {
            let mut minigames = world.obj_mut::<ControllerMinigames>(this);
            minigames.doodle_cloud_display_marker = library.display_marker("DoodleCloudDisplayMarker");
            minigames.doodle_pygmy_display_marker = library.display_marker("DoodlePygmyDisplayMarker");
            minigames.pain_drain_platform_display_marker = library.display_marker("PainDrainPlatformDisplayMarker");
            minigames.pain_drain_pygmy_display_marker = library.display_marker("PainDrainPygmyDisplayMarker");
            minigames.the_runs_pygmy_display_marker = library.display_marker("TheRunsPygmyDisplayMarker");
            minigames.konkey_dong_pygmy_display_marker = library.display_marker("KonkeyDongPygmyDisplayMarker");
            minigames.moron_pests_pygmy_display_marker = library.display_marker("MoronPestsPygmyDisplayMarker");
            minigames.no_pee_pygmy_display_marker = library.display_marker("NoPeePygmyDisplayMarker");
            minigames.battle_of_the_gods_pygmy_display_marker = library.display_marker("BattleOfTheGodsPygmyDisplayMarker");
        }
        {
            let mut locations = world.obj_mut::<ControllerIslandLocations>(this);
            locations.island_display_marker = library.display_marker("IslandDisplayMarker");
            locations.sand_island_display_group = library.display_group("SandIslandDisplayGroup");
            locations.rock_island_display_group = library.display_group("RockIslandDisplayGroup");
            locations.graveyard_island_display_group = library.display_group("GraveyardIslandDisplayGroup");
            locations.ice_island_display_group = library.display_group("IceIslandDisplayGroup");
            locations.ape_island_display_group = library.display_group("ApeIslandDisplayGroup");
            locations.misfit_island_display_group = library.display_group("MisfitIslandDisplayGroup");
            locations.god_island_display_group = library.display_group("GodIslandDisplayGroup");
            locations.fail_boat_ocean_display_group = library.display_group("FailBoatOceanDisplayGroup");
            locations.apocalypse_island_display_group = library.display_group("ApocalypseIslandDisplayGroup");
            locations.time_temple_display_group = library.display_group("TimeTempleDisplayGroup");
            locations.locust_temple_display_group = library.display_group("LocustTempleDisplayGroup");
            locations.dance_temple_display_group = library.display_group("DanceTempleDisplayGroup");
            locations.plague_temple_display_group = library.display_group("PlagueTempleDisplayGroup");
            locations.purification_temple_display_group = library.display_group("PurificationTempleDisplayGroup");
            locations.closet_temple_display_group = library.display_group("ClosetTempleDisplayGroup");
            locations.xenu_temple_display_group = library.display_group("XenuTempleDisplayGroup");
        }
    });
}

/// `loadGame`: `SunReflection`, an object of `IslandDisplayGroup` that the controller keeps
/// and takes out of the group.
fn cache_sun_reflection(world: &mut World, this: Entity) {
    let island_display_group = world.obj::<ControllerIslandLocations>(this).island_display_group;
    let sun_reflection = match island_display_group {
        Some(group) => display::group::get_display_object(world, group, "SunReflection"),
        None => None,
    };
    world.obj_mut::<ControllerSkyWeather>(this).sun_reflection_display_object = sun_reflection;

    if let Some(sun_reflection) = sun_reflection {
        display::remove_from_display_group(world, sun_reflection);
    }
}

/// `loadGame`: the sky gradient defs (`getGradientDefById:`), kept by id.
fn cache_gradient_defs(world: &mut World, this: Entity) {
    world.resource_scope(|world, library: Mut<Library>| {
        let mut sky_weather = world.obj_mut::<ControllerSkyWeather>(this);
        sky_weather.sunset_gradient = gradient_def_id(&library, "SunsetGradient");
        sky_weather.night_gradient = gradient_def_id(&library, "NightGradient");
        sky_weather.day_gradient = gradient_def_id(&library, "DayGradient");
        sky_weather.cloudy_gradient = gradient_def_id(&library, "CloudyGradient");
        sky_weather.eclipse_gradient = gradient_def_id(&library, "EclipseGradient");
        sky_weather.big_sun_gradient = gradient_def_id(&library, "BigSunGradient");
        sky_weather.small_sun_gradient = gradient_def_id(&library, "SmallSunGradient");
        sky_weather.apocalypse_sunset_gradient = gradient_def_id(&library, "ApocalypseSunsetGradient");
        sky_weather.apocalypse_night_gradient = gradient_def_id(&library, "ApocalypseNightGradient");
        sky_weather.apocalypse_day_gradient = gradient_def_id(&library, "ApocalypseDayGradient");
        sky_weather.apocalypse_cloudy_gradient = gradient_def_id(&library, "ApocalypseCloudyGradient");
    });
}

/// `[BCLibrary getGradientDefById:id]` as the controller keeps it: the def's id, `None` where
/// the original gets NULL.
fn gradient_def_id(library: &Library, id: &str) -> Option<String> {
    match library.gradient_def(id) {
        Some(_) => Some(id.to_owned()),
        None => None,
    }
}

/// `loadGame`: the controller's logic arrays (empty, capacity 5), the jellyfish filled from
/// the logic table.
fn create_logic_arrays(world: &mut World, this: Entity) {
    {
        let mut registry = world.obj_mut::<ControllerWorldRegistry>(this);
        registry.main_ui_logic_array = Vec::with_capacity(5);
        registry.help_ui_logic_array = Vec::with_capacity(5);
        registry.whats_new_screen_ui_logic_array = Vec::with_capacity(5);
    }
    {
        let mut stats = world.obj_mut::<ControllerStatsAndAchievements>(this);
        stats.stats_tribe_ui_logic_array = Vec::with_capacity(5);
        stats.stats_pygmy_ui_logic_array = Vec::with_capacity(5);
        stats.stats_god_ui_logic_array = Vec::with_capacity(5);
        stats.pygmy_stats_button_logic_array = Vec::with_capacity(5);
    }

    world.obj_mut::<ControllerWorldRegistry>(this).logic_array = Vec::with_capacity(5);
    world.obj_mut::<ControllerPygmies>(this).pygmy_logic_array = Vec::with_capacity(5);
    world.obj_mut::<ControllerGodPowers>(this).firework_logic_array = Vec::with_capacity(5);

    {
        let mut registry = world.obj_mut::<ControllerWorldRegistry>(this);
        registry.coconut_logic_array = Vec::with_capacity(5);
        registry.wood_logic_array = Vec::with_capacity(5);
    }

    world.obj_mut::<ControllerSkyWeather>(this).cloud_logic_array = Vec::with_capacity(5);
    world.obj_mut::<ControllerMinigames>(this).doodle_cloud_logic_array = Vec::with_capacity(5);

    {
        let mut registry = world.obj_mut::<ControllerWorldRegistry>(this);
        registry.spear_logic_array = Vec::with_capacity(5);
        registry.underwater_scenery_logic_array = Vec::with_capacity(5);
    }

    world.obj_mut::<ControllerWorldObjectEvents>(this).piranha_logic_array = Vec::with_capacity(5);

    {
        let mut registry = world.obj_mut::<ControllerWorldRegistry>(this);
        registry.ape_torch_logic_array = Vec::with_capacity(5);
        registry.monkey_logic_array = Vec::with_capacity(5);
    }

    world.obj_mut::<ControllerGodPowers>(this).underwater_power_trail_logic_array = Vec::with_capacity(5);

    let logics = world.resource::<Library>().logic_table.clone();
    let jellyfish = vec![
        logics[slot::JELLYFISH1],
        logics[slot::JELLYFISH2],
        logics[slot::JELLYFISH3],
        logics[slot::JELLYFISH4],
        logics[slot::JELLYFISH5],
    ];
    world.obj_mut::<ControllerWorldRegistry>(this).jellyfish_logic_array = jellyfish;
}

/// `loadGame`: the god-power point lists, then `mRainbowLogic` on a `RainbowModel` and
/// `mLocustLogic` on a `LocustModel` (neither model is put into the display tree here).
fn create_rainbow_and_locust(world: &mut World, this: Entity) {
    {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        god_powers.lightning_list = Vec::new();
        god_powers.rainbow_list = Vec::new();
        god_powers.locust_vector = Vec::new();
    }

    // PORT: the original hands the model `mpRainbowList` itself (one shared std::list); the
    // generated `initWithId:pathList:` takes the points by value, so the model gets a copy of
    // the (still empty) list.
    let rainbow_path_list = world.obj::<ControllerGodPowers>(this).rainbow_list.clone();
    let rainbow_model_object = world.spawn(Isa(&rainbow_model::RAINBOW_MODEL)).id();
    rainbow_model::init_with_id(world, rainbow_model_object, "Rainbow", rainbow_path_list);
    display::object_mut(world, rainbow_model_object).pos.z = SPRITE_Z;
    let rainbow_logic_object = rainbow_logic::spawn(world, rainbow_model_object);
    world.obj_mut::<ControllerGodPowers>(this).rainbow_logic = Some(rainbow_logic_object);
    rainbow_logic::set_rainbow_logic_listener(world, rainbow_logic_object, this);

    let locust_model_object = world.spawn(Isa(&locust_model::LOCUST_MODEL)).id();
    locust_model::init_with_id(world, locust_model_object, "Locust");
    display::object_mut(world, locust_model_object).pos.z = SPRITE_Z;
    let locust_logic_object = locust_logic::spawn(world, locust_model_object);
    world.obj_mut::<ControllerGodPowers>(this).locust_logic = Some(locust_logic_object);
    locust_logic::set_locust_logic_listener(world, locust_logic_object, this);
}

/// `loadGame`: the controller's sequences (delegate = the controller). `mSequence` runs the
/// boot script `PocketGod`; the others start empty.
fn create_sequences(world: &mut World, this: Entity) {
    let boot_script = library::sequence_def(world, "PocketGod");
    let boot_sequence = sequence::spawn(world, boot_script, Some(this));
    world.obj_mut::<ControllerBootAndFrame>(this).sequence = Some(boot_sequence);

    let misc_sequence = sequence::spawn(world, None, Some(this));
    world.obj_mut::<ControllerIslandLocations>(this).misc_sequence = Some(misc_sequence);

    let the_runs_sequence = sequence::spawn(world, None, Some(this));
    world.obj_mut::<ControllerMinigames>(this).the_runs_sequence = Some(the_runs_sequence);

    let konkey_dong_sequence = sequence::spawn(world, None, Some(this));
    world.obj_mut::<ControllerMinigames>(this).konkey_dong_sequence = Some(konkey_dong_sequence);

    let moron_pests_sequence = sequence::spawn(world, None, Some(this));
    world.obj_mut::<ControllerMinigames>(this).moron_pests_sequence = Some(moron_pests_sequence);

    let no_pee_sequence = sequence::spawn(world, None, Some(this));
    world.obj_mut::<ControllerMinigames>(this).no_pee_sequence = Some(no_pee_sequence);

    let battle_of_the_gods_sequence = sequence::spawn(world, None, Some(this));
    world.obj_mut::<ControllerMinigames>(this).battle_of_the_gods_sequence = Some(battle_of_the_gods_sequence);

    let weather_sequence = sequence::spawn(world, None, Some(this));
    world.obj_mut::<ControllerSkyWeather>(this).weather_sequence = Some(weather_sequence);

    let message_sequence = sequence::spawn(world, None, Some(this));
    world.obj_mut::<ControllerMenusAndScreens>(this).message_sequence = Some(message_sequence);
}

/// `-[PocketGodViewController personaPreferences]` @ 0x10006da98
pub fn persona_preferences(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<ControllerBootAndFrame>(this).persona_preferences
}

/// `-[PocketGodViewController optionPreferences]` @ 0x10006daa8
pub fn option_preferences(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<ControllerBootAndFrame>(this).option_preferences
}

/// `-[PocketGodViewController systemPreferences]` @ 0x10006dab8
pub fn system_preferences(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<ControllerBootAndFrame>(this).system_preferences
}

/// `-[PocketGodViewController challengePreferences]` @ 0x10006dac8
pub fn challenge_preferences(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<ControllerBootAndFrame>(this).challenge_preferences
}

/// `-[PocketGodViewController godPersona]` @ 0x100070db4
pub fn god_persona(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<ControllerStatsAndAchievements>(this).god_persona
}

/// `-[PocketGodViewController tribePersona]` @ 0x1000715e4
pub fn tribe_persona(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<ControllerStatsAndAchievements>(this).tribe_persona
}

/// `-[PocketGodViewController startTime:::]` @ 0x10007bc30
pub fn seq_start_time(world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let now = cf_absolute_time_get_current(world);
    world.get_resource_or_insert_with(BootAndFrameStatics::default).script_start_time = now;
    ItemResult::Done
}

/// `-[PocketGodViewController stopTime:::]` @ 0x10007bc50
pub fn seq_stop_time(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    // The script time since `startTime:::` only went to the debug log.
    ItemResult::Done
}

/// `-[PocketGodViewController waitForHoldLoadingAd:::]` @ 0x10007bca8
pub fn seq_wait_for_hold_loading_ad(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let now = cf_absolute_time_get_current(world);
    let app_start_time = world.obj::<ControllerBootAndFrame>(this).app_start_time;
    if 5.0 <= now - app_start_time {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[PocketGodViewController waitForPreferencesLoadComplete:::]` @ 0x10007bce8
pub fn seq_wait_for_preferences_load_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let boot = world.obj::<ControllerBootAndFrame>(this);
    let persona_preferences = boot.persona_preferences;
    let challenge_preferences = boot.challenge_preferences;
    let option_preferences = boot.option_preferences;
    let system_preferences = boot.system_preferences;

    let complete = is_load_complete(world, persona_preferences)
        && is_load_complete(world, challenge_preferences)
        && is_load_complete(world, option_preferences)
        && is_load_complete(world, system_preferences);
    if complete {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `[preferences isPreferencesLoadComplete]` (NO for nil).
fn is_load_complete(world: &World, preferences: Option<Entity>) -> bool {
    match preferences {
        Some(preferences) => prefs::is_preferences_load_complete(world, preferences),
        None => false,
    }
}

/// `-[PocketGodViewController onPreferencesChanged:]` @ 0x10007bd80
pub fn on_preferences_changed(world: &mut World, this: Entity, _preferences_changed: Entity) {
    let god = world.obj::<ControllerStatsAndAchievements>(this).god_persona;
    if let Some(god) = god {
        god_persona_class::refresh_from_preferences(world, god);
    }

    let tribe = world.obj::<ControllerStatsAndAchievements>(this).tribe_persona;
    if let Some(tribe) = tribe {
        tribe_persona_class::refresh_from_preferences(world, tribe);
    }
}

/// `-[PocketGodViewController loadPreferences:::]` @ 0x10007bdcc
pub fn seq_load_preferences(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let idols = world.obj::<ControllerManagersBridge>(this).idol_manager;
    if let Some(idols) = idols {
        idol_manager::load_episode_state(world, idols);
    }

    let option_preferences = world.obj::<ControllerBootAndFrame>(this).option_preferences;
    if let Some(options) = option_preferences {
        for &(key, value) in OPTION_DEFAULTS {
            prefs::set_default_preference_as_int(world, options, key, value);
        }
    }

    let system_preferences = world.obj::<ControllerBootAndFrame>(this).system_preferences;
    if let Some(system) = system_preferences {
        prefs::set_default_preference_as_int(world, system, "MisfitIslandFound", 0);
    }

    create_personas(world, this);

    let shark_state = option_int(world, this, "SharkState");
    set_shark_state(world, this, shark_state, true);
    let bird_state = option_int(world, this, "BirdState");
    set_bird_state(world, this, bird_state, true);
    let weather_state = option_int(world, this, "WeatherHazard");
    set_weather_state(world, this, weather_state, true);
    update_island_state_at_startup(world, this);

    if option_int(world, this, "GravityHazard") == 1 {
        let button = world.obj::<ControllerMenusAndScreens>(this).gravity_button_logic;
        set_button_behavior(world, button, "StartGravityOn");
    }
    if option_int(world, this, "TemperatureState") == 1 {
        let button = world.obj::<ControllerMenusAndScreens>(this).temperature_button_logic;
        set_button_behavior(world, button, "StartTemperatureOn");
    }
    if option_int(world, this, "HurricaneState") == 1 {
        let button = world.obj::<ControllerMenusAndScreens>(this).hurricane_button_logic;
        set_button_behavior(world, button, "StartHurricaneOn");
    }
    if option_int(world, this, "RainbowState") == 1 {
        let button = world.obj::<ControllerMenusAndScreens>(this).rainbow_button_logic;
        set_button_behavior(world, button, "StartRainbowOn");
    }
    if option_int(world, this, "CrackHoleState") == 1 {
        let button = world.obj::<ControllerMenusAndScreens>(this).crack_hole_button_logic;
        set_button_behavior(world, button, "StartCrackHoleOn");
    }
    if option_int(world, this, "IdolsState") == 1 {
        let button = world.obj::<ControllerIslandFeatures>(this).idols_button_logic;
        set_button_behavior(world, button, "StartIdolsOn");
    }
    if option_int(world, this, "TheRunsState") == 1 {
        let button = world.obj::<ControllerMenusAndScreens>(this).the_runs_button_logic;
        set_button_behavior(world, button, "StartTheRunsOn");
    }
    if option_int(world, this, "CharlieState") == 1 {
        let button = world.obj::<ControllerMenusAndScreens>(this).charlie_button_logic;
        set_button_behavior(world, button, "StartCharlieOn");
    }
    if option_int(world, this, "KonkeyDongState") == 1 {
        let button = world.obj::<ControllerIslandFeatures>(this).ape_konkey_dong_button_logic;
        set_button_behavior(world, button, "StartKonkeyDongOn");
    }
    if option_int(world, this, "MoronPestsState") == 1 {
        let button = world.obj::<ControllerIslandFeatures>(this).misfit_moron_pests_button_logic;
        set_button_behavior(world, button, "StartMoronPestsOn");
    }
    if option_int(world, this, "DrumHazard") == 1 {
        let button = world.obj::<ControllerMenusAndScreens>(this).drum_button_logic;
        set_button_behavior(world, button, "StartDrumOn");
    }

    if option_int(world, this, "Mute") == 1 {
        sound::mute(world, true);
        let button = world.obj::<ControllerMenusAndScreens>(this).mute_button_logic;
        set_button_behavior(world, button, "StartMuteOn");
        sound::stop_all_sounds(world);
    } else {
        sound::mute(world, false);
        let button = world.obj::<ControllerMenusAndScreens>(this).mute_button_logic;
        set_button_behavior(world, button, "StartMuteOff");
        play_background_loop(world, this, None);
    }

    if option_int(world, this, "Vibrate") == 1 {
        sound::mute_vibrate(world, true);
        let button = world.obj::<ControllerMenusAndScreens>(this).vibrate_button_logic;
        set_button_behavior(world, button, "StartVibrateOn");
    } else {
        sound::mute_vibrate(world, false);
    }

    if option_int(world, this, "DisplayNames") == 1 {
        let button = world.obj::<ControllerMenusAndScreens>(this).names_button_logic;
        set_button_behavior(world, button, "StartNamesOn");
    }
    if option_int(world, this, "ButtonLabels") == 1 {
        let button = world.obj::<ControllerMenusAndScreens>(this).labels_button_logic;
        set_button_behavior(world, button, "StartLabelsOn");
    }

    let check_requested = world.obj::<ControllerStatsAndAchievements>(this).check_all_achievements_and_leaderboards;
    if check_requested {
        world.obj_mut::<ControllerStatsAndAchievements>(this).check_all_achievements_and_leaderboards = false;
        check_all_achievements_and_leaderboards(world, this);
    }

    ItemResult::Done
}

/// `loadPreferences:::`: the god, the tribe with its six pygmies, and the guest pygmies
/// Charlie and Tom, all on the `Persona` document; then the controller listens to it.
fn create_personas(world: &mut World, this: Entity) {
    let persona_preferences = world.obj::<ControllerBootAndFrame>(this).persona_preferences;
    let Some(preferences) = persona_preferences else {
        panic!("loadPreferences::: runs before loadGame created the Persona preferences");
    };

    let god = world.spawn(Isa(&GOD_PERSONA)).id();
    let god_description = "God of Thunder and Rocks that Roll";
    god_persona_class::init_with_id(world, god, "God", "GENE SIMIANS", god_description, preferences);
    world.obj_mut::<ControllerStatsAndAchievements>(this).god_persona = Some(god);

    let tribe = world.spawn(Isa(&TRIBE_PERSONA)).id();
    let tribe_description = "One Tribe, One World";
    tribe_persona_class::init_with_id(world, tribe, "Tribe", "TRIBE OOKGA CHAKA", tribe_description, preferences);
    world.obj_mut::<ControllerStatsAndAchievements>(this).tribe_persona = Some(tribe);

    for (id, name, description) in PYGMY_PERSONAS {
        let pygmy = world.spawn(Isa(&PYGMY_PERSONA)).id();
        pygmy_persona_class::init_with_id(world, pygmy, id, name, description, preferences);
        tribe_persona_class::add_pygmy_persona(world, tribe, pygmy);
    }

    let charlie = world.spawn(Isa(&PYGMY_PERSONA)).id();
    let charlie_description = "Winning!.";
    pygmy_persona_class::init_with_id(world, charlie, "CharliePygmy", "Charlie", charlie_description, preferences);
    world.obj_mut::<ControllerStatsAndAchievements>(this).charlie_pygmy_persona = Some(charlie);
    pygmy_persona_class::set_persona_personality(world, charlie, 1);
    tribe_persona_class::add_guest_pygmy_persona(world, tribe, charlie);
    if option_int(world, this, "CharlieState") == 1 {
        tribe_persona_class::activate_guest_pygmy_persona(world, tribe, charlie);
    }

    let tom = world.spawn(Isa(&PYGMY_PERSONA)).id();
    let tom_description = "Show me the money!.";
    pygmy_persona_class::init_with_id(world, tom, "TomPygmy", "Tom", tom_description, preferences);
    world.obj_mut::<ControllerStatsAndAchievements>(this).tom_pygmy_persona = Some(tom);
    pygmy_persona_class::set_persona_personality(world, tom, 2);
    tribe_persona_class::add_guest_pygmy_persona(world, tribe, tom);

    prefs::set_preferences_listener(world, preferences, Some(this));
}

/// `[mOptionPreferences getIntPreference:key]` (0 while there is no document).
fn option_int(world: &World, this: Entity, key: &str) -> i32 {
    let option_preferences = world.obj::<ControllerBootAndFrame>(this).option_preferences;
    match option_preferences {
        Some(options) => prefs::get_int_preference(world, options, key),
        None => 0,
    }
}

/// `[button setBehavior:behavior]` (nothing for a nil button).
fn set_button_behavior(world: &mut World, button: Option<Entity>, behavior: &str) {
    if let Some(button) = button {
        pg_engine::logic::set_behavior(world, button, Some(behavior));
    }
}

/// `-[PocketGodViewController (Game) refreshNews]` @ 0x1001579f8
pub fn refresh_news(world: &mut World, this: Entity) {
    let news = world.obj::<ControllerManagersBridge>(this).news_manager;
    if let Some(news) = news {
        news_manager::open_news_teaser_on_load_complete(world, news);
    }

    let news = world.obj::<ControllerManagersBridge>(this).news_manager;
    if let Some(news) = news {
        news_manager::load_news_defs(world, news);
    }
}

/// `-[PocketGodViewController (Game) onGameFrame:]` @ 0x1001585bc
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    world.obj_mut::<ControllerBootAndFrame>(this).game_started = true;

    // The drum dance stops one second after the last drum tap.
    let tap_touch_time = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        god_powers.tap_touch_time += dt;
        god_powers.tap_touch_time
    };
    let story_active = world.obj::<ControllerManagersBridge>(this).story_active;
    if !story_active && tap_touch_time > 1.0 {
        world.obj_mut::<ControllerGodPowers>(this).tap_touch_count = 0;
        stop_celebration(world, this);
    }

    let bat_tap_touch_time = {
        let mut sky_weather = world.obj_mut::<ControllerSkyWeather>(this);
        sky_weather.bat_tap_touch_time += dt;
        sky_weather.bat_tap_touch_time
    };
    if bat_tap_touch_time > 1.0 {
        world.obj_mut::<ControllerSkyWeather>(this).bat_tap_touch_count = 0;
    }

    update_island_y_target(world, this, dt);
    ease_island_y(world, this);

    move_the_runs_camera(world, this, dt);
    move_moron_pests_camera(world, this, dt);
    move_no_pee_camera(world, this, dt);
    move_konkey_dong_camera(world, this);

    ease_menu(world, this);

    check_eclipse(world, this);
    update_gradients(world, this);
}

/// `onGameFrame:`: the island's vertical scroll target (`mTargetIslandYPos`).
fn update_island_y_target(world: &mut World, this: Entity, dt: f32) {
    let locations = world.obj::<ControllerIslandLocations>(this);
    let location_state = locations.island_location_state;
    let location_transition = locations.island_location_transition;

    let underwater = location_state == 2 || location_state == 9;
    if underwater || location_transition {
        world.obj_mut::<ControllerBootAndFrame>(this).target_island_y_pos = 0.0;
        return;
    }

    let dragged_pygmy = world.obj::<ControllerPygmies>(this).dragged_pygmy_logic;
    let dragged_gem = world.obj::<ControllerWorldObjectEvents>(this).dragged_apocalypse_gem_logic;
    if dragged_pygmy.is_some() || dragged_gem.is_some() {
        scroll_island_with_drag(world, this, dt);
        return;
    }

    let doodle_jump_pygmy = world.obj::<ControllerMinigames>(this).doodle_jump_pygmy_logic;
    match doodle_jump_pygmy {
        Some(pygmy) => follow_doodle_jump(world, this, pygmy, dt),
        None => spring_island_back(world, this, dt),
    }
}

/// `onGameFrame:`: while a pygmy or an apocalypse gem is dragged, a finger near the top or the
/// bottom of the screen scrolls the island (600 points per second, within [-260, 0]).
fn scroll_island_with_drag(world: &mut World, this: Entity, dt: f32) {
    world.obj_mut::<ControllerBootAndFrame>(this).target_island_y_pos_vel = 0.0;

    let island_display_group = world.obj::<ControllerIslandLocations>(this).island_display_group;
    let touch_screen_pos = world.obj::<ControllerTouchInput>(this).touch_screen_pos;
    let (touch_y, island_y) = match island_display_group {
        Some(group) => {
            let touch_pos = display::view_to_display_object(world, group, touch_screen_pos);
            let island_y = display::object(world, group).pos.y;
            (touch_pos.y, island_y)
        }
        None => (0.0, 0.0),
    };

    let screen_y = touch_y + island_y;
    let mut target = island_y;
    if screen_y > 256.0 {
        target = island_y + dt * -600.0;
    }
    if screen_y < 192.0 {
        target = dt * 600.0 + target;
    }
    if target > 0.0 {
        target = 0.0;
    }
    if target < -260.0 {
        target = -260.0;
    }
    world.obj_mut::<ControllerBootAndFrame>(this).target_island_y_pos = target;
}

/// `onGameFrame:`: the Ooga Jump camera follows the jumping pygmy up (scoring the height) and
/// drops with it when it falls; the doodle sky is shown below -480.
fn follow_doodle_jump(world: &mut World, this: Entity, pygmy: Entity, dt: f32) {
    let old_target = world.obj::<ControllerBootAndFrame>(this).target_island_y_pos;
    let island_display_group = world.obj::<ControllerIslandLocations>(this).island_display_group;
    let island_y = match island_display_group {
        Some(group) => display::object(world, group).pos.y,
        None => 0.0,
    };
    let pygmy_object = pg_engine::logic::display_object(world, pygmy);
    let pygmy_y = display::object(world, pygmy_object).pos.y;
    let falling = world.obj::<ControllerMinigames>(this).doodle_jump_fall;

    let mut target = old_target;
    if falling {
        let height = pygmy_y + -32.0;
        target = if height < 0.0 { 0.0 } else { -height };
        if let Some(group) = island_display_group {
            display::object_mut(world, group).pos.y = target;
        }
    } else {
        let height = pygmy_y + -128.0;
        let climb_target = -height;
        if island_y > climb_target {
            let challenges = world.obj::<ControllerManagersBridge>(this).challenge_manager;
            if let Some(challenges) = challenges {
                let score = trunc_i32(height * 0.3125);
                challenge_manager::set_game_score(world, challenges, 4, score);
            }
            target = climb_target;
        }
    }
    if target > 0.0 {
        target = 0.0;
    }

    // The generated show/hide signatures take no nil pygmy, so a pygmy that left the game in
    // between skips the call.
    if island_y < -480.0 && target > -480.0 {
        let pygmy = world.obj::<ControllerMinigames>(this).doodle_jump_pygmy_logic;
        if let Some(pygmy) = pygmy {
            hide_doodle_display_group(world, this, pygmy);
        }
    }
    if island_y > -480.0 && target < -480.0 {
        let pygmy = world.obj::<ControllerMinigames>(this).doodle_jump_pygmy_logic;
        if let Some(pygmy) = pygmy {
            show_doodle_display_group(world, this, pygmy);
        }
    }

    {
        let mut boot = world.obj_mut::<ControllerBootAndFrame>(this);
        boot.target_island_y_pos_vel = (target - boot.target_island_y_pos) / dt;
        boot.target_island_y_pos = target;
    }

    recycle_doodle_clouds(world, this, -island_y);
    let pygmy = world.obj::<ControllerMinigames>(this).doodle_jump_pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::doodle_check_off_screen(world, pygmy, -island_y);
    }
}

/// `onGameFrame:`: with nothing to follow the target springs back up to 0 (the velocity is not
/// reset while the target is clamped).
fn spring_island_back(world: &mut World, this: Entity, dt: f32) {
    let mut boot = world.obj_mut::<ControllerBootAndFrame>(this);
    boot.target_island_y_pos_vel = dt * 3000.0 + boot.target_island_y_pos_vel;
    boot.target_island_y_pos = boot.target_island_y_pos_vel * dt + boot.target_island_y_pos;
    if boot.target_island_y_pos > 0.0 {
        boot.target_island_y_pos = 0.0;
    }
}

/// `onGameFrame:`: `IslandDisplayGroup` (and `DoodleDisplayGroup` with it) eases toward the
/// target, computed in double precision.
fn ease_island_y(world: &mut World, this: Entity) {
    let island_display_group = world.obj::<ControllerIslandLocations>(this).island_display_group;
    let doodle_display_group = world.obj::<ControllerMinigames>(this).doodle_display_group;
    let target = world.obj::<ControllerBootAndFrame>(this).target_island_y_pos;
    let Some(island_display_group) = island_display_group else {
        return;
    };

    let island_y = display::object(world, island_display_group).pos.y;
    let step = f64::from(target - island_y) * ISLAND_Y_EASING;
    let eased_y = (f64::from(island_y) + step) as f32;
    display::object_mut(world, island_display_group).pos.y = eased_y;
    if let Some(doodle_display_group) = doodle_display_group {
        display::object_mut(world, doodle_display_group).pos.y = eased_y;
    }
}

/// `onGameFrame:`: The Runs scrolls with the running pygmy, faster once it is past x = 70.
fn move_the_runs_camera(world: &mut World, this: Entity, dt: f32) {
    let minigames = world.obj::<ControllerMinigames>(this);
    let move_camera = minigames.the_runs_move_camera;
    let group = minigames.the_runs_display_group;
    let pygmy = minigames.the_runs_pygmy_logic;
    if !move_camera {
        return;
    }
    let (Some(group), Some(pygmy)) = (group, pygmy) else {
        return;
    };

    let pygmy_vel_x = world.obj::<BcLogic>(pygmy).pos_vel.x;
    let pygmy_object = pg_engine::logic::display_object(world, pygmy);
    let pygmy_x = display::object(world, pygmy_object).pos.x;

    let mut group_object = display::object_mut(world, group);
    let group_x = group_object.pos.x;
    group_object.pos.x = if pygmy_x + group_x > 70.0 {
        group_x + pygmy_vel_x * -1.2 * dt
    } else {
        group_x - pygmy_vel_x * dt
    };
}

/// `onGameFrame:`: Moron Pests scrolls with the pygmy, slower while it is left of x = 340.
fn move_moron_pests_camera(world: &mut World, this: Entity, dt: f32) {
    let minigames = world.obj::<ControllerMinigames>(this);
    let move_camera = minigames.moron_pests_move_camera;
    let group = minigames.moron_pests_display_group;
    let pygmy = minigames.moron_pests_pygmy_logic;
    if !move_camera {
        return;
    }
    let (Some(group), Some(pygmy)) = (group, pygmy) else {
        return;
    };

    let pygmy_vel_x = world.obj::<BcLogic>(pygmy).pos_vel.x;
    let pygmy_object = pg_engine::logic::display_object(world, pygmy);
    let pygmy_x = display::object(world, pygmy_object).pos.x;

    let mut group_object = display::object_mut(world, group);
    let group_x = group_object.pos.x;
    group_object.pos.x = if pygmy_x + group_x < 340.0 {
        group_x + pygmy_vel_x * -0.9 * dt
    } else {
        group_x - pygmy_vel_x * dt
    };
}

/// `onGameFrame:`: No Pee leads the swimming pygmy horizontally and follows its depth.
fn move_no_pee_camera(world: &mut World, this: Entity, dt: f32) {
    let minigames = world.obj::<ControllerMinigames>(this);
    let move_camera = minigames.no_pee_move_camera;
    let group = minigames.no_pee_display_group;
    let pygmy = minigames.no_pee_pygmy_logic;
    if !move_camera {
        return;
    }
    let Some(pygmy) = pygmy else {
        return;
    };
    if pg_engine::logic::is_hibernating(world, pygmy) {
        return;
    }
    let Some(group) = group else {
        return;
    };

    let pygmy_vel_x = world.obj::<BcLogic>(pygmy).pos_vel.x;
    let pygmy_object = pg_engine::logic::display_object(world, pygmy);
    let pygmy_pos = display::object(world, pygmy_object).pos;
    let group_pos = display::object(world, group).pos;
    let screen_x = pygmy_pos.x + group_pos.x;

    let swimming_left = crate::pygmy::no_pee_swimming_left(world, pygmy);
    let mut lead = screen_x > 200.0 && !swimming_left;
    if !lead {
        let swimming_left_again = crate::pygmy::no_pee_swimming_left(world, pygmy);
        lead = swimming_left_again && screen_x < 280.0;
    }
    let group_x = if lead {
        group_pos.x + pygmy_vel_x * -1.2 * dt
    } else {
        group_pos.x - pygmy_vel_x * dt
    };

    let depth = (pygmy_pos.y + -150.0) / 700.0;
    let depth = if depth < 0.0 { 0.0 } else { depth };
    let target_y = if depth > 1.0 { -680.0 } else { depth * -680.0 };
    let offset_y = target_y - group_pos.y;
    let group_y = group_pos.y + (offset_y + offset_y) * dt;

    let mut group_object = display::object_mut(world, group);
    group_object.pos.x = group_x;
    group_object.pos.y = group_y;
}

/// `onGameFrame:`: Konkey Dong glides to the next level, or keeps the pygmy inside
/// [200, 280] × [150, 170] on screen within the level's bounds.
fn move_konkey_dong_camera(world: &mut World, this: Entity) {
    let minigames = world.obj::<ControllerMinigames>(this);
    let move_camera = minigames.konkey_dong_move_camera;
    let to_next_level = minigames.konkey_dong_move_camera_to_next_level;
    let group = minigames.konkey_dong_display_group;
    let pygmy = minigames.konkey_dong_pygmy_logic;
    if !move_camera {
        return;
    }
    let Some(group) = group else {
        return;
    };

    if to_next_level {
        glide_konkey_dong_camera(world, this, group);
        return;
    }

    let statue = world.obj::<ControllerWorldRegistry>(this).kong_statue_logic;
    let level_width = match statue {
        Some(statue) => konkey_dong_level_width(world, statue),
        None => None,
    };
    let Some(pygmy) = pygmy else {
        return;
    };
    let pygmy_object = pg_engine::logic::display_object(world, pygmy);
    let pygmy_pos = display::object(world, pygmy_object).pos;
    let group_pos = display::object(world, group).pos;

    let mut x = group_pos.x;
    let mut y = group_pos.y;
    let screen_x = pygmy_pos.x + x;
    let screen_y = pygmy_pos.y + y;
    if screen_x > 280.0 {
        x = 280.0 - pygmy_pos.x;
    } else if screen_x < 200.0 {
        x = 200.0 - pygmy_pos.x;
    }
    if x > 0.0 {
        x = 0.0;
    }
    if let Some(level_width) = level_width {
        let min_x = -(level_width + -480.0);
        if x < min_x {
            x = min_x;
        }
    }

    if screen_y > 170.0 {
        y = 170.0 - pygmy_pos.y;
    } else if screen_y < 150.0 {
        y = 150.0 - pygmy_pos.y;
    }
    let level_y_pos = world.obj::<ControllerMinigames>(this).konkey_dong_level_y_pos;
    if y > -level_y_pos {
        y = -level_y_pos;
    }

    let mut group_object = display::object_mut(world, group);
    group_object.pos.x = x;
    group_object.pos.y = y;
}

/// `onGameFrame:`: the Konkey Dong camera eases to the next level's height; once there the
/// statue hears that the transition is complete.
fn glide_konkey_dong_camera(world: &mut World, this: Entity, group: Entity) {
    let group_pos = display::object(world, group).pos;
    let level_y_pos = world.obj::<ControllerMinigames>(this).konkey_dong_level_y_pos;
    let x = group_pos.x + group_pos.x * -0.07;
    let y = group_pos.y + (-level_y_pos - group_pos.y) * 0.07;
    {
        let mut group_object = display::object_mut(world, group);
        group_object.pos.x = x;
        group_object.pos.y = y;
    }

    let level_y_pos = world.obj::<ControllerMinigames>(this).konkey_dong_level_y_pos;
    let remaining = (-level_y_pos - y).abs();
    if remaining <= 0.1 {
        world.obj_mut::<ControllerMinigames>(this).konkey_dong_move_camera_to_next_level = false;
        let statue = world.obj::<ControllerWorldRegistry>(this).kong_statue_logic;
        if let Some(statue) = statue {
            kong_statue_logic::level_transition_complete(world, statue);
        }
    }
}

/// `[mKongStatueLogic konkeyDongLevel]->mLevelWidth` (the float at +0x30 of the level).
fn konkey_dong_level_width(world: &mut World, statue: Entity) -> Option<f32> {
    let level_index = kong_statue_logic::konkey_dong_level(world, statue)?;
    let statue_logic = world.obj::<KongStatueLogic>(statue);
    let level = statue_logic.konkey_dong_level_array.get(level_index)?;
    Some(level.level_width)
}

/// `onGameFrame:`: the menu button turns and `MenuDisplayGroup` slides toward their targets;
/// the closed menu (y exactly 90) leaves the display tree, any other position puts it back after
/// `MenuDisplayMarker`. The f32 halving converges to exactly 90.
fn ease_menu(world: &mut World, this: Entity) {
    let menus = world.obj::<ControllerMenusAndScreens>(this);
    let menu_button = menus.menu_button_logic;
    let target_rot_z = menus.target_menu_button_z_rot;
    let target_y = menus.target_menu_y_pos;
    let Some(menu_button) = menu_button else {
        return;
    };
    let menu_display_group = world.obj::<ControllerBootAndFrame>(this).menu_display_group;
    let Some(menu_display_group) = menu_display_group else {
        return;
    };

    let button_object = pg_engine::logic::display_object(world, menu_button);
    {
        let mut button = display::object_mut(world, button_object);
        let rot_z = button.rot.z;
        button.rot.z = rot_z + (target_rot_z - rot_z) * 0.5;
    }
    let menu_y = {
        let mut menu = display::object_mut(world, menu_display_group);
        let y = menu.pos.y;
        menu.pos.y = y + (target_y - y) * 0.5;
        menu.pos.y
    };

    let parent = display::object(world, menu_display_group).display_group;
    if menu_y == MENU_CLOSED_Y_POS {
        if parent.is_some() {
            display::remove_from_display_group(world, menu_display_group);
        }
        return;
    }
    if parent.is_none() {
        let menu_display_marker = world.obj::<ControllerBootAndFrame>(this).menu_display_marker;
        if let Some(marker) = menu_display_marker {
            display::insert_after(world, marker, Some(menu_display_group));
        }
    }
}

/// `-[PocketGodViewController (Game) enableTouch:::]` @ 0x100159998
pub fn seq_enable_touch(world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.resource_mut::<BcView>().touch_active = true;
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) hideLoadingScreen]` @ 0x1001599d0
pub fn hide_loading_screen(world: &mut World, this: Entity) {
    let delegate = world.obj::<ControllerAppPlatform>(this).delegate;
    if let Some(delegate) = delegate {
        pocket_god_app_delegate::on_hide_loading_screen(world, delegate);
    }
}

/// `-[PocketGodViewController (Game) checkForIdolStartScreen]` @ 0x1001599f0
pub fn check_for_idol_start_screen(world: &mut World, this: Entity) -> bool {
    let boot = world.obj::<ControllerBootAndFrame>(this);
    let system_preferences = boot.system_preferences;
    let option_preferences = boot.option_preferences;

    let whats_new_version = match system_preferences {
        Some(system) => prefs::get_string_preference(world, system, "WhatsNewVersion"),
        None => None,
    };
    if let Some(system) = system_preferences {
        prefs::set_preference_as_string(world, system, "WhatsNewVersion", BUNDLE_VERSION);
    }
    let show_idol_start_screen = match option_preferences {
        Some(options) => prefs::get_int_preference(world, options, "ShowIdolStartScreen"),
        None => 0,
    };

    // `[nil isEqualToString:]` is NO: the first launch shows the start screen.
    let same_version = whats_new_version.as_deref() == Some(BUNDLE_VERSION);
    if show_idol_start_screen == 0 && same_version {
        world.obj_mut::<ControllerBootAndFrame>(this).add_pygmy_after_idol_start_screen = false;
        return false;
    }

    world.obj_mut::<ControllerManagersBridge>(this).idol_screen_open = true;

    let boot_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
    let close_menu = library::sequence_def(world, "CloseMenu");
    if let Some(boot_sequence) = boot_sequence {
        sequence::set_sequence_def(world, boot_sequence, close_menu);
    }

    let idols = world.obj::<ControllerManagersBridge>(this).idol_manager;
    if let Some(idols) = idols {
        idol_manager::open_idol_start_screen(world, idols);
    }
    true
}

/// `-[PocketGodViewController (Game) checkShowAuthenticationViewController:::]` @ 0x100159c44
pub fn seq_check_show_authentication_view_controller(world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let server = game_server::instance(world);
    if let Some(server) = server {
        game_server::check_show_authentication_view_controller(world, server);
    }
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) checkStartScreen:::]` @ 0x100159c80
pub fn seq_check_start_screen(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut boot = world.obj_mut::<ControllerBootAndFrame>(this);
        boot.add_pygmy_after_idol_start_screen = true;
        boot.init_complete = true;
    }

    let start_screen_open = check_for_idol_start_screen(world, this);

    let boot_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
    let script_id = if start_screen_open { "StartScreen" } else { "NoStartScreen" };
    let script = library::sequence_def(world, script_id);
    if let Some(boot_sequence) = boot_sequence {
        sequence::set_sequence_def(world, boot_sequence, script);
    }
    ItemResult::Replaced
}

/// `-[PocketGodViewController (Game) hideLoadingScreen:::]` @ 0x100159d40
pub fn seq_hide_loading_screen(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ControllerAppPlatform>(this).hide_loading_screen = true;
    hide_loading_screen(world, this);

    // `performSelectorOnMainThread:@selector(openLaunchUrl) withObject:self
    // waitUntilDone:NO`: runs after this frame (`run_deferred_calls`).
    let launch_url_pending = world.obj::<ControllerAppPlatform>(this).launch_url.is_some();
    if launch_url_pending {
        let mut statics = world.get_resource_or_insert_with(BootAndFrameStatics::default);
        statics.open_launch_url_targets.push(this);
    }
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) checkForNewNews:::]` @ 0x100159dc0
pub fn seq_check_for_new_news(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let news = world.obj::<ControllerManagersBridge>(this).news_manager;
    let Some(news) = news else {
        return ItemResult::Done;
    };

    let has_unread_news = news_manager::has_unread_news(world, news);
    if has_unread_news {
        world.obj_mut::<ControllerManagersBridge>(this).news_teaser_open = true;
        news_manager::open_news_teaser(world, news);
    }
    ItemResult::Done
}
