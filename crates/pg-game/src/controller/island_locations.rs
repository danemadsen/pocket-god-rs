//! `PocketGodViewController`: the 18 island locations. `setIslandLocationState:startup:` changes
//! location, `activateIslandLocationState:` and `deactivateLastIslandLocationState` enter and exit
//! each location's objects (walk bounds, horizon, display group, start script), the tsunami
//! carries the trips to and from the underwater realm, `loadLocationTextureAtlas` uploads the
//! location (or minigame) atlas into `screenAtlas0`, and the map, dream bubble, room fader and
//! idol award screen travel through the callbacks here.

use crate::prelude::*;
use super::*;

use pg_engine::library::Library;

use crate::logic::ape_monkeys::{banana_logic, banana_tree_logic};
use crate::logic::ape_ritual::{ape_altar_logic, ape_bongo_logic, ape_logic, ape_wench_logic};
use crate::logic::apocalypse_dance::{
    dance_balloon_bundle_logic, dance_balloon_logic, dance_dance_execution_logic, disco_ball_logic, disco_lights_logic,
    go_go_cage_logic,
};
use crate::logic::apocalypse_locust_closet_xenu::{
    closet_temple_closet_door_logic, closet_temple_couch_logic, closet_temple_thetan_meter_logic, locust_queen_logic,
    xenu_logic, xenu_temple_logic,
};
use crate::logic::apocalypse_temples::{
    apocalypse_calendar_logic, apocalypse_cloud_logic, apocalypse_temple_door_logic, apocalypse_temple_fountain_logic,
    apocalypse_time_temple_lever_logic, apocalypse_time_temple_platform_logic,
};
use crate::logic::battle::temple_door_logic;
use crate::logic::ice_island::{glacier_logic, ice_hole_logic, ice_monster_logic, ice_tree_logic, igloo_logic};
use crate::logic::island_anthill::{ant_hill_logic, magnifying_glass_logic};
use crate::logic::island_core::bon_fire_logic;
use crate::logic::island_objects::{boulder_logic, octo_statue_logic, out_house_logic, volcano_logic};
use crate::logic::konkeydong::kong_statue_logic;
use crate::logic::misfit_oogajump::island_object_logic;
use crate::logic::monsters::{spider_logic, spider_tree_logic, t_rex_egg_logic, t_rex_logic, zombie_grave_logic};
use crate::logic::moronpests_chase::moron_snake_logic;
use crate::logic::nopee::water_wings_logic;
use crate::logic::ocean_creatures::shark_logic;
use crate::logic::ocean_fishing::{fish_logic, fishing_pole_logic};
use crate::logic::sky_powers::{hurricane_logic, whirl_pool_logic};
use crate::logic::underwater_baitmaster_laser::{anchor_logic, chum_bucket_logic};
use crate::logic::underwater_paindrain::{tile_drain_logic, tile_plug_logic};
use crate::logic::underwater_stage::tsunami_logic;
use crate::managers::skin_pack_manager;

// `mIslandLocationState` values.
const LOCATION_SAND_ISLAND: i32 = 0;
const LOCATION_ROCK_ISLAND: i32 = 1;
const LOCATION_UNDERWATER: i32 = 2;
const LOCATION_GRAVEYARD_ISLAND: i32 = 3;
const LOCATION_ICE_ISLAND: i32 = 4;
const LOCATION_APE_ISLAND: i32 = 5;
const LOCATION_MISFIT_ISLAND: i32 = 6;
const LOCATION_GOD_ISLAND: i32 = 7;
const LOCATION_FAIL_BOAT: i32 = 8;
const LOCATION_UNDERWATER_CORAL_REEF: i32 = 9;
const LOCATION_APOCALYPSE_ISLAND: i32 = 10;
const LOCATION_TIME_TEMPLE: i32 = 11;
const LOCATION_LOCUST_TEMPLE: i32 = 12;
const LOCATION_DANCE_TEMPLE: i32 = 13;
const LOCATION_PLAGUE_TEMPLE: i32 = 14;
const LOCATION_PURIFICATION_TEMPLE: i32 = 15;
const LOCATION_CLOSET_TEMPLE: i32 = 16;
const LOCATION_XENU_TEMPLE: i32 = 17;

/// The id of the atlas slot every location and minigame atlas goes into (the def at
/// 0x10077b790 that `loadLocationTextureAtlas` passes to `FUN_100038d68`).
const SCREEN_ATLAS_ID: &str = "screenAtlas0";

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerIslandLocations {
    /// `mMiscSequence`: `@"BCSequence"`
    pub misc_sequence: Option<Entity>,

    /// `mUnderwaterDisplayGroup`: `@"BCDisplayGroup"`
    pub underwater_display_group: Option<Entity>,

    /// `mUnderwaterRuinsDisplayGroup`: `@"BCDisplayGroup"`
    pub underwater_ruins_display_group: Option<Entity>,

    /// `mUnderwaterCoralReefDisplayGroup`: `@"BCDisplayGroup"`
    pub underwater_coral_reef_display_group: Option<Entity>,

    /// `mIslandDisplayGroup`: `@"BCDisplayGroup"`
    pub island_display_group: Option<Entity>,

    /// `mIslandRootDisplayMarker`: `@"BCDisplayMarker"`
    pub island_root_display_marker: Option<Entity>,

    /// `mUnderwaterDisplayMarker`: `@"BCDisplayMarker"`
    pub underwater_display_marker: Option<Entity>,

    /// `mUnderwaterRealmDisplayMarker`: `@"BCDisplayMarker"`
    pub underwater_realm_display_marker: Option<Entity>,

    /// `mIslandDisplayMarker`: `@"BCDisplayMarker"`
    pub island_display_marker: Option<Entity>,

    /// `mSandIslandDisplayGroup`: `@"BCDisplayGroup"`
    pub sand_island_display_group: Option<Entity>,

    /// `mRockIslandDisplayGroup`: `@"BCDisplayGroup"`
    pub rock_island_display_group: Option<Entity>,

    /// `mGraveyardIslandDisplayGroup`: `@"BCDisplayGroup"`
    pub graveyard_island_display_group: Option<Entity>,

    /// `mIceIslandDisplayGroup`: `@"BCDisplayGroup"`
    pub ice_island_display_group: Option<Entity>,

    /// `mApeIslandDisplayGroup`: `@"BCDisplayGroup"`
    pub ape_island_display_group: Option<Entity>,

    /// `mMisfitIslandDisplayGroup`: `@"BCDisplayGroup"`
    pub misfit_island_display_group: Option<Entity>,

    /// `mGodIslandDisplayGroup`: `@"BCDisplayGroup"`
    pub god_island_display_group: Option<Entity>,

    /// `mFailBoatOceanDisplayGroup`: `@"BCDisplayGroup"`
    pub fail_boat_ocean_display_group: Option<Entity>,

    /// `mApocalypseIslandDisplayGroup`: `@"BCDisplayGroup"`
    pub apocalypse_island_display_group: Option<Entity>,

    /// `mTimeTempleDisplayGroup`: `@"BCDisplayGroup"`
    pub time_temple_display_group: Option<Entity>,

    /// `mLocustTempleDisplayGroup`: `@"BCDisplayGroup"`
    pub locust_temple_display_group: Option<Entity>,

    /// `mDanceTempleDisplayGroup`: `@"BCDisplayGroup"`
    pub dance_temple_display_group: Option<Entity>,

    /// `mPlagueTempleDisplayGroup`: `@"BCDisplayGroup"`
    pub plague_temple_display_group: Option<Entity>,

    /// `mPurificationTempleDisplayGroup`: `@"BCDisplayGroup"`
    pub purification_temple_display_group: Option<Entity>,

    /// `mClosetTempleDisplayGroup`: `@"BCDisplayGroup"`
    pub closet_temple_display_group: Option<Entity>,

    /// `mXenuTempleDisplayGroup`: `@"BCDisplayGroup"`
    pub xenu_temple_display_group: Option<Entity>,

    /// `mFrontTsunamiLogic`: `@"TsunamiLogic"`
    pub front_tsunami_logic: Option<Entity>,

    /// `mBehindTsunamiLogic`: `@"TsunamiLogic"`
    pub behind_tsunami_logic: Option<Entity>,

    /// `mbIslandLocationTransition`: `B`
    pub island_location_transition: bool,

    /// `mLastIslandLocationState`: `i`
    pub last_island_location_state: i32,

    /// `mIslandLocationState`: `i`
    pub island_location_state: i32,

    /// `mIslandMinXPos`: `f`
    pub island_min_x_pos: f32,

    /// `mIslandMaxXPos`: `f`
    pub island_max_x_pos: f32,

    /// `mIslandCenterXPos`: `f`
    pub island_center_x_pos: f32,

    /// `mHorizonYPos`: `f`
    pub horizon_y_pos: f32,

    /// `mSunTransitionYPos`: `f`
    pub sun_transition_y_pos: f32,

    /// `mSunUnderHorizonTriggerYPos`: `f`
    pub sun_under_horizon_trigger_y_pos: f32,

    /// `mSunUnderHorizonYPos`: `f`
    pub sun_under_horizon_y_pos: f32,

    /// `mSunOverHorizonYPos`: `f`
    pub sun_over_horizon_y_pos: f32,

    /// `mMoonUnderHorizonTriggerYPos`: `f`
    pub moon_under_horizon_trigger_y_pos: f32,

    /// `mMoonUnderHorizonYPos`: `f`
    pub moon_under_horizon_y_pos: f32,

    /// `mMoonOverHorizonYPos`: `f`
    pub moon_over_horizon_y_pos: f32,
}

/// `-[PocketGodViewController islandLocationState]` @ 0x100077d18
pub fn island_location_state(world: &mut World, this: Entity) -> i32 {
    world.obj::<ControllerIslandLocations>(this).island_location_state
}

/// `-[PocketGodViewController lastIslandLocationState]` @ 0x100077d28
pub fn last_island_location_state(world: &mut World, this: Entity) -> i32 {
    world.obj::<ControllerIslandLocations>(this).last_island_location_state
}

/// `-[PocketGodViewController onIdolAwardScreenChangeIslandLocationState:]` @ 0x100078d64
pub fn on_idol_award_screen_change_island_location_state(world: &mut World, this: Entity, idol_award_screen_change_island_location_state: i32) {
    set_island_location_state(world, this, idol_award_screen_change_island_location_state, false);
}

/// `-[PocketGodViewController onMapScreenClosed]` @ 0x100078fd8
pub fn on_map_screen_closed(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).map_screen_open = false;
}

/// `-[PocketGodViewController onMapScreenChangeIslandLocationState:]` @ 0x100078fe8
pub fn on_map_screen_change_island_location_state(world: &mut World, this: Entity, map_screen_change_island_location_state: i32) {
    set_island_location_state(world, this, map_screen_change_island_location_state, false);
}

