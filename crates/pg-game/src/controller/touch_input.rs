//! `PocketGodViewController`: methods in this part of the class.

use std::f32::consts::{FRAC_PI_8, PI, TAU};

use pg_engine::display;
use pg_engine::display::group;
use pg_engine::touch::BcTouch;
use pg_engine::view::BcView;

use crate::logic::ape_ritual::ape_wench_logic;
use crate::logic::idol_ui_widgets::one_shot_anim_logic;
use crate::logic::sky_powers::hurricane_logic;
use crate::logic::sky_weather::ocean_logic;
use crate::logic::theruns::crack_hole_logic;
use crate::prelude::*;

use super::*;

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerTouchInput {
    /// `mAllTouches`: `@"NSArray"` — `BCView`'s `mGameTouchArray` itself in the original, so
    /// `cancelAllTouches` reads the live array; this keeps the fingers of the last event.
    pub all_touches: Vec<Entity>,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mTouchLogicArray`: `[1000@]` — nil-terminated; slots past the terminator keep what an
    /// earlier fill left there.
    pub touch_logic_array: Vec<Option<Entity>>,

    /// `mCancelTouchLogicArray`: `[1000@]` — like `touch_logic_array`.
    pub cancel_touch_logic_array: Vec<Option<Entity>>,

    /// `mApeWenchTouch`: `@"BCTouch"`
    pub ape_wench_touch: Option<Entity>,

    /// `mLastApeWenchPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_ape_wench_pos: Vec4,

    /// `mLastBubbleTime`: `f`
    pub last_bubble_time: f32,

    /// `mPinchSunState`: `i`
    pub pinch_sun_state: i32,

    /// `mPinchSunDistance`: `f`
    pub pinch_sun_distance: f32,

    /// `mPinchSunTouch0`: `@"BCTouch"`
    pub pinch_sun_touch0: Option<Entity>,

    /// `mPinchSunTouch1`: `@"BCTouch"`
    pub pinch_sun_touch1: Option<Entity>,

    /// `mbDragCrack`: `B`
    pub drag_crack: bool,

    /// `mDragCrackTouch`: `@"BCTouch"`
    pub drag_crack_touch: Option<Entity>,

    /// `mbDragOcean`: `B`
    pub drag_ocean: bool,

    /// `mOceanDragVel`: `f`
    pub ocean_drag_vel: f32,

    /// `mDragOceanAveVel`: `f`
    pub drag_ocean_ave_vel: f32,

    /// `mDragOceanTimeStamp`: `f`
    pub drag_ocean_time_stamp: f32,

    /// `mDragOceanTouch`: `@"BCTouch"`
    pub drag_ocean_touch: Option<Entity>,

    /// `mDragOceanTouchPos`: `{?="x"f"y"f"z"f"w"f}`
    pub drag_ocean_touch_pos: Vec4,

    /// `mFrontOceanLogic`: `@"OceanLogic"`
    pub front_ocean_logic: Option<Entity>,

    /// `mBehindOceanLogic`: `@"OceanLogic"`
    pub behind_ocean_logic: Option<Entity>,

    /// `mUnderwaterTouchStartYPos`: `f`
    pub underwater_touch_start_y_pos: f32,

    /// `mUnderwaterTouch`: `@"BCTouch"`
    pub underwater_touch: Option<Entity>,

    /// `mUnderwaterTouchPos`: `{?="x"f"y"f"z"f"w"f}`
    pub underwater_touch_pos: Vec4,
}

/// Which of the controller's two C arrays of selectable logics a touch dispatch fills.
/// `cancelAllTouches` has its own so that it can run inside a dispatch over the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchLogicArray {
    /// `mTouchLogicArray`.
    Touch,

    /// `mCancelTouchLogicArray`.
    Cancel,
}

/// `-[PocketGodViewController retainTouchLogicArray:]` @ 0x10006e6b0
///
/// Sends `retain` to every logic up to the nil terminator: nothing to do in the port.
pub fn retain_touch_logic_array(_world: &mut World, _this: Entity, _logic_array: TouchLogicArray) {}

/// `-[PocketGodViewController releaseTouchLogicArray:]` @ 0x10006e6f0
///
/// Sends `release` to every logic up to the nil terminator: nothing to do in the port.
pub fn release_touch_logic_array(_world: &mut World, _this: Entity, _logic_array: TouchLogicArray) {}

/// `-[PocketGodViewController createTouchLogicArray:]` @ 0x10006e730
///
/// Fills the array from slot 0 with the selectable logics of the active display tree
/// (pre-order, so the frontmost come last), nil-terminates it and returns the terminator's
/// index (the original returns its address).
pub fn create_touch_logic_array(world: &mut World, this: Entity, logic_array: TouchLogicArray) -> usize {
    let active_display_group = world.resource::<BcView>().active_display_group;
    let active_display_group = active_display_group
        .expect("createTouchLogicArray: the original writes the terminator through [nil addSelectableLogicsToArray:]");

    let mut logics = Vec::new();
    display::add_selectable_logics(world, active_display_group, &mut logics);
    let end = logics.len();

    {
        let mut input = world.obj_mut::<ControllerTouchInput>(this);
        let slots = match logic_array {
            TouchLogicArray::Touch => &mut input.touch_logic_array,
            TouchLogicArray::Cancel => &mut input.cancel_touch_logic_array,
        };
        if slots.len() <= end {
            slots.resize(end + 1, None);
        }
        for (index, logic) in logics.into_iter().enumerate() {
            slots[index] = Some(logic);
        }
        slots[end] = None;
    }

    retain_touch_logic_array(world, this, logic_array);
    end
}

/// `-[PocketGodViewController isPointLeftOfLineStartXPos:lineStartYPos:lineEndXPos:lineEndYPos:xPos:yPos:]` @ 0x10006ea74
pub fn is_point_left_of_line_start_x_pos(
    _world: &mut World,
    _this: Entity,
    line_start_x_pos: f32,
    line_start_y_pos: f32,
    line_end_x_pos: f32,
    line_end_y_pos: f32,
    x_pos: f32,
    y_pos: f32,
) -> bool {
    let end_term = (line_end_x_pos - line_start_x_pos) * (y_pos - line_start_y_pos);
    let start_term = (line_end_y_pos - line_start_y_pos) * (x_pos - line_start_x_pos);
    end_term - start_term > 0.0
}

/// `-[PocketGodViewController preStartTouchBeganXPos:yPos:]` @ 0x10006ea9c
pub fn pre_start_touch_began_x_pos(_world: &mut World, _this: Entity, _x_pos: f32, _y_pos: f32) {}

