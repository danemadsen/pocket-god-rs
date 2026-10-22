//! `PocketGodViewController`: methods in this part of the class.
//!
//! The god powers drawn or triggered with the finger (lightning, double rainbow, locust swarm,
//! hurricane, whirlpool, coral-reef temple power), the drum and the celebration dance with its
//! songs and fireworks, earthquakes and the volcano.

use std::f32::consts::PI;
use std::ops::RangeInclusive;

use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::math::length3;
use pg_engine::sequence;
use pg_engine::sound;
use pg_engine::texture::Textures;
use pg_engine::touch::BcTouch;
use pg_engine::view::BcView;

use crate::logic::ape_monkeys::monkey_logic;
use crate::logic::ape_ritual::{ape_bongo_logic, ape_logic, ape_torch_logic};
use crate::logic::apocalypse_locust_closet_xenu::locust_logic;
use crate::logic::island_core::{bon_fire_logic, coconut_logic};
use crate::logic::island_objects::volcano_logic;
use crate::logic::ocean_birds::bird_logic;
use crate::logic::sky_powers::rainbow_model::RainbowModel;
use crate::logic::sky_powers::{
    finger_trail_logic, hurricane_logic, hurricane_model, lightning_logic, lightning_model, rainbow_logic,
    whirl_pool_logic, whirl_pool_model,
};
use crate::logic::sky_weather::firework_logic;
use crate::logic::underwater_stage::underwater_temple_logic;
use crate::managers::{challenge_manager, idol_manager, skin_pack_manager};
use crate::prelude::*;

use super::*;

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerGodPowers {
    /// `mFireworkDisplayMarker`: `@"BCDisplayMarker"`
    pub firework_display_marker: Option<Entity>,

    /// `mFireworkLogicArray`: `@"NSMutableArray"`
    pub firework_logic_array: Vec<Entity>,

    /// `mUnderwaterPowerTrailLogicArray`: `@"NSMutableArray"`
    pub underwater_power_trail_logic_array: Vec<Entity>,

    /// `mTapTouchCount`: `i`
    pub tap_touch_count: i32,

    /// `mTapTouchTime`: `f`
    pub tap_touch_time: f32,

    /// `mbCelebrationActive`: `B`
    pub celebration_active: bool,

    /// `mCelebrationSequence`: `@"BCSequence"`
    pub celebration_sequence: Option<Entity>,

    /// `mFireworksStallTime`: `f`
    pub fireworks_stall_time: f32,

    /// `mbThrillerActive`: `B`
    pub thriller_active: bool,

    /// `mbPantsOnTheGroundActive`: `B`
    pub pants_on_the_ground_active: bool,

    /// `mbUpdateSongActive`: `B`
    pub update_song_active: bool,

    /// `mbThemeSongActive`: `B`
    pub theme_song_active: bool,

    /// `mbFishmasActive`: `B`
    pub fishmas_active: bool,

    /// `mbFishmasTwelveDaysActive`: `B`
    pub fishmas_twelve_days_active: bool,

    /// `mbEruption`: `B`
    pub eruption: bool,

    /// `mVolcanoHitCount`: `i`
    pub volcano_hit_count: i32,

    /// `mVolcanoLogic`: `@"VolcanoLogic"`
    pub volcano_logic: Option<Entity>,

    /// `mEarthquakeCount`: `i`
    pub earthquake_count: i32,

    /// `mEarthquakeSequence`: `@"BCSequence"`
    pub earthquake_sequence: Option<Entity>,

    /// `mShakeStallTime`: `f`
    pub shake_stall_time: f32,

    /// `mShakeRotVel`: `f`
    pub shake_rot_vel: f32,

    /// `mShakeRot`: `f`
    pub shake_rot: f32,

    /// `mHurricaneTouch`: `@"BCTouch"`
    pub hurricane_touch: Option<Entity>,

    /// `mLastHurricanePos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_hurricane_pos: Vec4,

    /// `mLastHurricaneTouchTime`: `f`
    pub last_hurricane_touch_time: f32,

    /// `mpHurricaneList`: `^{list<VECTOR4, std::__1::allocator<VECTOR4> >={__list_node_base<VECTOR4, void *>=^{__list_node<VECTOR4, void *>}^{__list_node<VECTOR4, void *>}}{__compressed_pair<unsigned long, std::__1::allocator<std::__1::__list_node<VECTOR4, void *> > >=Q}}`
    pub hurricane_list: Vec<Vec4>,

    /// `mHurricaneLogic`: `@"HurricaneLogic"`
    pub hurricane_logic: Option<Entity>,

    /// `mWhirlPoolTouch`: `@"BCTouch"`
    pub whirl_pool_touch: Option<Entity>,

    /// `mpWhirlPoolList`: `^{list<VECTOR4, std::__1::allocator<VECTOR4> >={__list_node_base<VECTOR4, void *>=^{__list_node<VECTOR4, void *>}^{__list_node<VECTOR4, void *>}}{__compressed_pair<unsigned long, std::__1::allocator<std::__1::__list_node<VECTOR4, void *> > >=Q}}`
    pub whirl_pool_list: Vec<Vec4>,

    /// `mWhirlPoolLogic`: `@"WhirlPoolLogic"`
    pub whirl_pool_logic: Option<Entity>,

    /// `mUnderwaterPowerTouch`: `@"BCTouch"`
    pub underwater_power_touch: Option<Entity>,

    /// `mLastUnderwaterTempleMode`: `i`
    pub last_underwater_temple_mode: i32,

    /// `mLastUnderwaterPowerPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_underwater_power_pos: Vec4,

    /// `mbShockObject`: `B`
    pub shock_object: bool,

    /// `mbDrawLightning`: `B`
    pub draw_lightning: bool,

    /// `mLightningTouch`: `@"BCTouch"`
    pub lightning_touch: Option<Entity>,

    /// `mLastLightningPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_lightning_pos: Vec4,

    /// `mpLightningList`: `^{list<VECTOR4, std::__1::allocator<VECTOR4> >={__list_node_base<VECTOR4, void *>=^{__list_node<VECTOR4, void *>}^{__list_node<VECTOR4, void *>}}{__compressed_pair<unsigned long, std::__1::allocator<std::__1::__list_node<VECTOR4, void *> > >=Q}}`
    pub lightning_list: Vec<Vec4>,

    /// `mLightningLogic`: `@"LightningLogic"`
    pub lightning_logic: Option<Entity>,

    /// `mLightningTrailLogic`: `@"FingerTrailLogic"`
    pub lightning_trail_logic: Option<Entity>,

    /// `mHurricaneTrailLogic`: `@"FingerTrailLogic"`
    pub hurricane_trail_logic: Option<Entity>,

    /// `mWhirlPoolTrailLogic`: `@"FingerTrailLogic"`
    pub whirl_pool_trail_logic: Option<Entity>,

    /// `mUnderwaterPowerTrailLogic`: `@"FingerTrailLogic"`
    pub underwater_power_trail_logic: Option<Entity>,

    /// `mbDrawRainbow`: `B`
    pub draw_rainbow: bool,

    /// `mRainbowTouch`: `@"BCTouch"`
    pub rainbow_touch: Option<Entity>,

    /// `mLastRainbowPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_rainbow_pos: Vec4,

    /// `mpRainbowList`: `^{list<VECTOR4, std::__1::allocator<VECTOR4> >={__list_node_base<VECTOR4, void *>=^{__list_node<VECTOR4, void *>}^{__list_node<VECTOR4, void *>}}{__compressed_pair<unsigned long, std::__1::allocator<std::__1::__list_node<VECTOR4, void *> > >=Q}}`
    ///
    /// The original shares this one list with `mRainbowLogic`'s `RainbowModel`. The port keeps
    /// the points in that model's `RainbowModel::path_list` (where `RainbowLogic` clears, copies
    /// and replays them); the rainbow gesture appends there, so this field stays empty.
    pub rainbow_list: Vec<Vec4>,

    /// `mRainbowLogic`: `@"RainbowLogic"`
    pub rainbow_logic: Option<Entity>,

    /// `mRainbowTrailLogic`: `@"FingerTrailLogic"`
    pub rainbow_trail_logic: Option<Entity>,

    /// `mbDrawLocust`: `B`
    pub draw_locust: bool,

    /// `mLocustTouch`: `@"BCTouch"`
    pub locust_touch: Option<Entity>,

    /// `mLastLocustPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_locust_pos: Vec4,

    /// `mpLocustVector`: `^{vector<VECTOR4, std::__1::allocator<VECTOR4> >=^{?}^{?}{__compressed_pair<VECTOR4 *, std::__1::allocator<VECTOR4> >=^{?}}}`
    pub locust_vector: Vec<Vec4>,

    /// `mLocustLogic`: `@"LocustLogic"`
    pub locust_logic: Option<Entity>,
}

/// `mIslandLocationState` of Graveyard island (zombies dance to Thriller, the fireworks are
/// red).
const LOCATION_GRAVEYARD_ISLAND: i32 = 3;

/// `mIslandLocationState` of the underwater ruins.
const LOCATION_UNDERWATER: i32 = 2;

/// `mIslandLocationState` of the underwater coral reef (the temple power's location).
const LOCATION_UNDERWATER_CORAL_REEF: i32 = 9;

/// `mIslandLocationState` of the seven apocalypse temples (Time … Xenu).
const TEMPLE_LOCATIONS: RangeInclusive<i32> = 11..=17;

/// `mWeatherState` while the storm clouds are out.
const WEATHER_STORM: i32 = 1;

/// The `ChallengeManager` game type of the hurricane-lightning challenge.
const HURRICANE_LIGHTNING_CHALLENGE: i32 = 1;

/// Option document key of the drum god power.
const DRUM_HAZARD_OPTION: &str = "DrumHazard";

/// `mTapTouchCount` value whose tap starts the celebration (from the fourth tap on, each tap
/// plays a drum).
const CELEBRATION_TAP_COUNT: i32 = 4;

/// `mShakeStallTime` of a background shake without a time limit.
const SHAKE_FOREVER: f32 = -1000.0;

/// The `minTime`/`maxTime` scale of the random stall times (milliseconds).
const STALL_TIME_SCALE: f32 = 1000.0;

/// A finger stroke adds a lightning point after moving this far from the last one.
const LIGHTNING_POINT_DISTANCE: f32 = 20.0;

/// A rainbow stroke starts once the finger moved this far from where it went down.
const RAINBOW_START_DISTANCE: f32 = 20.0;

/// A rainbow stroke adds a point after moving this far from the last one.
const RAINBOW_POINT_DISTANCE: f32 = 5.0;

/// A locust stroke adds a point after moving this far from the last one.
const LOCUST_POINT_DISTANCE: f32 = 5.0;

