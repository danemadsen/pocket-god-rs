//! Mouse and touches as `UITouch`es (`docs/re/rendering.md` §9.1, §13).
//!
//! Every finger becomes a [`PlatformTouch`] in view points: origin at the top-left of the game
//! viewport, y down, the viewport `touch_width × touch_height` points large. The left mouse
//! button is one finger (id 0); touch ids are Bevy's plus one. A touch that starts outside the
//! viewport (in the black bars) is ignored for its whole life; one that started inside keeps
//! reporting positions outside it, like UIKit. Like UIKit's coalescing, a finger moves at most
//! once per frame (the latest position). `tapCount` counts quick repeated taps at the same
//! place, and touches are cancelled when the app resigns active.
//!
//! Bevy's events carry no time, so the events of one frame get distinct timestamps spread
//! evenly, in arrival order, over the time since the previous frame ([`event_time`]): a press
//! and its first move, or the last move and the release, never share a timestamp, which the
//! original's swipe speeds (`dx / (t - t0)`) divide by.

use bevy::input::ButtonState;
use bevy::input::mouse::MouseButton;
use bevy::input::touch::TouchPhase as BevyTouchPhase;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy::window::WindowEvent;
use pg_engine::touch::{PlatformTouch, PlatformTouches, TouchPhase};

use crate::lifecycle::PlatformLifecycle;
use crate::screen::GameScreen;

/// The finger id of the left mouse button.
pub const MOUSE_TOUCH_ID: u64 = 0;

/// Added to Bevy's touch ids so they never collide with the mouse.
const TOUCH_ID_OFFSET: u64 = 1;

/// The longest gap between a tap's release and the next touch for UIKit to count them as one
/// multi-tap (seconds).
pub const MULTI_TAP_INTERVAL: f64 = 0.35;

/// How far (view points) a tap may wander and the next one may land from it and still count.
pub const TAP_SLOP: f32 = 10.0;

/// A finger that is down.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ActiveTouch {
    tap_count: i32,
    start: Vec2,
    last: Vec2,
    moved_away: bool,
}

/// The last finger lifted as a tap.
#[derive(Clone, Copy, Debug, PartialEq)]
struct TapRecord {
    time: f64,
    position: Vec2,
    tap_count: i32,
}

impl TapRecord {
    /// Whether a touch down at `point` at `time` is the next tap of this series.
    fn continues_with(&self, point: Vec2, time: f64) -> bool {
        let soon = time - self.time <= MULTI_TAP_INTERVAL;
        let near = point.distance(self.position) <= TAP_SLOP;
        soon && near
    }
}

/// Turns platform finger events into `UITouch` phases, positions and tap counts.
#[derive(Clone, Debug, Default)]
pub struct TouchTracker {
    active: HashMap<u64, ActiveTouch>,
    ignored: HashSet<u64>,
    last_tap: Option<TapRecord>,
}

fn platform_touch(id: u64, phase: TouchPhase, time: f64, tap_count: i32, point: Vec2) -> PlatformTouch {
    PlatformTouch {
        id,
        phase,
        time_stamp: time,
        tap_count,
        x: point.x,
        y: point.y,
    }
}

impl TouchTracker {
    /// Whether the finger is down (and was not ignored).
    pub fn is_active(&self, id: u64) -> bool {
        self.active.contains_key(&id)
    }

    /// A finger touched down at `point` (view points); `inside` = on the game viewport. A
    /// finger that is still down under the same id (its release was lost) is cancelled first.
    pub fn began(&mut self, id: u64, point: Vec2, time: f64, inside: bool) -> Vec<PlatformTouch> {
        let mut touches = Vec::new();
        if let Some(stale) = self.cancelled(id, None, time) {
            touches.push(stale);
        }
        if let Some(touch) = self.press(id, point, time, inside) {
            touches.push(touch);
        }
        touches
    }