/// `-[PocketGodViewController loadLocationTextureAtlas]` @ 0x10007cbc8
pub fn load_location_texture_atlas(world: &mut World, this: Entity) {
    let bridge = world.obj::<ControllerManagersBridge>(this);
    let menus = world.obj::<ControllerMenusAndScreens>(this);
    let story_screen_without_story = bridge.story_screen_open && !bridge.story_active;
    let screen_open = bridge.map_screen_open
        || menus.stats_screen_open
        || menus.whats_new_screen_open
        || bridge.skin_pack_screen_open
        || bridge.bonus_pack_screen_open
        || story_screen_without_story
        || bridge.news_screen_open
        || menus.help_screen_open
        || menus.debug_screen_open;
    if screen_open {
        return;
    }

    let minigames = world.obj::<ControllerMinigames>(this);
    let minigame_atlas = if minigames.show_doodle_assets {
        Some("OogaJumpTextureAtlas0")
    } else if minigames.show_pain_drain_assets {
        Some("PainDrainTextureAtlas0")
    } else if minigames.show_the_runs_assets {
        Some("TheRunsTextureAtlas0")
    } else if minigames.show_konkey_dong_assets {
        Some("KonkeyDongTextureAtlas0")
    } else if minigames.show_moron_pests_assets {
        Some("MoronPestsTextureAtlas0")
    } else if minigames.show_no_pee_assets {
        Some("NoPeeTextureAtlas0")
    } else if minigames.show_battle_of_the_gods_assets {
        Some("BattleTextureAtlas0")
    } else {
        None
    };
    if let Some(minigame_atlas) = minigame_atlas {
        load_screen_atlas(world, minigame_atlas);
        return;
    }

    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    match island_location_state {
        LOCATION_SAND_ISLAND => {
            refresh_skin_pack_option(world, this, "Halloween", "Sand Island Statue", "Statue");
            refresh_skin_pack_option(world, this, "Halloween", "Sand Island", "Sand Island Extras");
            load_screen_atlas(world, "SandTextureAtlas0");
        }
        LOCATION_ROCK_ISLAND => {
            refresh_skin_pack_option(world, this, "Halloween", "Rock Island", "Rock Island Extras");
            load_screen_atlas(world, "RockTextureAtlas0");
        }
        LOCATION_UNDERWATER => load_screen_atlas(world, "UnderwaterTextureAtlas0"),
        LOCATION_GRAVEYARD_ISLAND => {
            refresh_skin_pack_option(world, this, "Halloween", "Graveyard Island", "Graveyard Island Extras");
            load_screen_atlas(world, "GraveyardTextureAtlas0");
        }
        LOCATION_ICE_ISLAND => {
            refresh_skin_pack_option(world, this, "IceMonster", "Ice Monster", "Ice Monster");
            refresh_skin_pack_option(world, this, "Halloween", "Ice Island", "Ice Island Extras");
            load_screen_atlas(world, "IceTextureAtlas0");
        }
        LOCATION_APE_ISLAND => {
            refresh_skin_pack_option(world, this, "Halloween", "Ape Island", "Ape Island Extras");
            load_screen_atlas(world, "ApeIslandTextureAtlas0");
        }
        LOCATION_MISFIT_ISLAND => load_screen_atlas(world, "MisfitTextureAtlas0"),
        LOCATION_GOD_ISLAND => {
            load_skin_texture_atlas_channel(world, 1);

            let god_idols = world.obj::<ControllerWorldRegistry>(this).god_idol_logic_array.clone();
            for god_idol in god_idols {
                crate::logic::battle::god_idol_logic::update_idol_texture(world, god_idol);
            }

            load_screen_atlas(world, "GodTextureAtlas0");
        }
        LOCATION_FAIL_BOAT => load_screen_atlas(world, "BoatTextureAtlas0"),
        LOCATION_UNDERWATER_CORAL_REEF => load_screen_atlas(world, "UnderwaterCoralReefTextureAtlas0"),
        LOCATION_APOCALYPSE_ISLAND => load_screen_atlas(world, "ApocalypseTextureAtlas0"),
        LOCATION_TIME_TEMPLE => load_screen_atlas(world, "TimeTempleTextureAtlas0"),
        LOCATION_LOCUST_TEMPLE => load_screen_atlas(world, "LocustTempleTextureAtlas0"),
        LOCATION_DANCE_TEMPLE => {
            load_skin_texture_atlas_channel(world, 2);
            load_screen_atlas(world, "DanceTempleTextureAtlas0");
        }
        LOCATION_PLAGUE_TEMPLE => load_screen_atlas(world, "PlagueTempleTextureAtlas0"),
        LOCATION_PURIFICATION_TEMPLE => load_screen_atlas(world, "PurificationTempleTextureAtlas0"),
        LOCATION_CLOSET_TEMPLE => load_screen_atlas(world, "ClosetTempleTextureAtlas0"),
        LOCATION_XENU_TEMPLE => {
            load_skin_texture_atlas_channel(world, 3);
            load_screen_atlas(world, "XenuTempleTextureAtlas0");
        }
        _ => {}
    }
}

/// `-[PocketGodViewController onLoadLocationTextureAtlas]` @ 0x10007d3ac
pub fn on_load_location_texture_atlas(world: &mut World, this: Entity) {
    load_location_texture_atlas(world, this);
}

/// `-[PocketGodViewController loadTextureAtlasChannel:::]` @ 0x10007d3bc
pub fn seq_load_texture_atlas_channel(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let channel = call.param("channel");
    let atlas_channel = match channel {
        Some("Disco") => 2,
        Some("Xenu") => 3,
        _ => return ItemResult::Done,
    };

    let skin_pack_manager = world.obj::<ControllerManagersBridge>(this).skin_pack_manager;
    if let Some(skin_pack_manager) = skin_pack_manager {
        skin_pack_manager::load_texture_atlas_channel(world, skin_pack_manager, atlas_channel);
    }
    ItemResult::Done
}

/// `-[PocketGodViewController refreshSkinPackOption:::]` @ 0x10007d468
pub fn seq_refresh_skin_pack_option(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let skin_pack_id = call.param("skinPackId").unwrap_or_default();
    let skin_option_id = call.param("skinOptionId").unwrap_or_default();
    let skin_object_id = call.param("skinObjectId").unwrap_or_default();
    refresh_skin_pack_option(world, this, skin_pack_id, skin_object_id, skin_option_id);
    ItemResult::Done
}

/// `-[PocketGodViewController deactivateLastIslandLocationState]` @ 0x10007ed70
pub fn deactivate_last_island_location_state(world: &mut World, this: Entity) {
    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        let locations = world.obj::<ControllerIslandLocations>(this);
        let last_island_location_state = locations.last_island_location_state;
        let island_location_state = locations.island_location_state;
        crate::pygmy::on_deactivate_island_location_state(world, pygmy, last_island_location_state, island_location_state);
    }

    let last_island_location_state = world.obj::<ControllerIslandLocations>(this).last_island_location_state;
    match last_island_location_state {
        LOCATION_SAND_ISLAND => deactivate_sand_island(world, this),
        LOCATION_ROCK_ISLAND => deactivate_rock_island(world, this),
        LOCATION_UNDERWATER => deactivate_underwater(world, this),
        LOCATION_GRAVEYARD_ISLAND => deactivate_graveyard_island(world, this),
        LOCATION_ICE_ISLAND => deactivate_ice_island(world, this),
        LOCATION_APE_ISLAND => deactivate_ape_island(world, this),
        LOCATION_MISFIT_ISLAND => deactivate_misfit_island(world, this),
        LOCATION_GOD_ISLAND => deactivate_god_island(world, this),
        LOCATION_FAIL_BOAT => deactivate_fail_boat(world, this),
        LOCATION_UNDERWATER_CORAL_REEF => deactivate_underwater_coral_reef(world, this),
        LOCATION_APOCALYPSE_ISLAND => deactivate_apocalypse_island(world, this),
        LOCATION_TIME_TEMPLE => deactivate_time_temple(world, this),
        LOCATION_LOCUST_TEMPLE => deactivate_locust_temple(world, this),
        LOCATION_DANCE_TEMPLE => deactivate_dance_temple(world, this),
        LOCATION_PLAGUE_TEMPLE => deactivate_plague_temple(world, this),
        LOCATION_PURIFICATION_TEMPLE => deactivate_purification_temple(world, this),
        LOCATION_CLOSET_TEMPLE => deactivate_closet_temple(world, this),
        LOCATION_XENU_TEMPLE => deactivate_xenu_temple(world, this),
        _ => {}
    }

    let locations = world.obj::<ControllerIslandLocations>(this);
    let going_underwater = is_underwater_location(locations.island_location_state);
    let was_underwater = is_underwater_location(locations.last_island_location_state);
    if !going_underwater || was_underwater {
        return;
    }

    let bat = world.obj::<ControllerSkyWeather>(this).bat_logic;
    if let Some(bat) = bat {
        pg_engine::logic::set_behavior(world, bat, Some("BatFlyOffScreen"));
    }

    let hurricane = world.obj::<ControllerGodPowers>(this).hurricane_logic;
    if let Some(hurricane) = hurricane {
        hurricane_logic::stop_hurricane_now(world, hurricane);

        let hurricane_touch = world.obj::<ControllerGodPowers>(this).hurricane_touch;
        if hurricane_touch.is_some() {
            let hurricane = world.obj::<ControllerGodPowers>(this).hurricane_logic;
            if let Some(hurricane) = hurricane {
                hurricane_logic::set_touch_active(world, hurricane, false);
            }
            world.obj_mut::<ControllerGodPowers>(this).hurricane_touch = None;

            let hurricane_trail = world.obj::<ControllerGodPowers>(this).hurricane_trail_logic;
            discard_finger_trail(world, hurricane_trail);
            world.obj_mut::<ControllerGodPowers>(this).hurricane_trail_logic = None;
        }
    }

    let island_display_group = world.obj::<ControllerIslandLocations>(this).island_display_group;
    remove_display_group(world, island_display_group);
}

/// `-[PocketGodViewController activateIslandLocationState:]` @ 0x100080fa8
pub fn activate_island_location_state(world: &mut World, this: Entity, activate_island_location_state: bool) {
    let startup = activate_island_location_state;
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    match island_location_state {
        LOCATION_SAND_ISLAND => activate_sand_island(world, this, startup),
        LOCATION_ROCK_ISLAND => activate_rock_island(world, this, startup),
        LOCATION_UNDERWATER => activate_underwater(world, this, startup),
        LOCATION_GRAVEYARD_ISLAND => activate_graveyard_island(world, this, startup),
        LOCATION_ICE_ISLAND => activate_ice_island(world, this, startup),
        LOCATION_APE_ISLAND => activate_ape_island(world, this, startup),
        LOCATION_MISFIT_ISLAND => activate_misfit_island(world, this, startup),
        LOCATION_GOD_ISLAND => activate_god_island(world, this, startup),
        LOCATION_FAIL_BOAT => activate_fail_boat(world, this),
        LOCATION_UNDERWATER_CORAL_REEF => activate_underwater_coral_reef(world, this, startup),
        LOCATION_APOCALYPSE_ISLAND => activate_apocalypse_island(world, this, startup),
        LOCATION_TIME_TEMPLE => activate_time_temple(world, this, startup),
        LOCATION_LOCUST_TEMPLE => activate_locust_temple(world, this, startup),
        LOCATION_DANCE_TEMPLE => activate_dance_temple(world, this, startup),
        LOCATION_PLAGUE_TEMPLE => activate_plague_temple(world, this, startup),
        LOCATION_PURIFICATION_TEMPLE => activate_purification_temple(world, this, startup),
        LOCATION_CLOSET_TEMPLE => activate_closet_temple(world, this, startup),
        LOCATION_XENU_TEMPLE => activate_xenu_temple(world, this, startup),
        _ => {}
    }

    let mut locations = world.obj_mut::<ControllerIslandLocations>(this);
    let horizon_y_pos = locations.horizon_y_pos;
    locations.sun_transition_y_pos = horizon_y_pos;
    locations.sun_under_horizon_trigger_y_pos = horizon_y_pos - 25.0;
    locations.sun_under_horizon_y_pos = horizon_y_pos - 40.0;
    locations.sun_over_horizon_y_pos = horizon_y_pos + 40.0;
    locations.moon_under_horizon_trigger_y_pos = horizon_y_pos - 25.0;
    locations.moon_under_horizon_y_pos = horizon_y_pos - 40.0;
    locations.moon_over_horizon_y_pos = horizon_y_pos + 40.0;

    let now_underwater = is_underwater_location(locations.island_location_state);
    let was_underwater = is_underwater_location(locations.last_island_location_state);
    let island_display_group = locations.island_display_group;
    let island_root_display_marker = locations.island_root_display_marker;
    if !now_underwater && was_underwater {
        move_display_group(world, island_display_group, island_root_display_marker);
    }
}

/// `-[PocketGodViewController setIslandLocationState:startup:]` @ 0x100083b64
pub fn set_island_location_state(world: &mut World, this: Entity, island_location_state: i32, startup: bool) {
    let category_menu_state = world.obj::<ControllerMenusAndScreens>(this).category_menu_state;
    if category_menu_state == 1 {
        set_category_menu_state(world, this, 0);
    }

    let mut old_island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if old_island_location_state == island_location_state && !startup {
        return;
    }

    let transition_running = world.obj::<ControllerIslandLocations>(this).island_location_transition;
    if transition_running {
        world.obj_mut::<ControllerIslandLocations>(this).island_location_transition = false;
        deactivate_last_island_location_state(world, this);
        activate_island_location_state(world, this, false);
        old_island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    }

    let mut locations = world.obj_mut::<ControllerIslandLocations>(this);
    locations.last_island_location_state = old_island_location_state;
    locations.island_location_state = island_location_state;

    let option_preferences = world.obj::<ControllerBootAndFrame>(this).option_preferences;
    if let Some(option_preferences) = option_preferences {
        pg_engine::prefs::set_preference_as_int(world, option_preferences, "IslandLocation", island_location_state);
    }

    let locations = world.obj::<ControllerIslandLocations>(this);
    let last_island_location_state = locations.last_island_location_state;
    let new_island_location_state = locations.island_location_state;

    // Only 9 → 2 and 2 → 9 swap realms under water; a startup call that stays at 2 or 9 takes
    // the way out of the realm, as in the original.
    let reef_to_ruins = last_island_location_state == LOCATION_UNDERWATER_CORAL_REEF && new_island_location_state == LOCATION_UNDERWATER;
    let ruins_to_reef = last_island_location_state == LOCATION_UNDERWATER && new_island_location_state == LOCATION_UNDERWATER_CORAL_REEF;
    if reef_to_ruins || ruins_to_reef {
        let front_tsunami = world.obj::<ControllerIslandLocations>(this).front_tsunami_logic;
        if let Some(front_tsunami) = front_tsunami {
            tsunami_logic::move_between_underwater_locations(world, front_tsunami);
        }
        deactivate_last_island_location_state(world, this);
        activate_island_location_state(world, this, startup);
        return;
    }

    if is_underwater_location(last_island_location_state) {
        leave_underwater(world, this);
        return;
    }

    if is_underwater_location(new_island_location_state) {
        enter_underwater(world, this, startup);
        return;
    }

    if !startup {
        deactivate_last_island_location_state(world, this);
    }
    activate_island_location_state(world, this, startup);
}