/// Releasing the locust finger outside this box (island coordinates) scatters the swarm.
const LOCUST_RELEASE_MIN_X_POS: f32 = 10.0;
const LOCUST_RELEASE_MAX_X_POS: f32 = 470.0;
const LOCUST_RELEASE_MIN_Y_POS: f32 = 10.0;
const LOCUST_RELEASE_MAX_Y_POS: f32 = 310.0;

/// The coral-reef power leaves a trail puff every time the finger moved this far.
const UNDERWATER_POWER_TRAIL_DISTANCE: f32 = 30.0;

/// Pygmies within this distance of the finger feel the coral-reef power …
const UNDERWATER_POWER_TOUCH_DISTANCE: f32 = 30.0;

/// … measured from this far above their position.
const UNDERWATER_POWER_TOUCH_Y_OFFSET: f32 = 20.0;

/// A warlock's bolt starts at this height and comes down to the pygmy in seven points.
const WARLOCK_LIGHTNING_TOP_Y_POS: f32 = 300.0;
const WARLOCK_LIGHTNING_POINT_COUNT: i32 = 7;

/// The texture whose frame size centres the lightning finger glow.
const ELECTRIC_GLOW_TEXTURE_ID: &str = "ElectricGlow_01";

/// The texture whose frame size centres the hurricane and whirlpool finger glow.
const WIND_FINGER_TEXTURE_ID: &str = "WindFinger_01";

/// Model id and behaviour of the lightning finger glow.
const ELECTRIC_GLOW: &str = "ElectricGlow";

/// Model id and behaviour of the hurricane and whirlpool finger glow.
const WIND_GLOW: &str = "WindGlow";

/// Model id and behaviour of the rainbow finger sparkle.
const RAINBOW_TRAIL: &str = "RainbowTrail";

/// Model id of the coral-reef power finger glow.
const UNDERWATER_POWER_MODEL_ID: &str = "UnderwaterPower";

/// Model id of the coral-reef power trail puffs.
const UNDERWATER_POWER_TRAIL_MODEL_ID: &str = "UnderwaterPowerTrail";

/// The event a trail puff's behaviour sends when its animation ends.
const UNDERWATER_POWER_TRAIL_COMPLETE_EVENT_ID: &str = "UnderwaterPowerTrailComplete";

/// The coral-reef power goes before this marker of `UnderwaterDisplayGroup`.
const UNDERWATER_PYGMY_DISPLAY_MARKER_ID: &str = "UnderwaterPygmyDisplayMarker";

/// Texture and model id of a firework.
const FIREWORK_SPARK_TEXTURE_ID: &str = "FireworkSpark";
const FIREWORK_MODEL_ID: &str = "Firework";

/// A firework's model is centred on its position.
const FIREWORK_ORIGIN_POS: f32 = -8.0;

/// Firework colours on Graveyard island (`arc4random() % 5`): shades of red.
const GRAVEYARD_FIREWORK_COLORS: [(f32, f32, f32); 5] = [
    (1.0, 0.0, 0.0),
    (1.0, 0.2, 0.2),
    (0.8, 0.0, 0.0),
    (0.6, 0.0, 0.0),
    (0.4, 0.0, 0.0),
];

/// Firework colours elsewhere (`arc4random() & 7`).
const FIREWORK_COLORS: [(f32, f32, f32); 8] = [
    (1.0, 0.8, 0.0),
    (1.0, 0.2, 0.0),
    (1.0, 0.6, 0.2),
    (0.0, 1.0, 1.0),
    (1.0, 1.0, 0.8),
    (0.6, 1.0, 0.0),
    (0.4, 0.6, 1.0),
    (1.0, 0.4, 1.0),
];

/// One value of a skin pack option
/// (`isSkinPackOptionActiveBySkinPackId:skinObjectId:skinOptionId:skinOptionValueId:`).
struct SkinOptionValue {
    skin_pack_id: &'static str,
    skin_object_id: &'static str,
    skin_option_id: &'static str,
    skin_option_value_id: &'static str,
}

const PANTS_ON_THE_GROUND: SkinOptionValue = SkinOptionValue {
    skin_pack_id: "Dance",
    skin_object_id: "Pygmies",
    skin_option_id: "Pygmy Dance",
    skin_option_value_id: "Pants on the Ground",
};

const UPDATE_SONG: SkinOptionValue = SkinOptionValue {
    skin_pack_id: "Dance2",
    skin_object_id: "Pygmies",
    skin_option_id: "Pygmy Dance",
    skin_option_value_id: "Pocket God Update Song",
};

const THEME_SONG: SkinOptionValue = SkinOptionValue {
    skin_pack_id: "Dance3",
    skin_object_id: "Pygmies",
    skin_option_id: "Pygmy Dance",
    skin_option_value_id: "Pygmy Theme Song",
};

const NAPOLEON_DYNAMITE: SkinOptionValue = SkinOptionValue {
    skin_pack_id: "Dance2",
    skin_object_id: "Pygmies",
    skin_option_id: "Pygmy Dance",
    skin_option_value_id: "Napoleon Dynamite",
};

const FISHMAS_DANCE: SkinOptionValue = SkinOptionValue {
    skin_pack_id: "Fishmas",
    skin_object_id: "Pygmies",
    skin_option_id: "Pygmy Dance",
    skin_option_value_id: "Fishmas Dance",
};

const TWELVE_DAYS_OF_FISHMAS: SkinOptionValue = SkinOptionValue {
    skin_pack_id: "Fishmas",
    skin_object_id: "Pygmies",
    skin_option_id: "Pygmy Dance",
    skin_option_value_id: "12 Days of Fishmas",
};

const VAMPIRE_DANCE_LEAD: SkinOptionValue = SkinOptionValue {
    skin_pack_id: "Dance",
    skin_object_id: "The Vampire",
    skin_option_id: "Vampire Dance",
    skin_option_value_id: "Staying Alive (Lead)",
};

/// `-[PocketGodViewController canWarlockUseLightning:]` @ 0x100087184
///
/// Warlocks call lightning only in storm weather, and not inside a temple.
pub fn can_warlock_use_lightning(world: &mut World, this: Entity, _pygmy_logic: Entity) -> bool {
    let weather_state = world.obj::<ControllerSkyWeather>(this).weather_state;
    if weather_state != WEATHER_STORM {
        return false;
    }
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    !TEMPLE_LOCATIONS.contains(&island_location_state)
}

/// `-[PocketGodViewController isCelebrationEnabled]` @ 0x100087348
pub fn is_celebration_enabled(world: &mut World, this: Entity) -> bool {
    if option_int_preference(world, this, DRUM_HAZARD_OPTION) != 1 {
        return false;
    }
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    island_location_state != LOCATION_UNDERWATER_CORAL_REEF && island_location_state != LOCATION_UNDERWATER
}

/// `-[PocketGodViewController (Game) zombieIsPresent]` @ 0x100157a44
pub fn zombie_is_present(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_GRAVEYARD_ISLAND {
        return false;
    }

    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        if crate::pygmy::is_zombie(world, pygmy) {
            return true;
        }
    }
    false
}

/// `-[PocketGodViewController (Game) canDoNormalDance]` @ 0x100157b84
///
/// Whether some pygmy can do the normal dance (and so dance to a song).
pub fn can_do_normal_dance(world: &mut World, this: Entity) -> bool {
    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        if crate::pygmy::can_do_normal_dance(world, pygmy) {
            return true;
        }
    }
    false
}

/// `-[PocketGodViewController (Game) startCelebration]` @ 0x100157ca4
///
/// Picks the dance song from the active skin options (or Thriller with a zombie on Graveyard
/// island), gets the ape and the monkeys going and runs `StartCelebration` on `mSequence`.
pub fn start_celebration(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerGodPowers>(this).celebration_active = true;

    let dance_song = if can_do_normal_dance(world, this) && is_skin_option_active(world, this, &PANTS_ON_THE_GROUND) {
        Some(DanceSong::PantsOnTheGround)
    } else if can_do_normal_dance(world, this) && is_skin_option_active(world, this, &UPDATE_SONG) {
        Some(DanceSong::UpdateSong)
    } else if can_do_normal_dance(world, this) && is_skin_option_active(world, this, &THEME_SONG) {
        Some(DanceSong::ThemeSong)
    } else if can_do_normal_dance(world, this) && is_skin_option_active(world, this, &FISHMAS_DANCE) {
        Some(DanceSong::Fishmas)
    } else if can_do_normal_dance(world, this) && is_skin_option_active(world, this, &TWELVE_DAYS_OF_FISHMAS) {
        Some(DanceSong::FishmasTwelveDays)
    } else if zombie_is_present(world, this) {
        Some(DanceSong::Thriller)
    } else {
        None
    };

    // Of the songs, only Thriller is started here.
    if let Some(dance_song) = dance_song {
        set_dance_song_active(world, this, dance_song);
        app_platform::stop_background_loop(world, this);
        if dance_song == DanceSong::Thriller {
            sound::play_sound(world, "ThrillerLoop");
        }
    }

    if island_features::is_ape_bongo_enabled(world, this) {
        let ape_bongo = world.obj::<ControllerWorldRegistry>(this).ape_bongo_logic;
        if let Some(ape_bongo) = ape_bongo {
            ape_bongo_logic::start_playing(world, ape_bongo);
        }
    }
    if island_features::is_ape_enabled(world, this) {
        let ape = world.obj::<ControllerWorldRegistry>(this).ape_logic;
        if let Some(ape) = ape {
            ape_logic::start_tapping(world, ape);
        }
    }

    let monkeys = world.obj::<ControllerWorldRegistry>(this).monkey_logic_array.clone();
    for monkey in monkeys {
        monkey_logic::start_dancing(world, monkey);
    }

    let main_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
    set_sequence_def(world, main_sequence, "StartCelebration");
}

/// `-[PocketGodViewController (Game) stopCelebrationMusic]` @ 0x100158120
///
/// Stops the dance song that is playing and brings the background loop back (unless a
/// minigame owns the music).
pub fn stop_celebration_music(world: &mut World, this: Entity) {
    let dance_song_sound = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        god_powers.celebration_active = false;
        if god_powers.pants_on_the_ground_active {
            god_powers.pants_on_the_ground_active = false;
            "PantsOnTheGround"
        } else if god_powers.update_song_active {
            god_powers.update_song_active = false;
            "UpdateSong"
        } else if god_powers.theme_song_active {
            god_powers.theme_song_active = false;
            "ThemeSong"
        } else if god_powers.fishmas_active {
            god_powers.fishmas_active = false;
            "MerryFishmas"
        } else if god_powers.fishmas_twelve_days_active {
            god_powers.fishmas_twelve_days_active = false;
            "TwelveDays"
        } else if god_powers.thriller_active {
            god_powers.thriller_active = false;
            "ThrillerLoop"
        } else {
            return;
        }
    };
    sound::stop_sound(world, dance_song_sound);

    let minigames = world.obj::<ControllerMinigames>(this);
    let minigame_active = minigames.the_runs
        || minigames.konkey_dong
        || minigames.moron_pests
        || minigames.no_pee
        || minigames.dance_dance_execution
        || minigames.battle_of_the_gods;
    if !minigame_active {
        app_platform::play_background_loop(world, this, None);
    }
}

