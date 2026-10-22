//! `PocketGodViewController`: the per-island feature toggles. Each `set…State:startup:` turns a
//! feature (or one of several mutually exclusive features) of an island on or off, stores it in
//! the `Options` document and animates its menu button; `updateIslandStateAtStartup` restores
//! them; the `is…Enabled` predicates tell the logic objects whether they belong on the current
//! location; `disableIslandLogics`/`enableIslandLogics` clear and restore the island around a
//! story.

use crate::prelude::*;
use super::*;

use pg_engine::{logic, prefs};

use crate::logic::ape_monkeys::{banana_logic, banana_tree_logic};
use crate::logic::ape_ritual::{ape_altar_logic, ape_bongo_logic, ape_logic, ape_wench_logic};
use crate::logic::apocalypse_dance::dance_balloon_logic;
use crate::logic::apocalypse_locust_closet_xenu::xenu_logic;
use crate::logic::apocalypse_temples::apocalypse_calendar_logic;
use crate::logic::ice_island::{ice_hole_logic, ice_monster_logic, igloo_logic};
use crate::logic::island_anthill::{ant_hill_logic, magnifying_glass_logic};
use crate::logic::island_core::bon_fire_logic;
use crate::logic::island_objects::{boulder_logic, out_house_logic};
use crate::logic::konkeydong::kong_statue_logic;
use crate::logic::monsters::{spider_logic, spider_tree_logic, t_rex_egg_logic, t_rex_logic, zombie_grave_logic};
use crate::logic::moronpests_chase::moron_snake_logic;
use crate::logic::nopee::water_wings_logic;
use crate::logic::ocean_birds::bird_logic;
use crate::logic::ocean_creatures::shark_logic;
use crate::logic::ocean_fishing::{fish_logic, fishing_pole_logic};
use crate::logic::underwater_baitmaster_laser::{anchor_logic, chum_bucket_logic, laser_shark_logic};
use crate::logic::underwater_paindrain::{tile_drain_logic, tile_plug_logic};
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
const LOCATION_UNDERWATER_CORAL_REEF: i32 = 9;
const LOCATION_APOCALYPSE_ISLAND: i32 = 10;
const LOCATION_TIME_TEMPLE: i32 = 11;
const LOCATION_LOCUST_TEMPLE: i32 = 12;
const LOCATION_DANCE_TEMPLE: i32 = 13;
const LOCATION_PLAGUE_TEMPLE: i32 = 14;
const LOCATION_PURIFICATION_TEMPLE: i32 = 15;
const LOCATION_CLOSET_TEMPLE: i32 = 16;
const LOCATION_XENU_TEMPLE: i32 = 17;

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerIslandFeatures {
    /// `mSharkState`: `i`
    pub shark_state: i32,

    /// `mBirdState`: `i`
    pub bird_state: i32,

    /// `mSandBoulderButtonLogic`: `@"BCButtonLogic"`
    pub sand_boulder_button_logic: Option<Entity>,

    /// `mSandFishButtonLogic`: `@"BCButtonLogic"`
    pub sand_fish_button_logic: Option<Entity>,

    /// `mSandAntHillButtonLogic`: `@"BCButtonLogic"`
    pub sand_ant_hill_button_logic: Option<Entity>,

    /// `mSandWaterWingsButtonLogic`: `@"BCButtonLogic"`
    pub sand_water_wings_button_logic: Option<Entity>,

    /// `mRockFishButtonLogic`: `@"BCButtonLogic"`
    pub rock_fish_button_logic: Option<Entity>,

    /// `mRockWoodButtonLogic`: `@"BCButtonLogic"`
    pub rock_wood_button_logic: Option<Entity>,

    /// `mRockEggButtonLogic`: `@"BCButtonLogic"`
    pub rock_egg_button_logic: Option<Entity>,

    /// `mRockOutHouseButtonLogic`: `@"BCButtonLogic"`
    pub rock_out_house_button_logic: Option<Entity>,

    /// `mRockTRexButtonLogic`: `@"BCButtonLogic"`
    pub rock_t_rex_button_logic: Option<Entity>,

    /// `mRockSpearButtonLogic`: `@"BCButtonLogic"`
    pub rock_spear_button_logic: Option<Entity>,

    /// `mGraveyardZombieGraveButtonLogic`: `@"BCButtonLogic"`
    pub graveyard_zombie_grave_button_logic: Option<Entity>,

    /// `mGraveyardGhostButtonLogic`: `@"BCButtonLogic"`
    pub graveyard_ghost_button_logic: Option<Entity>,

    /// `mGraveyardSpiderWebButtonLogic`: `@"BCButtonLogic"`
    pub graveyard_spider_web_button_logic: Option<Entity>,

    /// `mGraveyardSpiderButtonLogic`: `@"BCButtonLogic"`
    pub graveyard_spider_button_logic: Option<Entity>,

    /// `mGraveyardSpearButtonLogic`: `@"BCButtonLogic"`
    pub graveyard_spear_button_logic: Option<Entity>,

    /// `mGraveyardWaterWingsButtonLogic`: `@"BCButtonLogic"`
    pub graveyard_water_wings_button_logic: Option<Entity>,

    /// `mIceFishButtonLogic`: `@"BCButtonLogic"`
    pub ice_fish_button_logic: Option<Entity>,

    /// `mIceWaterWingsButtonLogic`: `@"BCButtonLogic"`
    pub ice_water_wings_button_logic: Option<Entity>,

    /// `mIceIceMonsterButtonLogic`: `@"BCButtonLogic"`
    pub ice_ice_monster_button_logic: Option<Entity>,

    /// `mIceIglooButtonLogic`: `@"BCButtonLogic"`
    pub ice_igloo_button_logic: Option<Entity>,

    /// `mIceIceHoleButtonLogic`: `@"BCButtonLogic"`
    pub ice_ice_hole_button_logic: Option<Entity>,

    /// `mIceIceSquidButtonLogic`: `@"BCButtonLogic"`
    pub ice_ice_squid_button_logic: Option<Entity>,

    /// `mApeApeAltarButtonLogic`: `@"BCButtonLogic"`
    pub ape_ape_altar_button_logic: Option<Entity>,

    /// `mApeApeButtonLogic`: `@"BCButtonLogic"`
    pub ape_ape_button_logic: Option<Entity>,

    /// `mApeBananaTreeButtonLogic`: `@"BCButtonLogic"`
    pub ape_banana_tree_button_logic: Option<Entity>,

    /// `mApeMonkeyButtonLogic`: `@"BCButtonLogic"`
    pub ape_monkey_button_logic: Option<Entity>,

    /// `mApeKonkeyDongButtonLogic`: `@"BCButtonLogic"`
    pub ape_konkey_dong_button_logic: Option<Entity>,

    /// `mMisfitMoronSnakeButtonLogic`: `@"BCButtonLogic"`
    pub misfit_moron_snake_button_logic: Option<Entity>,

    /// `mMisfitWaterWingsButtonLogic`: `@"BCButtonLogic"`
    pub misfit_water_wings_button_logic: Option<Entity>,

    /// `mMisfitMoronPestsButtonLogic`: `@"BCButtonLogic"`
    pub misfit_moron_pests_button_logic: Option<Entity>,

    /// `mIdolsButtonLogic`: `@"BCButtonLogic"`
    pub idols_button_logic: Option<Entity>,

    /// `mIdolWaterWingsButtonLogic`: `@"BCButtonLogic"`
    pub idol_water_wings_button_logic: Option<Entity>,

    /// `mApocalypseWaterWingsButtonLogic`: `@"BCButtonLogic"`
    pub apocalypse_water_wings_button_logic: Option<Entity>,

    /// `mUnderwaterPiranhaButtonLogic`: `@"BCButtonLogic"`
    pub underwater_piranha_button_logic: Option<Entity>,

    /// `mUnderwaterLaserSharkButtonLogic`: `@"BCButtonLogic"`
    pub underwater_laser_shark_button_logic: Option<Entity>,

    /// `mUnderwaterTileDrainButtonLogic`: `@"BCButtonLogic"`
    pub underwater_tile_drain_button_logic: Option<Entity>,

    /// `mUnderwaterCoralReefJellyfishButtonLogic`: `@"BCButtonLogic"`
    pub underwater_coral_reef_jellyfish_button_logic: Option<Entity>,

    /// `mSharkButtonLogic`: `@"BCButtonLogic"`
    pub shark_button_logic: Option<Entity>,

    /// `mBirdButtonLogic`: `@"BCButtonLogic"`
    pub bird_button_logic: Option<Entity>,

    /// `mSandIslandState`: `i`
    pub sand_island_state: i32,

    /// `mRockIslandState`: `i`
    pub rock_island_state: i32,

    /// `mRockIslandFeatureState`: `i`
    pub rock_island_feature_state: i32,

    /// `mRockIslandOutHouseState`: `i`
    pub rock_island_out_house_state: i32,

    /// `mRockIslandTRexState`: `i`
    pub rock_island_t_rex_state: i32,

    /// `mRockIslandSpearState`: `i`
    pub rock_island_spear_state: i32,

    /// `mGraveyardIslandState`: `i`
    pub graveyard_island_state: i32,

    /// `mGraveyardIslandZombieGraveState`: `i`
    pub graveyard_island_zombie_grave_state: i32,

    /// `mGraveyardIslandGhostState`: `i`
    pub graveyard_island_ghost_state: i32,

    /// `mGraveyardIslandSpiderWebState`: `i`
    pub graveyard_island_spider_web_state: i32,

    /// `mGraveyardIslandSpiderState`: `i`
    pub graveyard_island_spider_state: i32,

    /// `mGraveyardIslandSpearState`: `i`
    pub graveyard_island_spear_state: i32,

    /// `mIceIslandState`: `i`
    pub ice_island_state: i32,

    /// `mIceIslandIceMonsterState`: `i`
    pub ice_island_ice_monster_state: i32,

    /// `mIceIslandIglooState`: `i`
    pub ice_island_igloo_state: i32,

    /// `mIceIslandIceHoleState`: `i`
    pub ice_island_ice_hole_state: i32,

    /// `mIceIslandIceSquidState`: `i`
    pub ice_island_ice_squid_state: i32,

    /// `mApeIslandState`: `i`
    pub ape_island_state: i32,

    /// `mApeIslandApeState`: `i`
    pub ape_island_ape_state: i32,

    /// `mApeIslandMonkeyState`: `i`
    pub ape_island_monkey_state: i32,

    /// `mMisfitIslandState`: `i`
    pub misfit_island_state: i32,

    /// `mGodIslandState`: `i`
    pub god_island_state: i32,

    /// `mGodIslandIdolsState`: `i`
    pub god_island_idols_state: i32,

    /// `mApocalypseIslandState`: `i`
    pub apocalypse_island_state: i32,

    /// `mFailBoatState`: `i`
    pub fail_boat_state: i32,

    /// `mUnderwaterState`: `i`
    pub underwater_state: i32,

    /// `mUnderwaterCoralReefState`: `i`
    pub underwater_coral_reef_state: i32,
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

/// `[button setBehavior:behavior]` on a button logic that may be nil.
fn set_button_behavior(world: &mut World, button: Option<Entity>, behavior: &str) {
    if let Some(button) = button {
        logic::set_behavior(world, button, Some(behavior));
    }
}

/// `-[PocketGodViewController setSharkState:startup:]` @ 0x10007d894
pub fn set_shark_state(world: &mut World, this: Entity, shark_state: i32, startup: bool) {
    let current_state = world.obj::<ControllerIslandFeatures>(this).shark_state;
    let new_state = if current_state == shark_state { 0 } else { shark_state };
    world.obj_mut::<ControllerIslandFeatures>(this).shark_state = new_state;
    set_option_int_preference(world, this, "SharkState", new_state);

    if new_state == 1 {
        let behavior = if startup { "StartSharkOn" } else { "ToggleSharkOn" };
        let shark_button = world.obj::<ControllerIslandFeatures>(this).shark_button_logic;
        set_button_behavior(world, shark_button, behavior);

        let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
        if island_location_state != LOCATION_APE_ISLAND {
            let shark = world.obj::<ControllerWorldRegistry>(this).shark_logic;
            if let Some(shark) = shark {
                shark_logic::enter_island(world, shark, startup);
            }
        }
    } else if new_state == 0 {
        if !startup {
            let shark_button = world.obj::<ControllerIslandFeatures>(this).shark_button_logic;
            set_button_behavior(world, shark_button, "ToggleSharkOff");
        }

        let shark = world.obj::<ControllerWorldRegistry>(this).shark_logic;
        if let Some(shark) = shark {
            shark_logic::exit_island(world, shark, false);
        }
    }
}

/// `-[PocketGodViewController setBirdState:startup:]` @ 0x10007d9c4
pub fn set_bird_state(world: &mut World, this: Entity, bird_state: i32, startup: bool) {
    let current_state = world.obj::<ControllerIslandFeatures>(this).bird_state;
    let new_state = if current_state == bird_state { 0 } else { bird_state };
    world.obj_mut::<ControllerIslandFeatures>(this).bird_state = new_state;
    set_option_int_preference(world, this, "BirdState", new_state);

    if new_state == 1 {
        let behavior = if startup { "StartBirdOn" } else { "ToggleBirdOn" };
        let bird_button = world.obj::<ControllerIslandFeatures>(this).bird_button_logic;
        set_button_behavior(world, bird_button, behavior);

        let bird = world.obj::<ControllerWorldRegistry>(this).bird_logic;
        if let Some(bird) = bird {
            bird_logic::enter_island(world, bird, startup);
        }
    } else if new_state == 0 {
        if !startup {
            let bird_button = world.obj::<ControllerIslandFeatures>(this).bird_button_logic;
            set_button_behavior(world, bird_button, "ToggleBirdOff");
        }

        let bird = world.obj::<ControllerWorldRegistry>(this).bird_logic;
        if let Some(bird) = bird {
            bird_logic::exit_island(world, bird, false);
        }
    }
}

/// `-[PocketGodViewController updateIslandStateAtStartup]` @ 0x10007e398
pub fn update_island_state_at_startup(world: &mut World, this: Entity) {
    let sand_island_state = option_int_preference(world, this, "SandIslandHazard");
    world.obj_mut::<ControllerIslandFeatures>(this).sand_island_state = sand_island_state;
    let features = world.obj::<ControllerIslandFeatures>(this);
    let sand_start = match sand_island_state {
        1 => Some((features.sand_boulder_button_logic, "StartBoulderOn")),
        2 => Some((features.sand_fish_button_logic, "StartFishOn")),
        3 => Some((features.sand_ant_hill_button_logic, "StartAntHillOn")),
        4 => Some((features.sand_water_wings_button_logic, "StartWaterWingsOn")),
        _ => None,
    };
    if let Some((button, behavior)) = sand_start {
        set_button_behavior(world, button, behavior);
    }

    let rock_island_state = option_int_preference(world, this, "RockIslandHazard");
    world.obj_mut::<ControllerIslandFeatures>(this).rock_island_state = rock_island_state;
    if rock_island_state == 1 {
        let rock_fish_button = world.obj::<ControllerIslandFeatures>(this).rock_fish_button_logic;
        set_button_behavior(world, rock_fish_button, "StartFishOn");
    }

    let rock_island_feature_state = option_int_preference(world, this, "RockIslandFeature");
    world.obj_mut::<ControllerIslandFeatures>(this).rock_island_feature_state = rock_island_feature_state;
    let features = world.obj::<ControllerIslandFeatures>(this);
    let rock_feature_start = match rock_island_feature_state {
        2 => Some((features.rock_egg_button_logic, "StartEggOn")),
        1 => Some((features.rock_wood_button_logic, "StartWoodOn")),
        _ => None,
    };
    if let Some((button, behavior)) = rock_feature_start {
        set_button_behavior(world, button, behavior);
    }

    let rock_island_out_house_state = option_int_preference(world, this, "RockIslandOutHouseState");
    world.obj_mut::<ControllerIslandFeatures>(this).rock_island_out_house_state = rock_island_out_house_state;
    if rock_island_out_house_state == 1 {
        let out_house_button = world.obj::<ControllerIslandFeatures>(this).rock_out_house_button_logic;
        set_button_behavior(world, out_house_button, "StartOutHouseOn");
    }

    let rock_island_t_rex_state = option_int_preference(world, this, "RockIslandTRexState");
    world.obj_mut::<ControllerIslandFeatures>(this).rock_island_t_rex_state = rock_island_t_rex_state;
    if rock_island_t_rex_state == 1 {
        let t_rex_button = world.obj::<ControllerIslandFeatures>(this).rock_t_rex_button_logic;
        set_button_behavior(world, t_rex_button, "StartTRexOn");
    }

    let rock_island_spear_state = option_int_preference(world, this, "RockIslandSpearState");
    world.obj_mut::<ControllerIslandFeatures>(this).rock_island_spear_state = rock_island_spear_state;
    if rock_island_spear_state == 1 {
        let rock_spear_button = world.obj::<ControllerIslandFeatures>(this).rock_spear_button_logic;
        set_button_behavior(world, rock_spear_button, "StartSpearOn");
    }

    let graveyard_island_state = option_int_preference(world, this, "GraveyardIslandState");
    world.obj_mut::<ControllerIslandFeatures>(this).graveyard_island_state = graveyard_island_state;
    if graveyard_island_state == 1 {
        let graveyard_water_wings_button = world.obj::<ControllerIslandFeatures>(this).graveyard_water_wings_button_logic;
        set_button_behavior(world, graveyard_water_wings_button, "StartWaterWingsOn");
    }

    let zombie_grave_state = option_int_preference(world, this, "GraveyardIslandZombieGraveState");
    world.obj_mut::<ControllerIslandFeatures>(this).graveyard_island_zombie_grave_state = zombie_grave_state;
    if zombie_grave_state == 1 {
        let zombie_grave_button = world.obj::<ControllerIslandFeatures>(this).graveyard_zombie_grave_button_logic;
        set_button_behavior(world, zombie_grave_button, "StartZombieGraveOn");
    }

    let ghost_state = option_int_preference(world, this, "GraveyardIslandGhostState");
    world.obj_mut::<ControllerIslandFeatures>(this).graveyard_island_ghost_state = ghost_state;
    if ghost_state == 1 {
        let ghost_button = world.obj::<ControllerIslandFeatures>(this).graveyard_ghost_button_logic;
        set_button_behavior(world, ghost_button, "StartGhostOn");
    }

    let spider_web_state = option_int_preference(world, this, "GraveyardIslandSpiderWebState");
    world.obj_mut::<ControllerIslandFeatures>(this).graveyard_island_spider_web_state = spider_web_state;
    if spider_web_state == 1 {
        let spider_web_button = world.obj::<ControllerIslandFeatures>(this).graveyard_spider_web_button_logic;
        set_button_behavior(world, spider_web_button, "StartSpiderWebOn");
    }

    let spider_state = option_int_preference(world, this, "GraveyardIslandSpiderState");
    world.obj_mut::<ControllerIslandFeatures>(this).graveyard_island_spider_state = spider_state;
    if spider_state == 1 {
        let spider_button = world.obj::<ControllerIslandFeatures>(this).graveyard_spider_button_logic;
        set_button_behavior(world, spider_button, "StartSpiderOn");
    }

    let graveyard_spear_state = option_int_preference(world, this, "GraveyardIslandSpearState");
    world.obj_mut::<ControllerIslandFeatures>(this).graveyard_island_spear_state = graveyard_spear_state;
    if graveyard_spear_state == 1 {
        let graveyard_spear_button = world.obj::<ControllerIslandFeatures>(this).graveyard_spear_button_logic;
        set_button_behavior(world, graveyard_spear_button, "StartSpearOn");
    }

    let ice_island_state = option_int_preference(world, this, "IceIslandState");
    world.obj_mut::<ControllerIslandFeatures>(this).ice_island_state = ice_island_state;
    let features = world.obj::<ControllerIslandFeatures>(this);
    let ice_start = match ice_island_state {
        2 => Some((features.ice_water_wings_button_logic, "StartWaterWingsOn")),
        1 => Some((features.ice_fish_button_logic, "StartFishOn")),
        _ => None,
    };
    if let Some((button, behavior)) = ice_start {
        set_button_behavior(world, button, behavior);
    }

    let ice_monster_state = option_int_preference(world, this, "IceIslandIceMonsterState");
    world.obj_mut::<ControllerIslandFeatures>(this).ice_island_ice_monster_state = ice_monster_state;
    if ice_monster_state == 1 {
        let ice_monster_button = world.obj::<ControllerIslandFeatures>(this).ice_ice_monster_button_logic;
        set_button_behavior(world, ice_monster_button, "StartIceMonsterOn");
    }

    let igloo_state = option_int_preference(world, this, "IceIslandIglooState");
    world.obj_mut::<ControllerIslandFeatures>(this).ice_island_igloo_state = igloo_state;
    if igloo_state == 1 {
        let igloo_button = world.obj::<ControllerIslandFeatures>(this).ice_igloo_button_logic;
        set_button_behavior(world, igloo_button, "StartIglooOn");
    }

    let ice_hole_state = option_int_preference(world, this, "IceIslandIceHoleState");
    world.obj_mut::<ControllerIslandFeatures>(this).ice_island_ice_hole_state = ice_hole_state;
    if ice_hole_state == 1 {
        let ice_hole_button = world.obj::<ControllerIslandFeatures>(this).ice_ice_hole_button_logic;
        set_button_behavior(world, ice_hole_button, "StartIceHoleOn");
    }

    let ice_squid_state = option_int_preference(world, this, "IceIslandIceSquidState");
    world.obj_mut::<ControllerIslandFeatures>(this).ice_island_ice_squid_state = ice_squid_state;
    if ice_squid_state == 1 {
        let ice_squid_button = world.obj::<ControllerIslandFeatures>(this).ice_ice_squid_button_logic;
        set_button_behavior(world, ice_squid_button, "StartIceSquidOn");
    }

    let ape_island_state = option_int_preference(world, this, "ApeIslandState");
    world.obj_mut::<ControllerIslandFeatures>(this).ape_island_state = ape_island_state;
    let features = world.obj::<ControllerIslandFeatures>(this);
    let ape_start = match ape_island_state {
        2 => Some((features.ape_banana_tree_button_logic, "StartBananaTreeOn")),
        1 => Some((features.ape_ape_altar_button_logic, "StartApeAltarOn")),
        _ => None,
    };
    if let Some((button, behavior)) = ape_start {
        set_button_behavior(world, button, behavior);
    }

    let ape_state = option_int_preference(world, this, "ApeIslandApeState");
    world.obj_mut::<ControllerIslandFeatures>(this).ape_island_ape_state = ape_state;
    if ape_state == 1 {
        let ape_button = world.obj::<ControllerIslandFeatures>(this).ape_ape_button_logic;
        set_button_behavior(world, ape_button, "StartApeOn");
    }

    let monkey_state = option_int_preference(world, this, "ApeIslandMonkeyState");
    world.obj_mut::<ControllerIslandFeatures>(this).ape_island_monkey_state = monkey_state;
    if monkey_state == 1 {
        let monkey_button = world.obj::<ControllerIslandFeatures>(this).ape_monkey_button_logic;
        set_button_behavior(world, monkey_button, "StartMonkeyOn");
    }

    let misfit_island_state = option_int_preference(world, this, "MisfitIslandState");
    world.obj_mut::<ControllerIslandFeatures>(this).misfit_island_state = misfit_island_state;
    let features = world.obj::<ControllerIslandFeatures>(this);
    // Original bug, kept: the moron snake's start behaviour goes to the Moron Pests button, not
    // to `mMisfitMoronSnakeButtonLogic`.
    let misfit_start = match misfit_island_state {
        2 => Some((features.misfit_water_wings_button_logic, "StartWaterWingsOn")),
        1 => Some((features.misfit_moron_pests_button_logic, "StartMoronSnakeOn")),
        _ => None,
    };
    if let Some((button, behavior)) = misfit_start {
        set_button_behavior(world, button, behavior);
    }

    let god_island_state = option_int_preference(world, this, "GodIslandState");
    world.obj_mut::<ControllerIslandFeatures>(this).god_island_state = god_island_state;
    if god_island_state == 1 {
        let idol_water_wings_button = world.obj::<ControllerIslandFeatures>(this).idol_water_wings_button_logic;
        set_button_behavior(world, idol_water_wings_button, "StartWaterWingsOn");
    }

    let idols_state = option_int_preference(world, this, "GodIslandIdolsState");
    world.obj_mut::<ControllerIslandFeatures>(this).god_island_idols_state = idols_state;
    if idols_state == 1 {
        let idols_button = world.obj::<ControllerIslandFeatures>(this).idols_button_logic;
        set_button_behavior(world, idols_button, "StartIdolsOn");
    }

    let apocalypse_island_state = option_int_preference(world, this, "ApocalypseIslandState");
    world.obj_mut::<ControllerIslandFeatures>(this).apocalypse_island_state = apocalypse_island_state;
    if apocalypse_island_state == 1 {
        let apocalypse_water_wings_button = world.obj::<ControllerIslandFeatures>(this).apocalypse_water_wings_button_logic;
        set_button_behavior(world, apocalypse_water_wings_button, "StartWaterWingsOn");
    }

    let fail_boat_state = option_int_preference(world, this, "FailBoatState");
    world.obj_mut::<ControllerIslandFeatures>(this).fail_boat_state = fail_boat_state;

    let underwater_state = option_int_preference(world, this, "UnderwaterHazard");
    world.obj_mut::<ControllerIslandFeatures>(this).underwater_state = underwater_state;
    let features = world.obj::<ControllerIslandFeatures>(this);
    let underwater_start = match underwater_state {
        3 => Some((features.underwater_tile_drain_button_logic, "StartTileDrainOn")),
        2 => Some((features.underwater_laser_shark_button_logic, "StartLaserSharkOn")),
        1 => Some((features.underwater_piranha_button_logic, "StartPiranhaOn")),
        _ => None,
    };
    if let Some((button, behavior)) = underwater_start {
        set_button_behavior(world, button, behavior);
    }

    let coral_reef_state = option_int_preference(world, this, "UnderwaterCoralReefState");
    world.obj_mut::<ControllerIslandFeatures>(this).underwater_coral_reef_state = coral_reef_state;
    if coral_reef_state == 1 {
        let jellyfish_button = world.obj::<ControllerIslandFeatures>(this).underwater_coral_reef_jellyfish_button_logic;
        set_button_behavior(world, jellyfish_button, "StartJellyfishOn");
    }

    let island_location = option_int_preference(world, this, "IslandLocation");
    set_island_location_state(world, this, island_location, true);
}

/// `-[PocketGodViewController setSandIslandState:startup:]` @ 0x100083f50
pub fn set_sand_island_state(world: &mut World, this: Entity, sand_island_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).sand_island_state;
    match old_state {
        1 => {
            let boulder = world.obj::<ControllerWorldRegistry>(this).boulder_logic;
            if let Some(boulder) = boulder {
                boulder_logic::exit_island(world, boulder, false);
            }
            let boulder_button = world.obj::<ControllerIslandFeatures>(this).sand_boulder_button_logic;
            set_button_behavior(world, boulder_button, "ToggleBoulderOff");
        }
        2 => {
            let fishing_pole = world.obj::<ControllerWorldRegistry>(this).fishing_pole_logic;
            if let Some(fishing_pole) = fishing_pole {
                fishing_pole_logic::exit_island(world, fishing_pole, false);
            }
            let fish_button = world.obj::<ControllerIslandFeatures>(this).sand_fish_button_logic;
            set_button_behavior(world, fish_button, "ToggleFishOff");
        }
        3 => {
            let ant_hill = world.obj::<ControllerWorldRegistry>(this).ant_hill_logic;
            if let Some(ant_hill) = ant_hill {
                ant_hill_logic::exit_island(world, ant_hill);
            }
            let magnifying_glass = world.obj::<ControllerWorldRegistry>(this).magnifying_glass_logic;
            if let Some(magnifying_glass) = magnifying_glass {
                magnifying_glass_logic::exit_island(world, magnifying_glass, false);
            }
            let ant_hill_button = world.obj::<ControllerIslandFeatures>(this).sand_ant_hill_button_logic;
            set_button_behavior(world, ant_hill_button, "ToggleAntHillOff");
        }
        4 => {
            let water_wings = world.obj::<ControllerWorldRegistry>(this).water_wings_logic;
            if let Some(water_wings) = water_wings {
                water_wings_logic::exit_island(world, water_wings, false);
            }
            let water_wings_button = world.obj::<ControllerIslandFeatures>(this).sand_water_wings_button_logic;
            set_button_behavior(world, water_wings_button, "ToggleWaterWingsOff");
        }
        _ => {}
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).sand_island_state;
    let new_state = if current_state == sand_island_state { 0 } else { sand_island_state };
    world.obj_mut::<ControllerIslandFeatures>(this).sand_island_state = new_state;
    set_option_int_preference(world, this, "SandIslandHazard", new_state);

    match new_state {
        1 => {
            let behavior = if startup { "StartBoulderOn" } else { "ToggleBoulderOn" };
            let boulder_button = world.obj::<ControllerIslandFeatures>(this).sand_boulder_button_logic;
            set_button_behavior(world, boulder_button, behavior);

            let boulder = world.obj::<ControllerWorldRegistry>(this).boulder_logic;
            if let Some(boulder) = boulder {
                boulder_logic::enter_island(world, boulder, startup);
            }
        }
        2 => {
            let behavior = if startup { "StartFishOn" } else { "ToggleFishOn" };
            let fish_button = world.obj::<ControllerIslandFeatures>(this).sand_fish_button_logic;
            set_button_behavior(world, fish_button, behavior);

            let fishing_pole = world.obj::<ControllerWorldRegistry>(this).fishing_pole_logic;
            if let Some(fishing_pole) = fishing_pole {
                fishing_pole_logic::enter_island(world, fishing_pole, startup);
            }
            let fish = world.obj::<ControllerWorldRegistry>(this).fish_logic;
            if let Some(fish) = fish {
                fish_logic::enter_island(world, fish, startup);
            }
        }
        3 => {
            let behavior = if startup { "StartAntHillOn" } else { "ToggleAntHillOn" };
            let ant_hill_button = world.obj::<ControllerIslandFeatures>(this).sand_ant_hill_button_logic;
            set_button_behavior(world, ant_hill_button, behavior);

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
            let behavior = if startup { "StartWaterWingsOn" } else { "ToggleWaterWingsOn" };
            let water_wings_button = world.obj::<ControllerIslandFeatures>(this).sand_water_wings_button_logic;
            set_button_behavior(world, water_wings_button, behavior);

            let water_wings = world.obj::<ControllerWorldRegistry>(this).water_wings_logic;
            if let Some(water_wings) = water_wings {
                water_wings_logic::enter_island(world, water_wings, startup);
            }
        }
        _ => {}
    }
}