/// `-[PocketGodViewController touchBegan:allTouches:]` @ 0x10006eaa0
pub fn touch_began(world: &mut World, this: Entity, touch: Entity, all_touches: &[Entity]) {
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut input = world.obj_mut::<ControllerTouchInput>(this);
        input.all_touches = all_touches.to_vec();
        input.touch_screen_pos = touch_pos;
    }

    let doodle_jump_pygmy_logic = world.obj::<ControllerMinigames>(this).doodle_jump_pygmy_logic;
    if doodle_jump_pygmy_logic.is_some() {
        world.obj_mut::<ControllerMinigames>(this).doodle_debug_press = true;
    }

    let pinch_started = start_pinch_sun(world, this, touch, all_touches);
    if pinch_started {
        return;
    }

    let taken_by_logic = dispatch_touch_began(world, this, touch);
    if taken_by_logic {
        return;
    }

    let screen_or_minigame_active = {
        let menus = world.obj::<ControllerMenusAndScreens>(this);
        let managers = world.obj::<ControllerManagersBridge>(this);
        let minigames = world.obj::<ControllerMinigames>(this);
        menus.help_screen_open
            || menus.debug_screen_open
            || managers.challenge_screen_open
            || menus.stats_screen_open
            || managers.skin_pack_screen_open
            || managers.bonus_pack_screen_open
            || managers.story_screen_open
            || managers.news_screen_open
            || managers.idol_screen_open
            || managers.map_screen_open
            || minigames.pain_drain
            || minigames.the_runs
            || minigames.konkey_dong
            || minigames.moron_pests
            || minigames.no_pee
            || minigames.dance_dance_execution
            || minigames.battle_of_the_gods
    };
    if screen_or_minigame_active {
        return;
    }

    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    match island_location_state {
        2 | 9 => begin_underwater_gesture(world, this, touch),
        11..=17 => begin_temple_gesture(world, this, touch, island_location_state),
        _ => begin_island_gesture(world, this, touch),
    }
}

/// `-[PocketGodViewController touchMoved:allTouches:]` @ 0x10006f7fc
pub fn touch_moved(world: &mut World, this: Entity, touch: Entity, all_touches: &[Entity]) {
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut input = world.obj_mut::<ControllerTouchInput>(this);
        input.all_touches = all_touches.to_vec();
        input.touch_screen_pos = touch_pos;
    }

    let (pinch_sun_touch0, pinch_sun_touch1) = {
        let input = world.obj::<ControllerTouchInput>(this);
        (input.pinch_sun_touch0, input.pinch_sun_touch1)
    };
    if pinch_sun_touch0 == Some(touch) || pinch_sun_touch1 == Some(touch) {
        move_pinch_sun(world, this);
        return;
    }

    let drag_sun_touch = world.obj::<ControllerSkyWeather>(this).drag_sun_touch;
    if drag_sun_touch == Some(touch) {
        let island_pos = island_touch_pos(world, this);
        sky_weather::drag_sun(world, this, island_pos);
        return;
    }

    let drag_moon_touch = world.obj::<ControllerSkyWeather>(this).drag_moon_touch;
    if drag_moon_touch == Some(touch) {
        let island_pos = island_touch_pos(world, this);
        sky_weather::drag_moon(world, this, island_pos);
        return;
    }

    let (dragging_clouds, drag_clouds_touch) = {
        let sky = world.obj::<ControllerSkyWeather>(this);
        (sky.drag_clouds, sky.drag_clouds_touch)
    };
    if dragging_clouds && drag_clouds_touch == Some(touch) {
        let island_pos = island_touch_pos(world, this);
        let time_stamp = world.obj::<BcTouch>(touch).time_stamp;
        sky_weather::drag_clouds(world, this, island_pos, time_stamp);
    }

    let (dragging_crack, drag_crack_touch) = {
        let input = world.obj::<ControllerTouchInput>(this);
        (input.drag_crack, input.drag_crack_touch)
    };
    if dragging_crack && drag_crack_touch == Some(touch) {
        let island_pos = island_touch_pos(world, this);
        let time_stamp = world.obj::<BcTouch>(touch).time_stamp;
        drag_crack(world, this, island_pos, time_stamp);
        return;
    }

    let (dragging_ocean, drag_ocean_touch) = {
        let input = world.obj::<ControllerTouchInput>(this);
        (input.drag_ocean, input.drag_ocean_touch)
    };
    if dragging_ocean && drag_ocean_touch == Some(touch) {
        let island_pos = island_touch_pos(world, this);
        let time_stamp = world.obj::<BcTouch>(touch).time_stamp;
        drag_ocean(world, this, island_pos, time_stamp);
    }

    let ape_wench_touch = world.obj::<ControllerTouchInput>(this).ape_wench_touch;
    if ape_wench_touch == Some(touch) {
        let island_pos = island_touch_pos(world, this);
        drag_ape_wench(world, this, island_pos);
    }

    let hurricane_touch = world.obj::<ControllerGodPowers>(this).hurricane_touch;
    if hurricane_touch == Some(touch) {
        let island_pos = island_touch_pos(world, this);
        god_powers::maintain_hurricane(world, this, island_pos);
        return;
    }

    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state == 2 || island_location_state == 9 {
        let underwater_pos = underwater_touch_pos(world, this);
        check_underwater_bubble(world, this, touch, underwater_pos);
    }

    let underwater_touch = world.obj::<ControllerTouchInput>(this).underwater_touch;
    if underwater_touch == Some(touch) {
        let underwater_pos = underwater_touch_pos(world, this);
        drag_underwater(world, this, underwater_pos);
    }

    let underwater_power_touch = world.obj::<ControllerGodPowers>(this).underwater_power_touch;
    if underwater_power_touch == Some(touch) {
        let underwater_pos = underwater_touch_pos(world, this);
        god_powers::drag_underwater_power(world, this, underwater_pos);
        return;
    }

    let whirl_pool_touch = world.obj::<ControllerGodPowers>(this).whirl_pool_touch;
    if whirl_pool_touch == Some(touch) {
        let underwater_pos = underwater_touch_pos(world, this);
        god_powers::maintain_whirl_pool(world, this, underwater_pos);
        return;
    }

    let (drawing_lightning, lightning_touch) = {
        let powers = world.obj::<ControllerGodPowers>(this);
        (powers.draw_lightning, powers.lightning_touch)
    };
    if drawing_lightning && lightning_touch == Some(touch) {
        let shock_object = world.obj::<ControllerGodPowers>(this).shock_object;
        if shock_object {
            return;
        }
        let island_pos = island_touch_pos(world, this);
        god_powers::draw_lightning(world, this, island_pos);
        return;
    }

    let rainbow_touch = world.obj::<ControllerGodPowers>(this).rainbow_touch;
    if rainbow_touch == Some(touch) {
        let island_pos = island_touch_pos(world, this);
        god_powers::draw_rainbow(world, this, island_pos);
        let drawing_rainbow = world.obj::<ControllerGodPowers>(this).draw_rainbow;
        if drawing_rainbow {
            return;
        }
    }

    let locust_touch = world.obj::<ControllerGodPowers>(this).locust_touch;
    if locust_touch == Some(touch) {
        let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
        let locust_display_group = temple_or_island_display_group(world, this, island_location_state);
        let locust_pos = touch_screen_pos_in(world, this, locust_display_group);
        god_powers::draw_locust(world, this, locust_pos);
        let drawing_locust = world.obj::<ControllerGodPowers>(this).draw_locust;
        if drawing_locust {
            return;
        }
    }

    dispatch_touch_moved(world, this, touch);
}

