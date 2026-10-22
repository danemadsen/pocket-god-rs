//! `PocketGodAppDelegate`: methods in this part of the class.
//!
//! The `UIApplicationDelegate` (`docs/re/managers-platform.md` §2). UIKit's side falls to the app
//! wiring: it creates the delegate with [`spawn`], sends [`application`] once the platform shows
//! the launch image, runs [`fire_load_game_timer`] on every platform tick, and turns the
//! platform's lifecycle transitions into the `application…` calls below. Right after
//! [`application_did_become_active`], UIKit posts `UIApplicationDidBecomeActiveNotification`:
//! `crate::managers::game_server::post_application_did_become_active_notification`.

use pg_engine::platform::PlatformRequests;
use pg_engine::view;

use crate::prelude::*;
use super::*;

/// The interval of the `loadGame` timer (`0x3fb99999a0000000`, `(double)0.1f`).
const LOAD_GAME_DELAY: f64 = 0.1_f32 as f64;

/// The one-shot `NSTimer` of `PocketGodAppDelegate`.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct PocketGodAppDelegateStatics {
    /// `[NSTimer scheduledTimerWithTimeInterval:0.1 target:self selector:@selector(loadGame)
    /// userInfo:nil repeats:NO]` of `application:didFinishLaunchingWithOptions:`: its target and
    /// the seconds until it fires.
    pub load_game_timer: Option<(Entity, f64)>,
}

/// `[[PocketGodAppDelegate alloc] init]`: `UIApplicationMain` creates the delegate (`NSObject`'s
/// `init`, every ivar zero) before the storyboard sets its `window`.
pub fn spawn(world: &mut World) -> Entity {
    let delegate = (Isa(&POCKET_GOD_APP_DELEGATE), PocketGodAppDelegate::default());
    world.spawn(delegate).id()
}

/// Fires the `loadGame` timer of [`application`] once it is due. The app wiring calls this on
/// every platform tick with the seconds since the previous one, also while the render loop is
/// stopped (it only starts after `loadGame`), like `crate::controller::run_deferred_calls`.
pub fn fire_load_game_timer(world: &mut World, elapsed: f64) {
    let target = {
        let Some(mut statics) = world.get_resource_mut::<PocketGodAppDelegateStatics>() else {
            return;
        };
        let Some((target, remaining)) = statics.load_game_timer else {
            return;
        };

        let remaining = remaining - elapsed;
        if remaining > 0.0 {
            statics.load_game_timer = Some((target, remaining));
            return;
        }
        statics.load_game_timer = None;
        target
    };

    load_game(world, target);
}

/// `-[PocketGodAppDelegate application:didFinishLaunchingWithOptions:]` @ 0x100089324
///
/// The `UIApplication` and the launch options (unused) have no counterpart in the port.
pub fn application(world: &mut World, this: Entity) -> bool {
    // Crashlytics `startWithAPIKey:`, Flurry `setCrashReportingEnabled:NO` and `startSession:`
    // are removed (`docs/re/managers-platform.md` §18.1 rows 1–2).
    // `[application setIdleTimerDisabled:YES]`: the platform keeps the screen on.

    let controller = world.spawn(Isa(&crate::controller::POCKET_GOD_VIEW_CONTROLLER)).id();
    crate::controller::init(world, controller);
    world.obj_mut::<PocketGodAppDelegate>(this).pocket_god_view_controller = Some(controller);
    crate::controller::set_delegate(world, controller, this);

    // `window.frame = UIScreen.mainScreen.bounds`, `window.rootViewController = controller` and
    // `[window makeKeyAndVisible]`: the platform's window shows the game.
    show_loading_image(world, this);

    // Chartboost `startWithAppId:appSignature:delegate:` and `cacheInterstitial:@"Home Screen"`
    // are removed (§18.1 rows 3–5).
    let load_game_timer = (this, LOAD_GAME_DELAY);
    let mut statics = world.get_resource_or_insert_with(PocketGodAppDelegateStatics::default);
    statics.load_game_timer = Some(load_game_timer);
    true
}

/// `-[PocketGodAppDelegate showLoadingImage]` @ 0x100089570
///
/// The original puts a `UIImageView` over the window with the launch image that suits the
/// screen's `applicationFrame` (`Default`, `Default-568h.png` or `Default-Landscape~ipad.png`,
/// turned to landscape).
pub fn show_loading_image(_world: &mut World, _this: Entity) {
    // PORT: the platform draws the launch image (pg-platform's splash, picked by the viewport's
    // aspect) from the first frame until `on_hide_loading_screen` sets
    // `PlatformRequests::loading_screen_hidden`. There is no image view, so `mImageView` stays
    // nil.
}