/// `-[PocketGodViewController setRockIslandState:startup:]` @ 0x1000842bc
pub fn set_rock_island_state(world: &mut World, this: Entity, rock_island_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).rock_island_state;
    if old_state == 1 {
        let fishing_pole = world.obj::<ControllerWorldRegistry>(this).fishing_pole_logic;
        if let Some(fishing_pole) = fishing_pole {
            fishing_pole_logic::exit_island(world, fishing_pole, false);
        }
        let rock_fish_button = world.obj::<ControllerIslandFeatures>(this).rock_fish_button_logic;
        set_button_behavior(world, rock_fish_button, "ToggleFishOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).rock_island_state;
    let new_state = if current_state == rock_island_state { 0 } else { rock_island_state };
    world.obj_mut::<ControllerIslandFeatures>(this).rock_island_state = new_state;
    set_option_int_preference(world, this, "RockIslandHazard", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartFishOn" } else { "ToggleFishOn" };
    let rock_fish_button = world.obj::<ControllerIslandFeatures>(this).rock_fish_button_logic;
    set_button_behavior(world, rock_fish_button, behavior);

    let fishing_pole = world.obj::<ControllerWorldRegistry>(this).fishing_pole_logic;
    if let Some(fishing_pole) = fishing_pole {
        fishing_pole_logic::enter_island(world, fishing_pole, startup);
    }
    let fish = world.obj::<ControllerWorldRegistry>(this).fish_logic;
    if let Some(fish) = fish {
        fish_logic::enter_island(world, fish, startup);
    }
}

/// `-[PocketGodViewController setRockIslandFeatureState:startup:]` @ 0x100084414
pub fn set_rock_island_feature_state(world: &mut World, this: Entity, rock_island_feature_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).rock_island_feature_state;
    if old_state == 2 {
        let t_rex_egg = world.obj::<ControllerWorldRegistry>(this).t_rex_egg_logic;
        if let Some(t_rex_egg) = t_rex_egg {
            t_rex_egg_logic::exit_island(world, t_rex_egg, false);
        }
        let t_rex = world.obj::<ControllerWorldRegistry>(this).t_rex_logic;
        if let Some(t_rex) = t_rex {
            t_rex_logic::exit_island(world, t_rex);
        }
        let spears = world.obj::<ControllerWorldRegistry>(this).spear_logic_array.clone();
        for spear in spears {
            crate::send::exit_island_with_arg(world, Some(spear), false);
        }
        let egg_button = world.obj::<ControllerIslandFeatures>(this).rock_egg_button_logic;
        set_button_behavior(world, egg_button, "ToggleEggOff");
    } else if old_state == 1 {
        let bon_fire = world.obj::<ControllerWorldRegistry>(this).bon_fire_logic;
        if let Some(bon_fire) = bon_fire {
            bon_fire_logic::exit_island(world, bon_fire);
        }
        let woods = world.obj::<ControllerWorldRegistry>(this).wood_logic_array.clone();
        for wood in woods {
            crate::send::exit_island_with_arg(world, Some(wood), false);
        }
        let wood_button = world.obj::<ControllerIslandFeatures>(this).rock_wood_button_logic;
        set_button_behavior(world, wood_button, "ToggleWoodOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).rock_island_feature_state;
    let new_state = if current_state == rock_island_feature_state { 0 } else { rock_island_feature_state };
    world.obj_mut::<ControllerIslandFeatures>(this).rock_island_feature_state = new_state;
    set_option_int_preference(world, this, "RockIslandFeature", new_state);

    if new_state == 2 {
        let behavior = if startup { "StartEggOn" } else { "ToggleEggOn" };
        let egg_button = world.obj::<ControllerIslandFeatures>(this).rock_egg_button_logic;
        set_button_behavior(world, egg_button, behavior);

        let spears = world.obj::<ControllerWorldRegistry>(this).spear_logic_array.clone();
        for spear in spears {
            crate::send::enter_island_with_arg(world, Some(spear), startup);
        }
        let t_rex_egg = world.obj::<ControllerWorldRegistry>(this).t_rex_egg_logic;
        if let Some(t_rex_egg) = t_rex_egg {
            t_rex_egg_logic::enter_island(world, t_rex_egg, startup);
        }
    } else if new_state == 1 {
        let behavior = if startup { "StartWoodOn" } else { "ToggleWoodOn" };
        let wood_button = world.obj::<ControllerIslandFeatures>(this).rock_wood_button_logic;
        set_button_behavior(world, wood_button, behavior);

        let bon_fire = world.obj::<ControllerWorldRegistry>(this).bon_fire_logic;
        if let Some(bon_fire) = bon_fire {
            bon_fire_logic::enter_island(world, bon_fire, startup);
        }
        let woods = world.obj::<ControllerWorldRegistry>(this).wood_logic_array.clone();
        for wood in woods {
            crate::send::enter_island_with_arg(world, Some(wood), startup);
        }
    }
}

/// `-[PocketGodViewController setRockIslandOutHouseState:startup:]` @ 0x10008490c
pub fn set_rock_island_out_house_state(world: &mut World, this: Entity, rock_island_out_house_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).rock_island_out_house_state;
    if old_state == 1 {
        let out_house = world.obj::<ControllerWorldRegistry>(this).out_house_logic;
        if let Some(out_house) = out_house {
            out_house_logic::exit_island(world, out_house, false);
        }
        let out_house_button = world.obj::<ControllerIslandFeatures>(this).rock_out_house_button_logic;
        set_button_behavior(world, out_house_button, "ToggleOutHouseOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).rock_island_out_house_state;
    let new_state = if current_state == rock_island_out_house_state { 0 } else { rock_island_out_house_state };
    world.obj_mut::<ControllerIslandFeatures>(this).rock_island_out_house_state = new_state;
    set_option_int_preference(world, this, "RockIslandOutHouseState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartOutHouseOn" } else { "ToggleOutHouseOn" };
    let out_house_button = world.obj::<ControllerIslandFeatures>(this).rock_out_house_button_logic;
    set_button_behavior(world, out_house_button, behavior);

    let out_house = world.obj::<ControllerWorldRegistry>(this).out_house_logic;
    if let Some(out_house) = out_house {
        out_house_logic::enter_island(world, out_house, startup);
    }
}

/// `-[PocketGodViewController setRockIslandTRexState:startup:]` @ 0x100084a48
pub fn set_rock_island_t_rex_state(world: &mut World, this: Entity, rock_island_t_rex_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).rock_island_t_rex_state;
    if old_state == 1 {
        let t_rex = world.obj::<ControllerWorldRegistry>(this).t_rex_logic;
        if let Some(t_rex) = t_rex {
            t_rex_logic::exit_island(world, t_rex);
        }
        let t_rex_button = world.obj::<ControllerIslandFeatures>(this).rock_t_rex_button_logic;
        set_button_behavior(world, t_rex_button, "ToggleTRexOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).rock_island_t_rex_state;
    let new_state = if current_state == rock_island_t_rex_state { 0 } else { rock_island_t_rex_state };
    world.obj_mut::<ControllerIslandFeatures>(this).rock_island_t_rex_state = new_state;
    set_option_int_preference(world, this, "RockIslandTRexState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartTRexOn" } else { "ToggleTRexOn" };
    let t_rex_button = world.obj::<ControllerIslandFeatures>(this).rock_t_rex_button_logic;
    set_button_behavior(world, t_rex_button, behavior);

    let t_rex = world.obj::<ControllerWorldRegistry>(this).t_rex_logic;
    if let Some(t_rex) = t_rex {
        t_rex_logic::enter_island(world, t_rex, startup);
    }
}

/// `-[PocketGodViewController setRockIslandSpearState:startup:]` @ 0x100084b80
pub fn set_rock_island_spear_state(world: &mut World, this: Entity, rock_island_spear_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).rock_island_spear_state;
    if old_state == 1 {
        let spears = world.obj::<ControllerWorldRegistry>(this).spear_logic_array.clone();
        for spear in spears {
            crate::send::exit_island_with_arg(world, Some(spear), false);
        }
        let rock_spear_button = world.obj::<ControllerIslandFeatures>(this).rock_spear_button_logic;
        set_button_behavior(world, rock_spear_button, "ToggleSpearOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).rock_island_spear_state;
    let new_state = if current_state == rock_island_spear_state { 0 } else { rock_island_spear_state };
    world.obj_mut::<ControllerIslandFeatures>(this).rock_island_spear_state = new_state;
    set_option_int_preference(world, this, "RockIslandSpearState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartSpearOn" } else { "ToggleSpearOn" };
    let rock_spear_button = world.obj::<ControllerIslandFeatures>(this).rock_spear_button_logic;
    set_button_behavior(world, rock_spear_button, behavior);

    let spears = world.obj::<ControllerWorldRegistry>(this).spear_logic_array.clone();
    for spear in spears {
        crate::send::enter_island_with_arg(world, Some(spear), startup);
    }
}

/// `-[PocketGodViewController setGraveyardIslandState:startup:]` @ 0x100084e24
pub fn set_graveyard_island_state(world: &mut World, this: Entity, graveyard_island_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_state;
    if old_state == 1 {
        let water_wings = world.obj::<ControllerWorldRegistry>(this).water_wings_logic;
        if let Some(water_wings) = water_wings {
            water_wings_logic::exit_island(world, water_wings, false);
        }
        let water_wings_button = world.obj::<ControllerIslandFeatures>(this).graveyard_water_wings_button_logic;
        set_button_behavior(world, water_wings_button, "ToggleWaterWingsOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_state;
    let new_state = if current_state == graveyard_island_state { 0 } else { graveyard_island_state };
    world.obj_mut::<ControllerIslandFeatures>(this).graveyard_island_state = new_state;
    set_option_int_preference(world, this, "GraveyardIslandState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartWaterWingsOn" } else { "ToggleWaterWingsOn" };
    let water_wings_button = world.obj::<ControllerIslandFeatures>(this).graveyard_water_wings_button_logic;
    set_button_behavior(world, water_wings_button, behavior);

    let water_wings = world.obj::<ControllerWorldRegistry>(this).water_wings_logic;
    if let Some(water_wings) = water_wings {
        water_wings_logic::enter_island(world, water_wings, startup);
    }
}

/// `-[PocketGodViewController setGraveyardIslandZombieGraveState:startup:]` @ 0x100084f60
pub fn set_graveyard_island_zombie_grave_state(world: &mut World, this: Entity, graveyard_island_zombie_grave_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_zombie_grave_state;
    if old_state == 1 {
        let zombie_grave = world.obj::<ControllerWorldRegistry>(this).zombie_grave_logic;
        if let Some(zombie_grave) = zombie_grave {
            zombie_grave_logic::exit_island(world, zombie_grave, false);
        }
        let zombie_grave_button = world.obj::<ControllerIslandFeatures>(this).graveyard_zombie_grave_button_logic;
        set_button_behavior(world, zombie_grave_button, "ToggleZombieGraveOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_zombie_grave_state;
    let new_state = if current_state == graveyard_island_zombie_grave_state { 0 } else { graveyard_island_zombie_grave_state };
    world.obj_mut::<ControllerIslandFeatures>(this).graveyard_island_zombie_grave_state = new_state;
    set_option_int_preference(world, this, "GraveyardIslandZombieGraveState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartZombieGraveOn" } else { "ToggleZombieGraveOn" };
    let zombie_grave_button = world.obj::<ControllerIslandFeatures>(this).graveyard_zombie_grave_button_logic;
    set_button_behavior(world, zombie_grave_button, behavior);

    let zombie_grave = world.obj::<ControllerWorldRegistry>(this).zombie_grave_logic;
    if let Some(zombie_grave) = zombie_grave {
        zombie_grave_logic::enter_island(world, zombie_grave, startup);
    }
}

/// `-[PocketGodViewController setGraveyardIslandGhostState:startup:]` @ 0x10008509c
pub fn set_graveyard_island_ghost_state(world: &mut World, this: Entity, graveyard_island_ghost_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_ghost_state;
    if old_state == 1 {
        let ghost_button = world.obj::<ControllerIslandFeatures>(this).graveyard_ghost_button_logic;
        set_button_behavior(world, ghost_button, "ToggleGhostOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_ghost_state;
    let new_state = if current_state == graveyard_island_ghost_state { 0 } else { graveyard_island_ghost_state };
    world.obj_mut::<ControllerIslandFeatures>(this).graveyard_island_ghost_state = new_state;
    set_option_int_preference(world, this, "GraveyardIslandGhostState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartGhostOn" } else { "ToggleGhostOn" };
    let ghost_button = world.obj::<ControllerIslandFeatures>(this).graveyard_ghost_button_logic;
    set_button_behavior(world, ghost_button, behavior);
}

/// `-[PocketGodViewController setGraveyardIslandSpiderWebState:startup:]` @ 0x100085198
pub fn set_graveyard_island_spider_web_state(world: &mut World, this: Entity, graveyard_island_spider_web_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_spider_web_state;
    if old_state == 1 {
        let spider_tree = world.obj::<ControllerWorldRegistry>(this).spider_tree_logic;
        if let Some(spider_tree) = spider_tree {
            spider_tree_logic::exit_island(world, spider_tree, false);
        }
        let spider = world.obj::<ControllerWorldRegistry>(this).spider_logic;
        if let Some(spider) = spider {
            spider_logic::exit_island(world, spider);
        }
        let spears = world.obj::<ControllerWorldRegistry>(this).spear_logic_array.clone();
        for spear in spears {
            crate::send::exit_island_with_arg(world, Some(spear), false);
        }
        let spider_web_button = world.obj::<ControllerIslandFeatures>(this).graveyard_spider_web_button_logic;
        set_button_behavior(world, spider_web_button, "ToggleSpiderWebOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_spider_web_state;
    let new_state = if current_state == graveyard_island_spider_web_state { 0 } else { graveyard_island_spider_web_state };
    world.obj_mut::<ControllerIslandFeatures>(this).graveyard_island_spider_web_state = new_state;
    set_option_int_preference(world, this, "GraveyardIslandSpiderWebState", new_state);

    if new_state != 1 {
        return;
    }

    let spider_tree = world.obj::<ControllerWorldRegistry>(this).spider_tree_logic;
    if let Some(spider_tree) = spider_tree {
        spider_tree_logic::enter_island(world, spider_tree, startup);
    }
    let spears = world.obj::<ControllerWorldRegistry>(this).spear_logic_array.clone();
    for spear in spears {
        crate::send::enter_island_with_arg(world, Some(spear), startup);
    }

    let behavior = if startup { "StartSpiderWebOn" } else { "ToggleSpiderWebOn" };
    let spider_web_button = world.obj::<ControllerIslandFeatures>(this).graveyard_spider_web_button_logic;
    set_button_behavior(world, spider_web_button, behavior);
}

/// `-[PocketGodViewController setGraveyardIslandSpiderState:startup:]` @ 0x100085488
pub fn set_graveyard_island_spider_state(world: &mut World, this: Entity, graveyard_island_spider_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_spider_state;
    if old_state == 1 {
        let spider = world.obj::<ControllerWorldRegistry>(this).spider_logic;
        if let Some(spider) = spider {
            spider_logic::exit_island(world, spider);
        }
        let spider_button = world.obj::<ControllerIslandFeatures>(this).graveyard_spider_button_logic;
        set_button_behavior(world, spider_button, "ToggleSpiderOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_spider_state;
    let new_state = if current_state == graveyard_island_spider_state { 0 } else { graveyard_island_spider_state };
    world.obj_mut::<ControllerIslandFeatures>(this).graveyard_island_spider_state = new_state;
    set_option_int_preference(world, this, "GraveyardIslandSpiderState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartSpiderOn" } else { "ToggleSpiderOn" };
    let spider_button = world.obj::<ControllerIslandFeatures>(this).graveyard_spider_button_logic;
    set_button_behavior(world, spider_button, behavior);

    let spider = world.obj::<ControllerWorldRegistry>(this).spider_logic;
    if let Some(spider) = spider {
        spider_logic::enter_island(world, spider, startup);
    }
}

/// `-[PocketGodViewController setGraveyardIslandSpearState:startup:]` @ 0x1000855c0
pub fn set_graveyard_island_spear_state(world: &mut World, this: Entity, graveyard_island_spear_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_spear_state;
    if old_state == 1 {
        let spears = world.obj::<ControllerWorldRegistry>(this).spear_logic_array.clone();
        for spear in spears {
            crate::send::exit_island_with_arg(world, Some(spear), false);
        }
        let graveyard_spear_button = world.obj::<ControllerIslandFeatures>(this).graveyard_spear_button_logic;
        set_button_behavior(world, graveyard_spear_button, "ToggleSpearOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_spear_state;
    let new_state = if current_state == graveyard_island_spear_state { 0 } else { graveyard_island_spear_state };
    world.obj_mut::<ControllerIslandFeatures>(this).graveyard_island_spear_state = new_state;
    set_option_int_preference(world, this, "GraveyardIslandSpearState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartSpearOn" } else { "ToggleSpearOn" };
    let graveyard_spear_button = world.obj::<ControllerIslandFeatures>(this).graveyard_spear_button_logic;
    set_button_behavior(world, graveyard_spear_button, behavior);

    let spears = world.obj::<ControllerWorldRegistry>(this).spear_logic_array.clone();
    for spear in spears {
        crate::send::enter_island_with_arg(world, Some(spear), startup);
    }
}

/// `-[PocketGodViewController setIceIslandState:startup:]` @ 0x100085864
pub fn set_ice_island_state(world: &mut World, this: Entity, ice_island_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).ice_island_state;
    if old_state == 2 {
        let water_wings = world.obj::<ControllerWorldRegistry>(this).water_wings_logic;
        if let Some(water_wings) = water_wings {
            water_wings_logic::exit_island(world, water_wings, false);
        }
        let water_wings_button = world.obj::<ControllerIslandFeatures>(this).ice_water_wings_button_logic;
        set_button_behavior(world, water_wings_button, "ToggleWaterWingsOff");
    } else if old_state == 1 {
        let fishing_pole = world.obj::<ControllerWorldRegistry>(this).fishing_pole_logic;
        if let Some(fishing_pole) = fishing_pole {
            fishing_pole_logic::exit_island(world, fishing_pole, false);
        }
        let ice_fish_button = world.obj::<ControllerIslandFeatures>(this).ice_fish_button_logic;
        set_button_behavior(world, ice_fish_button, "ToggleFishOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).ice_island_state;
    let new_state = if current_state == ice_island_state { 0 } else { ice_island_state };
    world.obj_mut::<ControllerIslandFeatures>(this).ice_island_state = new_state;
    set_option_int_preference(world, this, "IceIslandState", new_state);

    if new_state == 2 {
        let behavior = if startup { "StartWaterWingsOn" } else { "ToggleWaterWingsOn" };
        let water_wings_button = world.obj::<ControllerIslandFeatures>(this).ice_water_wings_button_logic;
        set_button_behavior(world, water_wings_button, behavior);

        let water_wings = world.obj::<ControllerWorldRegistry>(this).water_wings_logic;
        if let Some(water_wings) = water_wings {
            water_wings_logic::enter_island(world, water_wings, startup);
        }
    } else if new_state == 1 {
        let behavior = if startup { "StartFishOn" } else { "ToggleFishOn" };
        let ice_fish_button = world.obj::<ControllerIslandFeatures>(this).ice_fish_button_logic;
        set_button_behavior(world, ice_fish_button, behavior);

        let fishing_pole = world.obj::<ControllerWorldRegistry>(this).fishing_pole_logic;
        if let Some(fishing_pole) = fishing_pole {
            fishing_pole_logic::enter_island(world, fishing_pole, startup);
        }
        let fish = world.obj::<ControllerWorldRegistry>(this).fish_logic;
        if let Some(fish) = fish {
            fish_logic::enter_island(world, fish, startup);
        }
    }
}

/// `-[PocketGodViewController setIceIslandIceMonsterState:startup:]` @ 0x100085a48
pub fn set_ice_island_ice_monster_state(world: &mut World, this: Entity, ice_island_ice_monster_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).ice_island_ice_monster_state;
    if old_state == 1 {
        let ice_monster = world.obj::<ControllerWorldRegistry>(this).ice_monster_logic;
        if let Some(ice_monster) = ice_monster {
            ice_monster_logic::exit_island(world, ice_monster, false);
        }
        let ice_monster_button = world.obj::<ControllerIslandFeatures>(this).ice_ice_monster_button_logic;
        set_button_behavior(world, ice_monster_button, "ToggleIceMonsterOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).ice_island_ice_monster_state;
    let new_state = if current_state == ice_island_ice_monster_state { 0 } else { ice_island_ice_monster_state };
    world.obj_mut::<ControllerIslandFeatures>(this).ice_island_ice_monster_state = new_state;
    set_option_int_preference(world, this, "IceIslandIceMonsterState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartIceMonsterOn" } else { "ToggleIceMonsterOn" };
    let ice_monster_button = world.obj::<ControllerIslandFeatures>(this).ice_ice_monster_button_logic;
    set_button_behavior(world, ice_monster_button, behavior);

    let ice_monster = world.obj::<ControllerWorldRegistry>(this).ice_monster_logic;
    if let Some(ice_monster) = ice_monster {
        ice_monster_logic::enter_island(world, ice_monster, startup);
    }
}

/// `-[PocketGodViewController setIceIslandIglooState:startup:]` @ 0x100085b84
pub fn set_ice_island_igloo_state(world: &mut World, this: Entity, ice_island_igloo_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).ice_island_igloo_state;
    if old_state == 1 {
        let igloo = world.obj::<ControllerWorldRegistry>(this).igloo_logic;
        if let Some(igloo) = igloo {
            igloo_logic::exit_island(world, igloo, false);
        }
        let igloo_button = world.obj::<ControllerIslandFeatures>(this).ice_igloo_button_logic;
        set_button_behavior(world, igloo_button, "ToggleIglooOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).ice_island_igloo_state;
    let new_state = if current_state == ice_island_igloo_state { 0 } else { ice_island_igloo_state };
    world.obj_mut::<ControllerIslandFeatures>(this).ice_island_igloo_state = new_state;
    set_option_int_preference(world, this, "IceIslandIglooState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartIglooOn" } else { "ToggleIglooOn" };
    let igloo_button = world.obj::<ControllerIslandFeatures>(this).ice_igloo_button_logic;
    set_button_behavior(world, igloo_button, behavior);

    let igloo = world.obj::<ControllerWorldRegistry>(this).igloo_logic;
    if let Some(igloo) = igloo {
        igloo_logic::enter_island(world, igloo, startup);
    }
}

/// `-[PocketGodViewController setIceIslandIceHoleState:startup:]` @ 0x100085cc0
pub fn set_ice_island_ice_hole_state(world: &mut World, this: Entity, ice_island_ice_hole_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).ice_island_ice_hole_state;
    if old_state == 1 {
        let ice_hole = world.obj::<ControllerWorldRegistry>(this).ice_hole_logic;
        if let Some(ice_hole) = ice_hole {
            ice_hole_logic::exit_island(world, ice_hole, false);
        }
        let ice_hole_button = world.obj::<ControllerIslandFeatures>(this).ice_ice_hole_button_logic;
        set_button_behavior(world, ice_hole_button, "ToggleIceHoleOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).ice_island_ice_hole_state;
    let new_state = if current_state == ice_island_ice_hole_state { 0 } else { ice_island_ice_hole_state };
    world.obj_mut::<ControllerIslandFeatures>(this).ice_island_ice_hole_state = new_state;
    set_option_int_preference(world, this, "IceIslandIceHoleState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartIceHoleOn" } else { "ToggleIceHoleOn" };
    let ice_hole_button = world.obj::<ControllerIslandFeatures>(this).ice_ice_hole_button_logic;
    set_button_behavior(world, ice_hole_button, behavior);

    let ice_hole = world.obj::<ControllerWorldRegistry>(this).ice_hole_logic;
    if let Some(ice_hole) = ice_hole {
        ice_hole_logic::enter_island(world, ice_hole, startup);
    }
}

/// `-[PocketGodViewController setIceIslandIceSquidState:startup:]` @ 0x100085dfc
pub fn set_ice_island_ice_squid_state(world: &mut World, this: Entity, ice_island_ice_squid_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).ice_island_ice_squid_state;
    if old_state == 1 {
        let ice_hole = world.obj::<ControllerWorldRegistry>(this).ice_hole_logic;
        if let Some(ice_hole) = ice_hole {
            ice_hole_logic::disable_ice_squid(world, ice_hole);
        }
        let ice_squid_button = world.obj::<ControllerIslandFeatures>(this).ice_ice_squid_button_logic;
        set_button_behavior(world, ice_squid_button, "ToggleIceSquidOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).ice_island_ice_squid_state;
    let new_state = if current_state == ice_island_ice_squid_state { 0 } else { ice_island_ice_squid_state };
    world.obj_mut::<ControllerIslandFeatures>(this).ice_island_ice_squid_state = new_state;
    set_option_int_preference(world, this, "IceIslandIceSquidState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartIceSquidOn" } else { "ToggleIceSquidOn" };
    let ice_squid_button = world.obj::<ControllerIslandFeatures>(this).ice_ice_squid_button_logic;
    set_button_behavior(world, ice_squid_button, behavior);

    let ice_hole = world.obj::<ControllerWorldRegistry>(this).ice_hole_logic;
    if let Some(ice_hole) = ice_hole {
        ice_hole_logic::enable_ice_squid(world, ice_hole);
    }
}

/// `-[PocketGodViewController setApeIslandState:startup:]` @ 0x100085f30
pub fn set_ape_island_state(world: &mut World, this: Entity, ape_island_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).ape_island_state;
    if old_state == 2 {
        let banana_tree = world.obj::<ControllerWorldRegistry>(this).banana_tree_logic;
        if let Some(banana_tree) = banana_tree {
            banana_tree_logic::exit_island(world, banana_tree, false);
        }
        let kong_statue = world.obj::<ControllerWorldRegistry>(this).kong_statue_logic;
        if let Some(kong_statue) = kong_statue {
            kong_statue_logic::exit_island(world, kong_statue);
        }
        let banana_tree_button = world.obj::<ControllerIslandFeatures>(this).ape_banana_tree_button_logic;
        set_button_behavior(world, banana_tree_button, "ToggleBananaTreeOff");
    } else if old_state == 1 {
        let ape_altar = world.obj::<ControllerWorldRegistry>(this).ape_altar_logic;
        if let Some(ape_altar) = ape_altar {
            ape_altar_logic::exit_island(world, ape_altar, false);
        }
        let ape_bongo = world.obj::<ControllerWorldRegistry>(this).ape_bongo_logic;
        if let Some(ape_bongo) = ape_bongo {
            ape_bongo_logic::exit_island(world, ape_bongo, false);
        }
        let ape_wench = world.obj::<ControllerWorldRegistry>(this).ape_wench_logic;
        if let Some(ape_wench) = ape_wench {
            ape_wench_logic::exit_island(world, ape_wench, false);
        }
        let ape_altar_button = world.obj::<ControllerIslandFeatures>(this).ape_ape_altar_button_logic;
        set_button_behavior(world, ape_altar_button, "ToggleApeAltarOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).ape_island_state;
    let new_state = if current_state == ape_island_state { 0 } else { ape_island_state };
    world.obj_mut::<ControllerIslandFeatures>(this).ape_island_state = new_state;
    set_option_int_preference(world, this, "ApeIslandState", new_state);

    if new_state == 2 {
        let behavior = if startup { "StartBananaTreeOn" } else { "ToggleBananaTreeOn" };
        let banana_tree_button = world.obj::<ControllerIslandFeatures>(this).ape_banana_tree_button_logic;
        set_button_behavior(world, banana_tree_button, behavior);

        let banana_tree = world.obj::<ControllerWorldRegistry>(this).banana_tree_logic;
        if let Some(banana_tree) = banana_tree {
            banana_tree_logic::enter_island(world, banana_tree, startup);
        }
        let kong_statue = world.obj::<ControllerWorldRegistry>(this).kong_statue_logic;
        if let Some(kong_statue) = kong_statue {
            kong_statue_logic::enter_island(world, kong_statue, startup);
        }
    } else if new_state == 1 {
        let behavior = if startup { "StartApeAltarOn" } else { "ToggleApeAltarOn" };
        let ape_altar_button = world.obj::<ControllerIslandFeatures>(this).ape_ape_altar_button_logic;
        set_button_behavior(world, ape_altar_button, behavior);

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
}

/// `-[PocketGodViewController setApeIslandApeState:startup:]` @ 0x10008619c
pub fn set_ape_island_ape_state(world: &mut World, this: Entity, ape_island_ape_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).ape_island_ape_state;
    if old_state == 1 {
        let ape = world.obj::<ControllerWorldRegistry>(this).ape_logic;
        if let Some(ape) = ape {
            ape_logic::exit_island(world, ape, false);
        }
        let ape_button = world.obj::<ControllerIslandFeatures>(this).ape_ape_button_logic;
        set_button_behavior(world, ape_button, "ToggleApeOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).ape_island_ape_state;
    let new_state = if current_state == ape_island_ape_state { 0 } else { ape_island_ape_state };
    world.obj_mut::<ControllerIslandFeatures>(this).ape_island_ape_state = new_state;
    set_option_int_preference(world, this, "ApeIslandApeState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartApeOn" } else { "ToggleApeOn" };
    let ape_button = world.obj::<ControllerIslandFeatures>(this).ape_ape_button_logic;
    set_button_behavior(world, ape_button, behavior);

    let ape = world.obj::<ControllerWorldRegistry>(this).ape_logic;
    if let Some(ape) = ape {
        ape_logic::enter_island(world, ape, startup);
    }
}

/// `-[PocketGodViewController setApeIslandMonkeyState:startup:]` @ 0x1000862d8
pub fn set_ape_island_monkey_state(world: &mut World, this: Entity, ape_island_monkey_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).ape_island_monkey_state;
    if old_state == 1 {
        let monkey_button = world.obj::<ControllerIslandFeatures>(this).ape_monkey_button_logic;
        set_button_behavior(world, monkey_button, "ToggleMonkeyOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).ape_island_monkey_state;
    let new_state = if current_state == ape_island_monkey_state { 0 } else { ape_island_monkey_state };
    world.obj_mut::<ControllerIslandFeatures>(this).ape_island_monkey_state = new_state;
    set_option_int_preference(world, this, "ApeIslandMonkeyState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartMonkeyOn" } else { "ToggleMonkeyOn" };
    let monkey_button = world.obj::<ControllerIslandFeatures>(this).ape_monkey_button_logic;
    set_button_behavior(world, monkey_button, behavior);
}

/// `-[PocketGodViewController setMisfitIslandState:startup:]` @ 0x1000863d4
pub fn set_misfit_island_state(world: &mut World, this: Entity, misfit_island_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).misfit_island_state;
    if old_state == 2 {
        let water_wings = world.obj::<ControllerWorldRegistry>(this).water_wings_logic;
        if let Some(water_wings) = water_wings {
            water_wings_logic::exit_island(world, water_wings, false);
        }
        let water_wings_button = world.obj::<ControllerIslandFeatures>(this).misfit_water_wings_button_logic;
        set_button_behavior(world, water_wings_button, "ToggleWaterWingsOff");
    } else if old_state == 1 {
        let moron_snake = world.obj::<ControllerWorldRegistry>(this).moron_snake_logic;
        if let Some(moron_snake) = moron_snake {
            moron_snake_logic::exit_island(world, moron_snake, false);
        }
        let moron_snake_button = world.obj::<ControllerIslandFeatures>(this).misfit_moron_snake_button_logic;
        set_button_behavior(world, moron_snake_button, "ToggleMoronSnakeOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).misfit_island_state;
    let new_state = if current_state == misfit_island_state { 0 } else { misfit_island_state };
    world.obj_mut::<ControllerIslandFeatures>(this).misfit_island_state = new_state;
    set_option_int_preference(world, this, "MisfitIslandState", new_state);

    if new_state == 2 {
        let behavior = if startup { "StartWaterWingsOn" } else { "ToggleWaterWingsOn" };
        let water_wings_button = world.obj::<ControllerIslandFeatures>(this).misfit_water_wings_button_logic;
        set_button_behavior(world, water_wings_button, behavior);

        let water_wings = world.obj::<ControllerWorldRegistry>(this).water_wings_logic;
        if let Some(water_wings) = water_wings {
            water_wings_logic::enter_island(world, water_wings, startup);
        }
    } else if new_state == 1 {
        let behavior = if startup { "StartMoronSnakeOn" } else { "ToggleMoronSnakeOn" };
        let moron_snake_button = world.obj::<ControllerIslandFeatures>(this).misfit_moron_snake_button_logic;
        set_button_behavior(world, moron_snake_button, behavior);

        let moron_snake = world.obj::<ControllerWorldRegistry>(this).moron_snake_logic;
        if let Some(moron_snake) = moron_snake {
            moron_snake_logic::enter_island(world, moron_snake, startup);
        }
    }
}

/// `-[PocketGodViewController setGodIslandState:startup:]` @ 0x10008658c
pub fn set_god_island_state(world: &mut World, this: Entity, god_island_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).god_island_state;
    if old_state == 1 {
        let water_wings = world.obj::<ControllerWorldRegistry>(this).water_wings_logic;
        if let Some(water_wings) = water_wings {
            water_wings_logic::exit_island(world, water_wings, false);
        }
        let water_wings_button = world.obj::<ControllerIslandFeatures>(this).idol_water_wings_button_logic;
        set_button_behavior(world, water_wings_button, "ToggleWaterWingsOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).god_island_state;
    let new_state = if current_state == god_island_state { 0 } else { god_island_state };
    world.obj_mut::<ControllerIslandFeatures>(this).god_island_state = new_state;
    set_option_int_preference(world, this, "GodIslandState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartWaterWingsOn" } else { "ToggleWaterWingsOn" };
    let water_wings_button = world.obj::<ControllerIslandFeatures>(this).idol_water_wings_button_logic;
    set_button_behavior(world, water_wings_button, behavior);

    let water_wings = world.obj::<ControllerWorldRegistry>(this).water_wings_logic;
    if let Some(water_wings) = water_wings {
        water_wings_logic::enter_island(world, water_wings, startup);
    }
}

/// `-[PocketGodViewController setGodIslandIdolsState:startup:]` @ 0x1000866c8
pub fn set_god_island_idols_state(world: &mut World, this: Entity, god_island_idols_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).god_island_idols_state;
    if old_state == 1 {
        let idols_button = world.obj::<ControllerIslandFeatures>(this).idols_button_logic;
        set_button_behavior(world, idols_button, "ToggleIdolsOff");

        let god_idols = world.obj::<ControllerWorldRegistry>(this).god_idol_logic_array.clone();
        for god_idol in god_idols {
            crate::send::exit_island_with_arg(world, Some(god_idol), false);
        }
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).god_island_idols_state;
    let new_state = if current_state == god_island_idols_state { 0 } else { god_island_idols_state };
    world.obj_mut::<ControllerIslandFeatures>(this).god_island_idols_state = new_state;
    set_option_int_preference(world, this, "GodIslandIdolsState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartIdolsOn" } else { "ToggleIdolsOn" };
    let idols_button = world.obj::<ControllerIslandFeatures>(this).idols_button_logic;
    set_button_behavior(world, idols_button, behavior);

    let skin_pack_manager = skin_pack_manager::instance(world);
    if let Some(skin_pack_manager) = skin_pack_manager {
        skin_pack_manager::load_texture_atlas_channel(world, skin_pack_manager, 1);
    }

    let god_idols = world.obj::<ControllerWorldRegistry>(this).god_idol_logic_array.clone();
    for god_idol in god_idols {
        crate::send::enter_island_with_arg(world, Some(god_idol), startup);
    }
}

/// `-[PocketGodViewController setApocalypseIslandState:startup:]` @ 0x100086988
pub fn set_apocalypse_island_state(world: &mut World, this: Entity, apocalypse_island_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).apocalypse_island_state;
    if old_state == 1 {
        let water_wings = world.obj::<ControllerWorldRegistry>(this).water_wings_logic;
        if let Some(water_wings) = water_wings {
            water_wings_logic::exit_island(world, water_wings, false);
        }
        let water_wings_button = world.obj::<ControllerIslandFeatures>(this).apocalypse_water_wings_button_logic;
        set_button_behavior(world, water_wings_button, "ToggleWaterWingsOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).apocalypse_island_state;
    let new_state = if current_state == apocalypse_island_state { 0 } else { apocalypse_island_state };
    world.obj_mut::<ControllerIslandFeatures>(this).apocalypse_island_state = new_state;
    set_option_int_preference(world, this, "ApocalypseIslandState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartWaterWingsOn" } else { "ToggleWaterWingsOn" };
    let water_wings_button = world.obj::<ControllerIslandFeatures>(this).apocalypse_water_wings_button_logic;
    set_button_behavior(world, water_wings_button, behavior);

    let water_wings = world.obj::<ControllerWorldRegistry>(this).water_wings_logic;
    if let Some(water_wings) = water_wings {
        water_wings_logic::enter_island(world, water_wings, startup);
    }
}

/// `-[PocketGodViewController setFailBoatState:startup:]` @ 0x100086ac4
pub fn set_fail_boat_state(world: &mut World, this: Entity, fail_boat_state: i32, _startup: bool) {
    let current_state = world.obj::<ControllerIslandFeatures>(this).fail_boat_state;
    let new_state = if current_state == fail_boat_state { 0 } else { fail_boat_state };
    world.obj_mut::<ControllerIslandFeatures>(this).fail_boat_state = new_state;
    set_option_int_preference(world, this, "FailBoatState", new_state);
}

/// `-[PocketGodViewController setUnderwaterState:startup:]` @ 0x100086b00
pub fn set_underwater_state(world: &mut World, this: Entity, underwater_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).underwater_state;
    if old_state == 3 {
        let tile_drain = world.obj::<ControllerWorldRegistry>(this).tile_drain_logic;
        if let Some(tile_drain) = tile_drain {
            tile_drain_logic::exit_island(world, tile_drain);
        }
        let tile_plug = world.obj::<ControllerWorldRegistry>(this).tile_plug_logic;
        if let Some(tile_plug) = tile_plug {
            tile_plug_logic::exit_island(world, tile_plug);
        }
        let tile_drain_button = world.obj::<ControllerIslandFeatures>(this).underwater_tile_drain_button_logic;
        set_button_behavior(world, tile_drain_button, "ToggleTileDrainOff");
    } else if old_state == 2 {
        let chum_bucket = world.obj::<ControllerWorldRegistry>(this).chum_bucket_logic;
        if let Some(chum_bucket) = chum_bucket {
            chum_bucket_logic::exit_island(world, chum_bucket);
        }
        let laser_shark_button = world.obj::<ControllerIslandFeatures>(this).underwater_laser_shark_button_logic;
        set_button_behavior(world, laser_shark_button, "ToggleLaserSharkOff");
    } else if old_state == 1 {
        let anchor = world.obj::<ControllerWorldRegistry>(this).anchor_logic;
        if let Some(anchor) = anchor {
            anchor_logic::exit_island(world, anchor);
        }
        let piranha_button = world.obj::<ControllerIslandFeatures>(this).underwater_piranha_button_logic;
        set_button_behavior(world, piranha_button, "TogglePiranhaOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).underwater_state;
    let new_state = if current_state == underwater_state { 0 } else { underwater_state };
    world.obj_mut::<ControllerIslandFeatures>(this).underwater_state = new_state;
    set_option_int_preference(world, this, "UnderwaterHazard", new_state);

    if new_state == 3 {
        let behavior = if startup { "StartTileDrainOn" } else { "ToggleTileDrainOn" };
        let tile_drain_button = world.obj::<ControllerIslandFeatures>(this).underwater_tile_drain_button_logic;
        set_button_behavior(world, tile_drain_button, behavior);

        let tile_plug = world.obj::<ControllerWorldRegistry>(this).tile_plug_logic;
        if let Some(tile_plug) = tile_plug {
            tile_plug_logic::enter_island(world, tile_plug, startup);
        }
        let tile_drain = world.obj::<ControllerWorldRegistry>(this).tile_drain_logic;
        if let Some(tile_drain) = tile_drain {
            tile_drain_logic::enter_island(world, tile_drain, startup);
        }
    } else if new_state == 2 {
        let behavior = if startup { "StartLaserSharkOn" } else { "ToggleLaserSharkOn" };
        let laser_shark_button = world.obj::<ControllerIslandFeatures>(this).underwater_laser_shark_button_logic;
        set_button_behavior(world, laser_shark_button, behavior);

        let chum_bucket = world.obj::<ControllerWorldRegistry>(this).chum_bucket_logic;
        if let Some(chum_bucket) = chum_bucket {
            chum_bucket_logic::enter_island(world, chum_bucket, startup);
        }
    } else if new_state == 1 {
        let behavior = if startup { "StartPiranhaOn" } else { "TogglePiranhaOn" };
        let piranha_button = world.obj::<ControllerIslandFeatures>(this).underwater_piranha_button_logic;
        set_button_behavior(world, piranha_button, behavior);

        let anchor = world.obj::<ControllerWorldRegistry>(this).anchor_logic;
        if let Some(anchor) = anchor {
            anchor_logic::enter_island(world, anchor, startup);
        }
    }
}

/// `-[PocketGodViewController setUnderwaterCoralReefState:startup:]` @ 0x100086d90
pub fn set_underwater_coral_reef_state(world: &mut World, this: Entity, underwater_coral_reef_state: i32, startup: bool) {
    let old_state = world.obj::<ControllerIslandFeatures>(this).underwater_coral_reef_state;
    if old_state == 1 {
        let jellyfish_array = world.obj::<ControllerWorldRegistry>(this).jellyfish_logic_array.clone();
        for jellyfish in jellyfish_array {
            crate::send::exit_island_with_arg(world, Some(jellyfish), false);
        }
        let jellyfish_button = world.obj::<ControllerIslandFeatures>(this).underwater_coral_reef_jellyfish_button_logic;
        set_button_behavior(world, jellyfish_button, "ToggleJellyfishOff");
    }

    let current_state = world.obj::<ControllerIslandFeatures>(this).underwater_coral_reef_state;
    let new_state = if current_state == underwater_coral_reef_state { 0 } else { underwater_coral_reef_state };
    world.obj_mut::<ControllerIslandFeatures>(this).underwater_coral_reef_state = new_state;
    set_option_int_preference(world, this, "UnderwaterCoralReefState", new_state);

    if new_state != 1 {
        return;
    }

    let behavior = if startup { "StartJellyfishOn" } else { "ToggleJellyfishOn" };
    let jellyfish_button = world.obj::<ControllerIslandFeatures>(this).underwater_coral_reef_jellyfish_button_logic;
    set_button_behavior(world, jellyfish_button, behavior);

    let jellyfish_array = world.obj::<ControllerWorldRegistry>(this).jellyfish_logic_array.clone();
    for jellyfish in jellyfish_array {
        crate::send::enter_island_with_arg(world, Some(jellyfish), startup);
    }
}

/// `-[PocketGodViewController isVolcanoEnabled]` @ 0x1000870ac
pub fn is_volcano_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_SAND_ISLAND
}

/// `-[PocketGodViewController isGravityEnabled]` @ 0x1000871d8
pub fn is_gravity_enabled(world: &mut World, this: Entity) -> bool {
    let gravity_hazard = option_int_preference(world, this, "GravityHazard");
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let underwater = island_location_state == LOCATION_UNDERWATER || island_location_state == LOCATION_UNDERWATER_CORAL_REEF;

    let minigames = world.obj::<ControllerMinigames>(this);
    let no_game_running = minigames.doodle_jump_pygmy_logic.is_none()
        && !minigames.the_runs
        && !minigames.konkey_dong
        && !minigames.moron_pests
        && !minigames.no_pee
        && !minigames.dance_dance_execution;

    if gravity_hazard != 1 || underwater || !no_game_running {
        return false;
    }

    let screen_open = is_screen_open(world, this);
    !screen_open
}

/// `-[PocketGodViewController isSharkEnabled]` @ 0x1000872b8
pub fn is_shark_enabled(world: &mut World, this: Entity) -> bool {
    let shark_state = world.obj::<ControllerIslandFeatures>(this).shark_state;
    if shark_state != 1 {
        return false;
    }

    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state != LOCATION_UNDERWATER
        && island_location_state != LOCATION_APE_ISLAND
        && island_location_state != LOCATION_UNDERWATER_CORAL_REEF
}

/// `-[PocketGodViewController isBirdEnabled]` @ 0x100087308
pub fn is_bird_enabled(world: &mut World, this: Entity) -> bool {
    let bird_state = world.obj::<ControllerIslandFeatures>(this).bird_state;
    if bird_state != 1 {
        return false;
    }

    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state != LOCATION_UNDERWATER_CORAL_REEF && island_location_state != LOCATION_UNDERWATER
}

/// `-[PocketGodViewController isBoulderEnabled]` @ 0x1000873b8
pub fn is_boulder_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_SAND_ISLAND {
        return false;
    }

    let sand_island_state = world.obj::<ControllerIslandFeatures>(this).sand_island_state;
    sand_island_state == 1
}

/// `-[PocketGodViewController isCoconutEnabled]` @ 0x1000873e8
pub fn is_coconut_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_SAND_ISLAND
}

/// `-[PocketGodViewController isFishEnabled]` @ 0x100087400
pub fn is_fish_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let features = world.obj::<ControllerIslandFeatures>(this);
    match island_location_state {
        LOCATION_ICE_ISLAND => features.ice_island_state == 1,
        LOCATION_ROCK_ISLAND => features.rock_island_state == 1,
        LOCATION_SAND_ISLAND => features.sand_island_state == 2,
        _ => false,
    }
}

/// `-[PocketGodViewController isAntHillEnabled]` @ 0x100087474
pub fn is_ant_hill_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_SAND_ISLAND {
        return false;
    }

    let sand_island_state = world.obj::<ControllerIslandFeatures>(this).sand_island_state;
    sand_island_state == 3
}

/// `-[PocketGodViewController areWaterWingsEnabled]` @ 0x1000874a4
pub fn are_water_wings_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let features = world.obj::<ControllerIslandFeatures>(this);
    match island_location_state {
        LOCATION_SAND_ISLAND => features.sand_island_state == 4,
        LOCATION_GRAVEYARD_ISLAND => features.graveyard_island_state == 1,
        LOCATION_ICE_ISLAND => features.ice_island_state == 2,
        LOCATION_MISFIT_ISLAND => features.misfit_island_state == 2,
        LOCATION_GOD_ISLAND => features.god_island_state == 1,
        LOCATION_APOCALYPSE_ISLAND => features.apocalypse_island_state == 1,
        _ => false,
    }
}

/// `-[PocketGodViewController isTRexEggEnabled]` @ 0x100087590
pub fn is_t_rex_egg_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_ROCK_ISLAND {
        return false;
    }

    let rock_island_feature_state = world.obj::<ControllerIslandFeatures>(this).rock_island_feature_state;
    rock_island_feature_state == 2
}

/// `-[PocketGodViewController isWoodEnabled]` @ 0x1000875c4
pub fn is_wood_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_ROCK_ISLAND {
        return false;
    }

    let rock_island_feature_state = world.obj::<ControllerIslandFeatures>(this).rock_island_feature_state;
    rock_island_feature_state == 1
}

/// `-[PocketGodViewController isBonFireEnabled]` @ 0x1000875f8
pub fn is_bon_fire_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_ROCK_ISLAND {
        return false;
    }

    let rock_island_feature_state = world.obj::<ControllerIslandFeatures>(this).rock_island_feature_state;
    rock_island_feature_state == 1
}

/// `-[PocketGodViewController isSpearEnabled]` @ 0x10008762c
pub fn is_spear_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let features = world.obj::<ControllerIslandFeatures>(this);
    if island_location_state == LOCATION_GRAVEYARD_ISLAND {
        return features.graveyard_island_spear_state == 1 && features.graveyard_island_spider_web_state == 1;
    }

    island_location_state == LOCATION_ROCK_ISLAND
        && features.rock_island_spear_state == 1
        && features.rock_island_feature_state == 2
}

/// `-[PocketGodViewController isTRexEnabled]` @ 0x1000876b4
pub fn is_t_rex_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let features = world.obj::<ControllerIslandFeatures>(this);
    island_location_state == LOCATION_ROCK_ISLAND
        && features.rock_island_t_rex_state == 1
        && features.rock_island_feature_state == 2
}

/// `-[PocketGodViewController isOutHouseEnabled]` @ 0x1000876fc
pub fn is_out_house_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_ROCK_ISLAND {
        return false;
    }

    let out_house_state = world.obj::<ControllerIslandFeatures>(this).rock_island_out_house_state;
    out_house_state == 1
}

/// `-[PocketGodViewController isZombieGraveEnabled]` @ 0x100087730
pub fn is_zombie_grave_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_GRAVEYARD_ISLAND {
        return false;
    }

    let zombie_grave_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_zombie_grave_state;
    zombie_grave_state == 1
}

/// `-[PocketGodViewController areGhostsEnabled]` @ 0x100087764
pub fn are_ghosts_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_GRAVEYARD_ISLAND {
        return false;
    }

    let ghost_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_ghost_state;
    ghost_state == 1
}

/// `-[PocketGodViewController isSpiderEnabled]` @ 0x100087798
pub fn is_spider_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let features = world.obj::<ControllerIslandFeatures>(this);
    island_location_state == LOCATION_GRAVEYARD_ISLAND
        && features.graveyard_island_spider_web_state == 1
        && features.graveyard_island_spider_state == 1
}

/// `-[PocketGodViewController isSpiderTreeEnabled]` @ 0x1000877e0
pub fn is_spider_tree_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_GRAVEYARD_ISLAND {
        return false;
    }

    let spider_web_state = world.obj::<ControllerIslandFeatures>(this).graveyard_island_spider_web_state;
    spider_web_state == 1
}

/// `-[PocketGodViewController isLaserSharkEnabled]` @ 0x100087884
pub fn is_laser_shark_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let underwater_state = world.obj::<ControllerIslandFeatures>(this).underwater_state;
    if island_location_state != LOCATION_UNDERWATER || underwater_state != 2 {
        return false;
    }

    let chum_bucket = world.obj::<ControllerWorldRegistry>(this).chum_bucket_logic;
    let Some(chum_bucket) = chum_bucket else {
        return false;
    };
    let laser_shark = chum_bucket_logic::laser_shark_logic(world, chum_bucket);
    match laser_shark {
        Some(laser_shark) => laser_shark_logic::is_enabled(world, laser_shark),
        None => false,
    }
}

/// `-[PocketGodViewController isIceMonsterEnabled]` @ 0x1000878f0
pub fn is_ice_monster_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_ICE_ISLAND {
        return false;
    }

    let ice_monster_state = world.obj::<ControllerIslandFeatures>(this).ice_island_ice_monster_state;
    ice_monster_state == 1
}

/// `-[PocketGodViewController isGlacierEnabled]` @ 0x100087924
pub fn is_glacier_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_ICE_ISLAND
}

/// `-[PocketGodViewController isIceHoleEnabled]` @ 0x10008793c
pub fn is_ice_hole_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_ICE_ISLAND {
        return false;
    }

    let ice_hole_state = world.obj::<ControllerIslandFeatures>(this).ice_island_ice_hole_state;
    ice_hole_state == 1
}

/// `-[PocketGodViewController isIceSquidEnabled]` @ 0x100087970
pub fn is_ice_squid_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let features = world.obj::<ControllerIslandFeatures>(this);
    island_location_state == LOCATION_ICE_ISLAND
        && features.ice_island_ice_hole_state == 1
        && features.ice_island_ice_squid_state == 1
}

/// `-[PocketGodViewController isIglooEnabled]` @ 0x1000879b8
pub fn is_igloo_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_ICE_ISLAND {
        return false;
    }

    let igloo_state = world.obj::<ControllerIslandFeatures>(this).ice_island_igloo_state;
    igloo_state == 1
}

/// `-[PocketGodViewController isApeEnabled]` @ 0x1000879ec
pub fn is_ape_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_APE_ISLAND {
        return false;
    }

    let ape_state = world.obj::<ControllerIslandFeatures>(this).ape_island_ape_state;
    ape_state == 1
}

/// `-[PocketGodViewController isApeAltarEnabled]` @ 0x100087a20
pub fn is_ape_altar_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_APE_ISLAND {
        return false;
    }

    let ape_island_state = world.obj::<ControllerIslandFeatures>(this).ape_island_state;
    ape_island_state == 1
}

/// `-[PocketGodViewController isApeBongoEnabled]` @ 0x100087a54
pub fn is_ape_bongo_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_APE_ISLAND {
        return false;
    }

    let ape_island_state = world.obj::<ControllerIslandFeatures>(this).ape_island_state;
    ape_island_state == 1
}

/// `-[PocketGodViewController isApeWenchEnabled]` @ 0x100087a88
pub fn is_ape_wench_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_APE_ISLAND {
        return false;
    }

    let ape_island_state = world.obj::<ControllerIslandFeatures>(this).ape_island_state;
    ape_island_state == 1
}

/// `-[PocketGodViewController isApeTorchEnabled]` @ 0x100087abc
pub fn is_ape_torch_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_APE_ISLAND
}

/// `-[PocketGodViewController isBananaTreeEnabled]` @ 0x100087ad4
pub fn is_banana_tree_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_APE_ISLAND {
        return false;
    }

    let ape_island_state = world.obj::<ControllerIslandFeatures>(this).ape_island_state;
    ape_island_state == 2
}

/// `-[PocketGodViewController isMonkeyEnabled]` @ 0x100087b08
pub fn is_monkey_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_APE_ISLAND {
        return false;
    }

    let monkey_state = world.obj::<ControllerIslandFeatures>(this).ape_island_monkey_state;
    monkey_state == 1
}

/// `-[PocketGodViewController isMoronSnakeEnabled]` @ 0x100087b3c
pub fn is_moron_snake_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_MISFIT_ISLAND {
        return false;
    }

    let misfit_island_state = world.obj::<ControllerIslandFeatures>(this).misfit_island_state;
    misfit_island_state == 1
}

