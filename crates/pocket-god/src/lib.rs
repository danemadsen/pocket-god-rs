//! Pocket God 1.48.2 on Bevy.
//!
//! [`app`] builds the app every platform runs: the desktop and web executable (`main.rs`)
//! and the Android library (`crates/pocket-god-android`) only call it and run the result.

mod egui_overlay;
mod game;
mod screenshot;
mod scripted_input;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy::window::{WindowLevel, WindowResolution};
use egui_overlay::EguiOverlayPlugin;
use game::GamePlugin;
use pg_assets::EmbeddedAssetsPlugin;
use pg_engine::EnginePlugin;
use pg_platform::{PgPlatformPlugin, PlatformSettings};
use pg_render::PgRenderPlugin;
use screenshot::ScreenshotPlugin;
use scripted_input::ScriptedInputPlugin;

const WINDOW_TITLE: &str = "Pocket God";

/// Initial desktop window size: the original's 480×320-point landscape screen at 2×
/// (iPhone 4). The game is letterboxed into any window size.
const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 640;

/// The `<canvas>` the web build renders into (`web/index.html`).
const WEB_CANVAS_SELECTOR: &str = "#pocket-god";

/// `PG_WINDOW_SIZE=<width>x<height>`: the initial desktop window size in points, e.g. to try
/// the wide (568:320) and tall (4:3) layouts.
const WINDOW_SIZE_ENV: &str = "PG_WINDOW_SIZE";

/// The initial window size: `PG_WINDOW_SIZE` when it parses, else 960 × 640.
fn window_size() -> (u32, u32) {
    let default_size = (WINDOW_WIDTH, WINDOW_HEIGHT);
    let Ok(requested) = std::env::var(WINDOW_SIZE_ENV) else {
        return default_size;
    };
    let Some((width, height)) = requested.split_once('x') else {
        return default_size;
    };
    match (width.trim().parse(), height.trim().parse()) {
        (Ok(width), Ok(height)) if width > 0 && height > 0 => (width, height),
        _ => default_size,
    }
}

/// Builds the Pocket God app with every plugin it needs; the caller only runs it.
pub fn app() -> App {
    let mut app = App::new();

    // The embedded assets replace the default asset source, so they must be registered
    // before `AssetPlugin` (part of `DefaultPlugins`).
    app.add_plugins(EmbeddedAssetsPlugin);
    app.add_plugins(default_plugins());
    app.add_plugins(EnginePlugin);
    app.add_plugins(PgRenderPlugin);
    app.add_plugins(platform_plugin());
    app.add_plugins(EguiOverlayPlugin);
    app.add_plugins(ScreenshotPlugin);
    app.add_plugins(ScriptedInputPlugin);

    app.add_plugins(GamePlugin);

    app
}

/// The platform layer; an unattended screenshot run never pauses for focus or visibility.
fn platform_plugin() -> PgPlatformPlugin {
    let unattended = screenshot::requested_path().is_some();
    let settings = PlatformSettings {
        pause_when_inactive: !unattended,
    };
    PgPlatformPlugin { settings }
}

fn primary_window() -> Window {
    let (width, height) = window_size();
    let resolution = WindowResolution::new(width, height);

    // Bevy renders nothing into a window the OS reports as occluded, so an unattended
    // screenshot run keeps its window above the others (a window opened behind them would be
    // captured black).
    let unattended = screenshot::requested_path().is_some();
    let window_level = if unattended { WindowLevel::AlwaysOnTop } else { WindowLevel::Normal };

    Window {
        title: WINDOW_TITLE.to_string(),
        resolution,
        window_level,
        canvas: Some(WEB_CANVAS_SELECTOR.to_string()),
        fit_canvas_to_parent: true,
        ..default()
    }
}

fn default_plugins() -> PluginGroupBuilder {
    let window_plugin = WindowPlugin {
        primary_window: Some(primary_window()),
        ..default()
    };

    let plugins = DefaultPlugins.build();
    plugins.set(window_plugin)
}