    fn press(&mut self, id: u64, point: Vec2, time: f64, inside: bool) -> Option<PlatformTouch> {
        if !inside {
            self.ignored.insert(id);
            return None;
        }
        self.ignored.remove(&id);

        let tap_count = match self.last_tap {
            Some(tap) if tap.continues_with(point, time) => tap.tap_count + 1,
            _ => 1,
        };

        let touch = ActiveTouch {
            tap_count,
            start: point,
            last: point,
            moved_away: false,
        };
        self.active.insert(id, touch);
        Some(platform_touch(id, TouchPhase::Began, time, tap_count, point))
    }

    /// A finger that is down moved to `point`.
    pub fn moved(&mut self, id: u64, point: Vec2, time: f64) -> Option<PlatformTouch> {
        let touch = self.active.get_mut(&id)?;
        touch.last = point;
        if point.distance(touch.start) > TAP_SLOP {
            touch.moved_away = true;
        }
        Some(platform_touch(id, TouchPhase::Moved, time, touch.tap_count, point))
    }

    /// A finger lifted at `point`.
    pub fn ended(&mut self, id: u64, point: Vec2, time: f64) -> Option<PlatformTouch> {
        if self.ignored.remove(&id) {
            return None;
        }
        let touch = self.active.remove(&id)?;

        let moved_away = touch.moved_away || point.distance(touch.start) > TAP_SLOP;
        self.last_tap = if moved_away {
            None
        } else {
            let tap = TapRecord {
                time,
                position: point,
                tap_count: touch.tap_count,
            };
            Some(tap)
        };
        Some(platform_touch(id, TouchPhase::Ended, time, touch.tap_count, point))
    }

    /// The system cancelled a finger (`point` = its last position when unknown).
    pub fn cancelled(&mut self, id: u64, point: Option<Vec2>, time: f64) -> Option<PlatformTouch> {
        if self.ignored.remove(&id) {
            return None;
        }
        let touch = self.active.remove(&id)?;
        self.last_tap = None;

        let position = point.unwrap_or(touch.last);
        Some(platform_touch(id, TouchPhase::Cancelled, time, touch.tap_count, position))
    }

    /// Cancels every finger (the app resigned active), in id order.
    pub fn cancel_all(&mut self, time: f64) -> Vec<PlatformTouch> {
        let mut ids: Vec<u64> = self.active.keys().copied().collect();
        ids.sort_unstable();
        self.ignored.clear();

        let mut cancelled = Vec::new();
        for id in ids {
            if let Some(touch) = self.cancelled(id, None, time) {
                cancelled.push(touch);
            }
        }
        cancelled
    }
}

/// The tracker plus the window state the events need.
#[derive(Resource, Default)]
pub struct TouchInputState {
    tracker: TouchTracker,

    /// The cursor in window logical pixels (the mouse finger's position).
    cursor: Option<Vec2>,

    /// The real time of the previous frame (seconds since start-up).
    previous_frame_time: f64,
}

/// The timestamp of the `index`-th of the `count` events that arrived between the previous
/// frame (at `previous`) and this one (at `now`): spread evenly in arrival order, the last one
/// at `now`.
pub fn event_time(previous: f64, now: f64, index: usize, count: usize) -> f64 {
    let last = index + 1 >= count;
    if last || now <= previous {
        return now;
    }
    let spacing = (now - previous) / count as f64;
    previous + spacing * (index + 1) as f64
}

/// The window events that move or press a finger.
fn is_finger_event(event: &WindowEvent) -> bool {
    match event {
        WindowEvent::CursorMoved(_) | WindowEvent::TouchInput(_) => true,
        WindowEvent::MouseButtonInput(input) => input.button == MouseButton::Left,
        _ => false,
    }
}

/// The touches of one frame, with each finger's moves coalesced into its latest.
struct FrameTouches<'a> {
    touches: &'a mut Vec<PlatformTouch>,

    /// Index in `touches` of each finger's move this frame.
    pending_moves: HashMap<u64, usize>,
}