/// `-[PocketGodViewController (Game) stopCelebration]` @ 0x1001583a4
///
/// Called every frame once the drum has been quiet for a second: does nothing unless a
/// celebration runs; otherwise ends the music, the ape and the monkeys and runs
/// `StopCelebration` on `mCelebrationSequence`.
pub fn stop_celebration(world: &mut World, this: Entity) {
    let celebration_active = world.obj::<ControllerGodPowers>(this).celebration_active;
    if !celebration_active {
        return;
    }
    world.obj_mut::<ControllerGodPowers>(this).celebration_active = false;
    stop_celebration_music(world, this);

    if island_features::is_ape_bongo_enabled(world, this) {
        let ape_bongo = world.obj::<ControllerWorldRegistry>(this).ape_bongo_logic;
        if let Some(ape_bongo) = ape_bongo {
            ape_bongo_logic::stop_playing(world, ape_bongo);
        }
    }
    if island_features::is_ape_enabled(world, this) {
        let ape = world.obj::<ControllerWorldRegistry>(this).ape_logic;
        if let Some(ape) = ape {
            ape_logic::stop_tapping(world, ape);
        }
    }

    let monkeys = world.obj::<ControllerWorldRegistry>(this).monkey_logic_array.clone();
    for monkey in monkeys {
        monkey_logic::stop_dancing(world, monkey);
    }

    let celebration_sequence = world.obj::<ControllerGodPowers>(this).celebration_sequence;
    set_sequence_def(world, celebration_sequence, "StopCelebration");
}

/// `-[PocketGodViewController (Game) startPygmiesDancing:::]` @ 0x10015a13c
///
/// Vampires do the vampire dance with the Staying Alive option; the other pygmies dance, the
/// first able one leading the theme song and the others backing it. Then a new
/// `mCelebrationSequence` runs `Celebration` (the fireworks).
pub fn seq_start_pygmies_dancing(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let vampire_dance = is_skin_option_active(world, this, &VAMPIRE_DANCE_LEAD);
    let theme_song = is_skin_option_active(world, this, &THEME_SONG);

    let mut theme_song_lead_chosen = false;
    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        if vampire_dance && crate::pygmy::is_vampire(world, pygmy) {
            crate::pygmy::start_vampire_dancing(world, pygmy);
            continue;
        }

        let sings_theme_song = theme_song && crate::pygmy::can_do_normal_dance(world, pygmy) && !crate::pygmy::is_zombie(world, pygmy);
        if !sings_theme_song {
            crate::pygmy::start_dancing(world, pygmy, false);
            continue;
        }

        let theme_song_backup = theme_song_lead_chosen;
        theme_song_lead_chosen = true;
        crate::pygmy::start_dancing(world, pygmy, theme_song_backup);
    }

    let old_celebration_sequence = world.obj::<ControllerGodPowers>(this).celebration_sequence;
    release_sequence(world, old_celebration_sequence, call.sequence);
    world.obj_mut::<ControllerGodPowers>(this).celebration_sequence = None;

    let celebration_def = library::sequence_def(world, "Celebration");
    let celebration_sequence = sequence::spawn(world, celebration_def, Some(this));
    world.obj_mut::<ControllerGodPowers>(this).celebration_sequence = Some(celebration_sequence);
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) stopPygmiesDancing:::]` @ 0x10015a454
///
/// Every pygmy stops dancing (to whichever songs are active; vampires to none of them), at once
/// while the skin store is open; `mCelebrationSequence` ends.
pub fn seq_stop_pygmies_dancing(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let active_songs = DanceSongOptions {
        pants_dance: is_skin_option_active(world, this, &PANTS_ON_THE_GROUND),
        update_song: is_skin_option_active(world, this, &UPDATE_SONG),
        theme_song: is_skin_option_active(world, this, &THEME_SONG),
        napoleon_dynamite: is_skin_option_active(world, this, &NAPOLEON_DYNAMITE),
        fishmas: is_skin_option_active(world, this, &FISHMAS_DANCE),
        fishmas_twelve_days: is_skin_option_active(world, this, &TWELVE_DAYS_OF_FISHMAS),
    };
    let vampire_dance = is_skin_option_active(world, this, &VAMPIRE_DANCE_LEAD);

    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        let songs = if vampire_dance && crate::pygmy::is_vampire(world, pygmy) {
            DanceSongOptions::default()
        } else {
            active_songs
        };
        crate::pygmy::stop_dancing(
            world,
            pygmy,
            songs.pants_dance,
            songs.update_song,
            songs.theme_song,
            songs.napoleon_dynamite,
            songs.fishmas,
            songs.fishmas_twelve_days,
        );

        let skin_pack_screen_open = world.obj::<ControllerManagersBridge>(this).skin_pack_screen_open;
        if skin_pack_screen_open {
            crate::pygmy::stop_dancing_now(world, pygmy);
        }
    }

    let celebration_sequence = world.obj::<ControllerGodPowers>(this).celebration_sequence;
    release_sequence(world, celebration_sequence, call.sequence);
    world.obj_mut::<ControllerGodPowers>(this).celebration_sequence = None;
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) fireworks:::]` @ 0x10015a7c8
///
/// `[0] minTime`, `[1] maxTime`: never completes (outside the temples, where it completes at
/// once). Waits a random time in [min, max], then launches a firework somewhere in the sky.
pub fn seq_fireworks(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if TEMPLE_LOCATIONS.contains(&island_location_state) {
        return ItemResult::Done;
    }

    if call.first_time {
        world.obj_mut::<ControllerGodPowers>(this).fireworks_stall_time = 0.0;
    }

    let mut fireworks_stall_time = world.obj::<ControllerGodPowers>(this).fireworks_stall_time;
    if fireworks_stall_time == 0.0 {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        fireworks_stall_time = logic::random_scaled_time(min_time, max_time, STALL_TIME_SCALE, random);
        world.obj_mut::<ControllerGodPowers>(this).fireworks_stall_time = fireworks_stall_time;
    }

    if fireworks_stall_time > 0.0 {
        world.obj_mut::<ControllerGodPowers>(this).fireworks_stall_time = fireworks_stall_time - call.delta_time;
        return ItemResult::Pending;
    }

    world.obj_mut::<ControllerGodPowers>(this).fireworks_stall_time = 0.0;
    launch_firework(world, this);
    ItemResult::Pending
}

/// `-[PocketGodViewController (Game) shakeBackground:::]` @ 0x10015e374
///
/// `[0] minTime`, `[1] maxTime`: rocks the island (or The Runs) up and down, for a random time
/// in [min, max], or forever without them.
pub fn seq_shake_background(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let mut shake_stall_time = SHAKE_FOREVER;
        if call.param("minTime").is_some() {
            let min_time = call.float("minTime");
            let max_time = call.float("maxTime");
            let random = world.resource_mut::<Random>().arc4random();
            shake_stall_time = logic::random_scaled_time(min_time, max_time, STALL_TIME_SCALE, random);
        }
        world.obj_mut::<ControllerGodPowers>(this).shake_stall_time = shake_stall_time;
    }

    let shake_stall_time = world.obj::<ControllerGodPowers>(this).shake_stall_time;
    let shaking = shake_stall_time > 0.0 || shake_stall_time == SHAKE_FOREVER;
    if !shaking {
        return ItemResult::Done;
    }

    let y_pos = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        let shake_rot = god_powers.shake_rot;
        let wave = shake_rot.sin();
        god_powers.shake_rot = shake_rot + god_powers.shake_rot_vel;
        wave + wave + -2.0
    };

    let the_runs = world.obj::<ControllerMinigames>(this).the_runs;
    let shaken_display_group = if the_runs {
        world.obj::<ControllerMinigames>(this).the_runs_display_group
    } else {
        world.obj::<ControllerIslandLocations>(this).island_display_group
    };
    if let Some(shaken_display_group) = shaken_display_group {
        display::object_mut(world, shaken_display_group).pos.y = y_pos;
    }

    if shake_stall_time != SHAKE_FOREVER {
        world.obj_mut::<ControllerGodPowers>(this).shake_stall_time = shake_stall_time - call.delta_time;
    }
    ItemResult::Pending
}

