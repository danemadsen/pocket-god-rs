//! `PocketGodViewController`: the sky and the weather. Sun and moon drags with the day,
//! sunset and night transitions, the pinch-sized sun, the eclipse, the sky gradients and the
//! clear colour, the storm (weather button and cloud flick), the fair-weather clouds, the storm
//! clouds and the bat summoned by tapping the moon.

use crate::prelude::*;
use super::*;

use crate::logic::island_objects::boulder_logic;
use crate::logic::sky_powers::hurricane_logic::{self, HurricaneLogic};
use crate::logic::sky_weather::{bat_logic, cloud_logic, moon_logic, storm_cloud_logic, sun_logic};
use crate::managers::idol_manager;
use pg_engine::display;
use pg_engine::display::gradient;
use pg_engine::display::group;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::prefs;
use pg_engine::sequence;
use pg_engine::texture::Textures;
use pg_engine::view::BcView;

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerSkyWeather {
    /// `mCloudFrontDisplayMarker`: `@"BCDisplayMarker"`
    pub cloud_front_display_marker: Option<Entity>,

    /// `mCloudBehindDisplayMarker`: `@"BCDisplayMarker"`
    pub cloud_behind_display_marker: Option<Entity>,

    /// `mBatTapTouchCount`: `i`
    pub bat_tap_touch_count: i32,

    /// `mBatTapTouchTime`: `f`
    pub bat_tap_touch_time: f32,

    /// `mBatLogic`: `@"BatLogic"`
    pub bat_logic: Option<Entity>,

    /// `mWeatherSequence`: `@"BCSequence"`
    pub weather_sequence: Option<Entity>,

    /// `mSunLogic`: `@"SunLogic"`
    pub sun_logic: Option<Entity>,

    /// `mDragSunTouch`: `@"BCTouch"`
    pub drag_sun_touch: Option<Entity>,

    /// `mDragSunTouchPos`: `{?="x"f"y"f"z"f"w"f}`
    pub drag_sun_touch_pos: Vec4,

    /// `mSunReflectionDisplayObject`: `@"BCDisplayObject"`
    pub sun_reflection_display_object: Option<Entity>,

    /// `mbSunReflectionVisible`: `B`
    pub sun_reflection_visible: bool,

    /// `mDragSunXOrigin`: `f`
    pub drag_sun_x_origin: f32,

    /// `mSunState`: `i`
    pub sun_state: i32,

    /// `mStarsLogic`: `@"MultiModelLogic"`
    pub stars_logic: Option<Entity>,

    /// `mMoonLogic`: `@"MoonLogic"`
    pub moon_logic: Option<Entity>,

    /// `mDragMoonTouch`: `@"BCTouch"`
    pub drag_moon_touch: Option<Entity>,

    /// `mDragMoonTouchPos`: `{?="x"f"y"f"z"f"w"f}`
    pub drag_moon_touch_pos: Vec4,

    /// `mbNightActive`: `B`
    pub night_active: bool,

    /// `mbSunsetActive`: `B`
    pub sunset_active: bool,

    /// `mCloudGradientTargetFade`: `f`
    pub cloud_gradient_target_fade: f32,

    /// `mCloudGradientFade`: `f`
    pub cloud_gradient_fade: f32,

    /// `mSunsetGradientTargetFade`: `f`
    pub sunset_gradient_target_fade: f32,

    /// `mSunsetGradientFade`: `f`
    pub sunset_gradient_fade: f32,

    /// `mNightGradientTargetFade`: `f`
    pub night_gradient_target_fade: f32,

    /// `mNightGradientFade`: `f`
    pub night_gradient_fade: f32,

    /// `mEclipseGradientTargetFade`: `f`
    pub eclipse_gradient_target_fade: f32,

    /// `mEclipseGradientFade`: `f`
    pub eclipse_gradient_fade: f32,

    /// `mBigSunGradientTargetFade`: `f`
    pub big_sun_gradient_target_fade: f32,

    /// `mBigSunGradientFade`: `f`
    pub big_sun_gradient_fade: f32,

    /// `mSmallSunGradientTargetFade`: `f`
    pub small_sun_gradient_target_fade: f32,

    /// `mSmallSunGradientFade`: `f`
    pub small_sun_gradient_fade: f32,

    /// `mApocalypseGradientTargetFade`: `f`
    pub apocalypse_gradient_target_fade: f32,

    /// `mApocalypseGradientFade`: `f`
    pub apocalypse_gradient_fade: f32,

    /// `mpSunsetGradient`: `^{BCGradientDef=i[0{BCGradientColor=fffff}]}`
    pub sunset_gradient: Option<String>,

    /// `mpNightGradient`: `^{BCGradientDef=i[0{BCGradientColor=fffff}]}`
    pub night_gradient: Option<String>,

    /// `mpDayGradient`: `^{BCGradientDef=i[0{BCGradientColor=fffff}]}`
    pub day_gradient: Option<String>,

    /// `mpCloudyGradient`: `^{BCGradientDef=i[0{BCGradientColor=fffff}]}`
    pub cloudy_gradient: Option<String>,

    /// `mpEclipseGradient`: `^{BCGradientDef=i[0{BCGradientColor=fffff}]}`
    pub eclipse_gradient: Option<String>,

    /// `mpBigSunGradient`: `^{BCGradientDef=i[0{BCGradientColor=fffff}]}`
    pub big_sun_gradient: Option<String>,

    /// `mpSmallSunGradient`: `^{BCGradientDef=i[0{BCGradientColor=fffff}]}`
    pub small_sun_gradient: Option<String>,

    /// `mpApocalypseSunsetGradient`: `^{BCGradientDef=i[0{BCGradientColor=fffff}]}`
    pub apocalypse_sunset_gradient: Option<String>,

    /// `mpApocalypseNightGradient`: `^{BCGradientDef=i[0{BCGradientColor=fffff}]}`
    pub apocalypse_night_gradient: Option<String>,

    /// `mpApocalypseDayGradient`: `^{BCGradientDef=i[0{BCGradientColor=fffff}]}`
    pub apocalypse_day_gradient: Option<String>,

    /// `mpApocalypseCloudyGradient`: `^{BCGradientDef=i[0{BCGradientColor=fffff}]}`
    pub apocalypse_cloudy_gradient: Option<String>,

    /// `mWeatherState`: `i`
    pub weather_state: i32,

    /// `mbDragClouds`: `B`
    pub drag_clouds: bool,

    /// `mCloudDragVel`: `f`
    pub cloud_drag_vel: f32,

    /// `mDragCloudsAveVel`: `f`
    pub drag_clouds_ave_vel: f32,

    /// `mDragCloudsTimeStamp`: `f`
    pub drag_clouds_time_stamp: f32,

    /// `mDragCloudsTouch`: `@"BCTouch"`
    pub drag_clouds_touch: Option<Entity>,

    /// `mDragCloudsTouchPos`: `{?="x"f"y"f"z"f"w"f}`
    pub drag_clouds_touch_pos: Vec4,

    /// `mCloudLogicArray`: `@"NSMutableArray"`
    pub cloud_logic_array: Vec<Entity>,

    /// `mFrontStormCloudLogic`: `@"StormCloudLogic"`
    pub front_storm_cloud_logic: Option<Entity>,

    /// `mBehindStormCloudLogic`: `@"StormCloudLogic"`
    pub behind_storm_cloud_logic: Option<Entity>,

    /// `mWeatherButtonLogic`: `@"BCButtonLogic"`
    pub weather_button_logic: Option<Entity>,
}