/// `-[PocketGodViewController touchEnded:allTouches:]` @ 0x1000701a0
pub fn touch_ended(world: &mut World, this: Entity, touch: Entity, all_touches: &[Entity]) {
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut input = world.obj_mut::<ControllerTouchInput>(this);
        input.all_touches = all_touches.to_vec();
        input.touch_screen_pos = touch_pos;
    }
    world.obj_mut::<ControllerMinigames>(this).doodle_debug_press = false;

    let (pinch_sun_touch0, pinch_sun_touch1) = {
        let input = world.obj::<ControllerTouchInput>(this);
        (input.pinch_sun_touch0, input.pinch_sun_touch1)
    };
    if pinch_sun_touch0 == Some(touch) {
        world.obj_mut::<ControllerTouchInput>(this).pinch_sun_touch0 = None;
        return;
    }
    if pinch_sun_touch1 == Some(touch) {
        world.obj_mut::<ControllerTouchInput>(this).pinch_sun_touch1 = None;
        return;
    }

    let drag_sun_touch = world.obj::<ControllerSkyWeather>(this).drag_sun_touch;
    if drag_sun_touch == Some(touch) {
        sky_weather::stop_drag_sun(world, this);
        return;
    }

    let drag_moon_touch = world.obj::<ControllerSkyWeather>(this).drag_moon_touch;
    if drag_moon_touch == Some(touch) {
        sky_weather::stop_drag_moon(world, this);
        return;
    }

    let (dragging_clouds, drag_clouds_touch) = {
        let sky = world.obj::<ControllerSkyWeather>(this);
        (sky.drag_clouds, sky.drag_clouds_touch)
    };
    if dragging_clouds && drag_clouds_touch == Some(touch) {
        sky_weather::stop_drag_clouds(world, this);
    }

    let (dragging_crack, drag_crack_touch) = {
        let input = world.obj::<ControllerTouchInput>(this);
        (input.drag_crack, input.drag_crack_touch)
    };
    if dragging_crack && drag_crack_touch == Some(touch) {
        stop_drag_crack(world, this);
        return;
    }

    let (dragging_ocean, drag_ocean_touch) = {
        let input = world.obj::<ControllerTouchInput>(this);
        (input.drag_ocean, input.drag_ocean_touch)
    };
    if dragging_ocean && drag_ocean_touch == Some(touch) {
        stop_drag_ocean(world, this);
    }

    let ape_wench_touch = world.obj::<ControllerTouchInput>(this).ape_wench_touch;
    if ape_wench_touch == Some(touch) {
        let island_pos = island_touch_pos(world, this);
        stop_ape_wench(world, this, island_pos);
    }

    let rainbow_touch = world.obj::<ControllerGodPowers>(this).rainbow_touch;
    if rainbow_touch == Some(touch) {
        world.obj_mut::<ControllerGodPowers>(this).rainbow_touch = None;
        let drawing_rainbow = world.obj::<ControllerGodPowers>(this).draw_rainbow;
        if drawing_rainbow {
            let island_pos = island_touch_pos(world, this);
            god_powers::stop_draw_rainbow(world, this, island_pos);
            return;
        }
    }

    let locust_touch = world.obj::<ControllerGodPowers>(this).locust_touch;
    if locust_touch == Some(touch) {
        world.obj_mut::<ControllerGodPowers>(this).locust_touch = None;
        let drawing_locust = world.obj::<ControllerGodPowers>(this).draw_locust;
        if drawing_locust {
            let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
            let locust_display_group = temple_or_island_display_group(world, this, island_location_state);
            let locust_pos = touch_screen_pos_in(world, this, locust_display_group);
            god_powers::stop_draw_locust(world, this, locust_pos);
            return;
        }
    }

    let hurricane_touch = world.obj::<ControllerGodPowers>(this).hurricane_touch;
    if hurricane_touch == Some(touch) {
        let island_pos = island_touch_pos(world, this);
        god_powers::stop_hurricane(world, this, island_pos);
        return;
    }

    let whirl_pool_touch = world.obj::<ControllerGodPowers>(this).whirl_pool_touch;
    if whirl_pool_touch == Some(touch) {
        let underwater_pos = underwater_touch_pos(world, this);
        god_powers::stop_whirl_pool(world, this, underwater_pos);
    }

    let underwater_touch = world.obj::<ControllerTouchInput>(this).underwater_touch;
    if underwater_touch == Some(touch) {
        let underwater_pos = underwater_touch_pos(world, this);
        stop_drag_underwater(world, this, underwater_pos);
    }

    let underwater_power_touch = world.obj::<ControllerGodPowers>(this).underwater_power_touch;
    if underwater_power_touch == Some(touch) {
        let underwater_pos = underwater_touch_pos(world, this);
        god_powers::stop_drag_underwater_power(world, this, underwater_pos);
        return;
    }

    let (drawing_lightning, lightning_touch) = {
        let powers = world.obj::<ControllerGodPowers>(this);
        (powers.draw_lightning, powers.lightning_touch)
    };
    if drawing_lightning && lightning_touch == Some(touch) {
        let island_pos = island_touch_pos(world, this);
        god_powers::stop_draw_lightning(world, this, island_pos);
        return;
    }

    dispatch_touch_ended(world, this, touch, TouchLogicArray::Touch);
}

/// `-[PocketGodViewController cancelAllTouches]` @ 0x1000707c8
///
/// Ends every finger for the logics (inside/outside) without clearing any gesture.
pub fn cancel_all_touches(world: &mut World, this: Entity) {
    // `mAllTouches` is `BCView`'s live `mGameTouchArray`: the fingers down right now.
    let all_touches = pg_engine::touch::all_touches(world);
    for touch in all_touches {
        dispatch_touch_ended(world, this, touch, TouchLogicArray::Cancel);
    }
}

/// `-[PocketGodViewController touchCancelled:allTouches:]` @ 0x100070a00
pub fn touch_cancelled(world: &mut World, this: Entity, touch: Entity, all_touches: &[Entity]) {
    touch_ended(world, this, touch, all_touches);
}