impl FrameTouches<'_> {
    fn push(&mut self, touch: PlatformTouch) {
        if touch.phase == TouchPhase::Moved {
            if let Some(index) = self.pending_moves.get(&touch.id) {
                self.touches[*index] = touch;
                return;
            }
            self.pending_moves.insert(touch.id, self.touches.len());
        } else {
            self.pending_moves.remove(&touch.id);
        }
        self.touches.push(touch);
    }

    fn extend(&mut self, touches: Vec<PlatformTouch>) {
        for touch in touches {
            self.push(touch);
        }
    }
}

fn finger_id(touch_id: u64) -> u64 {
    touch_id.saturating_add(TOUCH_ID_OFFSET)
}

/// Window events and lifecycle changes of this frame into [`PlatformTouches`].
pub fn collect_touches(
    mut events: MessageReader<WindowEvent>,
    mut lifecycle: MessageReader<PlatformLifecycle>,
    screen: Res<GameScreen>,
    time: Res<Time<Real>>,
    mut state: ResMut<TouchInputState>,
    mut platform_touches: ResMut<PlatformTouches>,
) {
    let now = time.elapsed_secs_f64();
    let state = &mut *state;
    let previous = std::mem::replace(&mut state.previous_frame_time, now);
    let mut frame = FrameTouches {
        touches: &mut platform_touches.0,
        pending_moves: HashMap::default(),
    };

    let Some(viewport) = screen.viewport else {
        events.clear();
        lifecycle.clear();
        return;
    };

    let finger_events: Vec<&WindowEvent> = events.read().filter(|event| is_finger_event(event)).collect();
    let count = finger_events.len();
    for (index, event) in finger_events.into_iter().enumerate() {
        let time = event_time(previous, now, index, count);
        match event {
            WindowEvent::CursorMoved(moved) => {
                state.cursor = Some(moved.position);
                let point = viewport.view_point(moved.position);
                if let Some(touch) = state.tracker.moved(MOUSE_TOUCH_ID, point, time) {
                    frame.push(touch);
                }
            }
            WindowEvent::MouseButtonInput(input) => {
                let Some(cursor) = state.cursor else {
                    continue;
                };
                let point = viewport.view_point(cursor);
                let touches = match input.state {
                    ButtonState::Pressed => {
                        let inside = viewport.contains_view_point(point);
                        state.tracker.began(MOUSE_TOUCH_ID, point, time, inside)
                    }
                    ButtonState::Released => {
                        let ended = state.tracker.ended(MOUSE_TOUCH_ID, point, time);
                        ended.into_iter().collect()
                    }
                };
                frame.extend(touches);
            }
            WindowEvent::TouchInput(input) => {
                let id = finger_id(input.id);
                let point = viewport.view_point(input.position);
                let touches = match input.phase {
                    BevyTouchPhase::Started => {
                        let inside = viewport.contains_view_point(point);
                        state.tracker.began(id, point, time, inside)
                    }
                    BevyTouchPhase::Moved => {
                        let moved = state.tracker.moved(id, point, time);
                        moved.into_iter().collect()
                    }
                    BevyTouchPhase::Ended => {
                        let ended = state.tracker.ended(id, point, time);
                        ended.into_iter().collect()
                    }
                    BevyTouchPhase::Canceled => {
                        let cancelled = state.tracker.cancelled(id, Some(point), time);
                        cancelled.into_iter().collect()
                    }
                };
                frame.extend(touches);
            }
            _ => {}
        }
    }

    for transition in lifecycle.read() {
        let resigns = matches!(transition, PlatformLifecycle::WillResignActive | PlatformLifecycle::DidEnterBackground);
        if !resigns {
            continue;
        }
        let cancelled = state.tracker.cancel_all(now);
        frame.extend(cancelled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(tracker: &mut TouchTracker, id: u64, point: Vec2, time: f64) -> PlatformTouch {
        let touches = tracker.began(id, point, time, true);
        assert_eq!(touches.len(), 1, "{touches:?}");
        touches[0]
    }

    fn tap(tracker: &mut TouchTracker, id: u64, point: Vec2, time: f64) -> (PlatformTouch, PlatformTouch) {
        let began = press(tracker, id, point, time);
        let ended = tracker.ended(id, point, time + 0.05).expect("the finger is down");
        (began, ended)
    }

    #[test]
    fn quick_taps_at_the_same_place_count_up() {
        let mut tracker = TouchTracker::default();
        let point = Vec2::new(100.0, 80.0);
        let (first, first_end) = tap(&mut tracker, 0, point, 1.0);
        let (second, second_end) = tap(&mut tracker, 0, point + Vec2::new(4.0, -3.0), 1.3);
        let (third, _) = tap(&mut tracker, 0, point, 1.6);

        assert_eq!(first.tap_count, 1);
        assert_eq!(first_end.tap_count, 1);
        assert_eq!(second.tap_count, 2);
        assert_eq!(second_end.tap_count, 2);
        assert_eq!(third.tap_count, 3);
        assert_eq!(first.phase, TouchPhase::Began);
        assert_eq!(first_end.phase, TouchPhase::Ended);
    }

    #[test]
    fn slow_far_or_dragged_taps_start_over() {
        let mut tracker = TouchTracker::default();
        let point = Vec2::new(100.0, 80.0);
        tap(&mut tracker, 0, point, 1.0);
        let (late, _) = tap(&mut tracker, 0, point, 1.5);
        assert_eq!(late.tap_count, 1);

        let (far, _) = tap(&mut tracker, 0, point + Vec2::new(30.0, 0.0), 1.6);
        assert_eq!(far.tap_count, 1);

        press(&mut tracker, 0, point, 2.0);
        tracker.moved(0, point + Vec2::new(50.0, 0.0), 2.05);
        tracker.ended(0, point, 2.1);
        let (after_drag, _) = tap(&mut tracker, 0, point, 2.2);
        assert_eq!(after_drag.tap_count, 1);
    }

    #[test]
    fn touches_starting_in_the_bars_are_ignored_for_their_whole_life() {
        let mut tracker = TouchTracker::default();
        let outside = Vec2::new(-20.0, 50.0);
        assert!(tracker.began(5, outside, 1.0, false).is_empty());
        assert_eq!(tracker.moved(5, Vec2::new(40.0, 50.0), 1.1), None);
        assert_eq!(tracker.ended(5, Vec2::new(40.0, 50.0), 1.2), None);
        assert!(!tracker.is_active(5));
    }

    #[test]
    fn moves_may_leave_the_view() {
        let mut tracker = TouchTracker::default();
        press(&mut tracker, 2, Vec2::new(10.0, 10.0), 1.0);
        let moved = tracker.moved(2, Vec2::new(-35.0, 400.0), 1.1).expect("still down");
        assert_eq!((moved.x, moved.y), (-35.0, 400.0));
        assert_eq!(moved.time_stamp, 1.1);
        assert_eq!(moved.id, 2);
    }

    #[test]
    fn resigning_active_cancels_every_finger_at_its_last_position() {
        let mut tracker = TouchTracker::default();
        press(&mut tracker, 3, Vec2::new(10.0, 10.0), 1.0);
        press(&mut tracker, 1, Vec2::new(50.0, 60.0), 1.0);
        tracker.moved(1, Vec2::new(55.0, 66.0), 1.1);

        let cancelled = tracker.cancel_all(2.0);
        let ids: Vec<u64> = cancelled.iter().map(|touch| touch.id).collect();
        assert_eq!(ids, vec![1, 3]);
        assert!(cancelled.iter().all(|touch| touch.phase == TouchPhase::Cancelled));
        assert_eq!((cancelled[0].x, cancelled[0].y), (55.0, 66.0));
        assert!(!tracker.is_active(1) && !tracker.is_active(3));
        assert_eq!(tracker.ended(1, Vec2::ZERO, 2.1), None);
    }

    #[test]
    fn a_finger_pressed_again_without_a_release_is_cancelled_first() {
        let mut tracker = TouchTracker::default();
        press(&mut tracker, MOUSE_TOUCH_ID, Vec2::new(10.0, 10.0), 1.0);
        let touches = tracker.began(MOUSE_TOUCH_ID, Vec2::new(30.0, 30.0), 2.0, true);
        let phases: Vec<TouchPhase> = touches.iter().map(|touch| touch.phase).collect();
        assert_eq!(phases, vec![TouchPhase::Cancelled, TouchPhase::Began]);
        assert_eq!((touches[0].x, touches[0].y), (10.0, 10.0));
        assert_eq!(touches[1].tap_count, 1);
    }

    #[test]
    fn a_frame_keeps_only_the_latest_move_of_each_finger() {
        let mut touches = Vec::new();
        let mut frame = FrameTouches {
            touches: &mut touches,
            pending_moves: HashMap::default(),
        };
        let began = platform_touch(1, TouchPhase::Began, 1.0, 1, Vec2::ZERO);
        let move_a = platform_touch(1, TouchPhase::Moved, 1.0, 1, Vec2::new(1.0, 0.0));
        let other = platform_touch(2, TouchPhase::Moved, 1.0, 1, Vec2::new(9.0, 9.0));
        let move_b = platform_touch(1, TouchPhase::Moved, 1.0, 1, Vec2::new(2.0, 0.0));
        let ended = platform_touch(1, TouchPhase::Ended, 1.0, 1, Vec2::new(2.0, 0.0));
        let move_c = platform_touch(1, TouchPhase::Moved, 1.0, 1, Vec2::new(3.0, 0.0));
        for touch in [began, move_a, other, move_b, ended, move_c] {
            frame.push(touch);
        }

        assert_eq!(touches, vec![began, move_b, other, ended, move_c]);
    }

    #[test]
    fn events_of_one_frame_get_increasing_timestamps() {
        let times: Vec<f64> = (0..4).map(|index| event_time(1.0, 1.02, index, 4)).collect();
        let expected = [1.005, 1.01, 1.015, 1.02];
        for (time, expected) in times.iter().zip(expected) {
            assert!((time - expected).abs() < 1e-12, "{times:?}");
        }
        assert_eq!(times[3], 1.02);

        // The next frame's events all come after this frame's.
        let next = event_time(1.02, 1.04, 0, 2);
        assert!(next > times[3]);

        assert_eq!(event_time(1.0, 1.02, 0, 1), 1.02);
        assert_eq!(event_time(2.0, 2.0, 0, 3), 2.0);
    }

    #[test]
    fn a_press_and_its_first_move_in_one_frame_keep_their_order_in_time() {
        let mut tracker = TouchTracker::default();
        let began = press(&mut tracker, 1, Vec2::new(10.0, 10.0), event_time(1.0, 1.016, 0, 3));
        let moved = tracker.moved(1, Vec2::new(30.0, 10.0), event_time(1.0, 1.016, 1, 3));
        let ended = tracker.ended(1, Vec2::new(50.0, 10.0), event_time(1.0, 1.016, 2, 3));
        let moved = moved.expect("the finger is down");
        let ended = ended.expect("the finger is down");
        assert!(began.time_stamp < moved.time_stamp && moved.time_stamp < ended.time_stamp);
        assert_eq!(ended.time_stamp, 1.016);
    }

    #[test]
    fn touch_ids_never_collide_with_the_mouse() {
        assert_eq!(finger_id(0), 1);
        assert_ne!(finger_id(0), MOUSE_TOUCH_ID);
        assert_eq!(finger_id(u64::MAX), u64::MAX);
    }
}