/// `mWeatherState`: clear sky.
const WEATHER_STATE_CLEAR: i32 = 0;

/// `mWeatherState`: storm.
const WEATHER_STATE_STORM: i32 = 1;

/// `mSunState`: the normal sun.
const SUN_STATE_NORMAL: i32 = 0;

/// `mSunState`: the big (hot) sun.
const SUN_STATE_BIG: i32 = 1;

/// `mSunState`: the small (cold) sun.
const SUN_STATE_SMALL: i32 = 2;

/// `mIslandLocationState` of Ape island, where the sun has no reflection.
const LOCATION_APE_ISLAND: i32 = 5;

/// The highest y the sun and the moon can be dragged to, and the height they rise to.
const SKY_TOP_Y_POS: f32 = 245.0;

/// The distance from the horizon over which the dragged sun blends the sky into sunset.
const SUNSET_BLEND_DISTANCE: f32 = 40.0;

/// The cloud speed a weather change starts its transition script with.
const WEATHER_TRANSITION_CLOUD_VEL: f32 = 4000.0;

/// The cloud flick speed (either way) that toggles the storm.
const STORM_FLICK_VEL: f32 = 3000.0;

/// A cloud drag whose angle from the horizontal lies between these two (either way, in
/// radians: 30° and 150°) is a vertical swipe and lets the clouds go.
const CLOUD_DRAG_STEEP_MIN_ANGLE: f32 = 0.5235988;
const CLOUD_DRAG_STEEP_MAX_ANGLE: f32 = 2.6179938;

/// The per-frame easing of the sky gradient weights toward their targets.
const GRADIENT_EASE: f32 = 0.13;

/// A gradient weight closer than this to its target snaps to it (compared in double).
const GRADIENT_SNAP_DISTANCE: f64 = 0.01;

/// The number of fair-weather clouds `createClouds` makes.
const CLOUD_COUNT: i32 = 4;

/// `-[PocketGodViewController setSunState:]` @ 0x10007d4a8
///
/// The pinch-sized sun: the big or small sun gradient targets follow the state, the
/// `ChangeSunState` script runs and the sun logic resizes.
pub fn set_sun_state(world: &mut World, this: Entity, sun_state: i32) {
    let current_sun_state = world.obj::<ControllerSkyWeather>(this).sun_state;
    if current_sun_state == sun_state {
        return;
    }

    {
        let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
        sky.sun_state = sun_state;
        match sun_state {
            SUN_STATE_NORMAL => {
                sky.big_sun_gradient_target_fade = 0.0;
                sky.small_sun_gradient_target_fade = 0.0;
            }
            SUN_STATE_BIG => {
                sky.big_sun_gradient_target_fade = 1.0;
                sky.small_sun_gradient_target_fade = 0.0;
            }
            SUN_STATE_SMALL => {
                sky.big_sun_gradient_target_fade = 0.0;
                sky.small_sun_gradient_target_fade = 1.0;
            }
            _ => {}
        }
    }

    let main_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
    set_sequence_def_by_id(world, main_sequence, "ChangeSunState");

    let (sun, stored_sun_state) = {
        let sky = world.obj::<ControllerSkyWeather>(this);
        (sky.sun_logic, sky.sun_state)
    };
    if let Some(sun) = sun {
        sun_logic::set_sun_state(world, sun, stored_sun_state);
    }
}

/// `-[PocketGodViewController setWeatherState:startup:]` @ 0x10007d5c4
///
/// Stores the state as given (the callers toggle). A storm ends an Ooga Jump in progress.
/// Outside startup either change runs its transition script (`StartStorm` / `StopStorm`) with
/// the clouds moving at 4000, saves the option and animates the weather button; at startup the
/// sky is set up in its end state at once.
pub fn set_weather_state(world: &mut World, this: Entity, weather_state: i32, startup: bool) {
    world.obj_mut::<ControllerSkyWeather>(this).weather_state = weather_state;

    if weather_state == WEATHER_STATE_STORM {
        let doodle_jump_pygmy_logic = world.obj::<ControllerMinigames>(this).doodle_jump_pygmy_logic;
        if doodle_jump_pygmy_logic.is_some() {
            remove_doodle_clouds(world, this);
            hide_sky_tiles(world, this);
            knock_down_doodle_jump_pygmy(world, this);
        }

        if startup {
            world.obj_mut::<ControllerSkyWeather>(this).cloud_gradient_target_fade = 1.0;
            let storm_clouds = storm_clouds(world, this);
            for storm_cloud in storm_clouds.into_iter().flatten() {
                storm_cloud_logic::start_visible(world, storm_cloud);
            }

            let weather_button_logic = world.obj::<ControllerSkyWeather>(this).weather_button_logic;
            set_logic_behavior(world, weather_button_logic, "StartWeatherOn");
        } else {
            world.obj_mut::<ControllerSkyWeather>(this).cloud_drag_vel = WEATHER_TRANSITION_CLOUD_VEL;
            let weather_sequence = world.obj::<ControllerSkyWeather>(this).weather_sequence;
            set_sequence_def_by_id(world, weather_sequence, "StartStorm");

            let saved_weather_state = world.obj::<ControllerSkyWeather>(this).weather_state;
            set_option_int_preference(world, this, "WeatherHazard", saved_weather_state);

            let weather_button_logic = world.obj::<ControllerSkyWeather>(this).weather_button_logic;
            set_logic_behavior(world, weather_button_logic, "ToggleWeatherOn");
        }

        trigger_idol_episode_event(world, this, "StormClouds");
        return;
    }

    if weather_state == WEATHER_STATE_CLEAR {
        if startup {
            create_clouds(world, this);
            world.obj_mut::<ControllerSkyWeather>(this).cloud_gradient_target_fade = 0.0;
            return;
        }

        world.obj_mut::<ControllerSkyWeather>(this).cloud_drag_vel = WEATHER_TRANSITION_CLOUD_VEL;
        let weather_sequence = world.obj::<ControllerSkyWeather>(this).weather_sequence;
        set_sequence_def_by_id(world, weather_sequence, "StopStorm");

        let saved_weather_state = world.obj::<ControllerSkyWeather>(this).weather_state;
        set_option_int_preference(world, this, "WeatherHazard", saved_weather_state);

        let weather_button_logic = world.obj::<ControllerSkyWeather>(this).weather_button_logic;
        set_logic_behavior(world, weather_button_logic, "ToggleWeatherOff");
    }
}

/// `-[PocketGodViewController isStormCloudEnabled]` @ 0x10008782c
pub fn is_storm_cloud_enabled(world: &mut World, this: Entity) -> bool {
    let weather_hazard = option_int_preference(world, this, "WeatherHazard");
    weather_hazard == WEATHER_STATE_STORM
}

