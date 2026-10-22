//! `ApocalypseCalendarLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::apocalypse_temples::apocalypse_gem_pedestal_logic;
use pg_engine::display;
use pg_engine::logic;

/// `ApocalypseCalendarLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ApocalypseCalendarLogic {
    /// `mApocalypseCalendarLogicListener`: `@"<ApocalypseCalendarLogicListener>"`
    pub apocalypse_calendar_logic_listener: Option<Entity>,

    /// `mbOpen`: `B`
    pub open: bool,

    /// `mbClosing`: `B`
    pub closing: bool,

    /// `mbCompletelyOpen`: `B`
    pub completely_open: bool,

    /// `mbTweenClockHands`: `B`
    pub tween_clock_hands: bool,

    /// `mApocalypseTime`: `d`
    pub apocalypse_time: f64,

    /// `mCalendarDial1Logic`: `@"MultiModelLogic"`
    pub calendar_dial1_logic: Option<Entity>,

    /// `mCalendarDial2Logic`: `@"MultiModelLogic"`
    pub calendar_dial2_logic: Option<Entity>,

    /// `mCalendarDial3Logic`: `@"MultiModelLogic"`
    pub calendar_dial3_logic: Option<Entity>,

    /// `mPedestalIndex`: `i`
    pub pedestal_index: i32,

    /// `mNextPedestalTime`: `f`
    pub next_pedestal_time: f32,
}

/// `CFTimeZoneCopySystem()`: the device's time zone, as the seconds it is ahead of GMT at the
/// calendar's date (21 Dec 2012). The platform layer sets it; absent means GMT.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct SystemTimeZone {
    pub seconds_from_gmt: f64,
}