/// `-[PocketGodViewController isInsideApocalypseTemple]` @ 0x1000871bc
pub fn is_inside_apocalypse_temple(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let temple_index = (island_location_state as u32).wrapping_sub(LOCATION_TIME_TEMPLE as u32);
    temple_index < 7
}

/// `-[PocketGodViewController isDreamBubbleEnabled]` @ 0x100087814
pub fn is_dream_bubble_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state != LOCATION_MISFIT_ISLAND
}

/// `-[PocketGodViewController isDoodleCloudEnabled]` @ 0x100087868
pub fn is_doodle_cloud_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let temple_index = (island_location_state as u32).wrapping_sub(LOCATION_TIME_TEMPLE as u32);
    6 < temple_index
}

/// `-[PocketGodViewController isOceanEnabled]` @ 0x1000881a4
pub fn is_ocean_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;

    // Bit n of 0x3e answers for location 5 + n: Ape island (5) and the temples (11–17) have no
    // ocean, locations 6–10 have one; every other value has one too.
    let offset = (island_location_state as u32).wrapping_sub(LOCATION_APE_ISLAND as u32);
    if offset < 13 {
        let ocean_bits: u32 = 0x3e;
        return (ocean_bits >> (offset & 0x1f)) & 1 != 0;
    }
    true
}

/// `-[PocketGodViewController hasMisfitIslandBeenFound]` @ 0x1000881d8
pub fn has_misfit_island_been_found(world: &mut World, this: Entity) -> bool {
    let system_preferences = world.obj::<ControllerBootAndFrame>(this).system_preferences;
    let Some(system_preferences) = system_preferences else {
        return false;
    };
    let misfit_island_found = pg_engine::prefs::get_int_preference(world, system_preferences, "MisfitIslandFound");
    misfit_island_found != 0
}

/// `-[PocketGodViewController islandMinXPos]` @ 0x100088214
pub fn island_min_x_pos(world: &mut World, this: Entity) -> f32 {
    world.obj::<ControllerIslandLocations>(this).island_min_x_pos
}

/// `-[PocketGodViewController islandMaxXPos]` @ 0x100088224
pub fn island_max_x_pos(world: &mut World, this: Entity) -> f32 {
    world.obj::<ControllerIslandLocations>(this).island_max_x_pos
}

/// `-[PocketGodViewController islandCenterXPos]` @ 0x100088234
pub fn island_center_x_pos(world: &mut World, this: Entity) -> f32 {
    world.obj::<ControllerIslandLocations>(this).island_center_x_pos
}

