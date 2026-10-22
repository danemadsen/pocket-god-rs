//! `PG_SCREENSHOT=<file.png>`: saves the window after its first 60 frames (about a second; set
//! `PG_SCREENSHOT_FRAME=<n>` for another frame), then exits (desktop; there are no environment
//! variables on the web). The run never pauses,
//! and its window stays above the others (`lib.rs`): Bevy draws nothing into a window the OS
//! reports as occluded.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

const SCREENSHOT_ENV: &str = "PG_SCREENSHOT";
const SCREENSHOT_FRAME_ENV: &str = "PG_SCREENSHOT_FRAME";
const DEFAULT_SCREENSHOT_FRAME: u32 = 60;

/// Frames between the capture and the exit, so the saved image is written.
const EXIT_DELAY_FRAMES: u32 = 30;

/// The screenshot path requested through `PG_SCREENSHOT`.
pub fn requested_path() -> Option<String> {
    let path = std::env::var(SCREENSHOT_ENV).ok()?;
    if path.is_empty() { None } else { Some(path) }
}

pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        let Some(path) = requested_path() else {
            return;
        };
        let frame = screenshot_frame();
        app.insert_resource(ScreenshotRequest { path, frame });
        app.add_systems(Update, capture_screenshot);
    }
}

#[derive(Resource)]
struct ScreenshotRequest {
    path: String,
    frame: u32,
}

/// `PG_SCREENSHOT_FRAME` when it parses, else frame 60.
fn screenshot_frame() -> u32 {
    let requested = std::env::var(SCREENSHOT_FRAME_ENV).ok();
    let parsed = requested.and_then(|text| text.trim().parse().ok());
    parsed.unwrap_or(DEFAULT_SCREENSHOT_FRAME)
}

/// Captures the window once the first frames have settled, then exits.
fn capture_screenshot(
    mut commands: Commands,
    request: Res<ScreenshotRequest>,
    mut frames: Local<u32>,
    mut exit: MessageWriter<AppExit>,
) {
    *frames += 1;

    if *frames == request.frame {
        let path = request.path.clone();
        let mut screenshot = commands.spawn(Screenshot::primary_window());
        screenshot.observe(save_to_disk(path));
    }

    if *frames == request.frame + EXIT_DELAY_FRAMES {
        exit.write(AppExit::Success);
    }
}