/// `-[PocketGodViewController (Game) startDragOcean:timeStamp:]` @ 0x10015fd24
pub fn start_drag_ocean(world: &mut World, this: Entity, pos: Vec4, time_stamp: f32) {
    {
        let mut input = world.obj_mut::<ControllerTouchInput>(this);
        input.drag_ocean = true;
        input.drag_ocean_ave_vel = -1.0;
        input.drag_ocean_time_stamp = time_stamp;
        input.ocean_drag_vel = 0.0;
        input.drag_ocean_touch_pos = pos;
    }

    let front_ocean_logic = world.obj::<ControllerTouchInput>(this).front_ocean_logic;
    if let Some(front_ocean_logic) = front_ocean_logic {
        ocean_logic::start_dragging(world, front_ocean_logic);
    }

    let behind_ocean_logic = world.obj::<ControllerTouchInput>(this).behind_ocean_logic;
    if let Some(behind_ocean_logic) = behind_ocean_logic {
        ocean_logic::start_dragging(world, behind_ocean_logic);
    }
}

/// `-[PocketGodViewController (Game) stopDragOcean]` @ 0x10015fdd4
pub fn stop_drag_ocean(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerTouchInput>(this).drag_ocean = false;

    let (front_ocean_logic, ocean_drag_vel) = {
        let input = world.obj::<ControllerTouchInput>(this);
        (input.front_ocean_logic, input.ocean_drag_vel)
    };
    if let Some(front_ocean_logic) = front_ocean_logic {
        ocean_logic::stop_dragging(world, front_ocean_logic, ocean_drag_vel);
    }

    let (behind_ocean_logic, ocean_drag_vel) = {
        let input = world.obj::<ControllerTouchInput>(this);
        (input.behind_ocean_logic, input.ocean_drag_vel)
    };
    if let Some(behind_ocean_logic) = behind_ocean_logic {
        ocean_logic::stop_dragging(world, behind_ocean_logic, ocean_drag_vel);
    }

    world.obj_mut::<ControllerTouchInput>(this).drag_ocean_touch = None;

    let ocean_drag_vel = world.obj::<ControllerTouchInput>(this).ocean_drag_vel;
    let (island_location_state, island_location_transition) = {
        let locations = world.obj::<ControllerIslandLocations>(this);
        (locations.island_location_state, locations.island_location_transition)
    };
    let flicked = ocean_drag_vel.abs() >= 3000.0;
    let underwater = island_location_state == 2 || island_location_state == 9;
    if flicked && !underwater && !island_location_transition {
        let underwater_location = if ocean_drag_vel > 0.0 { 2 } else { 9 };
        island_locations::set_island_location_state(world, this, underwater_location, false);
    }
}

/// `-[PocketGodViewController (Game) dragOcean:timeStamp:]` @ 0x10015fee8
pub fn drag_ocean(world: &mut World, this: Entity, pos: Vec4, time_stamp: f32) {
    let (drag_ocean_time_stamp, drag_ocean_touch_pos) = {
        let input = world.obj::<ControllerTouchInput>(this);
        (input.drag_ocean_time_stamp, input.drag_ocean_touch_pos)
    };

    let elapsed = time_stamp - drag_ocean_time_stamp;
    let delta_time = if elapsed == 0.0 { 0.02 } else { elapsed };
    let delta_x = pos.x - drag_ocean_touch_pos.x;
    let delta_y = pos.y - drag_ocean_touch_pos.y;
    let angle = delta_y.atan2(delta_x);

    // A drag steeper than 30° is not an ocean drag: let go with the horizontal part.
    let steep_downwards = angle < -0.5235988 && angle > -2.6179938;
    let steep_upwards = angle < 2.6179938 && angle > 0.5235988;
    if steep_downwards || steep_upwards {
        world.obj_mut::<ControllerTouchInput>(this).drag_ocean = false;
        let release_vel = delta_x * 10.0;

        let front_ocean_logic = world.obj::<ControllerTouchInput>(this).front_ocean_logic;
        if let Some(front_ocean_logic) = front_ocean_logic {
            ocean_logic::stop_dragging(world, front_ocean_logic, release_vel);
        }

        let behind_ocean_logic = world.obj::<ControllerTouchInput>(this).behind_ocean_logic;
        if let Some(behind_ocean_logic) = behind_ocean_logic {
            ocean_logic::stop_dragging(world, behind_ocean_logic, release_vel);
        }

        world.obj_mut::<ControllerTouchInput>(this).drag_ocean_touch = None;
        return;
    }

    if delta_time != 0.0 {
        let mut drag_vel = delta_x / delta_time;
        if drag_vel > 7000.0 {
            drag_vel = 7000.0;
        }
        if drag_vel < -7000.0 {
            drag_vel = -7000.0;
        }

        let mut input = world.obj_mut::<ControllerTouchInput>(this);
        if input.drag_ocean_ave_vel == -1.0 {
            input.drag_ocean_ave_vel = drag_vel;
        }
        let ave_vel = input.drag_ocean_ave_vel;
        input.drag_ocean_ave_vel = ave_vel + (drag_vel - ave_vel) * 0.5;
    }

    {
        let mut input = world.obj_mut::<ControllerTouchInput>(this);
        let ave_vel = input.drag_ocean_ave_vel;
        input.ocean_drag_vel = if ave_vel > 5000.0 { 5000.0 } else { ave_vel };
        if input.ocean_drag_vel < -5000.0 {
            input.ocean_drag_vel = -5000.0;
        }
    }

    let front_ocean_logic = world.obj::<ControllerTouchInput>(this).front_ocean_logic;
    if let Some(front_ocean_logic) = front_ocean_logic {
        ocean_logic::apply_drag_offset(world, front_ocean_logic, delta_x);
    }

    let behind_ocean_logic = world.obj::<ControllerTouchInput>(this).behind_ocean_logic;
    if let Some(behind_ocean_logic) = behind_ocean_logic {
        ocean_logic::apply_drag_offset(world, behind_ocean_logic, delta_x);
    }

    let mut input = world.obj_mut::<ControllerTouchInput>(this);
    input.drag_ocean_touch_pos = pos;
    input.drag_ocean_time_stamp = time_stamp;
}

/// `-[PocketGodViewController (Game) startDragCrack:timeStamp:]` @ 0x100160134
pub fn start_drag_crack(world: &mut World, this: Entity, pos: Vec4, _time_stamp: f32) {
    world.obj_mut::<ControllerTouchInput>(this).drag_crack = true;

    let crack_hole = world.obj::<ControllerWorldRegistry>(this).crack_hole_logic;
    if let Some(crack_hole) = crack_hole {
        crack_hole_logic::start_dragging(world, crack_hole, pos);
    }
}

/// `-[PocketGodViewController (Game) stopDragCrack]` @ 0x100160168
pub fn stop_drag_crack(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerTouchInput>(this).drag_crack = false;

    let crack_hole = world.obj::<ControllerWorldRegistry>(this).crack_hole_logic;
    if let Some(crack_hole) = crack_hole {
        crack_hole_logic::stop_dragging(world, crack_hole);
    }

    world.obj_mut::<ControllerTouchInput>(this).drag_crack_touch = None;
}

