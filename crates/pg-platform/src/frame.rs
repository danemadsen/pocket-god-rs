//! The display link: `BCView::gameLoop` at a fixed 60 Hz (`docs/re/rendering.md` §2.1–§2.2,
//! §13).
//!
//! The original ran one update per `CADisplayLink` tick (`mDisplayLinkCount` 1, i.e. 60 Hz;
//! 2 on slow devices) and much of the game is per-frame, so the port ticks on a fixed grid:
//! real time accumulates and every `display_link_count / 60` s one
//! [`pg_engine::view::game_loop`] runs with that dt. A Bevy frame runs at most
//! [`MAX_TICKS_PER_FRAME`] ticks; after a longer stall the backlog is dropped.
//!
//! Ticks run only while the engine's render loop is active (`BCView::startRenderLoop` /
//! `stopRenderLoop`) and the game is on screen (no display link fires while the original is in
//! the background). The first tick after `startRenderLoop` or after a pause gets dt = 1/15:
//! `startRenderLoop` sets `mLastTime = -1/15`, and after a gap the original's clamp yields the
//! same.
//!
//! When the screen changes in a frame without a tick (a window resized while the game is
//! paused, or between two ticks on a fast display), the unchanged game is drawn again for the
//! new screen ([`pg_engine::view::draw`]), so the renderer never stretches a draw list built for
//! the old one.

use bevy::prelude::*;
use pg_engine::view::{self, BcView};

use crate::screen::GameScreen;

/// Display-link ticks per second at `mDisplayLinkCount` 1.
pub const TICKS_PER_SECOND: u32 = 60;

/// The most ticks one Bevy frame runs; after a longer stall the remaining backlog is dropped.
pub const MAX_TICKS_PER_FRAME: u32 = 4;

/// How far short of a full period the accumulated time may be and still tick: absorbs the
/// frame-time jitter of a 60 Hz display, so each of its frames ticks exactly once.
pub const TICK_TOLERANCE: f64 = 0.002;

/// The dt clamp of `BCView::gameLoop` (`0x3d888889`, 1/15 s).
pub const MAX_DT: f32 = f32::from_bits(0x3d88_8889);

/// `mLastTime` after `BCView::startRenderLoop` (`0xbd888889`, -1/15 s).
const RESTART_LAST_TIME: f32 = f32::from_bits(0xbd88_8889);

/// Real time not yet consumed by ticks.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameClock {
    accumulator: f64,
}

impl FrameClock {
    /// Adds `elapsed` seconds of real time and returns how many ticks of `period` seconds are
    /// due, at most [`MAX_TICKS_PER_FRAME`]; whatever is still due after that is dropped.
    pub fn advance(&mut self, elapsed: f64, period: f64) -> u32 {
        self.accumulator += elapsed.max(0.0);

        let mut ticks = 0;
        while self.accumulator + TICK_TOLERANCE >= period {
            if ticks == MAX_TICKS_PER_FRAME {
                self.accumulator = 0.0;
                break;
            }
            self.accumulator -= period;
            ticks += 1;
        }
        ticks
    }

    /// Forgets the time accumulated so far.
    pub fn reset(&mut self) {
        self.accumulator = 0.0;
    }
}

/// The dt of the first tick after `startRenderLoop`, `elapsed` seconds after it: the
/// original's `t - mLastTime` with `mLastTime = -1/15`, clamped to 1/15 (so always 1/15).
pub fn restart_tick_dt(elapsed: f32) -> f32 {
    let since_last_time = elapsed.max(0.0) - RESTART_LAST_TIME;
    since_last_time.min(MAX_DT)
}

/// The tick period and dt for `mDisplayLinkCount` (the display link's frame interval).
pub fn tick_interval(display_link_count: i32) -> (f64, f32) {
    let frames = display_link_count.max(1);
    let period = f64::from(frames) / f64::from(TICKS_PER_SECOND);
    let dt = frames as f32 / TICKS_PER_SECOND as f32;
    (period, dt)
}

/// The display link's state.
#[derive(Resource, Default)]
pub struct FrameDriver {
    clock: FrameClock,

    /// The game is off screen (suspended or hidden): like the original's suspended process,
    /// nothing ticks.
    paused: bool,

    /// The next tick is the first after a pause.
    resumed: bool,
}

impl FrameDriver {
    /// Whether the display link is paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Pauses the display link while the game is off screen; the first tick after the pause
    /// is a restart's.
    pub fn set_paused(&mut self, paused: bool) {
        if self.paused && !paused {
            self.resumed = true;
        }
        self.paused = paused;
    }
}

/// Runs the engine ticks that are due (`PlatformSystems::FrameTick`), or draws the game again
/// when none is due in a frame in which the screen changed.
pub fn run_frame_ticks(world: &mut World) {
    let ticked = run_due_ticks(world);

    let screen_changed = world.resource_mut::<GameScreen>().take_redraw();
    if screen_changed && !ticked {
        view::draw(world);
    }
}