/// `-[PocketGodViewController isJellyfishEnabled]` @ 0x100087b70
pub fn is_jellyfish_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_UNDERWATER_CORAL_REEF {
        return false;
    }

    let coral_reef_state = world.obj::<ControllerIslandFeatures>(this).underwater_coral_reef_state;
    coral_reef_state == 1
}

/// `-[PocketGodViewController isIdolPedestalEnabled]` @ 0x100087ba4
pub fn is_idol_pedestal_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_GOD_ISLAND
}

/// `-[PocketGodViewController isTempleDoorEnabled]` @ 0x100087bbc
pub fn is_temple_door_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_GOD_ISLAND
}

/// `-[PocketGodViewController isGodIdolEnabled]` @ 0x100087bd4
pub fn is_god_idol_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_GOD_ISLAND {
        return false;
    }

    let idols_state = world.obj::<ControllerIslandFeatures>(this).god_island_idols_state;
    idols_state == 1
}

/// `-[PocketGodViewController isApocalypseGemPedestalEnabled]` @ 0x100087c08
pub fn is_apocalypse_gem_pedestal_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_APOCALYPSE_ISLAND
}

/// `-[PocketGodViewController isApocalypseTempleDoorEnabled]` @ 0x100087c20
pub fn is_apocalypse_temple_door_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_APOCALYPSE_ISLAND
}