/// `-[PocketGodViewController (Game) dragCrack:timeStamp:]` @ 0x1001601c4
pub fn drag_crack(world: &mut World, this: Entity, pos: Vec4, _time_stamp: f32) {
    let crack_hole = world.obj::<ControllerWorldRegistry>(this).crack_hole_logic;
    if let Some(crack_hole) = crack_hole {
        crack_hole_logic::drag(world, crack_hole, pos);
    }
}

/// `-[PocketGodViewController (Game) startDragUnderwater:]` @ 0x1001601e4
pub fn start_drag_underwater(world: &mut World, this: Entity, pos: Vec4) {
    let mut input = world.obj_mut::<ControllerTouchInput>(this);
    input.underwater_touch_pos = pos;
    input.underwater_touch_start_y_pos = input.underwater_touch_pos.y;
}

/// `-[PocketGodViewController (Game) stopDragUnderwater:]` @ 0x100160214
pub fn stop_drag_underwater(world: &mut World, this: Entity, _pos: Vec4) {
    world.obj_mut::<ControllerTouchInput>(this).underwater_touch = None;
}

/// `-[PocketGodViewController (Game) dragUnderwater:]` @ 0x100160228
///
/// A swipe down by more than 100 units leaves the underwater view for the previous location.
pub fn drag_underwater(world: &mut World, this: Entity, pos: Vec4) {
    let underwater_touch_pos = world.obj::<ControllerTouchInput>(this).underwater_touch_pos;
    let delta_x = pos.x - underwater_touch_pos.x;
    let delta_y = pos.y - underwater_touch_pos.y;
    let angle = delta_y.atan2(delta_x);
    if angle < -2.0943952 || angle > -1.0471976 {
        world.obj_mut::<ControllerTouchInput>(this).underwater_touch = None;
        return;
    }

    let underwater_touch_start_y_pos = world.obj::<ControllerTouchInput>(this).underwater_touch_start_y_pos;
    let (island_location_transition, last_island_location_state) = {
        let locations = world.obj::<ControllerIslandLocations>(this);
        (locations.island_location_transition, locations.last_island_location_state)
    };
    if underwater_touch_start_y_pos - pos.y > 100.0 && !island_location_transition {
        world.obj_mut::<ControllerTouchInput>(this).underwater_touch = None;

        let surface_location = match last_island_location_state {
            -1 | 2 | 9 => 0,
            location => location,
        };
        let target_location = if (11..=17).contains(&last_island_location_state) {
            10
        } else {
            surface_location
        };
        island_locations::set_island_location_state(world, this, target_location, false);
    }

    world.obj_mut::<ControllerTouchInput>(this).underwater_touch_pos = pos;
}

/// `-[PocketGodViewController (Game) startApeWench:]` @ 0x100162210
pub fn start_ape_wench(world: &mut World, this: Entity, pos: Vec4) {
    world.obj_mut::<ControllerTouchInput>(this).last_ape_wench_pos = pos;
}

/// `-[PocketGodViewController (Game) dragApeWench:]` @ 0x100162228
///
/// Turns the crank by the finger's angle change around its centre, at most 22.5° per move.
pub fn drag_ape_wench(world: &mut World, this: Entity, pos: Vec4) {
    let ape_wench = world.obj::<ControllerWorldRegistry>(this).ape_wench_logic;
    let ape_wench = ape_wench.expect("dragApeWench: the original dereferences [[nil displayObject] pos]");
    let wench_display_object = pg_engine::logic::display_object(world, ape_wench);
    let wench_pos = display::object(world, wench_display_object).pos;
    let crank_x = wench_pos.x + 35.0 + 19.0;
    let crank_y = wench_pos.y;

    let last_ape_wench_pos = world.obj::<ControllerTouchInput>(this).last_ape_wench_pos;
    let last_angle = (last_ape_wench_pos.y - crank_y).atan2(last_ape_wench_pos.x - crank_x);
    let angle = (pos.y - crank_y).atan2(pos.x - crank_x);

    let mut delta_rot = angle - last_angle;
    if delta_rot > PI {
        delta_rot += -TAU;
    }
    if delta_rot < -PI {
        delta_rot += TAU;
    }
    if delta_rot > FRAC_PI_8 {
        delta_rot = FRAC_PI_8;
    }
    if delta_rot < -FRAC_PI_8 {
        delta_rot = -FRAC_PI_8;
    }

    ape_wench_logic::set_ape_wench_delta_rot(world, ape_wench, delta_rot);
    world.obj_mut::<ControllerTouchInput>(this).last_ape_wench_pos = pos;
}

/// `-[PocketGodViewController (Game) stopApeWench:]` @ 0x100162354
pub fn stop_ape_wench(world: &mut World, this: Entity, _pos: Vec4) {
    world.obj_mut::<ControllerTouchInput>(this).ape_wench_touch = None;

    let ape_wench = world.obj::<ControllerWorldRegistry>(this).ape_wench_logic;
    if let Some(ape_wench) = ape_wench {
        ape_wench_logic::stop_turning_ape_wench(world, ape_wench);
    }
}

/// `-[PocketGodViewController (Game) checkUnderwaterBubble:pos:]` @ 0x100163ac4
///
/// A burst of bubbles where the finger is, at most every 0.4 s of touch game time.
pub fn check_underwater_bubble(world: &mut World, this: Entity, touch: Entity, pos: Vec4) {
    let game_time = world.obj::<BcTouch>(touch).game_time;
    let last_bubble_time = world.obj::<ControllerTouchInput>(this).last_bubble_time;
    if game_time - last_bubble_time > 0.4 {
        world.obj_mut::<ControllerTouchInput>(this).last_bubble_time = game_time;

        let bubbles = world.spawn(Isa(&one_shot_anim_logic::ONE_SHOT_ANIM_LOGIC)).id();
        one_shot_anim_logic::init_with_sequence_id(world, bubbles, "UnderwaterBubbles");

        let bubbles_display_object = pg_engine::logic::display_object(world, bubbles);
        {
            let mut object = display::object_mut(world, bubbles_display_object);
            object.pos.x = pos.x;
            object.pos.y = pos.y;
            object.pos.z = -277.12;
        }

        let underwater_display_group = world.obj::<ControllerIslandLocations>(this).underwater_display_group;
        display::move_to_display_group(world, bubbles_display_object, underwater_display_group);
    }
}

/// `[displayGroup viewToDisplayObject:point]`: zero for a nil group, like any struct a message
/// to nil returns.
fn view_to_group(world: &World, display_group: Option<Entity>, view_point: Vec4) -> Vec4 {
    match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, view_point),
        None => Vec4::ZERO,
    }
}

/// `[displayGroup viewToDisplayObject:&mTouchScreenPos]`.
fn touch_screen_pos_in(world: &World, this: Entity, display_group: Option<Entity>) -> Vec4 {
    let touch_screen_pos = world.obj::<ControllerTouchInput>(this).touch_screen_pos;
    view_to_group(world, display_group, touch_screen_pos)
}