/// `-[PocketGodViewController (Game) loadLocationTextureAtlas:::]` @ 0x10015a118
pub fn seq_load_location_texture_atlas(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    load_location_texture_atlas(world, this);
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) isTsunamiActive]` @ 0x100163fc4
pub fn is_tsunami_active(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    is_underwater_location(island_location_state)
}

/// `-[PocketGodViewController (Game) onTsunamiActivateComplete:]` @ 0x100164f88
pub fn on_tsunami_activate_complete(world: &mut World, this: Entity, _tsunami_activate_complete: Entity) {
    world.obj_mut::<ControllerIslandLocations>(this).island_location_transition = false;

    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        crate::pygmy::start_underwater(world, pygmy);
    }
}

/// `-[PocketGodViewController (Game) onTsunamiDeactivateComplete:]` @ 0x1001650a8
pub fn on_tsunami_deactivate_complete(world: &mut World, this: Entity, _tsunami_deactivate_complete: Entity) {
    world.obj_mut::<ControllerIslandLocations>(this).island_location_transition = false;

    let underwater_display_group = world.obj::<ControllerIslandLocations>(this).underwater_display_group;
    remove_display_group(world, underwater_display_group);
}

/// `-[PocketGodViewController (Game) loadMonsterTextureAtlas]` @ 0x100169bf0
pub fn load_monster_texture_atlas(world: &mut World, this: Entity) {
    if is_spider_enabled(world, this) {
        refresh_skin_pack_option(world, this, "Spider", "Spider", "Spider");
    } else if is_ape_enabled(world, this) {
        refresh_skin_pack_option(world, this, "ZombiePack", "Ape", "Ape");
    } else if is_locust_queen_enabled(world, this) {
        refresh_skin_pack_option(world, this, "ZombiePack", "Locust Queen", "Locust Queen");
    } else if is_ice_monster_enabled(world, this) {
        refresh_skin_pack_option(world, this, "IceMonster", "Ice Monster", "Ice Monster");
    } else if is_t_rex_enabled(world, this) {
        refresh_skin_pack_option(world, this, "TRex", "Tyrannosaurus Rex", "TRex Body");
    }
}

/// `-[PocketGodViewController (Game) onApocalypseRoomFaderChangeIslandLocationState:]` @ 0x100169d90
pub fn on_apocalypse_room_fader_change_island_location_state(world: &mut World, this: Entity, apocalypse_room_fader_change_island_location_state: i32) {
    set_island_location_state(world, this, apocalypse_room_fader_change_island_location_state, false);

    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        crate::pygmy::drop_into_apocalypse_temple_room(world, pygmy);
    }

    let apocalypse_active = world.obj::<ControllerApocalypse>(this).apocalypse_active;
    if apocalypse_active {
        let thetan_meter = world.obj::<ControllerWorldRegistry>(this).closet_temple_thetan_meter_logic;
        if let Some(thetan_meter) = thetan_meter {
            closet_temple_thetan_meter_logic::drop_apocalypse_thetans_into_apoclaypse_temple_room(world, thetan_meter);
        }
    }
}

/// `-[PocketGodViewController (Game) onDreamBubbleChangeIslandLocationState:]` @ 0x100169ef4
pub fn on_dream_bubble_change_island_location_state(world: &mut World, this: Entity, dream_bubble_change_island_location_state: i32) {
    let system_preferences = world.obj::<ControllerBootAndFrame>(this).system_preferences;
    if let Some(system_preferences) = system_preferences {
        pg_engine::prefs::set_preference_as_int(world, system_preferences, "MisfitIslandFound", 1);
    }
    set_island_location_state(world, this, dream_bubble_change_island_location_state, false);
}

/// `-[PocketGodViewController (Game) onTsunamiHideIslandAssets:]` @ 0x100169f58
pub fn on_tsunami_hide_island_assets(world: &mut World, this: Entity, _tsunami_hide_island_assets: Entity) {
    deactivate_last_island_location_state(world, this);
    activate_island_location_state(world, this, false);
}

/// `-[PocketGodViewController (Game) onTsunamiShowIslandAssets:]` @ 0x100169f98
pub fn on_tsunami_show_island_assets(world: &mut World, this: Entity, _tsunami_show_island_assets: Entity) {
    deactivate_last_island_location_state(world, this);
    activate_island_location_state(world, this, false);
}

// ---------------------------------------------------------------------------------------------
// setIslandLocationState:startup: — the tsunami transitions.

/// `setIslandLocationState:startup:` leaving the underwater realm for the surface: both tsunami
/// layers recede, and the location swap happens when they call back (`onTsunami…IslandAssets:`).
fn leave_underwater(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerIslandLocations>(this).island_location_transition = true;

    let front_tsunami = world.obj::<ControllerIslandLocations>(this).front_tsunami_logic;
    if let Some(front_tsunami) = front_tsunami {
        tsunami_logic::deactivate(world, front_tsunami);
    }
    let behind_tsunami = world.obj::<ControllerIslandLocations>(this).behind_tsunami_logic;
    if let Some(behind_tsunami) = behind_tsunami {
        tsunami_logic::deactivate(world, behind_tsunami);
    }

    let last_island_location_state = world.obj::<ControllerIslandLocations>(this).last_island_location_state;
    if last_island_location_state != LOCATION_UNDERWATER_CORAL_REEF {
        return;
    }
    let underwater_coral_reef_state = world.obj::<ControllerIslandFeatures>(this).underwater_coral_reef_state;
    if underwater_coral_reef_state != 1 {
        return;
    }
    let jellyfish_array = world.obj::<ControllerWorldRegistry>(this).jellyfish_logic_array.clone();
    for jellyfish in jellyfish_array {
        crate::send::exit_island_with_arg(world, Some(jellyfish), false);
    }
}

/// `setIslandLocationState:startup:` going from the surface into the underwater realm: both
/// tsunami layers rise, `UnderwaterDisplayGroup` goes to its marker, and the location swap
/// happens when the tsunami calls back (`onTsunami…IslandAssets:`).
fn enter_underwater(world: &mut World, this: Entity, startup: bool) {
    if startup {
        refresh_skin_pack_option(world, this, "Halloween", "Rock Island", "Rock Island Extras");
    }
    world.obj_mut::<ControllerIslandLocations>(this).island_location_transition = true;

    let front_tsunami = world.obj::<ControllerIslandLocations>(this).front_tsunami_logic;
    let to_ruins = world.obj::<ControllerIslandLocations>(this).island_location_state == LOCATION_UNDERWATER;
    if let Some(front_tsunami) = front_tsunami {
        tsunami_logic::activate(world, front_tsunami, to_ruins);
    }
    let behind_tsunami = world.obj::<ControllerIslandLocations>(this).behind_tsunami_logic;
    let to_ruins = world.obj::<ControllerIslandLocations>(this).island_location_state == LOCATION_UNDERWATER;
    if let Some(behind_tsunami) = behind_tsunami {
        tsunami_logic::activate(world, behind_tsunami, to_ruins);
    }

    let locations = world.obj::<ControllerIslandLocations>(this);
    let underwater_display_group = locations.underwater_display_group;
    let underwater_display_marker = locations.underwater_display_marker;
    move_display_group(world, underwater_display_group, underwater_display_marker);
}

// ---------------------------------------------------------------------------------------------
// activateIslandLocationState:, one function per location.

/// `activateIslandLocationState:` for Sand island.
fn activate_sand_island(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartSandIsland");

    let sand_island_state = world.obj::<ControllerIslandFeatures>(this).sand_island_state;
    let last_island_location_state = world.obj::<ControllerIslandLocations>(this).last_island_location_state;
    match sand_island_state {
        1 => {
            let boulder = world.obj::<ControllerWorldRegistry>(this).boulder_logic;
            if let Some(boulder) = boulder {
                boulder_logic::enter_island(world, boulder, startup);
            }
        }
        2 => {
            if !fishing_carried(world, this, LOCATION_SAND_ISLAND, last_island_location_state) {
                enter_fishing_pole_and_fish(world, this, startup);
            }
        }
        3 => {
            let magnifying_glass = world.obj::<ControllerWorldRegistry>(this).magnifying_glass_logic;
            if let Some(magnifying_glass) = magnifying_glass {
                magnifying_glass_logic::enter_island(world, magnifying_glass, startup);
            }
            let ant_hill = world.obj::<ControllerWorldRegistry>(this).ant_hill_logic;
            if let Some(ant_hill) = ant_hill {
                ant_hill_logic::enter_island(world, ant_hill, startup);
            }
        }
        4 => {
            if !water_wings_carried(world, this, LOCATION_SAND_ISLAND, last_island_location_state) {
                enter_water_wings(world, this, startup);
            }
        }
        _ => {}
    }

    let first_coconut = world.obj::<ControllerWorldRegistry>(this).coconut_logic_array.first().copied();
    if let Some(first_coconut) = first_coconut {
        let coconut_display_object = pg_engine::logic::display_object(world, first_coconut);
        let mut coconut_display = pg_engine::display::object_mut(world, coconut_display_object);
        coconut_display.pos.x = 163.0;
        coconut_display.pos.y = 205.0;
        crate::send::enter_island(world, Some(first_coconut));
    }

    let volcano = world.obj::<ControllerGodPowers>(this).volcano_logic;
    if let Some(volcano) = volcano {
        volcano_logic::enter_island(world, volcano, startup);
    }
    let octo_statue = world.obj::<ControllerWorldRegistry>(this).octo_statue_logic;
    if let Some(octo_statue) = octo_statue {
        octo_statue_logic::enter_island(world, octo_statue, startup);
    }

    let locations = world.obj::<ControllerIslandLocations>(this);
    let sand_island_display_group = locations.sand_island_display_group;
    let island_display_marker = locations.island_display_marker;
    move_display_group(world, sand_island_display_group, island_display_marker);

    set_island_bounds(world, this, 50.0, 390.0, 220.0, 160.0);
    load_location_texture_atlas(world, this);
}

/// `activateIslandLocationState:` for Rock island.
fn activate_rock_island(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartRockIsland");

    let rock_island_state = world.obj::<ControllerIslandFeatures>(this).rock_island_state;
    if rock_island_state == 1 {
        let last_island_location_state = world.obj::<ControllerIslandLocations>(this).last_island_location_state;
        if !fishing_carried(world, this, LOCATION_ROCK_ISLAND, last_island_location_state) {
            enter_fishing_pole_and_fish(world, this, startup);
        }
    }

    let rock_island_feature_state = world.obj::<ControllerIslandFeatures>(this).rock_island_feature_state;
    if rock_island_feature_state == 2 {
        let t_rex_egg = world.obj::<ControllerWorldRegistry>(this).t_rex_egg_logic;
        if let Some(t_rex_egg) = t_rex_egg {
            t_rex_egg_logic::enter_island(world, t_rex_egg, startup);
        }
    } else if rock_island_feature_state == 1 {
        let bon_fire = world.obj::<ControllerWorldRegistry>(this).bon_fire_logic;
        if let Some(bon_fire) = bon_fire {
            bon_fire_logic::enter_island(world, bon_fire, startup);
        }
        let woods = world.obj::<ControllerWorldRegistry>(this).wood_logic_array.clone();
        for wood in woods {
            crate::send::enter_island_with_arg(world, Some(wood), startup);
        }
    }

    let rock_island_out_house_state = world.obj::<ControllerIslandFeatures>(this).rock_island_out_house_state;
    let out_house = world.obj::<ControllerWorldRegistry>(this).out_house_logic;
    if let Some(out_house) = out_house {
        if rock_island_out_house_state == 1 {
            out_house_logic::enter_island(world, out_house, startup);
        } else if rock_island_out_house_state == 0 && startup {
            out_house_logic::exit_island(world, out_house, false);
        }
    }

    let rock_island_t_rex_state = world.obj::<ControllerIslandFeatures>(this).rock_island_t_rex_state;
    if rock_island_t_rex_state == 1 {
        let t_rex = world.obj::<ControllerWorldRegistry>(this).t_rex_logic;
        if let Some(t_rex) = t_rex {
            t_rex_logic::enter_island(world, t_rex, startup);
        }
    }

    let rock_island_spear_state = world.obj::<ControllerIslandFeatures>(this).rock_island_spear_state;
    if rock_island_spear_state == 1 {
        enter_spears(world, this, startup);
    }

    set_island_bounds(world, this, 50.0, 414.0, 232.0, 160.0);

    let plague_sign = pg_engine::library::logic(world, crate::logic_table::PLAGUE_SIGN);
    let disease_exists = apocalypse_cloud_logic::class_does_disease_exist(world);
    let plague_sign_hidden = !disease_exists && !does_plague_body_exist(world, this);
    let plague_sign_behavior = if plague_sign_hidden { "PlagueSignHide" } else { "PlagueSignShow" };
    pg_engine::logic::set_behavior(world, plague_sign, Some(plague_sign_behavior));

    let locations = world.obj::<ControllerIslandLocations>(this);
    let rock_island_display_group = locations.rock_island_display_group;
    let island_display_marker = locations.island_display_marker;
    move_display_group(world, rock_island_display_group, island_display_marker);
    load_location_texture_atlas(world, this);
}

/// `activateIslandLocationState:` for the underwater ruins.
fn activate_underwater(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartUnderwater");

    let underwater_state = world.obj::<ControllerIslandFeatures>(this).underwater_state;
    if underwater_state == 3 {
        let tile_drain = world.obj::<ControllerWorldRegistry>(this).tile_drain_logic;
        if let Some(tile_drain) = tile_drain {
            tile_drain_logic::enter_island(world, tile_drain, startup);
        }
        let tile_plug = world.obj::<ControllerWorldRegistry>(this).tile_plug_logic;
        if let Some(tile_plug) = tile_plug {
            tile_plug_logic::enter_island(world, tile_plug, startup);
        }
    } else if underwater_state == 2 {
        let chum_bucket = world.obj::<ControllerWorldRegistry>(this).chum_bucket_logic;
        if let Some(chum_bucket) = chum_bucket {
            chum_bucket_logic::enter_island(world, chum_bucket, startup);
        }
    } else if underwater_state == 1 {
        let anchor = world.obj::<ControllerWorldRegistry>(this).anchor_logic;
        if let Some(anchor) = anchor {
            anchor_logic::enter_island(world, anchor, startup);
        }
    }

    let last_island_location_state = world.obj::<ControllerIslandLocations>(this).last_island_location_state;
    if last_island_location_state != LOCATION_UNDERWATER_CORAL_REEF {
        let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
        for pygmy in pygmies {
            crate::pygmy::wait_underwater(world, pygmy);
        }
    }

    apocalypse_cloud_logic::class_cure_all_disease(world);

    let locations = world.obj::<ControllerIslandLocations>(this);
    let underwater_ruins_display_group = locations.underwater_ruins_display_group;
    let underwater_realm_display_marker = locations.underwater_realm_display_marker;
    move_display_group(world, underwater_ruins_display_group, underwater_realm_display_marker);
    load_location_texture_atlas(world, this);
}

/// `activateIslandLocationState:` for Graveyard island.
fn activate_graveyard_island(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartGraveyardIsland");

    let graveyard_island_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_state;
    if graveyard_island_state == 1 {
        let last_island_location_state = world.obj::<ControllerIslandLocations>(this).last_island_location_state;
        if !water_wings_carried(world, this, LOCATION_GRAVEYARD_ISLAND, last_island_location_state) {
            enter_water_wings(world, this, startup);
        }
    }

    let zombie_grave_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_zombie_grave_state;
    let zombie_grave = world.obj::<ControllerWorldRegistry>(this).zombie_grave_logic;
    if let Some(zombie_grave) = zombie_grave {
        if zombie_grave_state == 1 {
            zombie_grave_logic::enter_island(world, zombie_grave, startup);
        } else if zombie_grave_state == 0 && startup {
            zombie_grave_logic::exit_island(world, zombie_grave, false);
        }
    }

    let spider_web_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_spider_web_state;
    let spider_tree = world.obj::<ControllerWorldRegistry>(this).spider_tree_logic;
    if let Some(spider_tree) = spider_tree {
        if spider_web_state == 1 {
            spider_tree_logic::enter_island(world, spider_tree, startup);
        } else if spider_web_state == 0 && startup {
            spider_tree_logic::exit_island(world, spider_tree, false);
        }
    }

    let spider_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_spider_state;
    if spider_state == 1 {
        let spider = world.obj::<ControllerWorldRegistry>(this).spider_logic;
        if let Some(spider) = spider {
            spider_logic::enter_island(world, spider, startup);
        }
    }

    let spear_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_spear_state;
    if spear_state == 1 {
        enter_spears(world, this, startup);
    }

    set_island_bounds(world, this, 32.0, 448.0, 240.0, 160.0);

    let locations = world.obj::<ControllerIslandLocations>(this);
    let graveyard_island_display_group = locations.graveyard_island_display_group;
    let island_display_marker = locations.island_display_marker;
    move_display_group(world, graveyard_island_display_group, island_display_marker);
    load_location_texture_atlas(world, this);
}

/// `activateIslandLocationState:` for Ice island.
fn activate_ice_island(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartIceIsland");

    let ice_island_state = world.obj::<ControllerIslandFeatures>(this).ice_island_state;
    let last_island_location_state = world.obj::<ControllerIslandLocations>(this).last_island_location_state;
    if ice_island_state == 2 {
        if !water_wings_carried(world, this, LOCATION_ICE_ISLAND, last_island_location_state) {
            enter_water_wings(world, this, startup);
        }
    } else if ice_island_state == 1 {
        if !fishing_carried(world, this, LOCATION_ICE_ISLAND, last_island_location_state) {
            enter_fishing_pole_and_fish(world, this, startup);
        }
    }

    let ice_monster_state = world.obj::<ControllerIslandFeatures>(this).ice_island_ice_monster_state;
    let ice_monster = world.obj::<ControllerWorldRegistry>(this).ice_monster_logic;
    if let Some(ice_monster) = ice_monster {
        if ice_monster_state == 1 {
            ice_monster_logic::enter_island(world, ice_monster, startup);
        } else if ice_monster_state == 0 && startup {
            ice_monster_logic::exit_island(world, ice_monster, false);
        }
    }

    let igloo_state = world.obj::<ControllerIslandFeatures>(this).ice_island_igloo_state;
    let igloo = world.obj::<ControllerWorldRegistry>(this).igloo_logic;
    if let Some(igloo) = igloo {
        if igloo_state == 1 {
            igloo_logic::enter_island(world, igloo, startup);
        } else if igloo_state == 0 && startup {
            igloo_logic::exit_island(world, igloo, false);
        }
    }

    let ice_hole_state = world.obj::<ControllerIslandFeatures>(this).ice_island_ice_hole_state;
    let ice_hole = world.obj::<ControllerWorldRegistry>(this).ice_hole_logic;
    if let Some(ice_hole) = ice_hole {
        if ice_hole_state == 1 {
            ice_hole_logic::enter_island(world, ice_hole, startup);
        } else if ice_hole_state == 0 && startup {
            ice_hole_logic::exit_island(world, ice_hole, false);
        }
    }

    let ice_squid_state = world.obj::<ControllerIslandFeatures>(this).ice_island_ice_squid_state;
    let ice_hole = world.obj::<ControllerWorldRegistry>(this).ice_hole_logic;
    if let Some(ice_hole) = ice_hole {
        if ice_squid_state == 1 {
            ice_hole_logic::enable_ice_squid(world, ice_hole);
        } else if ice_squid_state == 0 && startup {
            ice_hole_logic::disable_ice_squid(world, ice_hole);
        }
    }

    set_island_bounds(world, this, 50.0, 410.0, 230.0, 160.0);

    let glacier = world.obj::<ControllerWorldRegistry>(this).glacier_logic;
    if let Some(glacier) = glacier {
        glacier_logic::enter_island(world, glacier, startup);
    }
    let ice_tree = world.obj::<ControllerWorldRegistry>(this).ice_tree_logic;
    if let Some(ice_tree) = ice_tree {
        ice_tree_logic::enter_island(world, ice_tree, startup);
    }

    let locations = world.obj::<ControllerIslandLocations>(this);
    let ice_island_display_group = locations.ice_island_display_group;
    let island_display_marker = locations.island_display_marker;
    move_display_group(world, ice_island_display_group, island_display_marker);
    load_location_texture_atlas(world, this);
}

/// `activateIslandLocationState:` for Ape island: the ocean layers and the sun reflection leave
/// the tree, the sky gradient and the sun and moon move down (the mountain hides the sea).
fn activate_ape_island(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartApeIsland");

    let ape_island_state = world.obj::<ControllerIslandFeatures>(this).ape_island_state;
    if ape_island_state == 2 {
        let banana_tree = world.obj::<ControllerWorldRegistry>(this).banana_tree_logic;
        if let Some(banana_tree) = banana_tree {
            banana_tree_logic::enter_island(world, banana_tree, startup);
        }
        let kong_statue = world.obj::<ControllerWorldRegistry>(this).kong_statue_logic;
        if let Some(kong_statue) = kong_statue {
            kong_statue_logic::enter_island(world, kong_statue, startup);
        }
    } else if ape_island_state == 1 {
        let ape_altar = world.obj::<ControllerWorldRegistry>(this).ape_altar_logic;
        if let Some(ape_altar) = ape_altar {
            ape_altar_logic::enter_island(world, ape_altar, startup);
        }
        let ape_bongo = world.obj::<ControllerWorldRegistry>(this).ape_bongo_logic;
        if let Some(ape_bongo) = ape_bongo {
            ape_bongo_logic::enter_island(world, ape_bongo, startup);
        }
        let ape_wench = world.obj::<ControllerWorldRegistry>(this).ape_wench_logic;
        if let Some(ape_wench) = ape_wench {
            ape_wench_logic::enter_island(world, ape_wench, startup);
        }
    }

    let ape_state = world.obj::<ControllerIslandFeatures>(this).ape_island_ape_state;
    let ape = world.obj::<ControllerWorldRegistry>(this).ape_logic;
    if let Some(ape) = ape {
        if ape_state == 1 {
            ape_logic::enter_island(world, ape, startup);
        } else if ape_state == 0 && startup {
            ape_logic::exit_island(world, ape, false);
        }
    }

    set_island_bounds(world, this, 80.0, 400.0, 240.0, 120.0);

    let front_ocean = world.obj::<ControllerTouchInput>(this).front_ocean_logic;
    if let Some(front_ocean) = front_ocean {
        let front_ocean_display_object = pg_engine::logic::display_object(world, front_ocean);
        pg_engine::display::remove_from_display_group(world, front_ocean_display_object);
    }
    let behind_ocean = world.obj::<ControllerTouchInput>(this).behind_ocean_logic;
    if let Some(behind_ocean) = behind_ocean {
        let behind_ocean_display_object = pg_engine::logic::display_object(world, behind_ocean);
        pg_engine::display::remove_from_display_group(world, behind_ocean_display_object);
    }
    let sun_reflection = world.obj::<ControllerSkyWeather>(this).sun_reflection_display_object;
    remove_display_group(world, sun_reflection);

    set_gradient_y_pos(world, this, 60.0);

    let sun = world.obj::<ControllerSkyWeather>(this).sun_logic;
    if let Some(sun) = sun {
        let sun_display_object = pg_engine::logic::display_object(world, sun);
        let mut sun_display = pg_engine::display::object_mut(world, sun_display_object);
        if sun_display.pos.y != 245.0 {
            sun_display.pos.y = sun_display.pos.y - 40.0;
        }
    }
    let moon = world.obj::<ControllerSkyWeather>(this).moon_logic;
    if let Some(moon) = moon {
        let moon_display_object = pg_engine::logic::display_object(world, moon);
        let mut moon_display = pg_engine::display::object_mut(world, moon_display_object);
        if moon_display.pos.y != 245.0 {
            moon_display.pos.y = moon_display.pos.y - 40.0;
        }
    }

    let ape_torches = world.obj::<ControllerWorldRegistry>(this).ape_torch_logic_array.clone();
    for ape_torch in ape_torches {
        crate::send::enter_island_with_arg(world, Some(ape_torch), startup);
    }

    let locations = world.obj::<ControllerIslandLocations>(this);
    let ape_island_display_group = locations.ape_island_display_group;
    let island_display_marker = locations.island_display_marker;
    move_display_group(world, ape_island_display_group, island_display_marker);
    load_location_texture_atlas(world, this);

    let shark = world.obj::<ControllerWorldRegistry>(this).shark_logic;
    if let Some(shark) = shark {
        shark_logic::exit_island_now(world, shark);
    }
}

/// `activateIslandLocationState:` for Misfit island.
fn activate_misfit_island(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartMisfitIsland");

    let misfit_island_state = world.obj::<ControllerIslandFeatures>(this).misfit_island_state;
    if misfit_island_state == 2 {
        let last_island_location_state = world.obj::<ControllerIslandLocations>(this).last_island_location_state;
        if !water_wings_carried(world, this, LOCATION_MISFIT_ISLAND, last_island_location_state) {
            enter_water_wings(world, this, startup);
        }
    } else if misfit_island_state == 1 {
        let moron_snake = world.obj::<ControllerWorldRegistry>(this).moron_snake_logic;
        if let Some(moron_snake) = moron_snake {
            moron_snake_logic::enter_island(world, moron_snake, startup);
        }
    }

    set_island_bounds(world, this, 32.0, 420.0, 226.0, 160.0);

    let misfit_ocean_rock = world.obj::<ControllerWorldRegistry>(this).misfit_ocean_rock_logic;
    if let Some(misfit_ocean_rock) = misfit_ocean_rock {
        island_object_logic::enter_island(world, misfit_ocean_rock, startup);
    }

    let locations = world.obj::<ControllerIslandLocations>(this);
    let misfit_island_display_group = locations.misfit_island_display_group;
    let island_display_marker = locations.island_display_marker;
    move_display_group(world, misfit_island_display_group, island_display_marker);
    load_location_texture_atlas(world, this);
}

/// `activateIslandLocationState:` for God island.
fn activate_god_island(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartGodIsland");

    let god_island_state = world.obj::<ControllerIslandFeatures>(this).god_island_state;
    if god_island_state == 1 {
        let last_island_location_state = world.obj::<ControllerIslandLocations>(this).last_island_location_state;
        if !water_wings_carried(world, this, LOCATION_GOD_ISLAND, last_island_location_state) {
            enter_water_wings(world, this, startup);
        }
    }

    let god_island_idols_state = world.obj::<ControllerIslandFeatures>(this).god_island_idols_state;
    if god_island_idols_state == 1 {
        load_skin_texture_atlas_channel(world, 1);

        let god_idols = world.obj::<ControllerWorldRegistry>(this).god_idol_logic_array.clone();
        for god_idol in god_idols {
            crate::send::enter_island_with_arg(world, Some(god_idol), startup);
        }
    }

    set_island_bounds(world, this, 32.0, 448.0, 240.0, 160.0);

    load_skin_texture_atlas_channel(world, 1);

    let idol_pedestals = world.obj::<ControllerWorldRegistry>(this).idol_pedestal_logic_array.clone();
    for idol_pedestal in idol_pedestals {
        crate::send::enter_island_with_arg(world, Some(idol_pedestal), startup);
    }
    let temple_door = world.obj::<ControllerWorldRegistry>(this).temple_door_logic;
    if let Some(temple_door) = temple_door {
        temple_door_logic::enter_island(world, temple_door, startup);
    }

    let locations = world.obj::<ControllerIslandLocations>(this);
    let god_island_display_group = locations.god_island_display_group;
    let island_display_marker = locations.island_display_marker;
    move_display_group(world, god_island_display_group, island_display_marker);
    load_location_texture_atlas(world, this);
}

/// `activateIslandLocationState:` for the Fail Boat.
fn activate_fail_boat(world: &mut World, this: Entity) {
    set_misc_sequence_def(world, this, "StartFailBoat");

    set_island_bounds(world, this, 32.0, 448.0, 240.0, 160.0);

    let locations = world.obj::<ControllerIslandLocations>(this);
    let fail_boat_ocean_display_group = locations.fail_boat_ocean_display_group;
    let island_display_marker = locations.island_display_marker;
    move_display_group(world, fail_boat_ocean_display_group, island_display_marker);
    load_location_texture_atlas(world, this);
}

/// `activateIslandLocationState:` for the underwater coral reef.
fn activate_underwater_coral_reef(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartUnderwaterCoralReef");

    let underwater_coral_reef_state = world.obj::<ControllerIslandFeatures>(this).underwater_coral_reef_state;
    if underwater_coral_reef_state == 1 {
        let jellyfish_array = world.obj::<ControllerWorldRegistry>(this).jellyfish_logic_array.clone();
        for jellyfish in jellyfish_array {
            crate::send::enter_island_with_arg(world, Some(jellyfish), startup);
        }
    }

    let last_island_location_state = world.obj::<ControllerIslandLocations>(this).last_island_location_state;
    if last_island_location_state != LOCATION_UNDERWATER {
        let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
        for pygmy in pygmies {
            crate::pygmy::wait_underwater(world, pygmy);
        }
    }

    apocalypse_cloud_logic::class_cure_all_disease(world);

    let locations = world.obj::<ControllerIslandLocations>(this);
    let underwater_coral_reef_display_group = locations.underwater_coral_reef_display_group;
    let underwater_realm_display_marker = locations.underwater_realm_display_marker;
    move_display_group(world, underwater_coral_reef_display_group, underwater_realm_display_marker);
    load_location_texture_atlas(world, this);
}

/// `activateIslandLocationState:` for Apocalypse island (the only location whose start script
/// is set after its water wings).
fn activate_apocalypse_island(world: &mut World, this: Entity, startup: bool) {
    let apocalypse_island_state = world.obj::<ControllerIslandFeatures>(this).apocalypse_island_state;
    if apocalypse_island_state == 1 {
        let last_island_location_state = world.obj::<ControllerIslandLocations>(this).last_island_location_state;
        if !water_wings_carried(world, this, LOCATION_APOCALYPSE_ISLAND, last_island_location_state) {
            enter_water_wings(world, this, startup);
        }
    }

    set_misc_sequence_def(world, this, "StartApocalypseIsland");

    set_island_bounds(world, this, 32.0, 448.0, 240.0, 160.0);

    let gem_pedestals = world.obj::<ControllerWorldRegistry>(this).apocalypse_gem_pedestal_logic_array.clone();
    for gem_pedestal in gem_pedestals {
        crate::send::enter_island_with_arg(world, Some(gem_pedestal), startup);
    }

    let calendar = world.obj::<ControllerWorldRegistry>(this).apocalypse_calendar_logic;
    if let Some(calendar) = calendar {
        apocalypse_calendar_logic::enter_island(world, calendar, startup);
    }
    let temple_door = world.obj::<ControllerWorldRegistry>(this).apocalypse_temple_door_logic;
    if let Some(temple_door) = temple_door {
        apocalypse_temple_door_logic::enter_island(world, temple_door, startup);
    }
    let portal_temple_door = world.obj::<ControllerWorldRegistry>(this).apocalypse_portal_temple_door_logic;
    if let Some(portal_temple_door) = portal_temple_door {
        apocalypse_temple_door_logic::enter_island(world, portal_temple_door, startup);
    }

    let locations = world.obj::<ControllerIslandLocations>(this);
    let apocalypse_island_display_group = locations.apocalypse_island_display_group;
    let island_display_marker = locations.island_display_marker;
    move_display_group(world, apocalypse_island_display_group, island_display_marker);
    load_location_texture_atlas(world, this);
}

/// `activateIslandLocationState:` for the Time temple.
fn activate_time_temple(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartTimeTemple");

    set_island_bounds(world, this, 80.0, 400.0, 240.0, 160.0);

    let doors = world.obj::<ControllerWorldRegistry>(this).time_temple_door_logic_array.clone();
    for door in doors {
        crate::send::enter_island_with_arg(world, Some(door), startup);
    }

    let lever = world.obj::<ControllerWorldRegistry>(this).apocalypse_time_temple_lever_logic;
    if let Some(lever) = lever {
        apocalypse_time_temple_lever_logic::enter_island(world, lever, startup);
    }
    let platform = world.obj::<ControllerWorldRegistry>(this).apocalypse_time_temple_platform_logic;
    if let Some(platform) = platform {
        apocalypse_time_temple_platform_logic::enter_island(world, platform, startup);
    }

    show_apocalypse_temple_display_group(world, this);
}

/// `activateIslandLocationState:` for the Locust temple.
fn activate_locust_temple(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartLocustTemple");

    set_island_bounds(world, this, 80.0, 400.0, 240.0, 160.0);

    let doors = world.obj::<ControllerWorldRegistry>(this).locust_temple_door_logic_array.clone();
    for door in doors {
        crate::send::enter_island_with_arg(world, Some(door), startup);
    }

    let locust_queen = world.obj::<ControllerWorldRegistry>(this).locust_queen_logic;
    if let Some(locust_queen) = locust_queen {
        locust_queen_logic::enter_island(world, locust_queen, startup);
    }

    show_apocalypse_temple_display_group(world, this);
}

/// `activateIslandLocationState:` for the Dance temple.
fn activate_dance_temple(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartDanceTemple");

    set_island_bounds(world, this, 80.0, 400.0, 240.0, 160.0);

    let doors = world.obj::<ControllerWorldRegistry>(this).dance_temple_door_logic_array.clone();
    for door in doors {
        crate::send::enter_island_with_arg(world, Some(door), startup);
    }

    let go_go_cage = world.obj::<ControllerWorldRegistry>(this).go_go_cage_logic;
    if let Some(go_go_cage) = go_go_cage {
        go_go_cage_logic::enter_island(world, go_go_cage, startup);
    }
    let disco_ball = world.obj::<ControllerWorldRegistry>(this).disco_ball_logic;
    if let Some(disco_ball) = disco_ball {
        disco_ball_logic::enter_island(world, disco_ball, startup);
    }
    let disco_lights = world.obj::<ControllerWorldRegistry>(this).disco_lights_logic;
    if let Some(disco_lights) = disco_lights {
        disco_lights_logic::enter_island(world, disco_lights, startup);
    }
    let balloon_bundle = world.obj::<ControllerWorldRegistry>(this).dance_balloon_bundle_logic;
    if let Some(balloon_bundle) = balloon_bundle {
        dance_balloon_bundle_logic::enter_island(world, balloon_bundle, startup);
    }
    let dance_dance_execution = world.obj::<ControllerWorldRegistry>(this).dance_dance_execution_logic;
    if let Some(dance_dance_execution) = dance_dance_execution {
        dance_dance_execution_logic::enter_island(world, dance_dance_execution, startup);
    }

    show_apocalypse_temple_display_group(world, this);
}

/// `activateIslandLocationState:` for the Plague temple.
fn activate_plague_temple(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartPlagueTemple");

    set_island_bounds(world, this, 65.0, 415.0, 240.0, 160.0);

    let doors = world.obj::<ControllerWorldRegistry>(this).plague_temple_door_logic_array.clone();
    for door in doors {
        crate::send::enter_island_with_arg(world, Some(door), startup);
    }

    let fountain = world.obj::<ControllerWorldRegistry>(this).apocalypse_plague_temple_fountain_logic;
    if let Some(fountain) = fountain {
        apocalypse_temple_fountain_logic::enter_island(world, fountain, startup);
    }

    show_apocalypse_temple_display_group(world, this);
}

/// `activateIslandLocationState:` for the Purification temple.
fn activate_purification_temple(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartPurificationTemple");

    set_island_bounds(world, this, 32.0, 448.0, 240.0, 160.0);

    let doors = world.obj::<ControllerWorldRegistry>(this).purification_temple_door_logic_array.clone();
    for door in doors {
        crate::send::enter_island_with_arg(world, Some(door), startup);
    }

    let fountain = world.obj::<ControllerWorldRegistry>(this).apocalypse_purification_temple_fountain_logic;
    if let Some(fountain) = fountain {
        apocalypse_temple_fountain_logic::enter_island(world, fountain, startup);
    }

    show_apocalypse_temple_display_group(world, this);
}

/// `activateIslandLocationState:` for the Closet temple.
fn activate_closet_temple(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartClosetTemple");

    set_island_bounds(world, this, 45.0, 420.0, 232.5, 160.0);

    let doors = world.obj::<ControllerWorldRegistry>(this).closet_temple_door_logic_array.clone();
    for door in doors {
        crate::send::enter_island_with_arg(world, Some(door), startup);
    }

    let thetan_meter = world.obj::<ControllerWorldRegistry>(this).closet_temple_thetan_meter_logic;
    if let Some(thetan_meter) = thetan_meter {
        closet_temple_thetan_meter_logic::enter_island(world, thetan_meter, startup);
    }
    let couch = world.obj::<ControllerWorldRegistry>(this).closet_temple_couch_logic;
    if let Some(couch) = couch {
        closet_temple_couch_logic::enter_island(world, couch, startup);
    }
    let closet_door = world.obj::<ControllerWorldRegistry>(this).closet_temple_closet_door_logic;
    if let Some(closet_door) = closet_door {
        closet_temple_closet_door_logic::enter_island(world, closet_door, startup);
    }

    show_apocalypse_temple_display_group(world, this);
}

/// `activateIslandLocationState:` for the Xenu temple.
fn activate_xenu_temple(world: &mut World, this: Entity, startup: bool) {
    set_misc_sequence_def(world, this, "StartXenuTemple");

    set_island_bounds(world, this, 55.0, 448.0, 251.5, 160.0);

    let doors = world.obj::<ControllerWorldRegistry>(this).xenu_temple_door_logic_array.clone();
    for door in doors {
        crate::send::enter_island_with_arg(world, Some(door), startup);
    }

    let xenu_temple = world.obj::<ControllerWorldRegistry>(this).xenu_temple_logic;
    if let Some(xenu_temple) = xenu_temple {
        xenu_temple_logic::enter_island(world, xenu_temple, startup);
    }
    let xenu = world.obj::<ControllerWorldRegistry>(this).xenu_logic;
    if let Some(xenu) = xenu {
        xenu_logic::enter_island(world, xenu, startup);
    }

    show_apocalypse_temple_display_group(world, this);
}

// ---------------------------------------------------------------------------------------------
// deactivateLastIslandLocationState, one function per location left.

/// `deactivateLastIslandLocationState` leaving Sand island: the split coconut is dropped.
fn deactivate_sand_island(world: &mut World, this: Entity) {
    let sand_island_state = world.obj::<ControllerIslandFeatures>(this).sand_island_state;
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    match sand_island_state {
        1 => {
            let boulder = world.obj::<ControllerWorldRegistry>(this).boulder_logic;
            if let Some(boulder) = boulder {
                boulder_logic::exit_island_now(world, boulder);
            }
        }
        2 => {
            if !fishing_carried(world, this, LOCATION_SAND_ISLAND, island_location_state) {
                exit_fishing_pole(world, this);
            }
        }
        3 => {
            let ant_hill = world.obj::<ControllerWorldRegistry>(this).ant_hill_logic;
            if let Some(ant_hill) = ant_hill {
                ant_hill_logic::exit_island_now(world, ant_hill);
            }
            let magnifying_glass = world.obj::<ControllerWorldRegistry>(this).magnifying_glass_logic;
            if let Some(magnifying_glass) = magnifying_glass {
                magnifying_glass_logic::exit_island_now(world, magnifying_glass);
            }
        }
        4 => {
            if !water_wings_carried(world, this, LOCATION_SAND_ISLAND, island_location_state) {
                exit_water_wings(world, this);
            }
        }
        _ => {}
    }

    let first_coconut = world.obj::<ControllerWorldRegistry>(this).coconut_logic_array.first().copied();
    crate::send::exit_island(world, first_coconut);

    let coconuts = world.obj::<ControllerWorldRegistry>(this).coconut_logic_array.clone();
    if coconuts.len() > 1 {
        let split_coconut = coconuts[1];
        let split_coconut_display_object = pg_engine::logic::display_object(world, split_coconut);
        crate::send::exit_island(world, Some(split_coconut));
        pg_engine::display::remove_from_display_group(world, split_coconut_display_object);
        pg_engine::logic::shutdown(world, split_coconut);

        // The array held the last reference, but the entity stays: `mDraggedCoconutLogic` (and
        // the touch arrays) may still point at a coconut being dragged.
        let mut registry = world.obj_mut::<ControllerWorldRegistry>(this);
        registry.coconut_logic_array.retain(|coconut| *coconut != split_coconut);
    }

    check_count_coconut_bounces(world, this);

    let volcano = world.obj::<ControllerGodPowers>(this).volcano_logic;
    if let Some(volcano) = volcano {
        volcano_logic::exit_island(world, volcano);
    }
    let octo_statue = world.obj::<ControllerWorldRegistry>(this).octo_statue_logic;
    if let Some(octo_statue) = octo_statue {
        octo_statue_logic::exit_island(world, octo_statue);
    }

    let sand_island_display_group = world.obj::<ControllerIslandLocations>(this).sand_island_display_group;
    remove_display_group(world, sand_island_display_group);
}

/// `deactivateLastIslandLocationState` leaving Rock island.
fn deactivate_rock_island(world: &mut World, this: Entity) {
    let rock_island_state = world.obj::<ControllerIslandFeatures>(this).rock_island_state;
    if rock_island_state == 1 {
        let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
        if !fishing_carried(world, this, LOCATION_ROCK_ISLAND, island_location_state) {
            exit_fishing_pole(world, this);
        }
    }

    let rock_island_feature_state = world.obj::<ControllerIslandFeatures>(this).rock_island_feature_state;
    if rock_island_feature_state == 2 {
        let t_rex_egg = world.obj::<ControllerWorldRegistry>(this).t_rex_egg_logic;
        if let Some(t_rex_egg) = t_rex_egg {
            t_rex_egg_logic::exit_island_now(world, t_rex_egg);
        }
    } else if rock_island_feature_state == 1 {
        let bon_fire = world.obj::<ControllerWorldRegistry>(this).bon_fire_logic;
        if let Some(bon_fire) = bon_fire {
            bon_fire_logic::exit_island_now(world, bon_fire);
        }
        let woods = world.obj::<ControllerWorldRegistry>(this).wood_logic_array.clone();
        for wood in woods {
            crate::send::exit_island_now(world, Some(wood));
        }
    }

    let t_rex = world.obj::<ControllerWorldRegistry>(this).t_rex_logic;
    if let Some(t_rex) = t_rex {
        t_rex_logic::exit_island(world, t_rex);
    }

    exit_spears(world, this);

    let out_house = world.obj::<ControllerWorldRegistry>(this).out_house_logic;
    if let Some(out_house) = out_house {
        out_house_logic::exit_island(world, out_house, true);
    }

    let rock_island_display_group = world.obj::<ControllerIslandLocations>(this).rock_island_display_group;
    remove_display_group(world, rock_island_display_group);
}

/// `deactivateLastIslandLocationState` leaving the underwater ruins. The ruins group stays in
/// the realm marker (the next underwater group replaces it there).
fn deactivate_underwater(world: &mut World, this: Entity) {
    let underwater_state = world.obj::<ControllerIslandFeatures>(this).underwater_state;
    if underwater_state == 3 {
        let tile_drain = world.obj::<ControllerWorldRegistry>(this).tile_drain_logic;
        if let Some(tile_drain) = tile_drain {
            tile_drain_logic::exit_island(world, tile_drain);
        }
        let tile_plug = world.obj::<ControllerWorldRegistry>(this).tile_plug_logic;
        if let Some(tile_plug) = tile_plug {
            tile_plug_logic::exit_island(world, tile_plug);
        }
    } else if underwater_state == 2 {
        let chum_bucket = world.obj::<ControllerWorldRegistry>(this).chum_bucket_logic;
        if let Some(chum_bucket) = chum_bucket {
            chum_bucket_logic::exit_island(world, chum_bucket);
        }
    } else if underwater_state == 1 {
        let anchor = world.obj::<ControllerWorldRegistry>(this).anchor_logic;
        if let Some(anchor) = anchor {
            anchor_logic::exit_island(world, anchor);
        }
    }

    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state == LOCATION_UNDERWATER_CORAL_REEF {
        return;
    }

    end_whirl_pool(world, this);

    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        crate::pygmy::stop_underwater(world, pygmy);
    }
}

/// `deactivateLastIslandLocationState` leaving Graveyard island.
fn deactivate_graveyard_island(world: &mut World, this: Entity) {
    let graveyard_island_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_state;
    if graveyard_island_state == 1 {
        let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
        if !water_wings_carried(world, this, LOCATION_GRAVEYARD_ISLAND, island_location_state) {
            exit_water_wings(world, this);
        }
    }

    let spider_tree = world.obj::<ControllerWorldRegistry>(this).spider_tree_logic;
    if let Some(spider_tree) = spider_tree {
        spider_tree_logic::exit_island(world, spider_tree, true);
    }
    let spider = world.obj::<ControllerWorldRegistry>(this).spider_logic;
    if let Some(spider) = spider {
        spider_logic::exit_island(world, spider);
    }

    exit_spears(world, this);

    let zombie_grave = world.obj::<ControllerWorldRegistry>(this).zombie_grave_logic;
    if let Some(zombie_grave) = zombie_grave {
        zombie_grave_logic::exit_island(world, zombie_grave, true);
    }

    let graveyard_island_display_group = world.obj::<ControllerIslandLocations>(this).graveyard_island_display_group;
    remove_display_group(world, graveyard_island_display_group);
}

/// `deactivateLastIslandLocationState` leaving Ice island.
fn deactivate_ice_island(world: &mut World, this: Entity) {
    let ice_island_state = world.obj::<ControllerIslandFeatures>(this).ice_island_state;
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if ice_island_state == 2 {
        if !water_wings_carried(world, this, LOCATION_ICE_ISLAND, island_location_state) {
            exit_water_wings(world, this);
        }
    } else if ice_island_state == 1 {
        if !fishing_carried(world, this, LOCATION_ICE_ISLAND, island_location_state) {
            exit_fishing_pole(world, this);
        }
    }

    let ice_hole = world.obj::<ControllerWorldRegistry>(this).ice_hole_logic;
    if let Some(ice_hole) = ice_hole {
        ice_hole_logic::exit_island(world, ice_hole, true);
    }
    let ice_monster = world.obj::<ControllerWorldRegistry>(this).ice_monster_logic;
    if let Some(ice_monster) = ice_monster {
        ice_monster_logic::exit_island(world, ice_monster, true);
    }
    let glacier = world.obj::<ControllerWorldRegistry>(this).glacier_logic;
    if let Some(glacier) = glacier {
        glacier_logic::exit_island(world, glacier);
    }
    let igloo = world.obj::<ControllerWorldRegistry>(this).igloo_logic;
    if let Some(igloo) = igloo {
        igloo_logic::exit_island(world, igloo, true);
    }
    let ice_tree = world.obj::<ControllerWorldRegistry>(this).ice_tree_logic;
    if let Some(ice_tree) = ice_tree {
        ice_tree_logic::exit_island(world, ice_tree);
    }

    let ice_island_display_group = world.obj::<ControllerIslandLocations>(this).ice_island_display_group;
    remove_display_group(world, ice_island_display_group);
}

/// `deactivateLastIslandLocationState` leaving Ape island: the ocean layers (and the sun
/// reflection at sunset) come back after `FireworkDisplayMarker`, the sky gradient and the sun
/// and moon move back up, and the shark returns. The giant ape only exits with the ritual
/// toggle (`ApeIslandState` 1), as in the original.
fn deactivate_ape_island(world: &mut World, this: Entity) {
    let ape_island_state = world.obj::<ControllerIslandFeatures>(this).ape_island_state;
    if ape_island_state == 2 {
        let banana_tree = world.obj::<ControllerWorldRegistry>(this).banana_tree_logic;
        if let Some(banana_tree) = banana_tree {
            banana_tree_logic::exit_island(world, banana_tree, true);
        }
        let kong_statue = world.obj::<ControllerWorldRegistry>(this).kong_statue_logic;
        if let Some(kong_statue) = kong_statue {
            kong_statue_logic::exit_island(world, kong_statue);
        }
    } else if ape_island_state == 1 {
        let ape = world.obj::<ControllerWorldRegistry>(this).ape_logic;
        if let Some(ape) = ape {
            ape_logic::exit_island(world, ape, true);
        }
        let ape_altar = world.obj::<ControllerWorldRegistry>(this).ape_altar_logic;
        if let Some(ape_altar) = ape_altar {
            ape_altar_logic::exit_island(world, ape_altar, true);
        }
        let ape_bongo = world.obj::<ControllerWorldRegistry>(this).ape_bongo_logic;
        if let Some(ape_bongo) = ape_bongo {
            ape_bongo_logic::exit_island(world, ape_bongo, true);
        }
        let ape_wench = world.obj::<ControllerWorldRegistry>(this).ape_wench_logic;
        if let Some(ape_wench) = ape_wench {
            ape_wench_logic::exit_island(world, ape_wench, true);
        }
    }

    let sunset_active = world.obj::<ControllerSkyWeather>(this).sunset_active;
    if sunset_active {
        let firework_display_marker = world.obj::<ControllerGodPowers>(this).firework_display_marker;
        let sun_reflection = world.obj::<ControllerSkyWeather>(this).sun_reflection_display_object;
        if let Some(firework_display_marker) = firework_display_marker {
            pg_engine::display::insert_after(world, firework_display_marker, sun_reflection);
        }
    }

    let front_ocean = world.obj::<ControllerTouchInput>(this).front_ocean_logic;
    let front_ocean_display_object = front_ocean.map(|ocean| pg_engine::logic::display_object(world, ocean));
    let firework_display_marker = world.obj::<ControllerGodPowers>(this).firework_display_marker;
    if let Some(firework_display_marker) = firework_display_marker {
        pg_engine::display::insert_after(world, firework_display_marker, front_ocean_display_object);
    }
    let behind_ocean = world.obj::<ControllerTouchInput>(this).behind_ocean_logic;
    let behind_ocean_display_object = behind_ocean.map(|ocean| pg_engine::logic::display_object(world, ocean));
    let firework_display_marker = world.obj::<ControllerGodPowers>(this).firework_display_marker;
    if let Some(firework_display_marker) = firework_display_marker {
        pg_engine::display::insert_after(world, firework_display_marker, behind_ocean_display_object);
    }

    set_gradient_y_pos(world, this, 120.0);

    let sun = world.obj::<ControllerSkyWeather>(this).sun_logic;
    if let Some(sun) = sun {
        let sun_display_object = pg_engine::logic::display_object(world, sun);
        let mut sun_display = pg_engine::display::object_mut(world, sun_display_object);
        let raised_y_pos = sun_display.pos.y + 40.0;
        sun_display.pos.y = if raised_y_pos <= 245.0 { raised_y_pos } else { 245.0 };
    }
    let moon = world.obj::<ControllerSkyWeather>(this).moon_logic;
    if let Some(moon) = moon {
        let moon_display_object = pg_engine::logic::display_object(world, moon);
        let mut moon_display = pg_engine::display::object_mut(world, moon_display_object);
        let raised_y_pos = moon_display.pos.y + 40.0;
        moon_display.pos.y = if raised_y_pos <= 245.0 { raised_y_pos } else { 245.0 };
    }

    let ape_torches = world.obj::<ControllerWorldRegistry>(this).ape_torch_logic_array.clone();
    for ape_torch in ape_torches {
        crate::send::exit_island(world, Some(ape_torch));
    }
    let monkeys = world.obj::<ControllerWorldRegistry>(this).monkey_logic_array.clone();
    for monkey in monkeys {
        crate::send::exit_island_with_arg(world, Some(monkey), true);
    }
    let banana = world.obj::<ControllerWorldRegistry>(this).banana_logic;
    if let Some(banana) = banana {
        banana_logic::exit_island(world, banana);
    }

    let ape_island_display_group = world.obj::<ControllerIslandLocations>(this).ape_island_display_group;
    remove_display_group(world, ape_island_display_group);

    let shark_state = world.obj::<ControllerIslandFeatures>(this).shark_state;
    if shark_state == 1 {
        let shark = world.obj::<ControllerWorldRegistry>(this).shark_logic;
        if let Some(shark) = shark {
            shark_logic::enter_island(world, shark, false);
        }
    }
}

/// `deactivateLastIslandLocationState` leaving Misfit island.
fn deactivate_misfit_island(world: &mut World, this: Entity) {
    let misfit_island_state = world.obj::<ControllerIslandFeatures>(this).misfit_island_state;
    if misfit_island_state == 2 {
        let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
        if !water_wings_carried(world, this, LOCATION_MISFIT_ISLAND, island_location_state) {
            exit_water_wings(world, this);
        }
    } else if misfit_island_state == 1 {
        let moron_snake = world.obj::<ControllerWorldRegistry>(this).moron_snake_logic;
        if let Some(moron_snake) = moron_snake {
            moron_snake_logic::exit_island(world, moron_snake, true);
        }
    }

    let misfit_ocean_rock = world.obj::<ControllerWorldRegistry>(this).misfit_ocean_rock_logic;
    if let Some(misfit_ocean_rock) = misfit_ocean_rock {
        island_object_logic::exit_island(world, misfit_ocean_rock);
    }

    let misfit_island_display_group = world.obj::<ControllerIslandLocations>(this).misfit_island_display_group;
    remove_display_group(world, misfit_island_display_group);
}

/// `deactivateLastIslandLocationState` leaving God island.
fn deactivate_god_island(world: &mut World, this: Entity) {
    let god_island_state = world.obj::<ControllerIslandFeatures>(this).god_island_state;
    if god_island_state == 1 {
        let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
        if !water_wings_carried(world, this, LOCATION_GOD_ISLAND, island_location_state) {
            exit_water_wings(world, this);
        }
    }

    let god_island_idols_state = world.obj::<ControllerIslandFeatures>(this).god_island_idols_state;
    if god_island_idols_state == 1 {
        let god_idols = world.obj::<ControllerWorldRegistry>(this).god_idol_logic_array.clone();
        for god_idol in god_idols {
            crate::send::exit_island_with_arg(world, Some(god_idol), true);
        }
    }

    let idol_pedestals = world.obj::<ControllerWorldRegistry>(this).idol_pedestal_logic_array.clone();
    for idol_pedestal in idol_pedestals {
        crate::send::exit_island_with_arg(world, Some(idol_pedestal), true);
    }
    let temple_door = world.obj::<ControllerWorldRegistry>(this).temple_door_logic;
    if let Some(temple_door) = temple_door {
        temple_door_logic::exit_island(world, temple_door, true);
    }

    let god_island_display_group = world.obj::<ControllerIslandLocations>(this).god_island_display_group;
    remove_display_group(world, god_island_display_group);
}

/// `deactivateLastIslandLocationState` leaving the Fail Boat.
fn deactivate_fail_boat(world: &mut World, this: Entity) {
    let fail_boat_ocean_display_group = world.obj::<ControllerIslandLocations>(this).fail_boat_ocean_display_group;
    remove_display_group(world, fail_boat_ocean_display_group);
}

/// `deactivateLastIslandLocationState` leaving the underwater coral reef: going to the surface
/// also ends an underwater power stroke and clears its trail puffs.
fn deactivate_underwater_coral_reef(world: &mut World, this: Entity) {
    let underwater_coral_reef_state = world.obj::<ControllerIslandFeatures>(this).underwater_coral_reef_state;
    if underwater_coral_reef_state == 1 {
        let jellyfish_array = world.obj::<ControllerWorldRegistry>(this).jellyfish_logic_array.clone();
        for jellyfish in jellyfish_array {
            crate::send::exit_island_with_arg(world, Some(jellyfish), true);
        }
    }

    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state == LOCATION_UNDERWATER {
        return;
    }

    end_whirl_pool(world, this);

    let underwater_power_touch = world.obj::<ControllerGodPowers>(this).underwater_power_touch;
    if underwater_power_touch.is_some() {
        world.obj_mut::<ControllerGodPowers>(this).underwater_power_touch = None;
        let underwater_power_trail = world.obj::<ControllerGodPowers>(this).underwater_power_trail_logic;
        discard_finger_trail(world, underwater_power_trail);
        world.obj_mut::<ControllerGodPowers>(this).underwater_power_trail_logic = None;
    }

    let trail_puffs = world.obj::<ControllerGodPowers>(this).underwater_power_trail_logic_array.clone();
    for trail_puff in trail_puffs {
        let trail_puff_display_object = pg_engine::logic::display_object(world, trail_puff);
        pg_engine::display::remove_from_display_group(world, trail_puff_display_object);
        pg_engine::logic::shutdown(world, trail_puff);
    }

    // `removeAllObjects`: the array was each puff's only owner.
    let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
    let removed_trail_puffs = std::mem::take(&mut god_powers.underwater_power_trail_logic_array);
    for trail_puff in removed_trail_puffs {
        pg_engine::logic::dealloc(world, trail_puff);
    }

    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        crate::pygmy::stop_underwater(world, pygmy);
    }
}

/// `deactivateLastIslandLocationState` leaving Apocalypse island.
fn deactivate_apocalypse_island(world: &mut World, this: Entity) {
    let apocalypse_island_state = world.obj::<ControllerIslandFeatures>(this).apocalypse_island_state;
    if apocalypse_island_state == 1 {
        let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
        if !water_wings_carried(world, this, LOCATION_APOCALYPSE_ISLAND, island_location_state) {
            exit_water_wings(world, this);
        }
    }

    let gem_pedestals = world.obj::<ControllerWorldRegistry>(this).apocalypse_gem_pedestal_logic_array.clone();
    for gem_pedestal in gem_pedestals {
        crate::send::exit_island_with_arg(world, Some(gem_pedestal), true);
    }

    let calendar = world.obj::<ControllerWorldRegistry>(this).apocalypse_calendar_logic;
    if let Some(calendar) = calendar {
        apocalypse_calendar_logic::exit_island(world, calendar, true);
    }
    let temple_door = world.obj::<ControllerWorldRegistry>(this).apocalypse_temple_door_logic;
    if let Some(temple_door) = temple_door {
        apocalypse_temple_door_logic::exit_island(world, temple_door, true);
    }
    let portal_temple_door = world.obj::<ControllerWorldRegistry>(this).apocalypse_portal_temple_door_logic;
    if let Some(portal_temple_door) = portal_temple_door {
        apocalypse_temple_door_logic::exit_island(world, portal_temple_door, true);
    }

    let apocalypse_island_display_group = world.obj::<ControllerIslandLocations>(this).apocalypse_island_display_group;
    remove_display_group(world, apocalypse_island_display_group);
}

/// `deactivateLastIslandLocationState` leaving the Time temple.
fn deactivate_time_temple(world: &mut World, this: Entity) {
    stop_hurricane_in_temple(world, this);

    let doors = world.obj::<ControllerWorldRegistry>(this).time_temple_door_logic_array.clone();
    for door in doors {
        crate::send::exit_island_with_arg(world, Some(door), true);
    }

    let lever = world.obj::<ControllerWorldRegistry>(this).apocalypse_time_temple_lever_logic;
    if let Some(lever) = lever {
        apocalypse_time_temple_lever_logic::exit_island(world, lever, true);
    }
    let platform = world.obj::<ControllerWorldRegistry>(this).apocalypse_time_temple_platform_logic;
    if let Some(platform) = platform {
        apocalypse_time_temple_platform_logic::exit_island(world, platform, true);
    }

    hide_apocalypse_temple_display_group(world, this);
}

/// `deactivateLastIslandLocationState` leaving the Locust temple.
fn deactivate_locust_temple(world: &mut World, this: Entity) {
    stop_hurricane_in_temple(world, this);

    let doors = world.obj::<ControllerWorldRegistry>(this).locust_temple_door_logic_array.clone();
    for door in doors {
        crate::send::exit_island_with_arg(world, Some(door), true);
    }

    let locust_queen = world.obj::<ControllerWorldRegistry>(this).locust_queen_logic;
    if let Some(locust_queen) = locust_queen {
        locust_queen_logic::exit_island(world, locust_queen, true);
    }

    hide_apocalypse_temple_display_group(world, this);
}

/// `deactivateLastIslandLocationState` leaving the Dance temple.
fn deactivate_dance_temple(world: &mut World, this: Entity) {
    stop_hurricane_in_temple(world, this);

    let doors = world.obj::<ControllerWorldRegistry>(this).dance_temple_door_logic_array.clone();
    for door in doors {
        crate::send::exit_island_with_arg(world, Some(door), true);
    }

    let go_go_cage = world.obj::<ControllerWorldRegistry>(this).go_go_cage_logic;
    if let Some(go_go_cage) = go_go_cage {
        go_go_cage_logic::exit_island(world, go_go_cage, true);
    }
    let disco_ball = world.obj::<ControllerWorldRegistry>(this).disco_ball_logic;
    if let Some(disco_ball) = disco_ball {
        disco_ball_logic::exit_island(world, disco_ball, true);
    }
    let disco_lights = world.obj::<ControllerWorldRegistry>(this).disco_lights_logic;
    if let Some(disco_lights) = disco_lights {
        disco_lights_logic::exit_island(world, disco_lights, true);
    }
    let balloon_bundle = world.obj::<ControllerWorldRegistry>(this).dance_balloon_bundle_logic;
    if let Some(balloon_bundle) = balloon_bundle {
        dance_balloon_bundle_logic::exit_island(world, balloon_bundle, true);
    }
    let dance_balloon = world.obj::<ControllerWorldRegistry>(this).dance_balloon_logic;
    if let Some(dance_balloon) = dance_balloon {
        dance_balloon_logic::exit_island(world, dance_balloon);
    }
    let dance_dance_execution = world.obj::<ControllerWorldRegistry>(this).dance_dance_execution_logic;
    if let Some(dance_dance_execution) = dance_dance_execution {
        dance_dance_execution_logic::exit_island(world, dance_dance_execution, true);
    }

    hide_apocalypse_temple_display_group(world, this);
}

/// `deactivateLastIslandLocationState` leaving the Plague temple.
fn deactivate_plague_temple(world: &mut World, this: Entity) {
    stop_hurricane_in_temple(world, this);

    let doors = world.obj::<ControllerWorldRegistry>(this).plague_temple_door_logic_array.clone();
    for door in doors {
        crate::send::exit_island_with_arg(world, Some(door), true);
    }

    let fountain = world.obj::<ControllerWorldRegistry>(this).apocalypse_plague_temple_fountain_logic;
    if let Some(fountain) = fountain {
        apocalypse_temple_fountain_logic::exit_island(world, fountain, true);
    }

    hide_apocalypse_temple_display_group(world, this);
}

/// `deactivateLastIslandLocationState` leaving the Purification temple.
fn deactivate_purification_temple(world: &mut World, this: Entity) {
    stop_hurricane_in_temple(world, this);

    let doors = world.obj::<ControllerWorldRegistry>(this).purification_temple_door_logic_array.clone();
    for door in doors {
        crate::send::exit_island_with_arg(world, Some(door), true);
    }

    let fountain = world.obj::<ControllerWorldRegistry>(this).apocalypse_purification_temple_fountain_logic;
    if let Some(fountain) = fountain {
        apocalypse_temple_fountain_logic::exit_island(world, fountain, true);
    }

    hide_apocalypse_temple_display_group(world, this);
}

/// `deactivateLastIslandLocationState` leaving the Closet temple.
fn deactivate_closet_temple(world: &mut World, this: Entity) {
    stop_hurricane_in_temple(world, this);

    let doors = world.obj::<ControllerWorldRegistry>(this).closet_temple_door_logic_array.clone();
    for door in doors {
        crate::send::exit_island_with_arg(world, Some(door), true);
    }

    let thetan_meter = world.obj::<ControllerWorldRegistry>(this).closet_temple_thetan_meter_logic;
    if let Some(thetan_meter) = thetan_meter {
        closet_temple_thetan_meter_logic::exit_island(world, thetan_meter, true);
    }
    let couch = world.obj::<ControllerWorldRegistry>(this).closet_temple_couch_logic;
    if let Some(couch) = couch {
        closet_temple_couch_logic::exit_island(world, couch, true);
    }
    let closet_door = world.obj::<ControllerWorldRegistry>(this).closet_temple_closet_door_logic;
    if let Some(closet_door) = closet_door {
        closet_temple_closet_door_logic::exit_island(world, closet_door, true);
    }

    hide_apocalypse_temple_display_group(world, this);
}

/// `deactivateLastIslandLocationState` leaving the Xenu temple.
fn deactivate_xenu_temple(world: &mut World, this: Entity) {
    stop_hurricane_in_temple(world, this);

    let doors = world.obj::<ControllerWorldRegistry>(this).xenu_temple_door_logic_array.clone();
    for door in doors {
        crate::send::exit_island_with_arg(world, Some(door), true);
    }

    let xenu_temple = world.obj::<ControllerWorldRegistry>(this).xenu_temple_logic;
    if let Some(xenu_temple) = xenu_temple {
        xenu_temple_logic::exit_island(world, xenu_temple, true);
    }
    let xenu = world.obj::<ControllerWorldRegistry>(this).xenu_logic;
    if let Some(xenu) = xenu {
        xenu_logic::exit_island(world, xenu, true);
    }

    hide_apocalypse_temple_display_group(world, this);
}

// ---------------------------------------------------------------------------------------------
// Shared pieces.

/// Locations 2 and 9, reached through the tsunami.
fn is_underwater_location(location: i32) -> bool {
    location == LOCATION_UNDERWATER || location == LOCATION_UNDERWATER_CORAL_REEF
}

/// Whether the water wings stay in the world on a trip between `location` and
/// `other_location`: the other location's toggle shows them too (Sand 4, Graveyard 1, Ice 2,
/// Misfit 2, God 1, Apocalypse 1), so they are neither exited nor re-entered. Each location's
/// own entry is absent from its switch in the original, hence the first test.
fn water_wings_carried(world: &World, this: Entity, location: i32, other_location: i32) -> bool {
    if other_location == location {
        return false;
    }
    let features = world.obj::<ControllerIslandFeatures>(this);
    match other_location {
        LOCATION_SAND_ISLAND => features.sand_island_state == 4,
        LOCATION_GRAVEYARD_ISLAND => features.graveyard_island_state == 1,
        LOCATION_ICE_ISLAND => features.ice_island_state == 2,
        LOCATION_MISFIT_ISLAND => features.misfit_island_state == 2,
        LOCATION_GOD_ISLAND => features.god_island_state == 1,
        LOCATION_APOCALYPSE_ISLAND => features.apocalypse_island_state == 1,
        _ => false,
    }
}

/// Like [`water_wings_carried`] for the fishing pole (Sand 2, Rock 1, Ice 1).
fn fishing_carried(world: &World, this: Entity, location: i32, other_location: i32) -> bool {
    if other_location == location {
        return false;
    }
    let features = world.obj::<ControllerIslandFeatures>(this);
    match other_location {
        LOCATION_SAND_ISLAND => features.sand_island_state == 2,
        LOCATION_ROCK_ISLAND => features.rock_island_state == 1,
        LOCATION_ICE_ISLAND => features.ice_island_state == 1,
        _ => false,
    }
}

/// `[mWaterWingsLogic enterIsland:startup]`.
fn enter_water_wings(world: &mut World, this: Entity, startup: bool) {
    let water_wings = world.obj::<ControllerWorldRegistry>(this).water_wings_logic;
    if let Some(water_wings) = water_wings {
        water_wings_logic::enter_island(world, water_wings, startup);
    }
}

/// `[mWaterWingsLogic exitIslandNow]`.
fn exit_water_wings(world: &mut World, this: Entity) {
    let water_wings = world.obj::<ControllerWorldRegistry>(this).water_wings_logic;
    if let Some(water_wings) = water_wings {
        water_wings_logic::exit_island_now(world, water_wings);
    }
}

/// `[mFishingPoleLogic enterIsland:startup]`, then `[mFishLogic enterIsland:startup]`.
fn enter_fishing_pole_and_fish(world: &mut World, this: Entity, startup: bool) {
    let fishing_pole = world.obj::<ControllerWorldRegistry>(this).fishing_pole_logic;
    if let Some(fishing_pole) = fishing_pole {
        fishing_pole_logic::enter_island(world, fishing_pole, startup);
    }
    let fish = world.obj::<ControllerWorldRegistry>(this).fish_logic;
    if let Some(fish) = fish {
        fish_logic::enter_island(world, fish, startup);
    }
}

/// `[mFishingPoleLogic exitIslandNow]` (the fish is left alone).
fn exit_fishing_pole(world: &mut World, this: Entity) {
    let fishing_pole = world.obj::<ControllerWorldRegistry>(this).fishing_pole_logic;
    if let Some(fishing_pole) = fishing_pole {
        fishing_pole_logic::exit_island_now(world, fishing_pole);
    }
}

/// Every spear of `mSpearLogicArray` `enterIsland:startup`.
fn enter_spears(world: &mut World, this: Entity, startup: bool) {
    let spears = world.obj::<ControllerWorldRegistry>(this).spear_logic_array.clone();
    for spear in spears {
        crate::send::enter_island_with_arg(world, Some(spear), startup);
    }
}

/// Every spear of `mSpearLogicArray` `exitIslandNow`.
fn exit_spears(world: &mut World, this: Entity) {
    let spears = world.obj::<ControllerWorldRegistry>(this).spear_logic_array.clone();
    for spear in spears {
        crate::send::exit_island_now(world, Some(spear));
    }
}

/// The walk bounds and the horizon of the location being activated.
fn set_island_bounds(world: &mut World, this: Entity, min_x_pos: f32, max_x_pos: f32, center_x_pos: f32, horizon_y_pos: f32) {
    let mut locations = world.obj_mut::<ControllerIslandLocations>(this);
    locations.island_min_x_pos = min_x_pos;
    locations.island_max_x_pos = max_x_pos;
    locations.island_center_x_pos = center_x_pos;
    locations.horizon_y_pos = horizon_y_pos;
}

/// `[mMiscSequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:sequence_id]]`: the
/// location's start script.
fn set_misc_sequence_def(world: &mut World, this: Entity, sequence_id: &str) {
    let misc_sequence = world.obj::<ControllerIslandLocations>(this).misc_sequence;
    let sequence_def = pg_engine::library::sequence_def(world, sequence_id);
    if let Some(misc_sequence) = misc_sequence {
        pg_engine::sequence::set_sequence_def(world, misc_sequence, sequence_def);
    }
}

/// `[[mIslandDisplayGroup getDisplayObject:@"Gradient"] pos].y = y_pos`: the sky gradient.
fn set_gradient_y_pos(world: &mut World, this: Entity, y_pos: f32) {
    let island_display_group = world.obj::<ControllerIslandLocations>(this).island_display_group;
    let Some(island_display_group) = island_display_group else {
        return;
    };
    let gradient = pg_engine::display::group::get_display_object(world, island_display_group, "Gradient");
    if let Some(gradient) = gradient {
        pg_engine::display::object_mut(world, gradient).pos.y = y_pos;
    }
}

/// `[group moveToDisplayGroup:marker]` (nothing for a nil group).
fn move_display_group(world: &mut World, group: Option<Entity>, marker: Option<Entity>) {
    if let Some(group) = group {
        pg_engine::display::move_to_display_group(world, group, marker);
    }
}

/// `[group removeFromDisplayGroup]` (nothing for a nil group).
fn remove_display_group(world: &mut World, group: Option<Entity>) {
    if let Some(group) = group {
        pg_engine::display::remove_from_display_group(world, group);
    }
}

/// `[mHurricaneLogic stopHurricane]`, the first thing every temple does on the way out.
fn stop_hurricane_in_temple(world: &mut World, this: Entity) {
    let hurricane = world.obj::<ControllerGodPowers>(this).hurricane_logic;
    if let Some(hurricane) = hurricane {
        hurricane_logic::stop_hurricane(world, hurricane);
    }
}

/// Leaving the underwater realm for the surface stops the whirlpool at once and ends a stroke in
/// progress together with its finger trail.
fn end_whirl_pool(world: &mut World, this: Entity) {
    let whirl_pool = world.obj::<ControllerGodPowers>(this).whirl_pool_logic;
    let Some(whirl_pool) = whirl_pool else {
        return;
    };
    whirl_pool_logic::stop_whirl_pool_now(world, whirl_pool);

    let whirl_pool_touch = world.obj::<ControllerGodPowers>(this).whirl_pool_touch;
    if whirl_pool_touch.is_none() {
        return;
    }

    let whirl_pool = world.obj::<ControllerGodPowers>(this).whirl_pool_logic;
    if let Some(whirl_pool) = whirl_pool {
        whirl_pool_logic::set_touch_active(world, whirl_pool, false);
    }
    world.obj_mut::<ControllerGodPowers>(this).whirl_pool_touch = None;

    let whirl_pool_trail = world.obj::<ControllerGodPowers>(this).whirl_pool_trail_logic;
    discard_finger_trail(world, whirl_pool_trail);
    world.obj_mut::<ControllerGodPowers>(this).whirl_pool_trail_logic = None;
}

/// `[[trail displayObject] removeFromDisplayGroup]; [trail shutdown]; [trail release]` for a
/// finger trail the controller alone owns (the caller then clears its ivar).
fn discard_finger_trail(world: &mut World, trail: Option<Entity>) {
    let Some(trail) = trail else {
        return;
    };
    let trail_display_object = pg_engine::logic::display_object(world, trail);
    pg_engine::display::remove_from_display_group(world, trail_display_object);
    pg_engine::logic::shutdown(world, trail);
    pg_engine::logic::dealloc(world, trail);
}

/// `FUN_100038d68(file, @"raw", screenAtlas0)`: upload a location or minigame atlas.
fn load_screen_atlas(world: &mut World, file: &str) {
    let screen_atlas = world.resource::<Library>().atlas_id(SCREEN_ATLAS_ID);
    let Some(screen_atlas) = screen_atlas else {
        panic!("loadLocationTextureAtlas: there is no atlas slot {SCREEN_ATLAS_ID}");
    };
    pg_engine::texture::load_atlas(world, screen_atlas, file);
}

/// `[[SkinPackManager instance] loadTextureAtlasChannel:channel]`.
fn load_skin_texture_atlas_channel(world: &mut World, channel: i32) {
    let skin_pack_manager = skin_pack_manager::instance(world);
    if let Some(skin_pack_manager) = skin_pack_manager {
        skin_pack_manager::load_texture_atlas_channel(world, skin_pack_manager, channel);
    }
}

/// `[mSkinPackManager refreshSkinPackOption:skin_pack_id skinObjectId:skin_object_id
/// skinOptionId:skin_option_id useFirstPurchased:YES]`.
fn refresh_skin_pack_option(world: &mut World, this: Entity, skin_pack_id: &str, skin_object_id: &str, skin_option_id: &str) {
    let skin_pack_manager = world.obj::<ControllerManagersBridge>(this).skin_pack_manager;
    let Some(skin_pack_manager) = skin_pack_manager else {
        return;
    };

    skin_pack_manager::refresh_skin_pack_option_skin_object_id_skin_option_id_use_first_purchased(
        world,
        skin_pack_manager,
        skin_pack_id,
        skin_object_id,
        skin_option_id,
        true,
    );
}