/// `-[PocketGodViewController isApocalypsePortalTempleDoorEnabled]` @ 0x100087c38
pub fn is_apocalypse_portal_temple_door_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_APOCALYPSE_ISLAND {
        return false;
    }

    let calendar = world.obj::<ControllerWorldRegistry>(this).apocalypse_calendar_logic;
    match calendar {
        Some(calendar) => apocalypse_calendar_logic::is_open(world, calendar),
        None => false,
    }
}

/// `-[PocketGodViewController isTimeTempleDoorEnabled]` @ 0x100087c80
pub fn is_time_temple_door_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_TIME_TEMPLE
}

/// `-[PocketGodViewController isApocalypseTimeTempleLeverEnabled]` @ 0x100087c98
pub fn is_apocalypse_time_temple_lever_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_TIME_TEMPLE
}

/// `-[PocketGodViewController isApocalypseTimeTemplePlatformEnabled]` @ 0x100087cb0
pub fn is_apocalypse_time_temple_platform_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_TIME_TEMPLE
}

/// `-[PocketGodViewController isLocustQueenEnabled]` @ 0x100087cc8
pub fn is_locust_queen_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_LOCUST_TEMPLE
}

/// `-[PocketGodViewController isLocustTempleDoorEnabled]` @ 0x100087ce0
pub fn is_locust_temple_door_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_LOCUST_TEMPLE
}