/// The touch in `IslandDisplayGroup` space.
fn island_touch_pos(world: &World, this: Entity) -> Vec4 {
    let island_display_group = world.obj::<ControllerIslandLocations>(this).island_display_group;
    touch_screen_pos_in(world, this, island_display_group)
}

/// The touch in `UnderwaterDisplayGroup` space.
fn underwater_touch_pos(world: &World, this: Entity) -> Vec4 {
    let underwater_display_group = world.obj::<ControllerIslandLocations>(this).underwater_display_group;
    touch_screen_pos_in(world, this, underwater_display_group)
}

/// The group a locust stroke is drawn in: the temple's inside a temple (11–17), the island's
/// elsewhere.
fn temple_or_island_display_group(world: &World, this: Entity, island_location_state: i32) -> Option<Entity> {
    let locations = world.obj::<ControllerIslandLocations>(this);
    match island_location_state {
        11 => locations.time_temple_display_group,
        12 => locations.locust_temple_display_group,
        13 => locations.dance_temple_display_group,
        14 => locations.plague_temple_display_group,
        15 => locations.purification_temple_display_group,
        16 => locations.closet_temple_display_group,
        17 => locations.xenu_temple_display_group,
        _ => locations.island_display_group,
    }
}

/// `[mIslandDisplayGroup getDisplayObject:id]`.
fn island_display_object(world: &World, this: Entity, id: &str) -> Option<Entity> {
    let island_display_group = world.obj::<ControllerIslandLocations>(this).island_display_group;
    match island_display_group {
        Some(island_display_group) => group::get_display_object(world, island_display_group, id),
        None => None,
    }
}

/// The distance from a display object's position to `point` in the xy plane.
fn distance_from_display_object(world: &World, object: Entity, point: Vec4) -> f32 {
    let object_pos = display::object(world, object).pos;
    let delta_x = object_pos.x - point.x;
    let delta_y = object_pos.y - point.y;
    (delta_x * delta_x + delta_y * delta_y).sqrt()
}

/// `[mOptionPreferences getIntPreference:key]` (0 while the document is nil).
fn option_int_preference(world: &World, this: Entity, key: &str) -> i32 {
    let option_preferences = world.obj::<ControllerBootAndFrame>(this).option_preferences;
    match option_preferences {
        Some(option_preferences) => pg_engine::prefs::get_int_preference(world, option_preferences, key),
        None => 0,
    }
}

/// Slot `index` of a logic array as it is now (a nested dispatch may have refilled it), or nil.
fn touch_logic_at(world: &World, this: Entity, logic_array: TouchLogicArray, index: usize) -> Option<Entity> {
    let input = world.obj::<ControllerTouchInput>(this);
    let slots = match logic_array {
        TouchLogicArray::Touch => &input.touch_logic_array,
        TouchLogicArray::Cancel => &input.cancel_touch_logic_array,
    };
    let logic = slots.get(index).copied().flatten()?;

    // PORT: the original's array retains its logics until the dispatch ends; the port despawns
    // an object as soon as its other owner releases it, so such a logic is skipped like nil.
    try_isa(world, logic)?;
    Some(logic)
}

/// The pinch-sun pre-check of `touchBegan:allTouches:`: with exactly two fingers down near the
/// sun (temperature power on), the other finger's gestures end and the pinch begins.
fn start_pinch_sun(world: &mut World, this: Entity, touch: Entity, all_touches: &[Entity]) -> bool {
    let konkey_dong = world.obj::<ControllerMinigames>(this).konkey_dong;
    let night_active = world.obj::<ControllerSkyWeather>(this).night_active;
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let underwater = island_location_state == 2 || island_location_state == 9;
    if konkey_dong || night_active || underwater || all_touches.len() != 2 {
        return false;
    }

    let hurricane = world.obj::<ControllerGodPowers>(this).hurricane_logic;
    let hurricane_active = match hurricane {
        Some(hurricane) => hurricane_logic::hurricane_active(world, hurricane),
        None => false,
    };
    if hurricane_active {
        return false;
    }

    let Some(sun) = island_display_object(world, this, "Sun") else {
        return false;
    };
    let temperature_state = option_int_preference(world, this, "TemperatureState");
    if temperature_state != 1 {
        return false;
    }

    let first_touch = all_touches[0];
    let second_touch = all_touches[1];
    let island_display_group = world.obj::<ControllerIslandLocations>(this).island_display_group;
    let first_view_pos = world.obj::<BcTouch>(first_touch).pos;
    let first_pos = view_to_group(world, island_display_group, first_view_pos);
    let first_sun_distance = distance_from_display_object(world, sun, first_pos);
    let second_view_pos = world.obj::<BcTouch>(second_touch).pos;
    let second_pos = view_to_group(world, island_display_group, second_view_pos);
    let second_sun_distance = distance_from_display_object(world, sun, second_pos);
    let both_near_sun = first_sun_distance < 120.0 && second_sun_distance < 120.0;
    if !both_near_sun {
        return false;
    }

    let other_touch = if first_touch == touch { second_touch } else { first_touch };
    touch_ended(world, this, other_touch, all_touches);

    let sun_state = world.obj::<ControllerSkyWeather>(this).sun_state;
    let delta_x = first_pos.x - second_pos.x;
    let delta_y = first_pos.y - second_pos.y;
    let pinch_sun_distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

    let mut input = world.obj_mut::<ControllerTouchInput>(this);
    input.pinch_sun_state = sun_state;
    input.pinch_sun_touch0 = Some(first_touch);
    input.pinch_sun_touch1 = Some(second_touch);
    input.pinch_sun_distance = pinch_sun_distance;
    true
}

/// The pinch branch of `touchMoved:allTouches:`: spreading the fingers by 30 grows the sun one
/// step from its size when the pinch began, closing them by 30 shrinks it, in between restores it.
fn move_pinch_sun(world: &mut World, this: Entity) {
    let (pinch_sun_touch0, pinch_sun_touch1) = {
        let input = world.obj::<ControllerTouchInput>(this);
        (input.pinch_sun_touch0, input.pinch_sun_touch1)
    };
    let (Some(first_touch), Some(second_touch)) = (pinch_sun_touch0, pinch_sun_touch1) else {
        return;
    };

    let island_display_group = world.obj::<ControllerIslandLocations>(this).island_display_group;
    let first_view_pos = world.obj::<BcTouch>(first_touch).pos;
    let first_pos = view_to_group(world, island_display_group, first_view_pos);
    let second_view_pos = world.obj::<BcTouch>(second_touch).pos;
    let second_pos = view_to_group(world, island_display_group, second_view_pos);

    let (pinch_sun_distance, pinch_sun_state) = {
        let input = world.obj::<ControllerTouchInput>(this);
        (input.pinch_sun_distance, input.pinch_sun_state)
    };
    let delta_x = first_pos.x - second_pos.x;
    let delta_y = first_pos.y - second_pos.y;
    let spread = (delta_x * delta_x + delta_y * delta_y).sqrt() - pinch_sun_distance;

    let sun_state = if spread < -30.0 {
        match pinch_sun_state {
            1 => 0,
            0 => 2,
            _ => return,
        }
    } else if spread < 30.0 {
        pinch_sun_state
    } else {
        match pinch_sun_state {
            0 => 1,
            2 => 0,
            _ => return,
        }
    };
    sky_weather::set_sun_state(world, this, sun_state);
}