/// Runs the ticks that are due; whether any ran.
fn run_due_ticks(world: &mut World) -> bool {
    let elapsed = world.resource::<Time<Real>>().delta_secs_f64();
    let (render_loop_active, restarted, display_link_count) = {
        let view = world.resource::<BcView>();
        (view.render_loop_active, view.render_loop_restarted, view.display_link_count)
    };

    let mut driver = world.resource_mut::<FrameDriver>();
    if driver.paused || !render_loop_active {
        driver.clock.reset();
        return false;
    }

    if restarted || driver.resumed {
        driver.resumed = false;
        driver.clock.reset();
        world.resource_mut::<BcView>().render_loop_restarted = false;

        let dt = restart_tick_dt(elapsed as f32);
        view::game_loop(world, dt);
        return true;
    }

    let (period, dt) = tick_interval(display_link_count);
    let ticks = driver.clock.advance(elapsed, period);
    for _ in 0..ticks {
        view::game_loop(world, dt);
    }
    ticks > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    const PERIOD: f64 = 1.0 / 60.0;

    fn run_frames(clock: &mut FrameClock, frame_time: f64, frames: usize) -> u32 {
        let mut ticks = 0;
        for _ in 0..frames {
            ticks += clock.advance(frame_time, PERIOD);
        }
        ticks
    }

    #[test]
    fn a_60_hz_display_ticks_once_per_frame_despite_jitter() {
        let mut clock = FrameClock::default();
        let jittered = [0.0160, 0.0172, 0.0165, 0.0168, 0.0161, 0.0174];
        for frame_time in jittered.iter().cycle().take(600) {
            assert_eq!(clock.advance(*frame_time, PERIOD), 1);
        }
    }

    #[test]
    fn faster_and_slower_displays_keep_60_ticks_per_second() {
        let mut fast = FrameClock::default();
        assert_eq!(run_frames(&mut fast, 1.0 / 120.0, 1200), 600);

        let mut odd = FrameClock::default();
        let ticks = run_frames(&mut odd, 1.0 / 144.0, 1440);
        assert!((599..=601).contains(&ticks), "{ticks}");

        let mut slow = FrameClock::default();
        assert_eq!(run_frames(&mut slow, 1.0 / 30.0, 300), 600);
    }

    #[test]
    fn stalls_run_four_ticks_and_drop_the_rest() {
        let mut clock = FrameClock::default();
        assert_eq!(clock.advance(1.0, PERIOD), MAX_TICKS_PER_FRAME);
        assert_eq!(clock.advance(1.0 / 60.0, PERIOD), 1);

        let mut exact = FrameClock::default();
        assert_eq!(exact.advance(4.0 / 60.0, PERIOD), 4);
        assert_eq!(exact.advance(0.0, PERIOD), 0);
    }

    #[test]
    fn reset_forgets_accumulated_time() {
        let mut clock = FrameClock::default();
        assert_eq!(clock.advance(0.012, PERIOD), 0);
        clock.reset();
        assert_eq!(clock.advance(0.012, PERIOD), 0);
        assert_eq!(clock.advance(-1.0, PERIOD), 0);
    }

    #[test]
    fn the_first_tick_after_a_restart_is_one_fifteenth() {
        assert_eq!(MAX_DT.to_bits(), 0x3d88_8889);
        assert_eq!(restart_tick_dt(0.0), MAX_DT);
        assert_eq!(restart_tick_dt(0.004), MAX_DT);
        assert_eq!(restart_tick_dt(30.0), MAX_DT);
    }

    #[test]
    fn only_the_end_of_a_pause_restarts_the_clock() {
        let mut driver = FrameDriver::default();
        driver.set_paused(false);
        assert!(!driver.resumed);

        driver.set_paused(true);
        driver.set_paused(true);
        assert!(driver.is_paused() && !driver.resumed);

        driver.set_paused(false);
        assert!(!driver.is_paused() && driver.resumed);
    }

    fn drawn_clear_color(world: &World) -> [f32; 4] {
        let view = world.resource::<BcView>();
        let gl = view.gl.as_ref().expect("the GL context is idle");
        gl.list.clear_color
    }

    #[test]
    fn a_screen_change_without_a_tick_draws_the_game_again() {
        let mut world = World::new();
        world.insert_resource(Time::<Real>::default());
        world.init_resource::<FrameDriver>();
        world.init_resource::<GameScreen>();
        let paused_game = BcView {
            clear_color: [0.1, 0.2, 0.3, 1.0],
            ..default()
        };
        world.insert_resource(paused_game);

        // The render loop is stopped: nothing ticks and, without a screen change, nothing draws.
        run_frame_ticks(&mut world);
        assert_eq!(drawn_clear_color(&world), [0.0; 4]);

        world.resource_mut::<GameScreen>().request_redraw();
        run_frame_ticks(&mut world);
        assert_eq!(drawn_clear_color(&world), [0.1, 0.2, 0.3, 1.0]);
        assert!(!world.resource_mut::<GameScreen>().take_redraw());
    }

    #[test]
    fn display_link_count_two_ticks_at_30_hz() {
        assert_eq!(tick_interval(1), (1.0 / 60.0, 1.0 / 60.0));
        assert_eq!(tick_interval(2), (2.0 / 60.0, 2.0 / 60.0));
        assert_eq!(tick_interval(0), tick_interval(1));
    }
}