/// `-[PocketGodViewController (Game) checkEclipse]` @ 0x100159098
///
/// A boulder dragged within 20 units of the normal-sized sun (outside the night) eclipses it
/// by `1 − distance / 20`; crossing 50 % runs `Eclipse`. Otherwise the eclipse clears.
pub fn check_eclipse(world: &mut World, this: Entity) {
    let dragged_boulder_logic = world.obj::<ControllerWorldObjectEvents>(this).dragged_boulder_logic;
    let boulder = world.obj::<ControllerWorldRegistry>(this).boulder_logic;
    let (night_active, sun_state, sun) = {
        let sky = world.obj::<ControllerSkyWeather>(this);
        (sky.night_active, sky.sun_state, sky.sun_logic)
    };

    let mut eclipse_percent = None;
    if let Some(dragged_boulder_logic) = dragged_boulder_logic
        && !night_active
        && sun_state == SUN_STATE_NORMAL
        && let Some(sun) = sun
    {
        let boulder_pos = logic_pos(world, dragged_boulder_logic);
        let sun_pos = logic_pos(world, sun);
        let delta_x = sun_pos.x - boulder_pos.x;
        let delta_y = sun_pos.y - boulder_pos.y;
        let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();
        if distance < 20.0 {
            eclipse_percent = Some(distance / -20.0 + 1.0);
        }
    }

    let Some(eclipse_percent) = eclipse_percent else {
        world.obj_mut::<ControllerSkyWeather>(this).eclipse_gradient_target_fade = 0.0;
        if let Some(boulder) = boulder {
            boulder_logic::set_eclipse_percent(world, boulder, 0.0);
        }
        if let Some(sun) = sun {
            sun_logic::set_eclipse_percent(world, sun, 0.0, None);
        }
        return;
    };

    let previous_eclipse_target = world.obj::<ControllerSkyWeather>(this).eclipse_gradient_target_fade;
    if eclipse_percent > 0.5 && previous_eclipse_target <= 0.5 {
        let main_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
        set_sequence_def_by_id(world, main_sequence, "Eclipse");
    }

    world.obj_mut::<ControllerSkyWeather>(this).eclipse_gradient_target_fade = eclipse_percent;
    if let Some(boulder) = boulder {
        boulder_logic::set_eclipse_percent(world, boulder, eclipse_percent);
    }
    if let Some(sun) = sun {
        sun_logic::set_eclipse_percent(world, sun, eclipse_percent, boulder);
    }
}

/// `-[PocketGodViewController (Game) updateGradients]` @ 0x1001592b8
///
/// While a weight still moves toward its target, the sky `Gradient` is rebuilt: the day
/// gradient, then each weighted gradient faded in by its weight (the apocalypse variants by
/// apocalypse × weight). The first stop's colour becomes the clear colour.
pub fn update_gradients(world: &mut World, this: Entity) {
    // The original tests the defs first; both tests are plain reads, and testing the weights
    // first spares copying the def ids on every settled frame.
    let sky = world.obj::<ControllerSkyWeather>(this);
    let weights_settled = sky.cloud_gradient_fade == sky.cloud_gradient_target_fade
        && sky.sunset_gradient_fade == sky.sunset_gradient_target_fade
        && sky.night_gradient_fade == sky.night_gradient_target_fade
        && sky.eclipse_gradient_fade == sky.eclipse_gradient_target_fade
        && sky.big_sun_gradient_fade == sky.big_sun_gradient_target_fade
        && sky.small_sun_gradient_fade == sky.small_sun_gradient_target_fade
        && sky.apocalypse_gradient_fade == sky.apocalypse_gradient_target_fade;
    if weights_settled {
        return;
    }

    let Some(gradient_defs) = SkyGradientDefs::from_ivars(sky) else {
        return;
    };

    let sky_gradient = island_display_object(world, this, "Gradient");

    let (cloud, sunset, night, eclipse, big_sun, small_sun, apocalypse) = {
        let mut sky_ref = world.obj_mut::<ControllerSkyWeather>(this);
        let sky = &mut *sky_ref;

        sky.cloud_gradient_fade = eased_weight(sky.cloud_gradient_fade, sky.cloud_gradient_target_fade);
        sky.sunset_gradient_fade = eased_weight(sky.sunset_gradient_fade, sky.sunset_gradient_target_fade);
        sky.night_gradient_fade = eased_weight(sky.night_gradient_fade, sky.night_gradient_target_fade);
        sky.eclipse_gradient_fade = eased_weight(sky.eclipse_gradient_fade, sky.eclipse_gradient_target_fade);
        sky.big_sun_gradient_fade = eased_weight(sky.big_sun_gradient_fade, sky.big_sun_gradient_target_fade);
        sky.small_sun_gradient_fade = eased_weight(sky.small_sun_gradient_fade, sky.small_sun_gradient_target_fade);
        sky.apocalypse_gradient_fade = eased_weight(sky.apocalypse_gradient_fade, sky.apocalypse_gradient_target_fade);

        sky.cloud_gradient_fade = snapped_weight(sky.cloud_gradient_fade, sky.cloud_gradient_target_fade);
        sky.sunset_gradient_fade = snapped_weight(sky.sunset_gradient_fade, sky.sunset_gradient_target_fade);
        sky.night_gradient_fade = snapped_weight(sky.night_gradient_fade, sky.night_gradient_target_fade);
        sky.eclipse_gradient_fade = snapped_weight(sky.eclipse_gradient_fade, sky.eclipse_gradient_target_fade);
        sky.big_sun_gradient_fade = snapped_weight(sky.big_sun_gradient_fade, sky.big_sun_gradient_target_fade);
        sky.small_sun_gradient_fade = snapped_weight(sky.small_sun_gradient_fade, sky.small_sun_gradient_target_fade);
        sky.apocalypse_gradient_fade = snapped_weight(sky.apocalypse_gradient_fade, sky.apocalypse_gradient_target_fade);

        (
            sky.cloud_gradient_fade,
            sky.sunset_gradient_fade,
            sky.night_gradient_fade,
            sky.eclipse_gradient_fade,
            sky.big_sun_gradient_fade,
            sky.small_sun_gradient_fade,
            sky.apocalypse_gradient_fade,
        )
    };

    // Without the `Gradient` object the original crashes reading the colour of
    // `[nil gradientColorAtIndex:0]`.
    let Some(sky_gradient) = sky_gradient else {
        return;
    };

    gradient::set_gradient(world, sky_gradient, &gradient_defs.day);
    if apocalypse != 0.0 {
        gradient::fade_gradient(world, sky_gradient, &gradient_defs.apocalypse_day, apocalypse);
    }
    if cloud != 0.0 {
        gradient::fade_gradient(world, sky_gradient, &gradient_defs.cloudy, cloud);
        if apocalypse != 0.0 {
            let apocalypse_cloud = apocalypse * cloud;
            gradient::fade_gradient(world, sky_gradient, &gradient_defs.apocalypse_cloudy, apocalypse_cloud);
        }
    }
    if big_sun != 0.0 {
        gradient::fade_gradient(world, sky_gradient, &gradient_defs.big_sun, big_sun);
    }
    if small_sun != 0.0 {
        gradient::fade_gradient(world, sky_gradient, &gradient_defs.small_sun, small_sun);
    }
    if sunset != 0.0 {
        gradient::fade_gradient(world, sky_gradient, &gradient_defs.sunset, sunset);
        if apocalypse != 0.0 {
            let apocalypse_sunset = apocalypse * sunset;
            gradient::fade_gradient(world, sky_gradient, &gradient_defs.apocalypse_sunset, apocalypse_sunset);
        }
    }
    if night != 0.0 {
        gradient::fade_gradient(world, sky_gradient, &gradient_defs.night, night);
        if apocalypse != 0.0 {
            let apocalypse_night = apocalypse * night;
            gradient::fade_gradient(world, sky_gradient, &gradient_defs.apocalypse_night, apocalypse_night);
        }
    }
    if eclipse != 0.0 {
        gradient::fade_gradient(world, sky_gradient, &gradient_defs.eclipse, eclipse);
    }

    let first_stop = gradient::gradient_color_at_index(world, sky_gradient, 0);
    let clear_color = [first_stop.r, first_stop.g, first_stop.b, 1.0];
    world.resource_mut::<BcView>().clear_color = clear_color;
}