/// `-[PocketGodViewController (Game) stopEarthquake:::]` @ 0x10015e524
///
/// The end of `EarthquakeStop`: `mEarthquakeSequence` ends, and starts over with
/// `EarthquakeStart` while some pygmy still quakes.
pub fn seq_stop_earthquake(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let earthquake_sequence = world.obj::<ControllerGodPowers>(this).earthquake_sequence;
    release_sequence(world, earthquake_sequence, call.sequence);
    world.obj_mut::<ControllerGodPowers>(this).earthquake_sequence = None;

    let earthquake_count = world.obj::<ControllerGodPowers>(this).earthquake_count;
    if earthquake_count > 0 {
        let earthquake_def = library::sequence_def(world, "EarthquakeStart");
        let new_earthquake_sequence = sequence::spawn(world, earthquake_def, Some(this));
        world.obj_mut::<ControllerGodPowers>(this).earthquake_sequence = Some(new_earthquake_sequence);
    }
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) onEruptionComplete:]` @ 0x10015f0ec
pub fn on_eruption_complete(world: &mut World, this: Entity, _volcano_logic: Entity) {
    let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
    god_powers.volcano_hit_count = 0;
    god_powers.eruption = false;
}

/// `-[PocketGodViewController (Game) onLightningDie:]` @ 0x10015f110
///
/// The bolt leaves the display tree and shuts down; `mLightningLogic` is released.
pub fn on_lightning_die(world: &mut World, this: Entity, lightning: Entity) {
    let lightning_display_object = logic::display_object(world, lightning);
    display::remove_from_display_group(world, lightning_display_object);
    logic::shutdown(world, lightning);

    let released_lightning = world.obj_mut::<ControllerGodPowers>(this).lightning_logic.take();
    // The release frees the bolt only when `mLightningLogic` is the bolt that just shut down. A
    // newer bolt there (a warlock's, struck while this one still blinked) lives on through its
    // animation's reference; this one, whose reference was overwritten, is never freed. PORT:
    // the original frees that newer bolt when it shuts down in turn; the port leaks it.
    if released_lightning == Some(lightning) {
        lightning_logic::dealloc(world, lightning);
    }
}

/// `-[PocketGodViewController (Game) onHurricaneCantPickUpObjects:]` @ 0x10015f18c
pub fn on_hurricane_cant_pick_up_objects(world: &mut World, this: Entity, _hurricane: Entity) {
    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    if let Some(challenge_manager) = challenge_manager {
        challenge_manager::stop_game(world, challenge_manager, HURRICANE_LIGHTNING_CHALLENGE);
    }
}

/// `-[PocketGodViewController (Game) onHurricaneDie:]` @ 0x10015f1b0
///
/// The hurricane leaves the display tree and shuts down; `mHurricaneLogic` is released.
pub fn on_hurricane_die(world: &mut World, this: Entity, hurricane: Entity) {
    let hurricane_display_object = logic::display_object(world, hurricane);
    display::remove_from_display_group(world, hurricane_display_object);
    logic::shutdown(world, hurricane);

    let released_hurricane = world.obj_mut::<ControllerGodPowers>(this).hurricane_logic.take();
    // The release frees the hurricane only when it is the one that just shut down (a running
    // one is also held by its animation).
    if released_hurricane == Some(hurricane) {
        hurricane_logic::dealloc(world, hurricane);
    }
}

/// `-[PocketGodViewController (Game) onWhirlPoolCantPickUpObjects:]` @ 0x10015f22c
pub fn on_whirl_pool_cant_pick_up_objects(_world: &mut World, _this: Entity, _whirl_pool: Entity) {}

/// `-[PocketGodViewController (Game) onWhirlPoolDie:]` @ 0x10015f230
///
/// The whirlpool leaves the display tree and shuts down; `mWhirlPoolLogic` is released.
pub fn on_whirl_pool_die(world: &mut World, this: Entity, whirl_pool: Entity) {
    let whirl_pool_display_object = logic::display_object(world, whirl_pool);
    display::remove_from_display_group(world, whirl_pool_display_object);
    logic::shutdown(world, whirl_pool);

    let released_whirl_pool = world.obj_mut::<ControllerGodPowers>(this).whirl_pool_logic.take();
    // The release frees the whirlpool only when it is the one that just shut down (a running
    // one is also held by its animation).
    if released_whirl_pool == Some(whirl_pool) {
        whirl_pool_logic::dealloc(world, whirl_pool);
    }
}

/// `-[PocketGodViewController (Game) startDrawLightning:]` @ 0x100160d3c
///
/// `pos` (island coordinates) starts a new bolt path, with the `ElectricGlow` finger glow.
pub fn start_draw_lightning(world: &mut World, this: Entity, pos: Vec4) {
    let last_lightning_pos = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        god_powers.draw_lightning = true;
        god_powers.lightning_list.clear();
        god_powers.lightning_list.push(pos);
        god_powers.last_lightning_pos = pos;
        god_powers.last_lightning_pos
    };

    let island_display_group = island_display_group(world, this);
    let trail = create_finger_trail(world, ELECTRIC_GLOW_TEXTURE_ID, ELECTRIC_GLOW, last_lightning_pos, island_display_group);
    world.obj_mut::<ControllerGodPowers>(this).lightning_trail_logic = Some(trail);
    finger_trail_logic::set_target_x_pos(world, trail, last_lightning_pos.x, last_lightning_pos.y);
    logic::set_behavior(world, trail, Some(ELECTRIC_GLOW));
}

/// `-[PocketGodViewController (Game) triggerLightning]` @ 0x100160f74
///
/// The bolt strikes along the drawn path; the finger glow goes.
pub fn trigger_lightning(world: &mut World, this: Entity) {
    create_lightning(world, this);

    let lightning_trail = world.obj::<ControllerGodPowers>(this).lightning_trail_logic;
    discard_finger_trail(world, lightning_trail);
    world.obj_mut::<ControllerGodPowers>(this).lightning_trail_logic = None;
}

/// `-[PocketGodViewController (Game) drawLightning:]` @ 0x1001610cc
///
/// `pos` (island coordinates): the glow follows the finger. The first pygmy, ape torch, monkey,
/// bird or bonfire under the finger that takes the hit ends the path at its position (the
/// finger's for torches and monkeys) and the bolt strikes; otherwise the path grows by a point
/// every 20 units.
pub fn draw_lightning(world: &mut World, this: Entity, pos: Vec4) {
    let lightning_trail = world.obj::<ControllerGodPowers>(this).lightning_trail_logic;
    if let Some(lightning_trail) = lightning_trail {
        finger_trail_logic::set_target_x_pos(world, lightning_trail, pos.x, pos.y);
    }

    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        let touch_pos = lightning_touch_pos(world, this);
        if !logic_does_point_intersect(world, Some(pygmy), touch_pos) {
            continue;
        }
        if !crate::pygmy::hit_by_lightning(world, pygmy) {
            continue;
        }
        let target_pos = logic_pos(world, pygmy);
        strike_shocked_object(world, this, target_pos);
        return;
    }

    let ape_torches = world.obj::<ControllerWorldRegistry>(this).ape_torch_logic_array.clone();
    for ape_torch in ape_torches {
        let touch_pos = lightning_touch_pos(world, this);
        if !logic_does_point_intersect(world, Some(ape_torch), touch_pos) {
            continue;
        }
        if !ape_torch_logic::hit_by_lightning(world, ape_torch) {
            continue;
        }
        strike_shocked_object(world, this, pos);
        return;
    }

    let monkeys = world.obj::<ControllerWorldRegistry>(this).monkey_logic_array.clone();
    for monkey in monkeys {
        let touch_pos = lightning_touch_pos(world, this);
        if !logic_does_point_intersect(world, Some(monkey), touch_pos) {
            continue;
        }
        if !monkey_logic::hit_by_lightning(world, monkey) {
            continue;
        }
        strike_shocked_object(world, this, pos);
        return;
    }

    let bird = world.obj::<ControllerWorldRegistry>(this).bird_logic;
    let touch_pos = lightning_touch_pos(world, this);
    if logic_does_point_intersect(world, bird, touch_pos) {
        if let Some(bird) = bird {
            if bird_logic::hit_by_lightning(world, bird) {
                let target_pos = logic_pos(world, bird);
                strike_shocked_object(world, this, target_pos);
                return;
            }
        }
    }

    let bon_fire = world.obj::<ControllerWorldRegistry>(this).bon_fire_logic;
    let touch_pos = lightning_touch_pos(world, this);
    if logic_does_point_intersect(world, bon_fire, touch_pos) {
        if let Some(bon_fire) = bon_fire {
            if bon_fire_logic::hit_by_lightning(world, bon_fire) {
                let target_pos = logic_pos(world, bon_fire);
                strike_shocked_object(world, this, target_pos);
                return;
            }
        }
    }

    let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
    let last_lightning_pos = god_powers.last_lightning_pos;
    let offset = Vec4::new(pos.x - last_lightning_pos.x, pos.y - last_lightning_pos.y, pos.z - last_lightning_pos.z, pos.w);
    if length3(offset) > LIGHTNING_POINT_DISTANCE {
        god_powers.lightning_list.push(pos);
        god_powers.last_lightning_pos = pos;
    }
}

/// `-[PocketGodViewController (Game) stopDrawLightning:]` @ 0x100161730
///
/// `pos` (island coordinates): unless something was shocked already, the release point ends
/// the path and the bolt strikes.
pub fn stop_draw_lightning(world: &mut World, this: Entity, pos: Vec4) {
    let shock_object = world.obj::<ControllerGodPowers>(this).shock_object;
    if !shock_object {
        {
            let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
            let last_lightning_pos = god_powers.last_lightning_pos;
            if last_lightning_pos.x != pos.x || last_lightning_pos.y != pos.y {
                god_powers.lightning_list.push(pos);
            }
        }
        trigger_lightning(world, this);
    }

    let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
    god_powers.lightning_touch = None;
    god_powers.draw_lightning = false;
    god_powers.shock_object = false;
}

/// `-[PocketGodViewController (Game) hitPygmyWithWarlockLightning:]` @ 0x100161818
///
/// A warlock's bolt: seven points from the top of the sky down to the pygmy, the inner five
/// jittered by up to 50 units sideways. Nothing while the player draws lightning.
pub fn hit_pygmy_with_warlock_lightning(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let drawing_lightning = world.obj::<ControllerGodPowers>(this).draw_lightning;
    if drawing_lightning {
        return;
    }

    let pygmy_pos = logic_pos(world, pygmy_logic);
    let y_step = (pygmy_pos.y + -WARLOCK_LIGHTNING_TOP_Y_POS) / WARLOCK_LIGHTNING_POINT_COUNT as f32;
    world.obj_mut::<ControllerGodPowers>(this).lightning_list.clear();

    let mut y_pos = WARLOCK_LIGHTNING_TOP_Y_POS;
    for index in 0..WARLOCK_LIGHTNING_POINT_COUNT {
        let mut x_pos = pygmy_pos.x;
        let end_point = index == 0 || index == WARLOCK_LIGHTNING_POINT_COUNT - 1;
        if !end_point {
            let roll = world.resource_mut::<Random>().arc4random_mod(1001);
            let jitter = roll.wrapping_sub(500) as i32;
            x_pos = x_pos + jitter as f32 / 10.0;
        }

        let point = Vec4::new(x_pos, y_pos, 0.0, 0.0);
        world.obj_mut::<ControllerGodPowers>(this).lightning_list.push(point);
        y_pos = y_step + y_pos;
    }

    create_lightning(world, this);
}

/// `-[PocketGodViewController (Game) startDrawRainbow:]` @ 0x100161a54
///
/// `pos` (island coordinates): where the finger went down; the rainbow starts once it moves.
pub fn start_draw_rainbow(world: &mut World, this: Entity, pos: Vec4) {
    clear_rainbow_points(world, this);
    world.obj_mut::<ControllerGodPowers>(this).last_rainbow_pos = pos;
}

/// `-[PocketGodViewController (Game) drawRainbow:]` @ 0x100161a9c
///
/// `pos` (island coordinates). Before the stroke: a move of more than 20 starts it (first
/// point, the `RainbowTrail` sparkle at the start point, `[mRainbowLogic startDrawRainbow]`).
/// During it: the sparkle follows and turns with the finger, and every 5 units the rainbow
/// grows by a point.
pub fn draw_rainbow(world: &mut World, this: Entity, pos: Vec4) {
    let (last_rainbow_pos, drawing_rainbow) = {
        let god_powers = world.obj::<ControllerGodPowers>(this);
        (god_powers.last_rainbow_pos, god_powers.draw_rainbow)
    };
    let x_offset = pos.x - last_rainbow_pos.x;
    let y_offset = pos.y - last_rainbow_pos.y;
    let distance = (x_offset * x_offset + y_offset * y_offset).sqrt();

    if !drawing_rainbow {
        if distance > RAINBOW_START_DISTANCE {
            world.obj_mut::<ControllerGodPowers>(this).draw_rainbow = true;
            push_rainbow_point(world, this, pos);
            start_rainbow_trail(world, this);

            let rainbow = world.obj::<ControllerGodPowers>(this).rainbow_logic;
            if let Some(rainbow) = rainbow {
                rainbow_logic::start_draw_rainbow(world, rainbow);
            }
        }
        return;
    }

    let rainbow_trail = world.obj::<ControllerGodPowers>(this).rainbow_trail_logic;
    if let Some(rainbow_trail) = rainbow_trail {
        finger_trail_logic::set_target_x_pos(world, rainbow_trail, pos.x, pos.y);
        let stroke_angle = y_offset.atan2(x_offset);
        let rainbow_trail_display_object = logic::display_object(world, rainbow_trail);
        display::object_mut(world, rainbow_trail_display_object).rot.z = (stroke_angle / PI) * 180.0;
    }

    if distance > RAINBOW_POINT_DISTANCE {
        let rainbow = world.obj::<ControllerGodPowers>(this).rainbow_logic;
        if let Some(rainbow) = rainbow {
            rainbow_logic::draw_rainbow(world, rainbow, pos);
        }
        push_rainbow_point(world, this, pos);
        world.obj_mut::<ControllerGodPowers>(this).last_rainbow_pos = pos;
    }
}

/// `-[PocketGodViewController (Game) stopDrawRainbow:]` @ 0x100161db4
///
/// `pos` (island coordinates): the release point ends the rainbow, which strikes; the sparkle
/// goes.
pub fn stop_draw_rainbow(world: &mut World, this: Entity, pos: Vec4) {
    let last_rainbow_pos = world.obj::<ControllerGodPowers>(this).last_rainbow_pos;
    if last_rainbow_pos.x != pos.x || last_rainbow_pos.y != pos.y {
        push_rainbow_point(world, this, pos);
    }

    let rainbow = world.obj::<ControllerGodPowers>(this).rainbow_logic;
    if let Some(rainbow) = rainbow {
        rainbow_logic::draw_rainbow_complete(world, rainbow);
    }

    let rainbow_trail = world.obj::<ControllerGodPowers>(this).rainbow_trail_logic;
    discard_finger_trail(world, rainbow_trail);

    let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
    god_powers.rainbow_trail_logic = None;
    god_powers.draw_rainbow = false;
}

/// `-[PocketGodViewController (Game) drawLocust:]` @ 0x100161ed8
///
/// `pos` (island or temple coordinates): the swarm path grows by a point every 5 units; the
/// swarm follows the finger.
pub fn draw_locust(world: &mut World, this: Entity, pos: Vec4) {
    let locust = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        let last_locust_pos = god_powers.last_locust_pos;
        let x_offset = pos.x - last_locust_pos.x;
        let y_offset = pos.y - last_locust_pos.y;
        let distance = (x_offset * x_offset + y_offset * y_offset).sqrt();
        if distance > LOCUST_POINT_DISTANCE {
            god_powers.locust_vector.push(pos);
            god_powers.last_locust_pos = pos;
        }
        god_powers.locust_logic
    };

    if let Some(locust) = locust {
        locust_logic::draw_locust(world, locust, pos);
    }
}

/// `-[PocketGodViewController (Game) startDrawLocust:]` @ 0x100161fac
///
/// `pos` (island or temple coordinates) starts a new swarm path; the swarm gathers and
/// `LocustStartDraw` runs on `mSequence`.
pub fn start_draw_locust(world: &mut World, this: Entity, pos: Vec4) {
    let locust = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        god_powers.draw_locust = true;
        god_powers.locust_vector.clear();
        god_powers.locust_vector.push(pos);
        god_powers.last_locust_pos = pos;
        god_powers.locust_logic
    };

    // The swarm keeps reading `mpLocustVector`, which the port names by its owner (this).
    if let Some(locust) = locust {
        locust_logic::start_draw_locust(world, locust, this, pos);
    }

    let main_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
    set_sequence_def(world, main_sequence, "LocustStartDraw");
}

/// `-[PocketGodViewController (Game) stopDrawLocust:]` @ 0x1001620f0
///
/// `pos` (island or temple coordinates): the release point ends the path; released outside the
/// screen's inner box, the swarm scatters.
pub fn stop_draw_locust(world: &mut World, this: Entity, pos: Vec4) {
    let locust = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        let last_locust_pos = god_powers.last_locust_pos;
        if last_locust_pos.x != pos.x || last_locust_pos.y != pos.y {
            god_powers.locust_vector.push(pos);
        }
        god_powers.locust_logic
    };

    if let Some(locust) = locust {
        locust_logic::draw_locust_complete(world, locust);

        let released_outside = pos.x < LOCUST_RELEASE_MIN_X_POS
            || pos.x > LOCUST_RELEASE_MAX_X_POS
            || pos.y < LOCUST_RELEASE_MIN_Y_POS
            || pos.y > LOCUST_RELEASE_MAX_Y_POS;
        if released_outside {
            locust_logic::deactivate_locusts(world, locust);
        }
    }

    world.obj_mut::<ControllerGodPowers>(this).draw_locust = false;
}

/// `-[PocketGodViewController (Game) startHurricane:]` @ 0x100162384
///
/// `pos` (island coordinates): creates the hurricane (once) and the `WindGlow` finger glow
/// (once), and lets the finger drive the hurricane.
pub fn start_hurricane(world: &mut World, this: Entity, pos: Vec4) {
    let hurricane_touch = world.obj::<ControllerGodPowers>(this).hurricane_touch;
    let touch_game_time = match hurricane_touch {
        Some(hurricane_touch) => world.obj::<BcTouch>(hurricane_touch).game_time,
        None => 0.0,
    };
    {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        god_powers.last_hurricane_touch_time = touch_game_time;
        god_powers.last_hurricane_pos = pos;
    }

    let hurricane = world.obj::<ControllerGodPowers>(this).hurricane_logic;
    if hurricane.is_none() {
        let hurricane_model_object = world.spawn(Isa(&hurricane_model::HURRICANE_MODEL)).id();
        hurricane_model::init_with_id(world, hurricane_model_object, "Hurricane");
        let island_display_group = island_display_group(world, this);
        place_power_model(world, hurricane_model_object, island_display_group);

        let new_hurricane = hurricane_logic::spawn(world, hurricane_model_object);
        world.obj_mut::<ControllerGodPowers>(this).hurricane_logic = Some(new_hurricane);
        hurricane_logic::set_hurricane_logic_listener(world, new_hurricane, this);
    }

    let hurricane_trail = world.obj::<ControllerGodPowers>(this).hurricane_trail_logic;
    if hurricane_trail.is_none() {
        let island_display_group = island_display_group(world, this);
        let new_hurricane_trail = create_finger_trail(world, WIND_FINGER_TEXTURE_ID, WIND_GLOW, pos, island_display_group);
        world.obj_mut::<ControllerGodPowers>(this).hurricane_trail_logic = Some(new_hurricane_trail);
        finger_trail_logic::set_target_x_pos(world, new_hurricane_trail, pos.x, pos.y);
        logic::set_behavior(world, new_hurricane_trail, Some(WIND_GLOW));
    }

    let hurricane = world.obj::<ControllerGodPowers>(this).hurricane_logic;
    if let Some(hurricane) = hurricane {
        hurricane_logic::set_touch_active(world, hurricane, true);
    }
}

/// `-[PocketGodViewController (Game) maintainHurricane:]` @ 0x1001626b8
///
/// `pos` (island coordinates): the glow follows the finger and the wind picks up.
pub fn maintain_hurricane(world: &mut World, this: Entity, pos: Vec4) {
    let (hurricane_trail, hurricane) = {
        let god_powers = world.obj::<ControllerGodPowers>(this);
        (god_powers.hurricane_trail_logic, god_powers.hurricane_logic)
    };
    if let Some(hurricane_trail) = hurricane_trail {
        finger_trail_logic::set_target_x_pos(world, hurricane_trail, pos.x, pos.y);
    }
    if let Some(hurricane) = hurricane {
        hurricane_logic::set_wind_vel(world, hurricane, 0.0);
    }
}

/// `-[PocketGodViewController (Game) stopHurricane:]` @ 0x100162718
///
/// The finger lets go of the hurricane; the glow goes.
pub fn stop_hurricane(world: &mut World, this: Entity, _pos: Vec4) {
    let hurricane = world.obj::<ControllerGodPowers>(this).hurricane_logic;
    if let Some(hurricane) = hurricane {
        hurricane_logic::set_touch_active(world, hurricane, false);
    }

    let hurricane_trail = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        god_powers.hurricane_touch = None;
        god_powers.hurricane_trail_logic
    };
    discard_finger_trail(world, hurricane_trail);
    world.obj_mut::<ControllerGodPowers>(this).hurricane_trail_logic = None;
}

/// `-[PocketGodViewController (Game) maintainWhirlPool:]` @ 0x1001627c4
///
/// `pos` (underwater coordinates): creates the whirlpool (once) and the `WindGlow` finger glow
/// (once) in `UnderwaterDisplayGroup`; the glow follows the finger and the current picks up.
pub fn maintain_whirl_pool(world: &mut World, this: Entity, pos: Vec4) {
    let whirl_pool = world.obj::<ControllerGodPowers>(this).whirl_pool_logic;
    if whirl_pool.is_none() {
        let whirl_pool_model_object = world.spawn(Isa(&whirl_pool_model::WHIRL_POOL_MODEL)).id();
        whirl_pool_model::init_with_id(world, whirl_pool_model_object, "WhirlPool");
        let underwater_display_group = underwater_display_group(world, this);
        place_power_model(world, whirl_pool_model_object, underwater_display_group);

        let new_whirl_pool = whirl_pool_logic::spawn(world, whirl_pool_model_object);
        world.obj_mut::<ControllerGodPowers>(this).whirl_pool_logic = Some(new_whirl_pool);
        whirl_pool_logic::set_whirl_pool_logic_listener(world, new_whirl_pool, this);
    }

    let whirl_pool_trail = world.obj::<ControllerGodPowers>(this).whirl_pool_trail_logic;
    if whirl_pool_trail.is_none() {
        let underwater_display_group = underwater_display_group(world, this);
        let new_whirl_pool_trail = create_finger_trail(world, WIND_FINGER_TEXTURE_ID, WIND_GLOW, pos, underwater_display_group);
        world.obj_mut::<ControllerGodPowers>(this).whirl_pool_trail_logic = Some(new_whirl_pool_trail);
        finger_trail_logic::set_target_x_pos(world, new_whirl_pool_trail, pos.x, pos.y);
        logic::set_behavior(world, new_whirl_pool_trail, Some(WIND_GLOW));
    }

    let (whirl_pool, whirl_pool_trail) = {
        let god_powers = world.obj::<ControllerGodPowers>(this);
        (god_powers.whirl_pool_logic, god_powers.whirl_pool_trail_logic)
    };
    if let Some(whirl_pool) = whirl_pool {
        whirl_pool_logic::set_touch_active(world, whirl_pool, true);
    }
    if let Some(whirl_pool_trail) = whirl_pool_trail {
        finger_trail_logic::set_target_x_pos(world, whirl_pool_trail, pos.x, pos.y);
    }
    if let Some(whirl_pool) = whirl_pool {
        whirl_pool_logic::set_wind_vel(world, whirl_pool, 0.0);
    }
}

/// `-[PocketGodViewController (Game) stopWhirlPool:]` @ 0x100162af8
///
/// The finger lets go of the whirlpool; the glow goes.
pub fn stop_whirl_pool(world: &mut World, this: Entity, _pos: Vec4) {
    let whirl_pool = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        god_powers.whirl_pool_touch = None;
        god_powers.whirl_pool_logic
    };
    if let Some(whirl_pool) = whirl_pool {
        whirl_pool_logic::set_touch_active(world, whirl_pool, false);
    }

    let whirl_pool_trail = world.obj::<ControllerGodPowers>(this).whirl_pool_trail_logic;
    if whirl_pool_trail.is_some() {
        discard_finger_trail(world, whirl_pool_trail);
        world.obj_mut::<ControllerGodPowers>(this).whirl_pool_trail_logic = None;
    }
}

/// `-[PocketGodViewController (Game) startDragUnderwaterPower:]` @ 0x100162bac
///
/// `pos` (underwater coordinates): the coral-reef temple's power follows the finger (its glow
/// is created once, tinted for the temple mode), and the mode's episode event fires.
pub fn start_drag_underwater_power(world: &mut World, this: Entity, pos: Vec4) {
    world.obj_mut::<ControllerGodPowers>(this).last_underwater_power_pos = pos;

    let underwater_power_trail = world.obj::<ControllerGodPowers>(this).underwater_power_trail_logic;
    if underwater_power_trail.is_none() {
        let glow = display::multi_model::spawn(world, Some(UNDERWATER_POWER_MODEL_ID));
        {
            let mut glow_display = display::object_mut(world, glow);
            glow_display.pos.x = pos.x;
            glow_display.pos.y = pos.y;
            glow_display.pos.z = SPRITE_Z;
        }
        insert_before_underwater_pygmy_marker(world, glow);

        let new_underwater_power_trail = finger_trail_logic::spawn(world, glow);
        world.obj_mut::<ControllerGodPowers>(this).underwater_power_trail_logic = Some(new_underwater_power_trail);
        finger_trail_logic::set_target_x_pos(world, new_underwater_power_trail, pos.x, pos.y);

        world.obj_mut::<ControllerGodPowers>(this).last_underwater_temple_mode = 0;
        let underwater_temple_mode = underwater_temple_mode(world, this);
        on_underwater_temple_mode_changed(world, this, underwater_temple_mode);
    }

    let unleash_event = match underwater_temple_mode(world, this) {
        1 => "UnleashUnderwaterPowerCold",
        2 => "UnleashUnderwaterPowerHeat",
        3 => "UnleashUnderwaterPowerOil",
        4 => "UnleashUnderwaterPowerAcid",
        5 => "UnleashUnderwaterPowerAir",
        _ => return,
    };
    let idol_manager = idol_manager::instance(world);
    if let Some(idol_manager) = idol_manager {
        idol_manager::trigger_episode_event(world, idol_manager, unleash_event);
    }
}

/// `-[PocketGodViewController (Game) stopDragUnderwaterPower:]` @ 0x100162eb4
///
/// The finger lets go of the coral-reef power; its glow goes.
pub fn stop_drag_underwater_power(world: &mut World, this: Entity, _pos: Vec4) {
    let underwater_power_trail = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        god_powers.underwater_power_touch = None;
        god_powers.underwater_power_trail_logic
    };
    if underwater_power_trail.is_some() {
        discard_finger_trail(world, underwater_power_trail);
        world.obj_mut::<ControllerGodPowers>(this).underwater_power_trail_logic = None;
    }
}

/// `-[PocketGodViewController (Game) dragUnderwaterPower:]` @ 0x100162f40
///
/// `pos` (underwater coordinates): every 30 units a trail puff of the temple mode; pygmies near
/// the finger that can feel the power get it; the glow follows the finger.
pub fn drag_underwater_power(world: &mut World, this: Entity, pos: Vec4) {
    let last_underwater_power_pos = world.obj::<ControllerGodPowers>(this).last_underwater_power_pos;
    let x_offset = pos.x - last_underwater_power_pos.x;
    let y_offset = pos.y - last_underwater_power_pos.y;
    let distance = (x_offset * x_offset + y_offset * y_offset).sqrt();
    if distance > UNDERWATER_POWER_TRAIL_DISTANCE {
        world.obj_mut::<ControllerGodPowers>(this).last_underwater_power_pos = pos;
        leave_underwater_power_trail_puff(world, this, pos);
    }

    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        if !crate::pygmy::can_be_touched_by_underwater_power(world, pygmy) {
            continue;
        }

        let pygmy_pos = logic_pos(world, pygmy);
        let pygmy_x_offset = pygmy_pos.x - pos.x;
        let pygmy_y_offset = (pygmy_pos.y + UNDERWATER_POWER_TOUCH_Y_OFFSET) - pos.y;
        let pygmy_distance = (pygmy_x_offset * pygmy_x_offset + pygmy_y_offset * pygmy_y_offset).sqrt();
        if pygmy_distance < UNDERWATER_POWER_TOUCH_DISTANCE {
            let underwater_temple_mode = underwater_temple_mode(world, this);
            crate::pygmy::can_be_touched_by_underwater_power_with_arg(world, pygmy, underwater_temple_mode);
        }
    }

    let underwater_power_trail = world.obj::<ControllerGodPowers>(this).underwater_power_trail_logic;
    if let Some(underwater_power_trail) = underwater_power_trail {
        finger_trail_logic::set_target_x_pos(world, underwater_power_trail, pos.x, pos.y);
    }
}

/// `-[PocketGodViewController (Game) onUnderwaterTempleModeChanged:]` @ 0x100163394
///
/// While the coral-reef power is held, the glow takes the new mode's look; mode 0 ends the
/// stroke.
pub fn on_underwater_temple_mode_changed(world: &mut World, this: Entity, mode: i32) {
    let (last_underwater_temple_mode, underwater_power_trail, last_underwater_power_pos) = {
        let god_powers = world.obj::<ControllerGodPowers>(this);
        (
            god_powers.last_underwater_temple_mode,
            god_powers.underwater_power_trail_logic,
            god_powers.last_underwater_power_pos,
        )
    };
    if last_underwater_temple_mode == mode {
        return;
    }
    let Some(underwater_power_trail) = underwater_power_trail else {
        return;
    };
    world.obj_mut::<ControllerGodPowers>(this).last_underwater_temple_mode = mode;

    let behavior = match mode {
        0 => {
            stop_drag_underwater_power(world, this, last_underwater_power_pos);
            return;
        }
        1 => "UnderwaterPowerCold",
        2 => "UnderwaterPowerHeat",
        3 => "UnderwaterPowerOil",
        4 => "UnderwaterPowerAcid",
        5 => "UnderwaterPowerAir",
        _ => return,
    };
    logic::set_behavior(world, underwater_power_trail, Some(behavior));
}

/// `-[PocketGodViewController (Game) onMultiModelEvent:eventId:]` @ 0x1001634ac
///
/// A coral-reef power trail puff whose animation ended leaves (but no puff ever has the
/// controller as its listener, see [`drag_underwater_power`]).
pub fn on_multi_model_event(world: &mut World, this: Entity, multi_model_logic: Entity, event_id: &str) {
    if event_id != UNDERWATER_POWER_TRAIL_COMPLETE_EVENT_ID {
        return;
    }

    let puff_display_object = logic::display_object(world, multi_model_logic);
    display::remove_from_display_group(world, puff_display_object);
    logic::shutdown(world, multi_model_logic);

    // `removeObject:` dropped the array's reference, the puff's last one.
    let was_in_array = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        let was_in_array = god_powers.underwater_power_trail_logic_array.contains(&multi_model_logic);
        god_powers.underwater_power_trail_logic_array.retain(|puff| *puff != multi_model_logic);
        was_in_array
    };
    if was_in_array {
        logic::dealloc(world, multi_model_logic);
    }
}

/// `-[PocketGodViewController (Game) checkForTapping:]` @ 0x100163550
///
/// A drum tap (option `DrumHazard`), counted only when a frame passed since the last one. From
/// the fourth tap on each tap plays the left or right drum (unless a dance song plays); the
/// fourth starts the celebration.
pub fn check_for_tapping(world: &mut World, this: Entity, touch: Entity) {
    if option_int_preference(world, this, DRUM_HAZARD_OPTION) == 0 {
        return;
    }
    let tap_touch_time = world.obj::<ControllerGodPowers>(this).tap_touch_time;
    if tap_touch_time == 0.0 {
        return;
    }

    let previous_tap_count = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        god_powers.tap_touch_time = 0.0;
        let previous_tap_count = god_powers.tap_touch_count;
        god_powers.tap_touch_count = previous_tap_count + 1;
        previous_tap_count
    };
    if previous_tap_count <= 2 {
        return;
    }

    if !is_dance_song_active(world, this) {
        let touch_x_pos = world.obj::<BcTouch>(touch).pos.x;
        let touch_width = world.resource::<BcView>().touch_width;
        let tap_sequence = if touch_x_pos <= touch_width * 0.5 { "TapLeft" } else { "TapRight" };
        let main_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
        set_sequence_def(world, main_sequence, tap_sequence);
    }

    let tap_touch_count = world.obj::<ControllerGodPowers>(this).tap_touch_count;
    if tap_touch_count == CELEBRATION_TAP_COUNT {
        start_celebration(world, this);
    }
}

/// `-[PocketGodViewController (Game) isUnderwaterPowerActive]` @ 0x100163eb8
pub fn is_underwater_power_active(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_UNDERWATER_CORAL_REEF {
        return false;
    }
    underwater_temple_mode(world, this) != 0
}

/// `-[PocketGodViewController (Game) isRainbowActive]` @ 0x100163f10
///
/// `mbCelebrationActive`, not the rainbow's own state.
pub fn is_rainbow_active(world: &mut World, this: Entity) -> bool {
    world.obj::<ControllerGodPowers>(this).celebration_active
}

/// `-[PocketGodViewController (Game) isLocustActive]` @ 0x100163f24
pub fn is_locust_active(world: &mut World, this: Entity) -> bool {
    let locust = world.obj::<ControllerGodPowers>(this).locust_logic;
    match locust {
        Some(locust) => locust_logic::is_locust_active(world, locust),
        None => false,
    }
}

/// `-[PocketGodViewController (Game) isDanceActive]` @ 0x100163f44
pub fn is_dance_active(world: &mut World, this: Entity) -> bool {
    world.obj::<ControllerGodPowers>(this).celebration_active
}

/// `-[PocketGodViewController (Game) isEruptionActive]` @ 0x100163f58
pub fn is_eruption_active(world: &mut World, this: Entity) -> bool {
    world.obj::<ControllerGodPowers>(this).eruption
}

/// `-[PocketGodViewController (Game) isEarthquakeActive]` @ 0x100164030
pub fn is_earthquake_active(world: &mut World, this: Entity) -> bool {
    world.obj::<ControllerGodPowers>(this).earthquake_sequence.is_some()
}

/// `-[PocketGodViewController (Game) onPygmyHitByLightningInHurricane:]` @ 0x100164668
pub fn on_pygmy_hit_by_lightning_in_hurricane(world: &mut World, this: Entity, _pygmy_logic: Entity) {
    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    if let Some(challenge_manager) = challenge_manager {
        challenge_manager::increment_game_score(world, challenge_manager, HURRICANE_LIGHTNING_CHALLENGE);
    }
}

/// `-[PocketGodViewController (Game) onPygmyEarthquakeStart:]` @ 0x100164c10
///
/// The first quaking pygmy starts the earthquake (episode event `Earthquake`, the dance music
/// stops, the coconut on the tree falls, a new `mEarthquakeSequence` runs `EarthquakeStart`).
pub fn on_pygmy_earthquake_start(world: &mut World, this: Entity, _pygmy_logic: Entity) {
    let (earthquake_count, earthquake_sequence) = {
        let god_powers = world.obj::<ControllerGodPowers>(this);
        (god_powers.earthquake_count, god_powers.earthquake_sequence)
    };

    if earthquake_count == 0 && earthquake_sequence.is_none() {
        let idol_manager = world.obj::<ControllerManagersBridge>(this).idol_manager;
        if let Some(idol_manager) = idol_manager {
            idol_manager::trigger_episode_event(world, idol_manager, "Earthquake");
        }
        stop_celebration_music(world, this);

        let first_coconut = world.obj::<ControllerWorldRegistry>(this).coconut_logic_array.first().copied();
        if let Some(first_coconut) = first_coconut {
            if coconut_logic::is_on_tree(world, first_coconut) {
                coconut_logic::fall_off_tree(world, first_coconut);
            }
        }

        let earthquake_def = library::sequence_def(world, "EarthquakeStart");
        let new_earthquake_sequence = sequence::spawn(world, earthquake_def, Some(this));
        world.obj_mut::<ControllerGodPowers>(this).earthquake_sequence = Some(new_earthquake_sequence);
    }

    let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
    god_powers.earthquake_count = god_powers.earthquake_count + 1;
}

/// `-[PocketGodViewController (Game) onPygmyEarthquakeStop:]` @ 0x100164d6c
///
/// The last quaking pygmy stops: `mEarthquakeSequence` runs `EarthquakeStop`.
pub fn on_pygmy_earthquake_stop(world: &mut World, this: Entity, _pygmy_logic: Entity) {
    let (earthquake_count, earthquake_sequence) = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        god_powers.earthquake_count = god_powers.earthquake_count - 1;
        (god_powers.earthquake_count, god_powers.earthquake_sequence)
    };
    if earthquake_count != 0 {
        return;
    }
    set_sequence_def(world, earthquake_sequence, "EarthquakeStop");
}

/// `-[PocketGodViewController (Game) onVolcanoHit:]` @ 0x100164e00
///
/// A pygmy thrown into the volcano: the first three hits play `hit1` … `hit3` on it (`erupt1`
/// … `erupt3` for a time-temple pygmy of type 4); the third starts the eruption.
pub fn on_volcano_hit(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let time_temple_pygmy_type = crate::pygmy::time_temple_pygmy_type(world, pygmy_logic);
    let volcano_hit_count = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        let volcano_hit_count = god_powers.volcano_hit_count;
        god_powers.volcano_hit_count = volcano_hit_count + 1;
        volcano_hit_count
    };
    let erupts = time_temple_pygmy_type == 4;

    let volcano_reaction: fn(&mut World, Entity) = match volcano_hit_count {
        0 if erupts => volcano_logic::erupt1,
        0 => volcano_logic::hit1,
        1 if erupts => volcano_logic::erupt2,
        1 => volcano_logic::hit2,
        2 => {
            world.obj_mut::<ControllerGodPowers>(this).eruption = true;
            if erupts { volcano_logic::erupt3 } else { volcano_logic::hit3 }
        }
        _ => return,
    };

    let volcano = world.obj::<ControllerGodPowers>(this).volcano_logic;
    if let Some(volcano) = volcano {
        volcano_reaction(world, volcano);
    }
}

/// `-[PocketGodViewController (Game) onFireworkDie:]` @ 0x100164f10
///
/// A burnt-out firework leaves the display tree and the firework array.
pub fn on_firework_die(world: &mut World, this: Entity, firework: Entity) {
    let firework_display_object = logic::display_object(world, firework);
    display::remove_from_display_group(world, firework_display_object);
    logic::shutdown(world, firework);

    // `removeObject:` dropped the array's reference, the firework's last one. `FireworkLogic`'s
    // own `dealloc` only adds releases of its (emptied) spark array and of this listener.
    let was_in_array = {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        let was_in_array = god_powers.firework_logic_array.contains(&firework);
        god_powers.firework_logic_array.retain(|logic| *logic != firework);
        was_in_array
    };
    if was_in_array {
        logic::dealloc(world, firework);
    }
}

/// The dance songs of a celebration (`startCelebration`'s flags).
#[derive(Clone, Copy, PartialEq, Eq)]
enum DanceSong {
    PantsOnTheGround,
    UpdateSong,
    ThemeSong,
    Fishmas,
    FishmasTwelveDays,
    Thriller,
}

/// The song options `stopDancing:updateSong:themeSong:napoleonDynamite:fishmas:fishmasTwelveDays:`
/// passes to a pygmy.
#[derive(Clone, Copy, Default)]
struct DanceSongOptions {
    pants_dance: bool,
    update_song: bool,
    theme_song: bool,
    napoleon_dynamite: bool,
    fishmas: bool,
    fishmas_twelve_days: bool,
}

/// Sets the `mb…Active` flag of a celebration's dance song.
fn set_dance_song_active(world: &mut World, this: Entity, dance_song: DanceSong) {
    let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
    match dance_song {
        DanceSong::PantsOnTheGround => god_powers.pants_on_the_ground_active = true,
        DanceSong::UpdateSong => god_powers.update_song_active = true,
        DanceSong::ThemeSong => god_powers.theme_song_active = true,
        DanceSong::Fishmas => god_powers.fishmas_active = true,
        DanceSong::FishmasTwelveDays => god_powers.fishmas_twelve_days_active = true,
        DanceSong::Thriller => god_powers.thriller_active = true,
    }
}

/// Whether a dance song plays (the drum then stays quiet).
fn is_dance_song_active(world: &World, this: Entity) -> bool {
    let god_powers = world.obj::<ControllerGodPowers>(this);
    god_powers.thriller_active
        || god_powers.pants_on_the_ground_active
        || god_powers.update_song_active
        || god_powers.theme_song_active
        || god_powers.fishmas_active
        || god_powers.fishmas_twelve_days_active
}

/// `[mSkinPackManager isSkinPackOptionActiveBySkinPackId:skinObjectId:skinOptionId:skinOptionValueId:]`
/// (NO while there is no manager).
fn is_skin_option_active(world: &mut World, this: Entity, value: &SkinOptionValue) -> bool {
    let skin_pack_manager = world.obj::<ControllerManagersBridge>(this).skin_pack_manager;
    let Some(skin_pack_manager) = skin_pack_manager else {
        return false;
    };
    skin_pack_manager::is_skin_pack_option_active_by_skin_pack_id(
        world,
        skin_pack_manager,
        value.skin_pack_id,
        value.skin_object_id,
        value.skin_option_id,
        value.skin_option_value_id,
    )
}

/// `[mOptionPreferences getIntPreference:key]` (0 before the document exists).
fn option_int_preference(world: &World, this: Entity, key: &str) -> i32 {
    let option_preferences = world.obj::<ControllerBootAndFrame>(this).option_preferences;
    match option_preferences {
        Some(option_preferences) => pg_engine::prefs::get_int_preference(world, option_preferences, key),
        None => 0,
    }
}

/// `[sequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:id]]` (nothing for a nil
/// sequence).
fn set_sequence_def(world: &mut World, sequence: Option<Entity>, id: &str) {
    let Some(sequence) = sequence else {
        return;
    };
    let def = library::sequence_def(world, id);
    sequence::set_sequence_def(world, sequence, def);
}

/// `[sequence stop]; [sequence release]` on a controller sequence ivar (the caller clears it).
/// The ivar was the sequence's last owner, so it dies, unless it is the sequence running the
/// current item (`running_sequence`): the runner still uses that one after the item returns,
/// so the port keeps it (stopped, unreachable).
fn release_sequence(world: &mut World, sequence: Option<Entity>, running_sequence: Entity) {
    let Some(sequence) = sequence else {
        return;
    };
    sequence::stop(world, sequence);
    if sequence != running_sequence {
        world.despawn(sequence);
    }
}

/// `[mUnderwaterTempleLogic underwaterTempleMode]` (0 without the temple).
fn underwater_temple_mode(world: &mut World, this: Entity) -> i32 {
    let underwater_temple = world.obj::<ControllerWorldRegistry>(this).underwater_temple_logic;
    match underwater_temple {
        Some(underwater_temple) => underwater_temple_logic::underwater_temple_mode(world, underwater_temple),
        None => 0,
    }
}

/// `[[object displayObject] pos]` of a logic object.
fn logic_pos(world: &World, object: Entity) -> Vec4 {
    let display_object = logic::display_object(world, object);
    display::object(world, display_object).pos
}

/// `[object doesPointIntersect:point checkSelect:NO]` of a logic object (`BCLogic` @
/// 0x100139898; NO for nil).
fn logic_does_point_intersect(world: &World, object: Option<Entity>, point: Vec4) -> bool {
    let Some(object) = object else {
        return false;
    };
    let display_object = logic::display_object(world, object);
    display::does_point_intersect(world, display_object, point, false)
}

/// `[mLightningTouch pos]` (view coordinates).
fn lightning_touch_pos(world: &World, this: Entity) -> Vec4 {
    let lightning_touch = world.obj::<ControllerGodPowers>(this).lightning_touch;
    match lightning_touch {
        Some(lightning_touch) => world.obj::<BcTouch>(lightning_touch).pos,
        // The finger always exists while the bolt is drawn; this stands for the original's
        // NULL point.
        None => Vec4::ZERO,
    }
}

/// `drawLightning:` found something to shock: the bolt ends at `target_pos` and strikes.
fn strike_shocked_object(world: &mut World, this: Entity, target_pos: Vec4) {
    {
        let mut god_powers = world.obj_mut::<ControllerGodPowers>(this);
        god_powers.shock_object = true;
        god_powers.lightning_list.push(target_pos);
    }
    trigger_lightning(world, this);
}

/// A `LightningModel "Lightning"` on (a copy of) `mpLightningList` in `IslandDisplayGroup`, and
/// its `LightningLogic` (listener self) as `mLightningLogic`.
fn create_lightning(world: &mut World, this: Entity) {
    let lightning_points = world.obj::<ControllerGodPowers>(this).lightning_list.clone();
    let lightning_model_object = world.spawn(Isa(&lightning_model::LIGHTNING_MODEL)).id();
    lightning_model::init_with_id(world, lightning_model_object, "Lightning", lightning_points);
    display::object_mut(world, lightning_model_object).pos.z = SPRITE_Z;
    let island_display_group = island_display_group(world, this);
    display::move_to_display_group(world, lightning_model_object, island_display_group);

    let lightning = lightning_logic::spawn(world, lightning_model_object);
    world.obj_mut::<ControllerGodPowers>(this).lightning_logic = Some(lightning);
    lightning_logic::set_lightning_logic_listener(world, lightning, this);
}

/// `mIslandDisplayGroup`.
fn island_display_group(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<ControllerIslandLocations>(this).island_display_group
}

/// `mUnderwaterDisplayGroup`.
fn underwater_display_group(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<ControllerIslandLocations>(this).underwater_display_group
}

/// `setXPos:0`, `setYPos:0`, `setZPos:-277.12`, `moveToDisplayGroup:` for a new hurricane or
/// whirlpool model.
fn place_power_model(world: &mut World, model: Entity, display_group: Option<Entity>) {
    {
        let mut model_display = display::object_mut(world, model);
        model_display.pos.x = 0.0;
        model_display.pos.y = 0.0;
        model_display.pos.z = SPRITE_Z;
    }
    display::move_to_display_group(world, model, display_group);
}

/// The `(frameWidth, frameHeight)` of a library texture (`+0x28`, `+0x2c` of its
/// `BCTextureDef`; zero for an unknown id).
fn texture_frame_size(world: &World, texture_id: &str) -> (f32, f32) {
    let texture = library::texture_def_id(world, texture_id);
    let Some(texture) = texture else {
        return (0.0, 0.0);
    };
    let def = world.resource::<Textures>().def(texture);
    (def.frame_width, def.frame_height)
}

/// A finger glow: a `BCMultiModel` with the given id, centred by the frame of `texture_id`,
/// at `pos` in `display_group`, driven by a new `FingerTrailLogic` (returned; the caller
/// stores it, aims it and sets its behaviour).
fn create_finger_trail(world: &mut World, texture_id: &str, model_id: &str, pos: Vec4, display_group: Option<Entity>) -> Entity {
    let (frame_width, frame_height) = texture_frame_size(world, texture_id);
    let glow = display::multi_model::spawn(world, Some(model_id));
    {
        let mut glow_display = display::object_mut(world, glow);
        glow_display.origin_pos.x = frame_width * -0.5;
        glow_display.origin_pos.y = frame_height * -0.5;
        glow_display.pos.x = pos.x;
        glow_display.pos.y = pos.y;
        glow_display.pos.z = SPRITE_Z;
    }
    display::move_to_display_group(world, glow, display_group);
    finger_trail_logic::spawn(world, glow)
}

/// `drawRainbow:` starting a stroke: the `RainbowTrail` sparkle at the start point (no texture
/// centring, no rotation) as `mRainbowTrailLogic`.
fn start_rainbow_trail(world: &mut World, this: Entity) {
    let last_rainbow_pos = world.obj::<ControllerGodPowers>(this).last_rainbow_pos;

    let sparkle = display::multi_model::spawn(world, Some(RAINBOW_TRAIL));
    {
        let mut sparkle_display = display::object_mut(world, sparkle);
        sparkle_display.pos.x = last_rainbow_pos.x;
        sparkle_display.pos.y = last_rainbow_pos.y;
        sparkle_display.pos.z = SPRITE_Z;
        sparkle_display.rot.z = 0.0;
    }
    let island_display_group = island_display_group(world, this);
    display::move_to_display_group(world, sparkle, island_display_group);

    let rainbow_trail = finger_trail_logic::spawn(world, sparkle);
    world.obj_mut::<ControllerGodPowers>(this).rainbow_trail_logic = Some(rainbow_trail);
    finger_trail_logic::set_target_x_pos(world, rainbow_trail, last_rainbow_pos.x, last_rainbow_pos.y);
    logic::set_behavior(world, rainbow_trail, Some(RAINBOW_TRAIL));
}

/// `[[trail displayObject] removeFromDisplayGroup]; [trail shutdown]; [trail release]` for a
/// finger trail the controller alone owns (the caller then clears its ivar).
fn discard_finger_trail(world: &mut World, trail: Option<Entity>) {
    let Some(trail) = trail else {
        return;
    };
    let trail_display_object = logic::display_object(world, trail);
    display::remove_from_display_group(world, trail_display_object);
    logic::shutdown(world, trail);
    logic::dealloc(world, trail);
}

/// `[[BCLibrary getDisplayMarkerById:@"UnderwaterPygmyDisplayMarker"] insertBefore:object]`.
fn insert_before_underwater_pygmy_marker(world: &mut World, object: Entity) {
    let underwater_pygmy_marker = library::display_marker(world, UNDERWATER_PYGMY_DISPLAY_MARKER_ID);
    if let Some(underwater_pygmy_marker) = underwater_pygmy_marker {
        display::insert_before(world, underwater_pygmy_marker, Some(object));
    }
}

/// `dragUnderwaterPower:`'s trail puff at `pos`: a `MultiModelLogic` on an
/// `UnderwaterPowerTrail` model, kept in `mUnderwaterPowerTrailLogicArray`, playing the temple
/// mode's trail.
fn leave_underwater_power_trail_puff(world: &mut World, this: Entity, pos: Vec4) {
    let puff = display::multi_model::spawn(world, Some(UNDERWATER_POWER_TRAIL_MODEL_ID));
    {
        let mut puff_display = display::object_mut(world, puff);
        puff_display.pos.x = pos.x;
        puff_display.pos.y = pos.y;
        puff_display.pos.z = SPRITE_Z;
        puff_display.rot.z = 0.0;
    }
    insert_before_underwater_pygmy_marker(world, puff);

    // Original bug, kept: the puff gets no listener, so its `UnderwaterPowerTrailComplete`
    // event never reaches `onMultiModelEvent:eventId:` and the puff stays (in the tree and the
    // array) until the coral reef is left.
    let puff_logic = logic::multi_model::spawn(world, puff);
    world.obj_mut::<ControllerGodPowers>(this).underwater_power_trail_logic_array.push(puff_logic);

    let trail_behavior = match underwater_temple_mode(world, this) {
        1 => "UnderwaterPowerColdTrail",
        2 => "UnderwaterPowerHeatTrail",
        3 => "UnderwaterPowerOilTrail",
        4 => "UnderwaterPowerAcidTrail",
        5 => "UnderwaterPowerAirTrail",
        _ => return,
    };
    logic::set_behavior(world, puff_logic, Some(trail_behavior));
}

/// `fireworks:::` launching one firework: a `FireworkSpark` model somewhere in the sky after
/// `mFireworkDisplayMarker`, red on Graveyard island and multicoloured elsewhere, with its
/// `FireworkLogic` (listener self) in `mFireworkLogicArray`.
fn launch_firework(world: &mut World, this: Entity) {
    let spark_texture = library::texture_def_id(world, FIREWORK_SPARK_TEXTURE_ID);
    let firework_model = display::model::spawn(world, Some(FIREWORK_MODEL_ID), spark_texture);

    let x_roll = world.resource_mut::<Random>().arc4random_mod(3801);
    let y_roll = world.resource_mut::<Random>().arc4random_mod(1301);
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let color_roll = world.resource_mut::<Random>().arc4random();
    let (red, green, blue) = if island_location_state == LOCATION_GRAVEYARD_ISLAND {
        GRAVEYARD_FIREWORK_COLORS[(color_roll % 5) as usize]
    } else {
        FIREWORK_COLORS[(color_roll & 7) as usize]
    };

    {
        let mut firework_display = display::object_mut(world, firework_model);
        firework_display.color = true;
        firework_display.red = red;
        firework_display.green = green;
        firework_display.blue = blue;
        firework_display.pos.x = (x_roll + 500) as f32 / 10.0;
        firework_display.pos.y = (y_roll + 1900) as f32 / 10.0;
        firework_display.pos.z = SPRITE_Z;
        firework_display.origin_pos.x = FIREWORK_ORIGIN_POS;
        firework_display.origin_pos.y = FIREWORK_ORIGIN_POS;
    }

    let firework_display_marker = world.obj::<ControllerGodPowers>(this).firework_display_marker;
    if let Some(firework_display_marker) = firework_display_marker {
        display::insert_after(world, firework_display_marker, Some(firework_model));
    }

    let firework = firework_logic::spawn(world, firework_model);
    firework_logic::set_firework_logic_listener(world, firework, this);
    world.obj_mut::<ControllerGodPowers>(this).firework_logic_array.push(firework);
}

/// `RainbowModel` of `mRainbowLogic`, whose `path_list` is the shared `mpRainbowList` (see
/// [`ControllerGodPowers::rainbow_list`]); `None` before `loadGame` created it.
fn rainbow_model(world: &World, this: Entity) -> Option<Entity> {
    let rainbow = world.obj::<ControllerGodPowers>(this).rainbow_logic?;
    Some(logic::display_object(world, rainbow))
}

/// `mpRainbowList->clear()`.
fn clear_rainbow_points(world: &mut World, this: Entity) {
    if let Some(rainbow_model) = rainbow_model(world, this) {
        world.obj_mut::<RainbowModel>(rainbow_model).path_list.clear();
    }
}

/// `mpRainbowList->push_back(point)`.
fn push_rainbow_point(world: &mut World, this: Entity, point: Vec4) {
    if let Some(rainbow_model) = rainbow_model(world, this) {
        world.obj_mut::<RainbowModel>(rainbow_model).path_list.push(point);
    }
}
