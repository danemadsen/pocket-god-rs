//! The platform layer of the Pocket God port: everything that differs between desktop, the web
//! and Android lives here, behind `cfg`s, as Bevy systems around the engine
//! (`docs/ARCHITECTURE.md`, `docs/re/managers-platform.md` §18).
//!
//! - [`frame`]: the display link. [`PlatformSystems::FrameTick`] runs `BCView::gameLoop` at a
//!   fixed 60 Hz while the engine's render loop is active and the game is on screen.
//! - [`lifecycle`]: window focus, hidden windows and app suspension as `UIApplicationDelegate`
//!   callbacks ([`PlatformLifecycle`]), with `PocketGodAppDelegate`'s render-loop rows; only
//!   an OS suspension is the background.
//! - [`screen`]: the game viewport (the window aspect clamped to 4:3 … 568:320, letterboxed)
//!   for the engine (`configure_screen`) and the renderer ([`pg_render::LogicalScreen`]).
//! - [`input`]: mouse and touches as `UITouch`es ([`pg_engine::touch::PlatformTouches`]).
//! - [`motion`]: the accelerometer in the iOS convention ([`pg_engine::view::MotionInput`]).
//! - [`audio`]: the engine's sound commands played with `bevy_audio`, silent while the game is
//!   off screen.
//! - [`storage`]: the save files behind [`pg_engine::prefs::PreferencesHooks`].
//! - [`services`]: URLs, vibration, the locale and the random seed.
//! - [`splash`]: the launch image until the game hides its loading screen, and
//!   [`splash::launch_image_shown`], the run condition the game's `loadGame` waits on.
//!
//! Add it with `pg_engine::EnginePlugin` and `pg_render::PgRenderPlugin`.

pub mod audio;
pub mod frame;
pub mod input;
pub mod lifecycle;
pub mod motion;
pub mod screen;
pub mod services;
pub mod splash;
pub mod storage;

mod base64;
mod wav;

#[cfg(target_os = "android")]
mod android;

use bevy::prelude::*;

pub use frame::{FrameClock, FrameDriver};
pub use lifecycle::PlatformLifecycle;
pub use screen::GameViewport;

/// The platform's systems in `Update`, in this order. The game orders its own per-frame
/// systems around [`PlatformSystems::FrameTick`].
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatformSystems {
    /// Lifecycle, screen, touch and motion input for the next tick.
    Input,

    /// The frame driver: `BCView::gameLoop` ticks (exclusive).
    FrameTick,

    /// Sound commands, platform requests and the splash, after the ticks.
    Output,
}

/// Behaviour switches of the platform layer.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformSettings {
    /// Pause like the original when the window loses focus or is hidden (and fall silent while
    /// it is hidden). Unattended runs (screenshots) turn it off; an OS suspension (Android
    /// background) always pauses.
    pub pause_when_inactive: bool,
}

impl Default for PlatformSettings {
    fn default() -> Self {
        Self {
            pause_when_inactive: true,
        }
    }
}

#[derive(Default)]
pub struct PgPlatformPlugin {
    pub settings: PlatformSettings,
}

impl Plugin for PgPlatformPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.settings);
        app.add_message::<PlatformLifecycle>();

        // Save files must be in place before the game loads its preference documents.
        storage::install(app);

        app.init_resource::<lifecycle::LifecycleTracker>();
        app.init_resource::<frame::FrameDriver>();
        app.init_resource::<screen::GameScreen>();
        app.init_resource::<input::TouchInputState>();
        app.init_resource::<audio::SoundBank>();
        app.init_resource::<audio::Voices>();
        app.init_resource::<splash::SplashState>();
        motion::build(app);

        let sets = (PlatformSystems::Input, PlatformSystems::FrameTick, PlatformSystems::Output).chain();
        app.configure_sets(Update, sets);

        // The screen is configured before anything loads data (`loadGame` reads the wide and
        // tall flags), so this runs in `PreStartup`.
        let startup = (
            screen::configure_screen_at_startup,
            services::start_services,
            splash::show_splash,
        )
            .chain();
        app.add_systems(PreStartup, startup);

        let input = (
            lifecycle::track_lifecycle,
            lifecycle::apply_lifecycle,
            screen::update_screen,
            input::collect_touches,
            motion::update_motion,
        )
            .chain();
        app.add_systems(Update, input.in_set(PlatformSystems::Input));

        app.add_systems(Update, frame::run_frame_ticks.in_set(PlatformSystems::FrameTick));

        let output = (
            audio::play_sound_commands,
            services::handle_platform_requests,
            splash::update_splash,
        )
            .chain();
        app.add_systems(Update, output.in_set(PlatformSystems::Output));
    }
}