pub const APOCALYPSE_CALENDAR_LOGIC: Class = Class {
    name: "ApocalypseCalendarLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    on_game_frame,
    touch_began_inside,
    touch_began_outside,
    touch_moved_entered,
    touch_moved_exited,
    touch_moved_inside,
    touch_moved_outside,
    touch_ended_inside,
    touch_ended_outside,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The dials' speeds in degrees per second: one turn per 183.9 s, per 25.03 h and per 365 days.
const DIAL1_DEGREES_PER_SECOND: f64 = 1.9577625570776256;
const DIAL2_DEGREES_PER_SECOND: f64 = 0.003995433789954338;
const DIAL3_DEGREES_PER_SECOND: f64 = 1.1415525114155251e-5;

/// The torch and gem items step to the next pedestal every half second.
const PEDESTAL_INTERVAL: f32 = 0.5;

/// Sequence-item handlers implemented by `ApocalypseCalendarLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "lightTorches" => seq_light_torches(world, this, call),
        "elevateGems" => seq_elevate_gems(world, this, call),
        "unlightTorches" => seq_unlight_torches(world, this, call),
        "calendarOpenComplete" => seq_calendar_open_complete(world, this, call),
        "calendarCloseComplete" => seq_calendar_close_complete(world, this, call),
        "tweenClockHands" => seq_tween_clock_hands(world, this, call),
        "untweenClockHands" => seq_untween_clock_hands(world, this, call),
        "hideClockHands" => seq_hide_clock_hands(world, this, call),
        "showClockHands" => seq_show_clock_hands(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `ApocalypseCalendarLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animSunInit" => anim_sun_init(world, this),
        "animSunWhatsUp" => anim_sun_whats_up(world, this),
        "animSunCaptured" => anim_sun_captured(world, this),
        "animSunPulledIn" => anim_sun_pulled_in(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[ApocalypseCalendarLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&APOCALYPSE_CALENDAR_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `CFGregorianDateGetAbsoluteTime(date, tz)`: seconds from 2001-01-01 00:00:00 GMT to the given
/// local date and time.
fn cf_gregorian_date_get_absolute_time(year: i64, month: i64, day: i64, hour: i64, minute: i64, second: f64, seconds_from_gmt: f64) -> f64 {
    // Days since 1970-01-01 of a proleptic Gregorian date (civil-from-days inverse).
    let days_from_civil = |year: i64, month: i64, day: i64| -> i64 {
        let shifted_year = if month <= 2 { year - 1 } else { year };
        let era = shifted_year.div_euclid(400);
        let year_of_era = shifted_year - era * 400;
        let shifted_month = (month + 9) % 12;
        let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146097 + day_of_era - 719468
    };

    let days = days_from_civil(year, month, day) - days_from_civil(2001, 1, 1);
    let seconds_of_day = hour * 3600 + minute * 60;
    let gmt_time = (days * 86400) as f64 + seconds_of_day as f64 + second;
    gmt_time - seconds_from_gmt
}

/// `[mApocalypseCalendarLogicListener apocalypseGemPedestalLogicArray:self]` (empty for a nil
/// listener).
fn apocalypse_gem_pedestal_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let listener = world.obj::<ApocalypseCalendarLogic>(this).apocalypse_calendar_logic_listener;
    match listener {
        Some(listener) => crate::controller::apocalypse_gem_pedestal_logic_array(world, listener, this),
        None => Vec::new(),
    }
}

/// `[[mDisplayObject displayGroup] getDisplayObject:@"ClockCenter"]`.
fn clock_center(world: &World, this: Entity) -> Option<Entity> {
    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group?;
    display::group::get_display_object(world, display_group, "ClockCenter")
}

/// `setIsHidden:` on `ClockCenter` and the three dials.
fn set_clock_hands_hidden(world: &mut World, this: Entity, clock_center: Option<Entity>, hidden: bool) {
    if let Some(clock_center) = clock_center {
        display::object_mut(world, clock_center).hidden = hidden;
    }

    let calendar = world.obj::<ApocalypseCalendarLogic>(this);
    let dials = [calendar.calendar_dial1_logic, calendar.calendar_dial2_logic, calendar.calendar_dial3_logic];
    for dial in dials.into_iter().flatten() {
        let dial_display_object = logic::display_object(world, dial);
        display::object_mut(world, dial_display_object).hidden = hidden;
    }
}

/// The shared body of `lightTorches:::`, `elevateGems:::` and `unlightTorches:::`: every half
/// second `command` goes to the next pedestal; done after the last one.
fn step_pedestals(world: &mut World, this: Entity, call: &SequenceCall, command: fn(&mut World, Entity)) -> ItemResult {
    if call.first_time {
        world.obj_mut::<ApocalypseCalendarLogic>(this).pedestal_index = 0;
    }

    let next_pedestal_time = {
        let mut calendar = world.obj_mut::<ApocalypseCalendarLogic>(this);
        calendar.next_pedestal_time = calendar.next_pedestal_time - call.delta_time;
        calendar.next_pedestal_time
    };
    if !(next_pedestal_time < 0.0) {
        return ItemResult::Pending;
    }

    world.obj_mut::<ApocalypseCalendarLogic>(this).next_pedestal_time = PEDESTAL_INTERVAL;
    let pedestals = apocalypse_gem_pedestal_logic_array(world, this);
    let pedestal_index = world.obj::<ApocalypseCalendarLogic>(this).pedestal_index;
    let pedestal = pedestals.get(pedestal_index as usize).copied();
    if let Some(pedestal) = pedestal {
        command(world, pedestal);
    }

    let next_index = pedestal_index + 1;
    world.obj_mut::<ApocalypseCalendarLogic>(this).pedestal_index = next_index;
    if i64::from(next_index) == pedestals.len() as i64 {
        return ItemResult::Done;
    }
    ItemResult::Pending
}

/// `-[ApocalypseCalendarLogic initWithDisplayObject:]` @ 0x1000d7250
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ApocalypseCalendarLogic::default());
    crate::logic::island_core::sprite_logic::init_with_display_object(world, this, display_object);

    let dial1 = pg_engine::library::logic(world, crate::logic_table::CLOCK_DIAL1_MULTI_MODEL);
    let dial2 = pg_engine::library::logic(world, crate::logic_table::CLOCK_DIAL2_MULTI_MODEL);
    let dial3 = pg_engine::library::logic(world, crate::logic_table::CLOCK_DIAL3_MULTI_MODEL);
    {
        let mut calendar = world.obj_mut::<ApocalypseCalendarLogic>(this);
        calendar.calendar_dial1_logic = Some(dial1);
        calendar.calendar_dial2_logic = Some(dial2);
        calendar.calendar_dial3_logic = Some(dial3);
    }
    logic::set_behavior(world, this, Some("ApocalypseCalendarInit"));

    // 21 Dec 2012 23:11:00, local time.
    let seconds_from_gmt = match world.get_resource::<SystemTimeZone>() {
        Some(time_zone) => time_zone.seconds_from_gmt,
        None => 0.0,
    };
    let apocalypse_time = cf_gregorian_date_get_absolute_time(2012, 12, 21, 23, 11, 0.0, seconds_from_gmt);
    world.obj_mut::<ApocalypseCalendarLogic>(this).apocalypse_time = apocalypse_time;
}

/// `-[ApocalypseCalendarLogic setApocalypseCalendarLogicListener:]` @ 0x1000d7380
pub fn set_apocalypse_calendar_logic_listener(world: &mut World, this: Entity, apocalypse_calendar_logic_listener: Entity) {
    world.obj_mut::<ApocalypseCalendarLogic>(this).apocalypse_calendar_logic_listener = Some(apocalypse_calendar_logic_listener);
}

/// `-[ApocalypseCalendarLogic isOpen]` @ 0x1000d73dc
pub fn is_open(world: &mut World, this: Entity) -> bool {
    world.obj::<ApocalypseCalendarLogic>(this).completely_open
}

/// `-[ApocalypseCalendarLogic closeApocalypsePortal]` @ 0x1000d73ec
pub fn close_apocalypse_portal(world: &mut World, this: Entity) {
    {
        let mut calendar = world.obj_mut::<ApocalypseCalendarLogic>(this);
        calendar.open = false;
        calendar.closing = true;
        calendar.completely_open = false;
    }
    logic::set_behavior(world, this, Some("ApocalypseCalendarClose"));
}

/// `-[ApocalypseCalendarLogic enterIsland:]` @ 0x1000d742c
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    let clock_center = clock_center(world, this);
    set_clock_hands_hidden(world, this, clock_center, false);

    let calendar = world.obj::<ApocalypseCalendarLogic>(this);
    let open = calendar.open;
    let listener = calendar.apocalypse_calendar_logic_listener;

    if !open {
        let tom_pygmy_active = match listener {
            Some(listener) => crate::controller::is_tom_pygmy_active(world, listener),
            None => false,
        };
        if !tom_pygmy_active {
            logic::set_behavior(world, this, Some("ApocalypseCalendarInit"));
            return;
        }

        world.obj_mut::<ApocalypseCalendarLogic>(this).open = true;
        let idol_manager = crate::managers::idol_manager::instance(world);
        if let Some(idol_manager) = idol_manager {
            crate::managers::idol_manager::trigger_episode_event(world, idol_manager, "BringTomToTempleEntrance");
        }
        logic::set_behavior(world, this, Some("ApocalypseCalendarOpen"));
        return;
    }

    world.obj_mut::<ApocalypseCalendarLogic>(this).completely_open = true;
    let pedestals = apocalypse_gem_pedestal_logic_array(world, this);
    for pedestal in pedestals {
        apocalypse_gem_pedestal_logic::start_torch_lit(world, pedestal);
    }

    set_clock_hands_hidden(world, this, clock_center, true);
    logic::set_behavior(world, this, Some("ApocalypseCalendarStartOpen"));
}

/// `-[ApocalypseCalendarLogic exitIsland:]` @ 0x1000d7788
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    let closing = world.obj::<ApocalypseCalendarLogic>(this).closing;
    if !closing {
        return;
    }

    world.obj_mut::<ApocalypseCalendarLogic>(this).tween_clock_hands = false;
    let listener = world.obj::<ApocalypseCalendarLogic>(this).apocalypse_calendar_logic_listener;
    if let Some(listener) = listener {
        crate::controller::end_apocalypse_complete_now(world, listener);
    }

    let pedestals = apocalypse_gem_pedestal_logic_array(world, this);
    for pedestal in pedestals {
        apocalypse_gem_pedestal_logic::unlight_torch_now(world, pedestal);
    }
}

/// `-[ApocalypseCalendarLogic exitIslandNow]` @ 0x1000d78dc
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[ApocalypseCalendarLogic animSunInit]` @ 0x1000d78f0
pub fn anim_sun_init(world: &mut World, _this: Entity) {
    let sun_pygmy = pg_engine::library::logic(world, crate::logic_table::SUN_PYGMY);
    logic::set_behavior(world, sun_pygmy, Some("PygmySunInit"));
}

/// `-[ApocalypseCalendarLogic animSunWhatsUp]` @ 0x1000d7940
pub fn anim_sun_whats_up(world: &mut World, _this: Entity) {
    let sun_pygmy = pg_engine::library::logic(world, crate::logic_table::SUN_PYGMY);
    logic::set_behavior(world, sun_pygmy, Some("PygmySunWhatsUp"));
}

/// `-[ApocalypseCalendarLogic animSunCaptured]` @ 0x1000d7990
pub fn anim_sun_captured(world: &mut World, _this: Entity) {
    let sun_pygmy = pg_engine::library::logic(world, crate::logic_table::SUN_PYGMY);
    logic::set_behavior(world, sun_pygmy, Some("PygmySunCaptured"));
}

/// `-[ApocalypseCalendarLogic animSunPulledIn]` @ 0x1000d79e0
pub fn anim_sun_pulled_in(world: &mut World, _this: Entity) {
    let sun_pygmy = pg_engine::library::logic(world, crate::logic_table::SUN_PYGMY);
    logic::set_behavior(world, sun_pygmy, Some("PygmySunPulledIn"));
}

/// `-[ApocalypseCalendarLogic lightTorches:::]` @ 0x1000d7a94
pub fn seq_light_torches(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    step_pedestals(world, this, call, apocalypse_gem_pedestal_logic::light_torch)
}

/// `-[ApocalypseCalendarLogic elevateGems:::]` @ 0x1000d7b88
pub fn seq_elevate_gems(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    step_pedestals(world, this, call, apocalypse_gem_pedestal_logic::elevate_gem)
}

/// `-[ApocalypseCalendarLogic unlightTorches:::]` @ 0x1000d7c7c
pub fn seq_unlight_torches(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    step_pedestals(world, this, call, apocalypse_gem_pedestal_logic::unlight_torch)
}

/// `-[ApocalypseCalendarLogic calendarOpenComplete:::]` @ 0x1000d7d70
pub fn seq_calendar_open_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ApocalypseCalendarLogic>(this).completely_open = true;
    ItemResult::Done
}

/// `-[ApocalypseCalendarLogic calendarCloseComplete:::]` @ 0x1000d7d88
pub fn seq_calendar_close_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ApocalypseCalendarLogic>(this).closing = false;

    let listener = world.obj::<ApocalypseCalendarLogic>(this).apocalypse_calendar_logic_listener;
    if let Some(listener) = listener {
        crate::controller::end_apocalypse_complete(world, listener);
    }
    ItemResult::Done
}

