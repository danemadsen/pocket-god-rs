//! `PG_TOUCHES="<script>"`: scripted touches for unattended runs (with `PG_SCREENSHOT`), frame
//! by frame, in the original's screen points (480 × 320 on a 3:2 window, 568 × 320 wide,
//! 480 × 360 tall), whatever the window size:
//!
//! - `tap <frame> <x> <y>`: a finger down at `frame` and up three frames later;
//! - `drag <frame> <x1> <y1> <x2> <y2> <frames>`: down at (x1, y1), moving in a straight line
//!   for `frames` frames, up at (x2, y2).
//!
//! Commands are separated by `;`. Desktop only (there are no environment variables on the web).

use bevy::prelude::*;
use pg_engine::touch::{PlatformTouch, PlatformTouches, TouchPhase};
use pg_engine::view::BcView;
use pg_platform::PlatformSystems;

const TOUCHES_ENV: &str = "PG_TOUCHES";

/// Scripted fingers get ids far above the platform's (mouse 0, then touch ids).
const FIRST_SCRIPTED_ID: u64 = 1_000_000;

/// A tap lifts its finger this many frames after it went down.
const TAP_FRAMES: u32 = 3;

/// Events of one frame get timestamps this far apart, in seconds.
const SAME_FRAME_SPACING: f64 = 0.0001;

pub struct ScriptedInputPlugin;

impl Plugin for ScriptedInputPlugin {
    fn build(&self, app: &mut App) {
        let Ok(script) = std::env::var(TOUCHES_ENV) else {
            return;
        };
        let events = parse_script(&script);
        app.insert_resource(ScriptedTouches { events, frame: 0 });

        let player = play_scripted_touches.after(PlatformSystems::Input);
        app.add_systems(Update, player.before(PlatformSystems::FrameTick));
    }
}

/// One touch event of the script, in the original's screen points.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ScriptedEvent {
    frame: u32,
    id: u64,
    phase: TouchPhase,
    x: f32,
    y: f32,
}

#[derive(Resource)]
struct ScriptedTouches {
    events: Vec<ScriptedEvent>,
    frame: u32,
}

fn parse_script(script: &str) -> Vec<ScriptedEvent> {
    let mut events = Vec::new();
    let commands = script.split(';').map(str::trim).filter(|command| !command.is_empty());
    for (index, command) in commands.enumerate() {
        let id = FIRST_SCRIPTED_ID + index as u64;
        let words: Vec<&str> = command.split_whitespace().collect();
        let numbers: Vec<f32> = words[1..].iter().filter_map(|word| word.parse().ok()).collect();
        match (words[0], numbers.as_slice()) {
            ("tap", [frame, x, y]) => push_tap(&mut events, id, *frame as u32, *x, *y),
            ("drag", [frame, x1, y1, x2, y2, frames]) => {
                let start = Vec2::new(*x1, *y1);
                let end = Vec2::new(*x2, *y2);
                push_drag(&mut events, id, *frame as u32, start, end, *frames as u32);
            }
            _ => warn!("{TOUCHES_ENV}: cannot read `{command}`"),
        }
    }
    events.sort_by_key(|event| event.frame);
    events
}

fn push_tap(events: &mut Vec<ScriptedEvent>, id: u64, frame: u32, x: f32, y: f32) {
    let down = ScriptedEvent { frame, id, phase: TouchPhase::Began, x, y };
    let up = ScriptedEvent { frame: frame + TAP_FRAMES, phase: TouchPhase::Ended, ..down };
    events.push(down);
    events.push(up);
}

fn push_drag(events: &mut Vec<ScriptedEvent>, id: u64, frame: u32, start: Vec2, end: Vec2, frames: u32) {
    let frames = frames.max(1);
    let down = ScriptedEvent { frame, id, phase: TouchPhase::Began, x: start.x, y: start.y };
    events.push(down);
    for step in 1..=frames {
        let point = start.lerp(end, step as f32 / frames as f32);
        let phase = if step == frames { TouchPhase::Ended } else { TouchPhase::Moved };
        events.push(ScriptedEvent { frame: frame + step, id, phase, x: point.x, y: point.y });
    }
}

/// Hands this frame's scripted events to the engine as platform touches in view points.
fn play_scripted_touches(
    mut scripted: ResMut<ScriptedTouches>,
    mut touches: ResMut<PlatformTouches>,
    view: Res<BcView>,
    time: Res<Time<Real>>,
) {
    scripted.frame += 1;
    let frame = scripted.frame;
    let points_per_unit_x = view.touch_width / view.screen_width;
    let points_per_unit_y = view.touch_height / view.screen_height;
    let now = time.elapsed_secs_f64();

    let due = scripted.events.iter().filter(|event| event.frame == frame);
    for (order, event) in due.enumerate() {
        let time_stamp = now + order as f64 * SAME_FRAME_SPACING;
        let touch = PlatformTouch {
            id: event.id,
            phase: event.phase,
            time_stamp,
            tap_count: 1,
            x: event.x * points_per_unit_x,
            y: event.y * points_per_unit_y,
        };
        touches.0.push(touch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tap_is_a_press_and_a_release() {
        let events = parse_script("tap 10 115 147");
        let phases: Vec<(u32, TouchPhase)> = events.iter().map(|event| (event.frame, event.phase)).collect();
        assert_eq!(phases, vec![(10, TouchPhase::Began), (13, TouchPhase::Ended)]);
    }

    #[test]
    fn a_drag_moves_in_a_line_and_ends_at_its_target() {
        let events = parse_script("drag 5 0 0 30 60 3; tap 1 1 1");
        assert_eq!(events.len(), 6);
        assert_eq!(events[0].frame, 1);
        let last = events.last().unwrap();
        assert_eq!((last.frame, last.phase, last.x, last.y), (8, TouchPhase::Ended, 30.0, 60.0));
        let first_move = events.iter().find(|event| event.phase == TouchPhase::Moved).unwrap();
        assert_eq!((first_move.x, first_move.y), (10.0, 20.0));
    }
}