/// `-[PocketGodAppDelegate applicationDidBecomeActive:]` @ 0x100089a24
pub fn application_did_become_active(world: &mut World, this: Entity) {
    let delegate = world.obj::<PocketGodAppDelegate>(this);
    let resume = delegate.startup_complete && !delegate.dashboard_open && !delegate.modal_view_open;
    if resume {
        view::start_render_loop(world);
    }
}

/// `-[PocketGodAppDelegate applicationWillResignActive:]` @ 0x100089a90
pub fn application_will_resign_active(world: &mut World, this: Entity) {
    let delegate = world.obj::<PocketGodAppDelegate>(this);
    let pause = delegate.startup_complete && !delegate.dashboard_open && !delegate.modal_view_open;
    if pause {
        view::stop_render_loop(world);
    }
}

/// `-[PocketGodAppDelegate applicationDidEnterBackground:]` @ 0x100089afc
pub fn application_did_enter_background(world: &mut World, this: Entity) {
    let delegate = world.obj::<PocketGodAppDelegate>(this);
    let modal_view_open = delegate.modal_view_open;
    let controller = delegate.pocket_god_view_controller;
    if modal_view_open && let Some(controller) = controller {
        crate::controller::dismiss_modal_view_controller(world, controller);
    }

    let delegate = world.obj::<PocketGodAppDelegate>(this);
    let pause = delegate.startup_complete && !delegate.dashboard_open;
    if pause {
        view::stop_render_loop(world);
    }
}

/// `-[PocketGodAppDelegate applicationWillEnterForeground:]` @ 0x100089b94
pub fn application_will_enter_foreground(world: &mut World, this: Entity) {
    let delegate = world.obj::<PocketGodAppDelegate>(this);
    let resume = delegate.startup_complete && !delegate.dashboard_open;
    if !resume {
        return;
    }

    let controller = world.obj::<PocketGodAppDelegate>(this).pocket_god_view_controller;
    if let Some(controller) = controller {
        crate::controller::refresh_news(world, controller);
    }
    view::start_render_loop(world);
}

/// `-[PocketGodAppDelegate applicationWillTerminate:]` @ 0x100089c0c
pub fn application_will_terminate(_world: &mut World, _this: Entity) {}

/// `-[PocketGodAppDelegate loadGame]` @ 0x100089c10
pub fn load_game(world: &mut World, this: Entity) {
    let controller = world.obj::<PocketGodAppDelegate>(this).pocket_god_view_controller;
    if let Some(controller) = controller {
        crate::controller::load_game(world, controller);
    }
}

/// `-[PocketGodAppDelegate onHideLoadingScreen]` @ 0x100089c2c
pub fn on_hide_loading_screen(world: &mut World, this: Entity) {
    world.obj_mut::<PocketGodAppDelegate>(this).startup_complete = true;

    // `[mImageView removeFromSuperview]`, then `release`: the platform hides the launch image.
    let mut requests = world.get_resource_or_insert_with(PlatformRequests::default);
    requests.loading_screen_hidden = true;
    world.obj_mut::<PocketGodAppDelegate>(this).image_view = None;
}

/// `-[PocketGodAppDelegate onModalViewWillAppear]` @ 0x100089c8c
pub fn on_modal_view_will_appear(world: &mut World, this: Entity) {
    world.obj_mut::<PocketGodAppDelegate>(this).modal_view_open = true;
    view::stop_render_loop(world);
}

/// `-[PocketGodAppDelegate onModalViewWillDisappear]` @ 0x100089cd0
pub fn on_modal_view_will_disappear(world: &mut World, this: Entity) {
    world.obj_mut::<PocketGodAppDelegate>(this).modal_view_open = false;
    view::start_render_loop(world);
}

/// `-[PocketGodAppDelegate window]` @ 0x100089d8c
pub fn window(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<PocketGodAppDelegate>(this).window
}

/// `-[PocketGodAppDelegate setWindow:]` @ 0x100089d9c
pub fn set_window(world: &mut World, this: Entity, window: Entity) {
    world.obj_mut::<PocketGodAppDelegate>(this).window = Some(window);
}

/// `-[PocketGodAppDelegate mPocketGodViewController]` @ 0x100089db8
pub fn m_pocket_god_view_controller(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<PocketGodAppDelegate>(this).pocket_god_view_controller
}

/// `-[PocketGodAppDelegate setMPocketGodViewController:]` @ 0x100089dc8
pub fn set_m_pocket_god_view_controller(world: &mut World, this: Entity, m_pocket_god_view_controller: Entity) {
    world.obj_mut::<PocketGodAppDelegate>(this).pocket_god_view_controller = Some(m_pocket_god_view_controller);
}