/// `-[PocketGodViewController (Game) setCloudGradientFade:::]` @ 0x10015e32c
pub fn seq_set_cloud_gradient_fade(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let fade = call.float("fade");
    world.obj_mut::<ControllerSkyWeather>(this).cloud_gradient_target_fade = fade;
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) transitionInStormClouds:::]` @ 0x10015e684
pub fn seq_transition_in_storm_clouds(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let cloud_drag_vel = world.obj::<ControllerSkyWeather>(this).cloud_drag_vel;
    let storm_clouds = storm_clouds(world, this);
    for storm_cloud in storm_clouds.into_iter().flatten() {
        storm_cloud_logic::transition_in(world, storm_cloud, cloud_drag_vel);
    }
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) transitionOutStormClouds:::]` @ 0x10015e6fc
pub fn seq_transition_out_storm_clouds(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let cloud_drag_vel = world.obj::<ControllerSkyWeather>(this).cloud_drag_vel;
    let storm_clouds = storm_clouds(world, this);
    for storm_cloud in storm_clouds.into_iter().flatten() {
        storm_cloud_logic::transition_out(world, storm_cloud, cloud_drag_vel);
    }
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) createClouds]` @ 0x10015e774
///
/// Replaces the clouds with 4 new `CloudLogic`s (indices 0 to 3) on `NewCloud` models just
/// after `CloudBehindDisplayMarker`.
pub fn create_clouds(world: &mut World, this: Entity) {
    let cloud_texture = library::texture_def_id(world, "Cloud1Texture");

    let old_clouds = world.obj::<ControllerSkyWeather>(this).cloud_logic_array.clone();
    for &old_cloud in &old_clouds {
        let old_cloud_display_object = logic::display_object(world, old_cloud);
        display::remove_from_display_group(world, old_cloud_display_object);
        logic::shutdown(world, old_cloud);
    }

    // `removeAllObjects` drops the array's references, the last ones of the shut-down clouds.
    world.obj_mut::<ControllerSkyWeather>(this).cloud_logic_array.clear();
    for old_cloud in old_clouds {
        dealloc_sprite(world, old_cloud);
    }

    for cloud_index in 0..CLOUD_COUNT {
        let cloud_model = display::model::spawn(world, Some("NewCloud"), cloud_texture);
        let cloud_behind_display_marker = world.obj::<ControllerSkyWeather>(this).cloud_behind_display_marker;
        if let Some(cloud_behind_display_marker) = cloud_behind_display_marker {
            display::insert_after(world, cloud_behind_display_marker, Some(cloud_model));
        }

        let cloud = cloud_logic::spawn(world, cloud_model, cloud_index);
        cloud_logic::set_cloud_logic_listener(world, cloud, this);
        world.obj_mut::<ControllerSkyWeather>(this).cloud_logic_array.push(cloud);
    }
}

/// `-[PocketGodViewController (Game) transitionInClouds:::]` @ 0x10015ea30
///
/// Ends the hurricane, makes new clouds and sends them in at the drag speed.
pub fn seq_transition_in_clouds(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let hurricane = world.obj::<ControllerGodPowers>(this).hurricane_logic;
    if let Some(hurricane) = hurricane {
        hurricane_logic::stop_hurricane(world, hurricane);
    }

    create_clouds(world, this);

    let (clouds, cloud_drag_vel) = {
        let sky = world.obj::<ControllerSkyWeather>(this);
        (sky.cloud_logic_array.clone(), sky.cloud_drag_vel)
    };
    for cloud in clouds {
        cloud_logic::transition_in(world, cloud, cloud_drag_vel);
    }
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) transitionOutClouds:::]` @ 0x10015eb94
pub fn seq_transition_out_clouds(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let (clouds, cloud_drag_vel) = {
        let sky = world.obj::<ControllerSkyWeather>(this);
        (sky.cloud_logic_array.clone(), sky.cloud_drag_vel)
    };
    for cloud in clouds {
        cloud_logic::transition_out(world, cloud, cloud_drag_vel);
    }
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) onCloudDie:]` @ 0x10015f2ac
///
/// A cloud that drifted off screen while transitioning out. It calls this from its own
/// `drift:::` item, which touches nothing of the cloud logic afterwards.
pub fn on_cloud_die(world: &mut World, this: Entity, cloud: Entity) {
    let cloud_display_object = logic::display_object(world, cloud);
    display::remove_from_display_group(world, cloud_display_object);

    let was_in_array = {
        let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
        let was_in_array = sky.cloud_logic_array.contains(&cloud);
        sky.cloud_logic_array.retain(|logic| *logic != cloud);
        was_in_array
    };
    logic::shutdown(world, cloud);

    // `removeObject:` dropped the array's reference and `shutdown` the frame listener's, the
    // cloud's last two.
    if was_in_array {
        dealloc_sprite(world, cloud);
    }
}

/// `-[PocketGodViewController (Game) startDragClouds:timeStamp:]` @ 0x10015f324
///
/// `pos`: the touch in island coordinates.
pub fn start_drag_clouds(world: &mut World, this: Entity, pos: Vec4, time_stamp: f32) {
    let weather_state = {
        let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
        sky.drag_clouds = true;
        sky.drag_clouds_ave_vel = -1.0;
        sky.drag_clouds_time_stamp = time_stamp;
        sky.cloud_drag_vel = 0.0;
        sky.drag_clouds_touch_pos = pos;
        sky.weather_state
    };

    if weather_state == WEATHER_STATE_STORM {
        let storm_clouds = storm_clouds(world, this);
        for storm_cloud in storm_clouds.into_iter().flatten() {
            storm_cloud_logic::start_dragging(world, storm_cloud);
        }
        return;
    }

    let clouds = world.obj::<ControllerSkyWeather>(this).cloud_logic_array.clone();
    for cloud in clouds {
        cloud_logic::start_dragging(world, cloud);
    }
}

/// `-[PocketGodViewController (Game) stopDragClouds]` @ 0x10015f50c
///
/// The clouds keep the drag speed. A flick of at least 3000 either way toggles the storm,
/// unless a hurricane is blowing.
pub fn stop_drag_clouds(world: &mut World, this: Entity) {
    let (weather_state, cloud_drag_vel) = {
        let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
        sky.drag_clouds = false;
        (sky.weather_state, sky.cloud_drag_vel)
    };

    if weather_state == WEATHER_STATE_STORM {
        let storm_clouds = storm_clouds(world, this);
        for storm_cloud in storm_clouds.into_iter().flatten() {
            storm_cloud_logic::stop_dragging(world, storm_cloud, cloud_drag_vel);
        }
    } else {
        let clouds = world.obj::<ControllerSkyWeather>(this).cloud_logic_array.clone();
        for cloud in clouds {
            cloud_logic::stop_dragging(world, cloud, cloud_drag_vel);
        }
    }

    let cloud_drag_vel = world.obj::<ControllerSkyWeather>(this).cloud_drag_vel;
    let flicked = cloud_drag_vel.abs() >= STORM_FLICK_VEL;
    if flicked && !is_hurricane_active(world, this) {
        let weather_state = world.obj::<ControllerSkyWeather>(this).weather_state;
        if weather_state == WEATHER_STATE_CLEAR {
            start_storm_by_flick(world, this);
        } else {
            // Unlike the storm direction, the weather button is left as it is.
            world.obj_mut::<ControllerSkyWeather>(this).weather_state = WEATHER_STATE_CLEAR;
            set_option_int_preference(world, this, "WeatherHazard", WEATHER_STATE_CLEAR);

            let weather_sequence = world.obj::<ControllerSkyWeather>(this).weather_sequence;
            set_sequence_def_by_id(world, weather_sequence, "StopStorm");
        }
    }

    world.obj_mut::<ControllerSkyWeather>(this).drag_clouds_touch = None;
}

/// `stopDragClouds`: a flick that brings the storm. It ends an Ooga Jump in progress.
fn start_storm_by_flick(world: &mut World, this: Entity) {
    let doodle_jump_pygmy_logic = world.obj::<ControllerMinigames>(this).doodle_jump_pygmy_logic;
    if doodle_jump_pygmy_logic.is_some() {
        remove_doodle_clouds(world, this);

        let doodle_jump_pygmy_logic = world.obj::<ControllerMinigames>(this).doodle_jump_pygmy_logic;
        if let Some(doodle_jump_pygmy_logic) = doodle_jump_pygmy_logic {
            hide_doodle_display_group(world, this, doodle_jump_pygmy_logic);
        }
        knock_down_doodle_jump_pygmy(world, this);
    }

    trigger_idol_episode_event(world, this, "StormClouds");
    world.obj_mut::<ControllerSkyWeather>(this).weather_state = WEATHER_STATE_STORM;
    set_option_int_preference(world, this, "WeatherHazard", WEATHER_STATE_STORM);

    let weather_button_logic = world.obj::<ControllerSkyWeather>(this).weather_button_logic;
    set_logic_behavior(world, weather_button_logic, "StartWeatherOn");

    let weather_sequence = world.obj::<ControllerSkyWeather>(this).weather_sequence;
    set_sequence_def_by_id(world, weather_sequence, "StartStorm");
}

/// `-[PocketGodViewController (Game) dragClouds:timeStamp:]` @ 0x10015f8b8
///
/// `pos`: the touch in island coordinates. A swipe steeper than 30° lets the clouds go with
/// ten times the last step. Otherwise the clouds follow the finger and the drag speed (the
/// running average of the finger speed, limited to ±7000 per step and ±5000 overall) is
/// kept for the release.
pub fn drag_clouds(world: &mut World, this: Entity, pos: Vec4, time_stamp: f32) {
    let (last_time_stamp, last_touch_pos) = {
        let sky = world.obj::<ControllerSkyWeather>(this);
        (sky.drag_clouds_time_stamp, sky.drag_clouds_touch_pos)
    };
    let elapsed = time_stamp - last_time_stamp;
    let delta_time = if elapsed == 0.0 { 0.02 } else { elapsed };
    let delta_x = pos.x - last_touch_pos.x;
    let delta_y = pos.y - last_touch_pos.y;

    let angle = delta_y.atan2(delta_x);
    let steep_down = angle < -CLOUD_DRAG_STEEP_MIN_ANGLE && angle > -CLOUD_DRAG_STEEP_MAX_ANGLE;
    let steep_up = angle < CLOUD_DRAG_STEEP_MAX_ANGLE && angle > CLOUD_DRAG_STEEP_MIN_ANGLE;
    if steep_down || steep_up {
        release_clouds_on_swipe(world, this, delta_x);
        return;
    }

    if delta_time != 0.0 {
        let step_vel = delta_x / delta_time;
        let step_vel = if step_vel > 7000.0 { 7000.0 } else { step_vel };
        let step_vel = if step_vel < -7000.0 { -7000.0 } else { step_vel };

        let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
        let mut average_vel = sky.drag_clouds_ave_vel;
        if average_vel == -1.0 {
            sky.drag_clouds_ave_vel = step_vel;
            average_vel = sky.drag_clouds_ave_vel;
        }
        sky.drag_clouds_ave_vel = average_vel + (step_vel - average_vel) * 0.5;
    }

    let weather_state = {
        let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
        let average_vel = sky.drag_clouds_ave_vel;
        sky.cloud_drag_vel = if average_vel > 5000.0 { 5000.0 } else { average_vel };
        if sky.cloud_drag_vel < -5000.0 {
            sky.cloud_drag_vel = -5000.0;
        }
        sky.weather_state
    };

    if weather_state == WEATHER_STATE_STORM {
        let storm_clouds = storm_clouds(world, this);
        for storm_cloud in storm_clouds.into_iter().flatten() {
            storm_cloud_logic::apply_drag_offset(world, storm_cloud, delta_x);
        }
    } else {
        let clouds = world.obj::<ControllerSkyWeather>(this).cloud_logic_array.clone();
        for cloud in clouds {
            let cloud_display_object = logic::display_object(world, cloud);
            let mut cloud_display = display::object_mut(world, cloud_display_object);
            cloud_display.pos.x = delta_x + cloud_display.pos.x;
        }
    }

    let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
    sky.drag_clouds_touch_pos = pos;
    sky.drag_clouds_time_stamp = time_stamp;
}

/// `dragClouds:timeStamp:`: a steep swipe ends the drag; the clouds go on at ten times the
/// last horizontal step and the touch is released.
fn release_clouds_on_swipe(world: &mut World, this: Entity, delta_x: f32) {
    let weather_state = {
        let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
        sky.drag_clouds = false;
        sky.weather_state
    };

    let release_vel = delta_x * 10.0;
    if weather_state == WEATHER_STATE_STORM {
        let storm_clouds = storm_clouds(world, this);
        for storm_cloud in storm_clouds.into_iter().flatten() {
            storm_cloud_logic::stop_dragging(world, storm_cloud, release_vel);
        }
    } else {
        let clouds = world.obj::<ControllerSkyWeather>(this).cloud_logic_array.clone();
        for cloud in clouds {
            cloud_logic::stop_dragging(world, cloud, release_vel);
        }
    }

    world.obj_mut::<ControllerSkyWeather>(this).drag_clouds_touch = None;
}

/// `-[PocketGodViewController (Game) startDragSun:]` @ 0x10016035c
///
/// `pos`: the touch in island coordinates.
pub fn start_drag_sun(world: &mut World, this: Entity, pos: Vec4) {
    let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
    sky.drag_sun_touch_pos = pos;
    sky.drag_sun_x_origin = pos.x;
}

/// `-[PocketGodViewController (Game) startNight]` @ 0x100160388
///
/// The moon rises, the sun sets below the horizon and `StartNight` runs.
pub fn start_night(world: &mut World, this: Entity) {
    {
        let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
        sky.night_active = true;
        sky.sunset_active = false;
        sky.sunset_gradient_target_fade = 0.0;
        sky.night_gradient_target_fade = 1.0;
    }
    trigger_idol_episode_event(world, this, "Night");

    let moon = world.obj::<ControllerSkyWeather>(this).moon_logic;
    if let Some(moon) = moon {
        moon_logic::transition_in(world, moon, SKY_TOP_Y_POS);
    }

    let sun = world.obj::<ControllerSkyWeather>(this).sun_logic;
    let sun_under_horizon_y_pos = world.obj::<ControllerIslandLocations>(this).sun_under_horizon_y_pos;
    if let Some(sun) = sun {
        sun_logic::transition_out(world, sun, sun_under_horizon_y_pos);
    }

    let main_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
    set_sequence_def_by_id(world, main_sequence, "StartNight");
}

/// `-[PocketGodViewController (Game) startDay]` @ 0x1001604c0
///
/// The bat flies off, the sun's reflection goes, the sun rises, the moon sets and `StartDay`
/// runs.
pub fn start_day(world: &mut World, this: Entity) {
    let bat = world.obj::<ControllerSkyWeather>(this).bat_logic;
    set_logic_behavior(world, bat, "BatFlyOffScreen");

    let sun_reflection = {
        let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
        sky.night_active = false;
        sky.sunset_active = false;
        sky.sunset_gradient_target_fade = 0.0;
        sky.night_gradient_target_fade = 0.0;
        sky.sun_reflection_visible = false;
        sky.sun_reflection_display_object
    };
    if let Some(sun_reflection) = sun_reflection {
        display::remove_from_display_group(world, sun_reflection);
    }

    let sun = world.obj::<ControllerSkyWeather>(this).sun_logic;
    if let Some(sun) = sun {
        sun_logic::transition_in(world, sun, SKY_TOP_Y_POS);
    }

    let moon = world.obj::<ControllerSkyWeather>(this).moon_logic;
    let moon_under_horizon_y_pos = world.obj::<ControllerIslandLocations>(this).moon_under_horizon_y_pos;
    if let Some(moon) = moon {
        moon_logic::transition_out(world, moon, moon_under_horizon_y_pos);
    }

    let main_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
    set_sequence_def_by_id(world, main_sequence, "StartDay");
}

/// `-[PocketGodViewController (Game) startSunset]` @ 0x100160624
///
/// The bat flies off, the sunset gradient fades in from nothing, the sun's reflection shows
/// at full alpha, the sun moves to its sunset height, the moon sets and `StartSunset` runs.
pub fn start_sunset(world: &mut World, this: Entity) {
    let bat = world.obj::<ControllerSkyWeather>(this).bat_logic;
    set_logic_behavior(world, bat, "BatFlyOffScreen");

    {
        let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
        sky.night_active = false;
        sky.sunset_active = true;
        sky.sunset_gradient_fade = 0.0;
        sky.sunset_gradient_target_fade = 1.0;
        sky.night_gradient_target_fade = 0.0;
    }
    trigger_idol_episode_event(world, this, "Sunset");

    insert_sun_reflection(world, this);
    let sun_reflection = world.obj::<ControllerSkyWeather>(this).sun_reflection_display_object;
    if let Some(sun_reflection) = sun_reflection {
        display::object_mut(world, sun_reflection).set_alpha(1.0);
    }

    let sun = world.obj::<ControllerSkyWeather>(this).sun_logic;
    let sun_transition_y_pos = world.obj::<ControllerIslandLocations>(this).sun_transition_y_pos;
    if let Some(sun) = sun {
        sun_logic::transition_in(world, sun, sun_transition_y_pos);
    }

    let moon = world.obj::<ControllerSkyWeather>(this).moon_logic;
    let moon_under_horizon_y_pos = world.obj::<ControllerIslandLocations>(this).moon_under_horizon_y_pos;
    if let Some(moon) = moon {
        moon_logic::transition_out(world, moon, moon_under_horizon_y_pos);
    }

    let main_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
    set_sequence_def_by_id(world, main_sequence, "StartSunset");
}

/// `-[PocketGodViewController (Game) stopDragSun]` @ 0x100160848
///
/// A sun let go below the trigger height brings the night.
pub fn stop_drag_sun(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerSkyWeather>(this).drag_sun_touch = None;

    // Without the `Sun` object the original crashes reading `[nil pos]`.
    let Some(sun_display_object) = island_display_object(world, this, "Sun") else {
        return;
    };
    let sun_y_pos = display::object(world, sun_display_object).pos.y;
    let trigger_y_pos = world.obj::<ControllerIslandLocations>(this).sun_under_horizon_trigger_y_pos;
    if sun_y_pos < trigger_y_pos {
        start_night(world, this);
    }
}

/// `-[PocketGodViewController (Game) dragSun:]` @ 0x1001608e4
///
/// `pos`: the touch in island coordinates. The sun follows the finger vertically, between
/// `mSunUnderHorizonYPos` and 245. Within 40 units above the horizon the sky blends toward
/// sunset (a sunset from 20 % on); at or below the horizon it is full sunset with the night
/// growing. The sun's reflection fades with the sunset weight. A change between day and
/// sunset runs `StartSunset` or `StartDay`.
pub fn drag_sun(world: &mut World, this: Entity, pos: Vec4) {
    let sun_display_object = island_display_object(world, this, "Sun");
    let drag_sun_touch_pos = world.obj::<ControllerSkyWeather>(this).drag_sun_touch_pos;
    let delta_y = pos.y - drag_sun_touch_pos.y;

    // Without the `Sun` object the original crashes reading `[nil pos]`.
    let Some(sun_display_object) = sun_display_object else {
        return;
    };

    let dragged_y_pos = delta_y + display::object(world, sun_display_object).pos.y;
    let mut sun_y_pos = if dragged_y_pos > SKY_TOP_Y_POS { SKY_TOP_Y_POS } else { dragged_y_pos };
    display::object_mut(world, sun_display_object).pos.y = sun_y_pos;

    let (sun_under_horizon_y_pos, horizon_y_pos) = {
        let locations = world.obj::<ControllerIslandLocations>(this);
        (locations.sun_under_horizon_y_pos, locations.horizon_y_pos)
    };
    if sun_y_pos < sun_under_horizon_y_pos {
        display::object_mut(world, sun_display_object).pos.y = sun_under_horizon_y_pos;
        sun_y_pos = sun_under_horizon_y_pos;
    }

    let horizon_distance = (horizon_y_pos - sun_y_pos).abs() / SUNSET_BLEND_DISTANCE;
    let horizon_closeness = 1.0 - horizon_distance;
    let sunset_weight = if horizon_closeness < 0.0 { 0.0 } else { horizon_closeness };

    let sunset_active;
    if sun_y_pos > horizon_y_pos {
        sunset_active = sunset_weight > 0.2;
        world.obj_mut::<ControllerSkyWeather>(this).sunset_gradient_target_fade = sunset_weight;
    } else if sun_y_pos <= horizon_y_pos {
        world.obj_mut::<ControllerSkyWeather>(this).sunset_gradient_target_fade = 1.0;
        sunset_active = true;
    } else {
        // Only a NaN position compares neither way.
        sunset_active = false;
    }

    world.obj_mut::<ControllerSkyWeather>(this).night_gradient_target_fade = 0.0;
    if sun_y_pos < horizon_y_pos {
        world.obj_mut::<ControllerSkyWeather>(this).night_gradient_target_fade = horizon_distance;
    }

    if sunset_weight == 0.0 {
        let sun_reflection = {
            let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
            sky.sun_reflection_visible = false;
            sky.sun_reflection_display_object
        };
        if let Some(sun_reflection) = sun_reflection {
            display::remove_from_display_group(world, sun_reflection);
        }
    } else {
        insert_sun_reflection(world, this);
        let sun_reflection = world.obj::<ControllerSkyWeather>(this).sun_reflection_display_object;
        if let Some(sun_reflection) = sun_reflection {
            display::object_mut(world, sun_reflection).set_alpha(sunset_weight);
        }
    }

    let sunset_was_active = {
        let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
        sky.drag_sun_touch_pos = pos;
        sky.sunset_active
    };
    if sunset_active == sunset_was_active {
        return;
    }

    world.obj_mut::<ControllerSkyWeather>(this).sunset_active = sunset_active;
    let script = if sunset_active { "StartSunset" } else { "StartDay" };
    let main_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
    set_sequence_def_by_id(world, main_sequence, script);
}

/// `-[PocketGodViewController (Game) startDragMoon:]` @ 0x100160bc4
///
/// `pos`: the touch in island coordinates.
pub fn start_drag_moon(world: &mut World, this: Entity, pos: Vec4) {
    world.obj_mut::<ControllerSkyWeather>(this).drag_moon_touch_pos = pos;
}

/// `-[PocketGodViewController (Game) stopDragMoon]` @ 0x100160bdc
///
/// A moon let go below the trigger height brings the sunset.
pub fn stop_drag_moon(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerSkyWeather>(this).drag_moon_touch = None;

    // Without the `Moon` object the original crashes reading `[nil pos]`.
    let Some(moon_display_object) = island_display_object(world, this, "Moon") else {
        return;
    };
    let moon_y_pos = display::object(world, moon_display_object).pos.y;
    let trigger_y_pos = world.obj::<ControllerIslandLocations>(this).moon_under_horizon_trigger_y_pos;
    if moon_y_pos < trigger_y_pos {
        start_sunset(world, this);
    }
}

/// `-[PocketGodViewController (Game) dragMoon:]` @ 0x100160c78
///
/// `pos`: the touch in island coordinates. The moon follows the finger vertically, between
/// `mMoonUnderHorizonYPos` and 245.
pub fn drag_moon(world: &mut World, this: Entity, pos: Vec4) {
    let moon_display_object = island_display_object(world, this, "Moon");
    let drag_moon_touch_pos = world.obj::<ControllerSkyWeather>(this).drag_moon_touch_pos;
    let delta_y = pos.y - drag_moon_touch_pos.y;

    // Without the `Moon` object the original crashes reading `[nil pos]`.
    let Some(moon_display_object) = moon_display_object else {
        return;
    };

    let dragged_y_pos = delta_y + display::object(world, moon_display_object).pos.y;
    let moon_y_pos = if dragged_y_pos > SKY_TOP_Y_POS { SKY_TOP_Y_POS } else { dragged_y_pos };
    display::object_mut(world, moon_display_object).pos.y = moon_y_pos;

    let moon_under_horizon_y_pos = world.obj::<ControllerIslandLocations>(this).moon_under_horizon_y_pos;
    if moon_y_pos < moon_under_horizon_y_pos {
        display::object_mut(world, moon_display_object).pos.y = moon_under_horizon_y_pos;
    }

    world.obj_mut::<ControllerSkyWeather>(this).drag_moon_touch_pos = pos;
}

/// `-[PocketGodViewController (Game) checkForBatTapping:]` @ 0x100163740
///
/// A tap on the moon (the touch itself is not used). Unless a pygmy is a vampire or the bat is
/// out, a tap counts when the tap timer has run since the last one (`onGameFrame:` advances it
/// and drops the count after 1 s); the third tap summons the bat just above the moon.
pub fn check_for_bat_tapping(world: &mut World, this: Entity, _touch: Entity) {
    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        if crate::pygmy::is_vampire(world, pygmy) {
            return;
        }
    }

    let (bat, bat_tap_touch_time) = {
        let sky = world.obj::<ControllerSkyWeather>(this);
        (sky.bat_logic, sky.bat_tap_touch_time)
    };
    if bat.is_some() || bat_tap_touch_time == 0.0 {
        return;
    }

    let bat_tap_touch_count = {
        let mut sky = world.obj_mut::<ControllerSkyWeather>(this);
        sky.bat_tap_touch_time = 0.0;
        sky.bat_tap_touch_count = sky.bat_tap_touch_count.wrapping_add(1);
        sky.bat_tap_touch_count
    };
    if bat_tap_touch_count != 3 {
        return;
    }

    // Without the moon the original crashes reading `[nil pos]`.
    let moon = world.obj::<ControllerSkyWeather>(this).moon_logic;
    let Some(moon) = moon else {
        return;
    };
    let moon_pos = logic_pos(world, moon);
    let (frame_width, frame_height) = texture_frame_size(world, "BatForward_01");

    let bat_model = display::multi_model::spawn(world, Some("Bat"));
    let drag_display_marker = library::display_marker(world, "DragDisplayMarker");
    if let Some(drag_display_marker) = drag_display_marker {
        display::insert_after(world, drag_display_marker, Some(bat_model));
    }

    {
        let mut bat_display = display::object_mut(world, bat_model);
        bat_display.pos.x = moon_pos.x;
        bat_display.pos.y = moon_pos.y + 40.0;
        bat_display.pos.z = SPRITE_Z;
        bat_display.origin_pos.x = frame_width * -0.5;
        bat_display.origin_pos.y = frame_height * -0.5;
        bat_display.col_x_max = frame_width;
        bat_display.col_y_max = frame_height + 40.0;
        bat_display.is_selectable = true;
    }

    let bat = bat_logic::spawn(world, bat_model);
    world.obj_mut::<ControllerSkyWeather>(this).bat_logic = Some(bat);
    bat_logic::set_bat_logic_listener(world, bat, this);
}

/// `-[PocketGodViewController (Game) sunState]` @ 0x100163e88
///
/// The sun state, normal at night.
pub fn sun_state(world: &mut World, this: Entity) -> i32 {
    let sky = world.obj::<ControllerSkyWeather>(this);
    if sky.night_active {
        return SUN_STATE_NORMAL;
    }
    sky.sun_state
}

/// `-[PocketGodViewController (Game) isEclipseActive]` @ 0x100163f6c
pub fn is_eclipse_active(world: &mut World, this: Entity) -> bool {
    let eclipse_gradient_target_fade = world.obj::<ControllerSkyWeather>(this).eclipse_gradient_target_fade;
    0.5 < eclipse_gradient_target_fade
}

/// `-[PocketGodViewController (Game) isDayActive]` @ 0x100163f8c
pub fn is_day_active(world: &mut World, this: Entity) -> bool {
    let sky = world.obj::<ControllerSkyWeather>(this);
    !sky.night_active && !sky.sunset_active
}

/// `-[PocketGodViewController (Game) isStormActive]` @ 0x100163fec
pub fn is_storm_active(world: &mut World, this: Entity) -> bool {
    world.obj::<ControllerSkyWeather>(this).weather_state == WEATHER_STATE_STORM
}

/// `-[PocketGodViewController (Game) isNightActive]` @ 0x100164008
pub fn is_night_active(world: &mut World, this: Entity) -> bool {
    world.obj::<ControllerSkyWeather>(this).night_active
}

/// `-[PocketGodViewController (Game) isSunsetActive]` @ 0x10016401c
pub fn is_sunset_active(world: &mut World, this: Entity) -> bool {
    world.obj::<ControllerSkyWeather>(this).sunset_active
}

/// `-[PocketGodViewController (Game) onBatDie:]` @ 0x100169b24
///
/// The bat leaves the display tree and shuts down; `mBatLogic` is released and cleared (the
/// current bat, whichever bat died).
pub fn on_bat_die(world: &mut World, this: Entity, bat: Entity) {
    let bat_display_object = logic::display_object(world, bat);
    display::remove_from_display_group(world, bat_display_object);
    logic::shutdown(world, bat);

    let released_bat = world.obj_mut::<ControllerSkyWeather>(this).bat_logic.take();
    // The release frees the bat only when it is the one that just shut down (a live bat is
    // still a frame listener) and the hurricane's picked-up array does not still hold it.
    if released_bat == Some(bat) && !is_held_by_hurricane(world, this, bat) {
        dealloc_sprite(world, bat);
    }
}

/// The 11 sky gradient defs (`mp…Gradient`), by id.
struct SkyGradientDefs {
    sunset: String,
    night: String,
    day: String,
    cloudy: String,
    eclipse: String,
    big_sun: String,
    small_sun: String,
    apocalypse_sunset: String,
    apocalypse_night: String,
    apocalypse_day: String,
    apocalypse_cloudy: String,
}

impl SkyGradientDefs {
    /// The defs, when none of them is NULL.
    fn from_ivars(sky: &ControllerSkyWeather) -> Option<Self> {
        let gradient_defs = SkyGradientDefs {
            sunset: sky.sunset_gradient.clone()?,
            night: sky.night_gradient.clone()?,
            day: sky.day_gradient.clone()?,
            cloudy: sky.cloudy_gradient.clone()?,
            eclipse: sky.eclipse_gradient.clone()?,
            big_sun: sky.big_sun_gradient.clone()?,
            small_sun: sky.small_sun_gradient.clone()?,
            apocalypse_sunset: sky.apocalypse_sunset_gradient.clone()?,
            apocalypse_night: sky.apocalypse_night_gradient.clone()?,
            apocalypse_day: sky.apocalypse_day_gradient.clone()?,
            apocalypse_cloudy: sky.apocalypse_cloudy_gradient.clone()?,
        };
        Some(gradient_defs)
    }
}

/// `updateGradients`: one easing step of a gradient weight, in single precision.
fn eased_weight(fade: f32, target_fade: f32) -> f32 {
    fade + (target_fade - fade) * GRADIENT_EASE
}

/// `updateGradients`: the weight, or its target once the two are less than 0.01 apart.
fn snapped_weight(fade: f32, target_fade: f32) -> f32 {
    let distance = f64::from((fade - target_fade).abs());
    if distance < GRADIENT_SNAP_DISTANCE {
        return target_fade;
    }
    fade
}

/// `[mIslandDisplayGroup getDisplayObject:id]` (nil without the group).
fn island_display_object(world: &World, this: Entity, id: &str) -> Option<Entity> {
    let island_display_group = world.obj::<ControllerIslandLocations>(this).island_display_group;
    match island_display_group {
        Some(island_display_group) => group::get_display_object(world, island_display_group, id),
        None => None,
    }
}

/// `[[logic displayObject] pos]`.
fn logic_pos(world: &World, logic_object: Entity) -> Vec4 {
    let display_object = logic::display_object(world, logic_object);
    display::object(world, display_object).pos
}

/// `mFrontStormCloudLogic` and `mBehindStormCloudLogic`, in the order the original messages
/// them.
fn storm_clouds(world: &World, this: Entity) -> [Option<Entity>; 2] {
    let sky = world.obj::<ControllerSkyWeather>(this);
    [sky.front_storm_cloud_logic, sky.behind_storm_cloud_logic]
}

/// Puts the sun's reflection back into the sky just before `CloudBehindDisplayMarker`, unless
/// it is there already or this is Ape island.
fn insert_sun_reflection(world: &mut World, this: Entity) {
    let sun_reflection_visible = world.obj::<ControllerSkyWeather>(this).sun_reflection_visible;
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if sun_reflection_visible || island_location_state == LOCATION_APE_ISLAND {
        return;
    }

    world.obj_mut::<ControllerSkyWeather>(this).sun_reflection_visible = true;
    let cloud_behind_display_marker = library::display_marker(world, "CloudBehindDisplayMarker");
    let sun_reflection = world.obj::<ControllerSkyWeather>(this).sun_reflection_display_object;
    if let Some(cloud_behind_display_marker) = cloud_behind_display_marker {
        display::insert_before(world, cloud_behind_display_marker, sun_reflection);
    }
}

/// `[[mDoodleDisplayGroup getDisplayObject:@"SkyTiles"] hide]`.
fn hide_sky_tiles(world: &mut World, this: Entity) {
    let doodle_display_group = world.obj::<ControllerMinigames>(this).doodle_display_group;
    let sky_tiles = match doodle_display_group {
        Some(doodle_display_group) => group::get_display_object(world, doodle_display_group, "SkyTiles"),
        None => None,
    };
    if let Some(sky_tiles) = sky_tiles {
        display::object_mut(world, sky_tiles).hidden = true;
    }
}

/// `[mDoodleJumpPygmyLogic doodleKnockedDown:nil]`.
fn knock_down_doodle_jump_pygmy(world: &mut World, this: Entity) {
    let doodle_jump_pygmy_logic = world.obj::<ControllerMinigames>(this).doodle_jump_pygmy_logic;
    let Some(doodle_jump_pygmy_logic) = doodle_jump_pygmy_logic else {
        return;
    };
    // The original passes nil; the pygmy never reads the cloud argument.
    crate::pygmy::doodle_knocked_down(world, doodle_jump_pygmy_logic, Entity::PLACEHOLDER);
}

/// `[mHurricaneLogic hurricaneActive]` (NO without a hurricane).
fn is_hurricane_active(world: &mut World, this: Entity) -> bool {
    let hurricane = world.obj::<ControllerGodPowers>(this).hurricane_logic;
    match hurricane {
        Some(hurricane) => hurricane_logic::hurricane_active(world, hurricane),
        None => false,
    }
}

/// Whether the hurricane holds `sprite` in its picked-up array, which retains it. (The
/// whirlpool never picks up the bat.)
fn is_held_by_hurricane(world: &World, this: Entity, sprite: Entity) -> bool {
    let hurricane = world.obj::<ControllerGodPowers>(this).hurricane_logic;
    let Some(hurricane) = hurricane else {
        return false;
    };
    match world.get::<HurricaneLogic>(hurricane) {
        Some(hurricane_state) => hurricane_state.picked_up_logic_array.contains(&sprite),
        None => false,
    }
}

/// The last release of a cloud or of the bat, already out of the display tree and shut down:
/// `-[BCLogic dealloc]` (`CloudLogic`'s and `BatLogic`'s deallocs only add the release of
/// their listener) releases the model, whose creator released it right after making it, so
/// the model goes too.
fn dealloc_sprite(world: &mut World, sprite: Entity) {
    let display_object = logic::display_object(world, sprite);
    logic::dealloc(world, sprite);

    let out_of_display_tree = match world.get::<DisplayObject>(display_object) {
        Some(object) => object.display_group.is_none(),
        None => false,
    };
    if out_of_display_tree {
        world.despawn(display_object);
    }
}

/// The `(frameWidth, frameHeight)` of a library texture (`+0x28`, `+0x2c` of its
/// `BCTextureDef`; zero for an unknown id, where the original reads through NULL).
fn texture_frame_size(world: &World, texture_id: &str) -> (f32, f32) {
    let texture = library::texture_def_id(world, texture_id);
    let Some(texture) = texture else {
        return (0.0, 0.0);
    };
    let texture_def = world.resource::<Textures>().def(texture);
    (texture_def.frame_width, texture_def.frame_height)
}

/// `[sequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:id]]` (nothing for a nil
/// sequence).
fn set_sequence_def_by_id(world: &mut World, sequence: Option<Entity>, id: &str) {
    let sequence_def = library::sequence_def(world, id);
    let Some(sequence) = sequence else {
        return;
    };
    sequence::set_sequence_def(world, sequence, sequence_def);
}

/// `[logic setBehavior:behavior]` on a logic that may be nil.
fn set_logic_behavior(world: &mut World, logic_object: Option<Entity>, behavior: &str) {
    if let Some(logic_object) = logic_object {
        logic::set_behavior(world, logic_object, Some(behavior));
    }
}

/// `[mIdolManager triggerEpisodeEvent:event]` (nothing before the manager exists).
fn trigger_idol_episode_event(world: &mut World, this: Entity, event: &str) {
    let idol_manager = world.obj::<ControllerManagersBridge>(this).idol_manager;
    if let Some(idol_manager) = idol_manager {
        idol_manager::trigger_episode_event(world, idol_manager, event);
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