/// `-[PocketGodViewController isGoGoCageEnabled]` @ 0x100087cf8
pub fn is_go_go_cage_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_DANCE_TEMPLE
}

/// `-[PocketGodViewController isDanceDanceExecutionEnabled]` @ 0x100087d10
pub fn is_dance_dance_execution_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_DANCE_TEMPLE
}

/// `-[PocketGodViewController isDanceTempleDoorEnabled]` @ 0x100087d28
pub fn is_dance_temple_door_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_DANCE_TEMPLE
}

/// `-[PocketGodViewController isDanceBalloonBundleEnabled]` @ 0x100087d40
pub fn is_dance_balloon_bundle_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_DANCE_TEMPLE
}

/// `-[PocketGodViewController isApocalypsePlagueTempleFountainEnabled]` @ 0x100087d58
pub fn is_apocalypse_plague_temple_fountain_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_PLAGUE_TEMPLE
}

/// `-[PocketGodViewController isPlagueTempleDoorEnabled]` @ 0x100087d70
pub fn is_plague_temple_door_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_PLAGUE_TEMPLE
}

/// `-[PocketGodViewController isApocalypsePurificationTempleFountainEnabled]` @ 0x100087d88
pub fn is_apocalypse_purification_temple_fountain_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_PURIFICATION_TEMPLE
}