/// `-[ApocalypseCalendarLogic tweenClockHands:::]` @ 0x1000d7dc4
pub fn seq_tween_clock_hands(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ApocalypseCalendarLogic>(this).tween_clock_hands = true;
    ItemResult::Done
}

/// `-[ApocalypseCalendarLogic untweenClockHands:::]` @ 0x1000d7ddc
pub fn seq_untween_clock_hands(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ApocalypseCalendarLogic>(this).tween_clock_hands = false;
    ItemResult::Done
}

/// `-[ApocalypseCalendarLogic hideClockHands:::]` @ 0x1000d7df0
pub fn seq_hide_clock_hands(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let clock_center = clock_center(world, this);
    set_clock_hands_hidden(world, this, clock_center, true);
    ItemResult::Done
}

/// `-[ApocalypseCalendarLogic showClockHands:::]` @ 0x1000d7ed4
pub fn seq_show_clock_hands(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let clock_center = clock_center(world, this);
    set_clock_hands_hidden(world, this, clock_center, false);
    ItemResult::Done
}

/// `-[ApocalypseCalendarLogic onGameFrame:]` @ 0x1000d7fb8
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let calendar = world.obj::<ApocalypseCalendarLogic>(this);
    let dials = (calendar.calendar_dial1_logic, calendar.calendar_dial2_logic, calendar.calendar_dial3_logic);
    let tween_clock_hands = calendar.tween_clock_hands;
    let apocalypse_time = calendar.apocalypse_time;
    let (Some(dial1), Some(dial2), Some(dial3)) = dials else {
        return;
    };

    let dial1_display_object = logic::display_object(world, dial1);
    let dial2_display_object = logic::display_object(world, dial2);
    let dial3_display_object = logic::display_object(world, dial3);

    // The dials count down to the apocalypse; tweened hands go to 0.
    let mut target1: f32 = 0.0;
    let mut target2: f32 = 0.0;
    let mut target3: f32 = 0.0;
    if !tween_clock_hands {
        // PORT: `CFAbsoluteTimeGetCurrent()` is the controller's `WallClock`, which reads the
        // real date only when the app wiring seeds it with the absolute time.
        let now = crate::controller::cf_absolute_time_get_current(world);
        let seconds_left = apocalypse_time - now;
        target1 = ((seconds_left * DIAL1_DEGREES_PER_SECOND) % 360.0) as f32;
        target2 = ((seconds_left * DIAL2_DEGREES_PER_SECOND) % 360.0) as f32;
        target3 = ((seconds_left * DIAL3_DEGREES_PER_SECOND) % 360.0) as f32;
    }

    // Each dial takes the short way round: a target more than half a turn below the dial moves
    // up a turn, a dial more than half a turn below its target moves up a turn.
    let rot1 = display::object(world, dial1_display_object).rot.z;
    let rot2 = display::object(world, dial2_display_object).rot.z;
    let rot3 = display::object(world, dial3_display_object).rot.z;

    let difference1 = rot1 - target1;
    if difference1 > 180.0 {
        target1 = target1 + 360.0;
    }
    let difference2 = rot2 - target2;
    if difference1 < -180.0 {
        display::object_mut(world, dial1_display_object).rot.z = rot1 + 360.0;
    }
    if difference2 > 180.0 {
        target2 = target2 + 360.0;
    }
    let difference3 = rot3 - target3;
    if difference2 < -180.0 {
        let mut display = display::object_mut(world, dial2_display_object);
        display.rot.z = display.rot.z + 360.0;
    }
    if difference3 > 180.0 {
        target3 = target3 + 360.0;
    }
    if difference3 < -180.0 {
        let mut display = display::object_mut(world, dial3_display_object);
        display.rot.z = display.rot.z + 360.0;
    }

    {
        let mut display = display::object_mut(world, dial1_display_object);
        display.rot.z = display.rot.z + (target1 - display.rot.z) * 0.1;
    }
    {
        let mut display = display::object_mut(world, dial2_display_object);
        display.rot.z = display.rot.z + (target2 - display.rot.z) * 0.1;
    }
    {
        let mut display = display::object_mut(world, dial3_display_object);
        display.rot.z = display.rot.z + (target3 - display.rot.z) * 0.1;
    }

    {
        let mut display = display::object_mut(world, dial1_display_object);
        if display.rot.z < 0.0 {
            display.rot.z = display.rot.z + 360.0;
        }
    }

    // Original bug, kept: the second dial is wrapped twice and the third never.
    let mut display = display::object_mut(world, dial2_display_object);
    let mut rot2 = display.rot.z;
    if rot2 < 0.0 {
        rot2 = rot2 + 360.0;
        display.rot.z = rot2;
    }
    if rot2 < 0.0 {
        display.rot.z = rot2 + 360.0;
    }
}

/// `-[ApocalypseCalendarLogic touchBeganInside:allTouches:]` @ 0x1000d8258
pub fn touch_began_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApocalypseCalendarLogic touchBeganOutside:allTouches:]` @ 0x1000d825c
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApocalypseCalendarLogic touchMovedEntered:allTouches:]` @ 0x1000d8260
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApocalypseCalendarLogic touchMovedExited:allTouches:]` @ 0x1000d8264
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApocalypseCalendarLogic touchMovedInside:allTouches:]` @ 0x1000d8268
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[ApocalypseCalendarLogic touchMovedOutside:allTouches:]` @ 0x1000d8278
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApocalypseCalendarLogic touchEndedInside:allTouches:]` @ 0x1000d827c
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApocalypseCalendarLogic touchEndedOutside:allTouches:]` @ 0x1000d8280
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