/// The logic dispatch of `touchBegan:allTouches:`, frontmost first: the first selectable logic
/// under the finger takes the touch (`touchBeganInside:`), the others get `touchBeganOutside:`.
/// Returns whether a logic took it.
fn dispatch_touch_began(world: &mut World, this: Entity, touch: Entity) -> bool {
    let end = create_touch_logic_array(world, this, TouchLogicArray::Touch);
    let mut taken = false;
    let mut index = end;
    while index > 0 {
        index -= 1;
        let Some(logic) = touch_logic_at(world, this, TouchLogicArray::Touch, index) else {
            continue;
        };

        let logic_display_object = pg_engine::logic::display_object(world, logic);
        let selectable = display::object(world, logic_display_object).is_selectable;
        if !selectable {
            continue;
        }

        let class = isa(world, logic);
        if !taken {
            let touch_pos = world.obj::<BcTouch>(touch).pos;
            let inside = pg_engine::logic::does_point_intersect(world, logic, touch_pos);
            if inside {
                world.obj_mut::<BcTouch>(touch).active_object = Some(logic);
                (class.touch_began_inside)(world, logic, touch);
                taken = true;
                continue;
            }
        }
        (class.touch_began_outside)(world, logic, touch);
    }
    release_touch_logic_array(world, this, TouchLogicArray::Touch);
    taken
}

/// The logic dispatch of `touchMoved:allTouches:`, frontmost first: the first logic under the
/// finger becomes the touch's active object (entered/inside), the others get exited/outside.
fn dispatch_touch_moved(world: &mut World, this: Entity, touch: Entity) {
    let end = create_touch_logic_array(world, this, TouchLogicArray::Touch);
    let mut found = false;
    let mut index = end;
    while index > 0 {
        index -= 1;
        let Some(logic) = touch_logic_at(world, this, TouchLogicArray::Touch, index) else {
            continue;
        };

        let class = isa(world, logic);
        if !found {
            let touch_pos = world.obj::<BcTouch>(touch).pos;
            let inside = pg_engine::logic::does_point_intersect(world, logic, touch_pos);
            if inside {
                let active_object = world.obj::<BcTouch>(touch).active_object;
                if active_object != Some(logic) {
                    if let Some(active_object) = active_object
                        && let Some(active_class) = try_isa(world, active_object)
                    {
                        (active_class.touch_moved_exited)(world, active_object, touch);
                    }
                    world.obj_mut::<BcTouch>(touch).active_object = Some(logic);
                    (class.touch_moved_entered)(world, logic, touch);
                }
                (class.touch_moved_inside)(world, logic, touch);
                found = true;
                continue;
            }
        }

        let active_object = world.obj::<BcTouch>(touch).active_object;
        if active_object == Some(logic) {
            world.obj_mut::<BcTouch>(touch).active_object = None;
            (class.touch_moved_exited)(world, logic, touch);
        }
        (class.touch_moved_outside)(world, logic, touch);
    }
    release_touch_logic_array(world, this, TouchLogicArray::Touch);
}

/// The logic dispatch of `touchEnded:allTouches:` and `cancelAllTouches`, frontmost first: the
/// first logic under the finger gets `touchEndedInside:`, the others `touchEndedOutside:`.
fn dispatch_touch_ended(world: &mut World, this: Entity, touch: Entity, logic_array: TouchLogicArray) {
    let end = create_touch_logic_array(world, this, logic_array);
    let mut found = false;
    let mut index = end;
    while index > 0 {
        index -= 1;
        let Some(logic) = touch_logic_at(world, this, logic_array, index) else {
            continue;
        };

        let class = isa(world, logic);
        if !found {
            let touch_pos = world.obj::<BcTouch>(touch).pos;
            let inside = pg_engine::logic::does_point_intersect(world, logic, touch_pos);
            if inside {
                (class.touch_ended_inside)(world, logic, touch);
                found = true;
                continue;
            }
        }
        (class.touch_ended_outside)(world, logic, touch);
    }
    release_touch_logic_array(world, this, logic_array);
}

/// The underwater branch (locations 2 and 9) of `touchBegan:allTouches:`: bubbles, the camera
/// swipe, then the coral reef power stroke or else the whirlpool finger.
fn begin_underwater_gesture(world: &mut World, this: Entity, touch: Entity) {
    let underwater_pos = underwater_touch_pos(world, this);
    check_underwater_bubble(world, this, touch, underwater_pos);

    let underwater_touch = world.obj::<ControllerTouchInput>(this).underwater_touch;
    if underwater_touch.is_none() {
        world.obj_mut::<ControllerTouchInput>(this).underwater_touch = Some(touch);
        start_drag_underwater(world, this, underwater_pos);
    }

    let underwater_power_touch = world.obj::<ControllerGodPowers>(this).underwater_power_touch;
    if underwater_power_touch.is_none() && god_powers::is_underwater_power_active(world, this) {
        world.obj_mut::<ControllerGodPowers>(this).underwater_power_touch = Some(touch);
        god_powers::start_drag_underwater_power(world, this, underwater_pos);
        return;
    }

    let whirl_pool_touch = world.obj::<ControllerGodPowers>(this).whirl_pool_touch;
    if whirl_pool_touch.is_none() {
        world.obj_mut::<ControllerGodPowers>(this).whirl_pool_touch = Some(touch);
    }
}

/// The temple branch (locations 11–17) of `touchBegan:allTouches:`: the drum tap and the
/// locust stroke.
fn begin_temple_gesture(world: &mut World, this: Entity, touch: Entity, island_location_state: i32) {
    let temple_display_group = temple_or_island_display_group(world, this, island_location_state);
    let temple_pos = touch_screen_pos_in(world, this, temple_display_group);
    god_powers::check_for_tapping(world, this, touch);

    let locust_touch = world.obj::<ControllerGodPowers>(this).locust_touch;
    if locust_touch.is_none() && god_powers::is_locust_active(world, this) {
        world.obj_mut::<ControllerGodPowers>(this).locust_touch = Some(touch);
        god_powers::start_draw_locust(world, this, temple_pos);
    }
}