/// `-[PocketGodViewController isPurificationTempleDoorEnabled]` @ 0x100087da0
pub fn is_purification_temple_door_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_PURIFICATION_TEMPLE
}

/// `-[PocketGodViewController isClosetTempleDoorEnabled]` @ 0x100087db8
pub fn is_closet_temple_door_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_CLOSET_TEMPLE
}

/// `-[PocketGodViewController isClosetTempleThetanMeterEnabled]` @ 0x100087dd0
pub fn is_closet_temple_thetan_meter_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_CLOSET_TEMPLE
}

/// `-[PocketGodViewController isClosetTempleCouchEnabled]` @ 0x100087de8
pub fn is_closet_temple_couch_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_CLOSET_TEMPLE
}

/// `-[PocketGodViewController isClosetTempleClosetDoorEnabled]` @ 0x100087e00
pub fn is_closet_temple_closet_door_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_CLOSET_TEMPLE
}

/// `-[PocketGodViewController isXenuTempleDoorEnabled]` @ 0x100088080
pub fn is_xenu_temple_door_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_XENU_TEMPLE
}

/// `-[PocketGodViewController isXenuTempleEnabled]` @ 0x100088098
pub fn is_xenu_temple_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state == LOCATION_XENU_TEMPLE
}

/// `-[PocketGodViewController isXenuEnabled]` @ 0x1000880b0
pub fn is_xenu_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_XENU_TEMPLE {
        return false;
    }

    let xenu = world.obj::<ControllerWorldRegistry>(this).xenu_logic;
    match xenu {
        Some(xenu) => xenu_logic::is_alive(world, xenu),
        None => false,
    }
}

