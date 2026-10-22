//! The game on the platform: what `UIApplicationMain` and UIKit do around the original's code.
//!
//! [`GamePlugin`] creates `PocketGodAppDelegate` and sends it
//! `application:didFinishLaunchingWithOptions:` once the launch image is on screen, runs the
//! original's timers and main-thread calls around the platform's display link, and turns the
//! platform's lifecycle transitions into `UIApplicationDelegate` calls
//! (`docs/re/managers-platform.md` §2, §18).

use bevy::ecs::message::{MessageCursor, Messages};
use bevy::prelude::*;
use pg_engine::hooks::EngineHooks;
use pg_game::controller::{self, WallClock};
use pg_game::logic::apocalypse_temples::apocalypse_calendar_logic::SystemTimeZone;
use pg_game::managers::game_server;
use pg_game::managers::idol_manager;
use pg_game::managers::pocket_god_app_delegate as app_delegate;
use pg_game::singletons::Singletons;
use pg_platform::splash::launch_image_shown;
use pg_platform::{PlatformLifecycle, PlatformSystems, services};
use pg_ui::PgUiPlugin;

/// Seconds from the Unix epoch to `CFAbsoluteTime`'s (1 January 2001, UTC).
const CF_ABSOLUTE_TIME_EPOCH: f64 = 978_307_200.0;

/// 21 December 2012, 23:11:00 UTC (Unix seconds): the day `ApocalypseCalendarLogic` converts with
/// the system time zone, whose offset on that day the calendar needs.
const APOCALYPSE_DAY: i64 = 1_356_131_460;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PgUiPlugin);
        app.init_resource::<Singletons>();
        app.init_resource::<AppDelegate>();
        app.add_systems(Startup, set_clock_and_time_zone);

        let launch = launch_once.run_if(launch_image_shown);
        app.add_systems(Update, launch.before(PlatformSystems::Input));

        let before_ticks = (forward_lifecycle, run_timers).chain();
        let after_input = before_ticks.after(PlatformSystems::Input);
        app.add_systems(Update, after_input.before(PlatformSystems::FrameTick));

        let main_thread_calls = fire_main_thread_calls.after(PlatformSystems::FrameTick);
        app.add_systems(Update, main_thread_calls.before(PlatformSystems::Output));
    }
}

/// `UIApplication`'s delegate, once launched.
#[derive(Resource, Default)]
struct AppDelegate(Option<Entity>);

/// `CFAbsoluteTimeGetCurrent()` starts at the real time; the system time zone for the calendar.
fn set_clock_and_time_zone(world: &mut World) {
    let now = services::unix_time_seconds() - CF_ABSOLUTE_TIME_EPOCH;
    world.insert_resource(WallClock { seconds: now });

    let seconds_from_gmt = services::seconds_from_gmt(APOCALYPSE_DAY);
    world.insert_resource(SystemTimeZone { seconds_from_gmt });
}

/// `UIApplicationMain`: the delegate gets `application:didFinishLaunchingWithOptions:` once the
/// launch image is on screen (its `loadGame` timer then runs the synchronous load).
fn launch_once(world: &mut World, mut launched: Local<bool>) {
    if *launched {
        return;
    }
    *launched = true;

    let mut hooks = world.resource_mut::<EngineHooks>();
    hooks.trigger_episode_event = trigger_episode_event;

    let delegate = app_delegate::spawn(world);
    world.resource_mut::<AppDelegate>().0 = Some(delegate);
    app_delegate::application(world, delegate);
}

/// `MultiModelLogic triggerEpisodeEvent:::` → `[[IdolManager instance] triggerEpisodeEvent:]`.
fn trigger_episode_event(world: &mut World, event_id: &str) {
    let Some(manager) = idol_manager::instance(world) else {
        return;
    };
    idol_manager::trigger_episode_event(world, manager, event_id);
}

/// The platform's lifecycle transitions as `UIApplicationDelegate` calls (the render loop itself
/// is started and stopped by the platform with the same conditions).
fn forward_lifecycle(world: &mut World, mut cursor: Local<MessageCursor<PlatformLifecycle>>) {
    let messages = world.resource::<Messages<PlatformLifecycle>>();
    let transitions: Vec<PlatformLifecycle> = cursor.read(messages).copied().collect();
    let Some(delegate) = world.resource::<AppDelegate>().0 else {
        return;
    };

    for transition in transitions {
        match transition {
            PlatformLifecycle::WillResignActive => app_delegate::application_will_resign_active(world, delegate),
            PlatformLifecycle::DidEnterBackground => app_delegate::application_did_enter_background(world, delegate),
            PlatformLifecycle::WillEnterForeground => app_delegate::application_will_enter_foreground(world, delegate),
            PlatformLifecycle::DidBecomeActive => {
                app_delegate::application_did_become_active(world, delegate);
                game_server::post_application_did_become_active_notification(world);
            }
        }
    }
}

/// The original's `NSTimer`s fire on the run loop in real time, also while the display link is
/// stopped: the app delegate's `loadGame` timer and the controller's (which also advance the
/// `CFAbsoluteTimeGetCurrent` clock).
fn run_timers(world: &mut World) {
    let elapsed = world.resource::<Time<Real>>().delta_secs_f64();
    app_delegate::fire_load_game_timer(world, elapsed);
    controller::run_deferred_calls(world, elapsed);
}

/// `performSelectorOnMainThread:…waitUntilDone:NO` calls run on the next pass of the main run
/// loop, after the display link's ticks.
fn fire_main_thread_calls(world: &mut World) {
    controller::fire_main_thread_calls(world);
}