/// The island branch (every other location) of `touchBegan:allTouches:`: sun, moon and
/// lightning take the finger first; otherwise the drum tap, a cloud drag above the horizon,
/// and the first of locust, crack, ocean, ape wench, hurricane and rainbow.
fn begin_island_gesture(world: &mut World, this: Entity, touch: Entity) {
    let island_pos = island_touch_pos(world, this);

    let sun = island_display_object(world, this, "Sun");
    let night_active = world.obj::<ControllerSkyWeather>(this).night_active;
    let near_sun = match sun {
        Some(sun) if !night_active => distance_from_display_object(world, sun, island_pos) < 40.0,
        _ => false,
    };
    let (drag_sun_touch, drag_moon_touch) = {
        let sky = world.obj::<ControllerSkyWeather>(this);
        (sky.drag_sun_touch, sky.drag_moon_touch)
    };
    if near_sun && drag_sun_touch.is_none() && drag_moon_touch.is_none() {
        world.obj_mut::<ControllerSkyWeather>(this).drag_sun_touch = Some(touch);
        sky_weather::start_drag_sun(world, this, island_pos);
        return;
    }

    let moon = island_display_object(world, this, "Moon");
    let near_moon = match moon {
        Some(moon) if night_active => distance_from_display_object(world, moon, island_pos) < 40.0,
        _ => false,
    };
    if near_moon && drag_sun_touch.is_none() && drag_moon_touch.is_none() {
        world.obj_mut::<ControllerSkyWeather>(this).drag_moon_touch = Some(touch);
        sky_weather::check_for_bat_tapping(world, this, touch);
        sky_weather::start_drag_moon(world, this, island_pos);
        return;
    }

    let weather_state = world.obj::<ControllerSkyWeather>(this).weather_state;
    let (lightning_logic, lightning_touch) = {
        let powers = world.obj::<ControllerGodPowers>(this);
        (powers.lightning_logic, powers.lightning_touch)
    };
    if island_pos.y > 250.0 && weather_state == 1 && lightning_logic.is_none() && lightning_touch.is_none() {
        world.obj_mut::<ControllerGodPowers>(this).lightning_touch = Some(touch);
        god_powers::start_draw_lightning(world, this, island_pos);
        return;
    }

    god_powers::check_for_tapping(world, this, touch);

    let horizon_y_pos = world.obj::<ControllerIslandLocations>(this).horizon_y_pos;
    let drag_clouds_touch = world.obj::<ControllerSkyWeather>(this).drag_clouds_touch;
    if island_pos.y > horizon_y_pos && drag_clouds_touch.is_none() {
        world.obj_mut::<ControllerSkyWeather>(this).drag_clouds_touch = Some(touch);
        let time_stamp = world.obj::<BcTouch>(touch).time_stamp;
        sky_weather::start_drag_clouds(world, this, island_pos, time_stamp);
    }

    let locust_touch = world.obj::<ControllerGodPowers>(this).locust_touch;
    if locust_touch.is_none() && god_powers::is_locust_active(world, this) {
        world.obj_mut::<ControllerGodPowers>(this).locust_touch = Some(touch);
        god_powers::start_draw_locust(world, this, island_pos);
        return;
    }

    let crack_hole_state = option_int_preference(world, this, "CrackHoleState");
    let (island_min_x_pos, island_max_x_pos) = {
        let locations = world.obj::<ControllerIslandLocations>(this);
        (locations.island_min_x_pos, locations.island_max_x_pos)
    };
    let on_island = island_min_x_pos < island_pos.x && island_pos.x < island_max_x_pos;
    let on_ground = 50.0 < island_pos.y && island_pos.y < 120.0;
    if crack_hole_state == 1 && on_island && on_ground {
        let crack_hole = world.obj::<ControllerWorldRegistry>(this).crack_hole_logic;
        let cracks_active = match crack_hole {
            Some(crack_hole) => crack_hole_logic::are_cracks_active(world, crack_hole),
            None => false,
        };
        if cracks_active {
            return;
        }
        let drag_crack_touch = world.obj::<ControllerTouchInput>(this).drag_crack_touch;
        if drag_crack_touch.is_some() {
            return;
        }
        world.obj_mut::<ControllerTouchInput>(this).drag_crack_touch = Some(touch);
        let time_stamp = world.obj::<BcTouch>(touch).time_stamp;
        start_drag_crack(world, this, island_pos, time_stamp);
        return;
    }

    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let horizon_y_pos = world.obj::<ControllerIslandLocations>(this).horizon_y_pos;
    let drag_ocean_touch = world.obj::<ControllerTouchInput>(this).drag_ocean_touch;
    if island_location_state != 5 && island_pos.y < horizon_y_pos && drag_ocean_touch.is_none() {
        world.obj_mut::<ControllerTouchInput>(this).drag_ocean_touch = Some(touch);
        let time_stamp = world.obj::<BcTouch>(touch).time_stamp;
        start_drag_ocean(world, this, island_pos, time_stamp);
        return;
    }

    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state == 5 {
        let ape_wench = world.obj::<ControllerWorldRegistry>(this).ape_wench_logic;
        let ape_wench = ape_wench
            .expect("touchBegan: on Ape island the original dereferences [[nil displayObject] pos]");
        let wench_display_object = pg_engine::logic::display_object(world, ape_wench);
        let wench_pos = display::object(world, wench_display_object).pos;
        let crank_x = wench_pos.x + 35.0 + 19.0;
        let delta_x = island_pos.x - crank_x;
        let delta_y = island_pos.y - wench_pos.y;
        let crank_distance = (delta_x * delta_x + delta_y * delta_y).sqrt();
        if crank_distance < 130.0 && ape_wench_logic::can_crank_wench(world, ape_wench) {
            world.obj_mut::<ControllerTouchInput>(this).ape_wench_touch = Some(touch);
            start_ape_wench(world, this, island_pos);
            return;
        }
    }

    let weather_state = world.obj::<ControllerSkyWeather>(this).weather_state;
    let hurricane_touch = world.obj::<ControllerGodPowers>(this).hurricane_touch;
    let horizon_y_pos = world.obj::<ControllerIslandLocations>(this).horizon_y_pos;
    if weather_state == 1
        && hurricane_touch.is_none()
        && island_pos.y > horizon_y_pos
        && option_int_preference(world, this, "HurricaneState") == 1
    {
        world.obj_mut::<ControllerGodPowers>(this).hurricane_touch = Some(touch);
        god_powers::start_hurricane(world, this, island_pos);
        return;
    }

    let horizon_y_pos = world.obj::<ControllerIslandLocations>(this).horizon_y_pos;
    let night_active = world.obj::<ControllerSkyWeather>(this).night_active;
    let rainbow_touch = world.obj::<ControllerGodPowers>(this).rainbow_touch;
    if island_pos.y > horizon_y_pos
        && !night_active
        && rainbow_touch.is_none()
        && option_int_preference(world, this, "RainbowState") == 1
    {
        world.obj_mut::<ControllerGodPowers>(this).rainbow_touch = Some(touch);
        god_powers::start_draw_rainbow(world, this, island_pos);
    }
}