/// `-[PocketGodViewController isKongStatueEnabled]` @ 0x1000880f8
pub fn is_kong_statue_enabled(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_APE_ISLAND {
        return false;
    }

    let ape_island_state = world.obj::<ControllerIslandFeatures>(this).ape_island_state;
    ape_island_state == 2
}

/// `-[PocketGodViewController isKonkeyDongEnabled]` @ 0x10008812c
pub fn is_konkey_dong_enabled(world: &mut World, this: Entity) -> bool {
    let konkey_dong_state = option_int_preference(world, this, "KonkeyDongState");
    konkey_dong_state == 1
}

/// `-[PocketGodViewController isMoronPestsEnabled]` @ 0x100088168
pub fn is_moron_pests_enabled(world: &mut World, this: Entity) -> bool {
    let moron_pests_state = option_int_preference(world, this, "MoronPestsState");
    moron_pests_state == 1
}

/// `-[PocketGodViewController (Game) disableIslandLogics]` @ 0x100165b9c
pub fn disable_island_logics(world: &mut World, this: Entity) {
    if is_t_rex_active(world, this) {
        let t_rex = world.obj::<ControllerWorldRegistry>(this).t_rex_logic;
        if let Some(t_rex) = t_rex {
            t_rex_logic::exit_island(world, t_rex);
        }
    }

    if is_spider_active(world, this) {
        let spider = world.obj::<ControllerWorldRegistry>(this).spider_logic;
        if let Some(spider) = spider {
            spider_logic::exit_island(world, spider);
        }
    }

    if is_ice_monster_enabled(world, this) {
        let ice_monster = world.obj::<ControllerWorldRegistry>(this).ice_monster_logic;
        if let Some(ice_monster) = ice_monster {
            ice_monster_logic::exit_island(world, ice_monster, true);
        }
    }

    if is_ant_hill_enabled(world, this) {
        let ant_hill = world.obj::<ControllerWorldRegistry>(this).ant_hill_logic;
        if let Some(ant_hill) = ant_hill {
            ant_hill_logic::exit_island_now(world, ant_hill);
        }
    }

    if is_bird_enabled(world, this) {
        let bird = world.obj::<ControllerWorldRegistry>(this).bird_logic;
        if let Some(bird) = bird {
            bird_logic::exit_island_now(world, bird);
        }
    }

    if is_shark_enabled(world, this) {
        let shark = world.obj::<ControllerWorldRegistry>(this).shark_logic;
        if let Some(shark) = shark {
            shark_logic::exit_island_now(world, shark);
        }
    }

    if is_banana_tree_enabled(world, this) {
        let banana = world.obj::<ControllerWorldRegistry>(this).banana_logic;
        if let Some(banana) = banana {
            banana_logic::exit_island(world, banana);
        }
    }

    if is_dance_balloon_bundle_enabled(world, this) {
        let dance_balloon = world.obj::<ControllerWorldRegistry>(this).dance_balloon_logic;
        if let Some(dance_balloon) = dance_balloon {
            dance_balloon_logic::exit_island(world, dance_balloon);
        }
    }

    if is_monkey_enabled(world, this) {
        let monkeys = world.obj::<ControllerWorldRegistry>(this).monkey_logic_array.clone();
        for monkey in monkeys {
            crate::send::exit_island_now(world, Some(monkey));
        }
    }

    if is_moron_snake_enabled(world, this) {
        let moron_snake = world.obj::<ControllerWorldRegistry>(this).moron_snake_logic;
        if let Some(moron_snake) = moron_snake {
            moron_snake_logic::exit_island_now(world, moron_snake);
        }
    }

    if is_jellyfish_enabled(world, this) {
        let jellyfish_array = world.obj::<ControllerWorldRegistry>(this).jellyfish_logic_array.clone();
        for jellyfish in jellyfish_array {
            crate::send::exit_island_now(world, Some(jellyfish));
        }
    }

    if is_god_idol_enabled(world, this) {
        let god_idols = world.obj::<ControllerWorldRegistry>(this).god_idol_logic_array.clone();
        for god_idol in god_idols {
            crate::send::exit_island_now(world, Some(god_idol));
        }
    }
}

/// `-[PocketGodViewController (Game) enableIslandLogics]` @ 0x100166068
pub fn enable_island_logics(world: &mut World, this: Entity) {
    if is_ant_hill_enabled(world, this) {
        let ant_hill = world.obj::<ControllerWorldRegistry>(this).ant_hill_logic;
        if let Some(ant_hill) = ant_hill {
            ant_hill_logic::enter_island(world, ant_hill, false);
        }
    }

    if is_ice_monster_enabled(world, this) {
        let ice_monster = world.obj::<ControllerWorldRegistry>(this).ice_monster_logic;
        if let Some(ice_monster) = ice_monster {
            ice_monster_logic::enter_island(world, ice_monster, false);
        }
    }

    if is_bird_enabled(world, this) {
        let bird = world.obj::<ControllerWorldRegistry>(this).bird_logic;
        if let Some(bird) = bird {
            bird_logic::enter_island(world, bird, false);
        }
    }

    if is_shark_enabled(world, this) {
        let shark = world.obj::<ControllerWorldRegistry>(this).shark_logic;
        if let Some(shark) = shark {
            shark_logic::enter_island(world, shark, false);
        }
    }

    if is_moron_snake_enabled(world, this) {
        let moron_snake = world.obj::<ControllerWorldRegistry>(this).moron_snake_logic;
        if let Some(moron_snake) = moron_snake {
            moron_snake_logic::enter_island(world, moron_snake, false);
        }
    }

    if is_god_idol_enabled(world, this) {
        let god_idols = world.obj::<ControllerWorldRegistry>(this).god_idol_logic_array.clone();
        for god_idol in god_idols {
            crate::send::enter_island_with_arg(world, Some(god_idol), false);
        }
    }

    stop_celebration(world, this);

    let earthquake_sequence = world.obj::<ControllerGodPowers>(this).earthquake_sequence;
    if earthquake_sequence.is_some() {
        on_story_mode_stop_earthquake(world, this);
    }
}
