//! The app lifecycle as `UIApplicationDelegate` callbacks (`docs/re/managers-platform.md` §2.3–
//! §2.4 and §18 rows 7–8, `docs/re/controller.md` §2.7).
//!
//! Only an OS suspension is UIKit's background: the Android app sent to the background, or a
//! web page put into the back/forward cache. It is reported as
//! [`PlatformLifecycle::DidEnterBackground`] and, on return,
//! [`PlatformLifecycle::WillEnterForeground`] (where the original refreshes its news). A window
//! that loses the focus or is hidden (minimised, or a hidden browser tab) only resigns active,
//! as when Notification Center covers the game: a desktop window has no background-to-
//! foreground transition, so the game must not refresh its news on refocus (§2.4). Every
//! transition is reported as a [`PlatformLifecycle`] message, in UIKit's order.
//!
//! Time and sound stop whenever the game is off screen (suspended or hidden), as for the
//! suspended original: the display link pauses ([`FrameDriver`]) and the voices fall silent
//! (`crate::audio`). A window that is shown but unfocused keeps its sound, like an inactive iOS
//! app.
//!
//! The platform applies `PocketGodAppDelegate`'s render-loop rows itself: resigning active or
//! entering the background stops the render loop, entering the foreground or becoming active
//! starts it, all only once the startup completed (`mbStartupComplete`, set together with
//! `hideLoadingScreen`). The calls are idempotent, so the game's own delegate may repeat them.

use bevy::ecs::message::{MessageCursor, Messages};
use bevy::prelude::*;
use bevy::window::{AppLifecycle, WindowEvent};
use pg_engine::platform::PlatformRequests;
use pg_engine::view;

use crate::PlatformSettings;
use crate::frame::FrameDriver;

/// The `UIApplicationDelegate` callbacks, in the order UIKit sends them.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformLifecycle {
    /// `applicationWillResignActive:`: the focus is lost, the window hidden or the app leaves.
    WillResignActive,

    /// `applicationDidEnterBackground:`: the OS suspended the app.
    DidEnterBackground,

    /// `applicationWillEnterForeground:`: the OS resumed the app.
    WillEnterForeground,

    /// `applicationDidBecomeActive:`.
    DidBecomeActive,
}

/// Suspension, visibility and focus, turning platform events into UIKit transitions.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifecycleTracker {
    /// The OS suspended the app.
    suspended: bool,

    /// The window is hidden.
    occluded: bool,

    focused: bool,
}

impl Default for LifecycleTracker {
    fn default() -> Self {
        Self {
            suspended: false,
            occluded: false,
            focused: true,
        }
    }
}

impl LifecycleTracker {
    /// UIKit's foreground: the OS has not suspended the app.
    pub fn is_foreground(&self) -> bool {
        !self.suspended
    }

    /// The game is on screen: in the foreground with its window shown. Time and sound only run
    /// while it is.
    pub fn is_visible(&self) -> bool {
        !self.suspended && !self.occluded
    }

    /// UIKit's active state: on screen with the focus.
    pub fn is_active(&self) -> bool {
        self.is_visible() && self.focused
    }

    /// Reports a change of the active state since `was_active`.
    fn report_activity_change(&self, was_active: bool, transitions: &mut Vec<PlatformLifecycle>) {
        let active = self.is_active();
        if was_active && !active {
            transitions.push(PlatformLifecycle::WillResignActive);
        }
        if !was_active && active {
            transitions.push(PlatformLifecycle::DidBecomeActive);
        }
    }

    /// The OS suspended (`true`) or resumed (`false`) the app: UIKit's background.
    pub fn set_suspended(&mut self, suspended: bool, transitions: &mut Vec<PlatformLifecycle>) {
        let was_active = self.is_active();
        let was_foreground = self.is_foreground();
        self.suspended = suspended;
        let foreground = self.is_foreground();

        if was_foreground && !foreground {
            if was_active {
                transitions.push(PlatformLifecycle::WillResignActive);
            }
            transitions.push(PlatformLifecycle::DidEnterBackground);
        }
        if !was_foreground && foreground {
            transitions.push(PlatformLifecycle::WillEnterForeground);
            if self.is_active() {
                transitions.push(PlatformLifecycle::DidBecomeActive);
            }
        }
    }

    /// The window was hidden (`true`) or shown again (`false`): it resigns or regains active,
    /// without a background transition.
    pub fn set_occluded(&mut self, occluded: bool, transitions: &mut Vec<PlatformLifecycle>) {
        let was_active = self.is_active();
        self.occluded = occluded;
        self.report_activity_change(was_active, transitions);
    }

    /// The window gained or lost the focus.
    pub fn set_focused(&mut self, focused: bool, transitions: &mut Vec<PlatformLifecycle>) {
        let was_active = self.is_active();
        self.focused = focused;
        self.report_activity_change(was_active, transitions);
    }
}

/// Reads the window and app lifecycle events into [`PlatformLifecycle`] messages and pauses
/// the display link while the game is off screen.
pub fn track_lifecycle(
    mut events: MessageReader<WindowEvent>,
    settings: Res<PlatformSettings>,
    mut tracker: ResMut<LifecycleTracker>,
    mut driver: ResMut<FrameDriver>,
    mut lifecycle: MessageWriter<PlatformLifecycle>,
) {
    let mut transitions = Vec::new();
    for event in events.read() {
        match event {
            WindowEvent::AppLifecycle(state) => match state {
                AppLifecycle::WillSuspend | AppLifecycle::Suspended => tracker.set_suspended(true, &mut transitions),
                AppLifecycle::WillResume | AppLifecycle::Running => tracker.set_suspended(false, &mut transitions),
                AppLifecycle::Idle => {}
            },
            WindowEvent::WindowOccluded(occlusion) if settings.pause_when_inactive => {
                tracker.set_occluded(occlusion.occluded, &mut transitions);
            }
            WindowEvent::WindowFocused(focus) if settings.pause_when_inactive => {
                tracker.set_focused(focus.focused, &mut transitions);
            }
            _ => {}
        }
    }

    let paused = !tracker.is_visible();
    if driver.is_paused() != paused {
        driver.set_paused(paused);
    }

    for transition in transitions {
        info!("app lifecycle: {transition:?}");
        lifecycle.write(transition);
    }
}

/// `PocketGodAppDelegate`'s render-loop rows (§2.3), once the startup completed.
pub fn apply_lifecycle(world: &mut World, mut cursor: Local<MessageCursor<PlatformLifecycle>>) {
    let transitions: Vec<PlatformLifecycle> = {
        let messages = world.resource::<Messages<PlatformLifecycle>>();
        cursor.read(messages).copied().collect()
    };
    if transitions.is_empty() {
        return;
    }

    let startup_complete = world.resource::<PlatformRequests>().loading_screen_hidden;
    if !startup_complete {
        return;
    }

    for transition in transitions {
        match transition {
            PlatformLifecycle::WillResignActive | PlatformLifecycle::DidEnterBackground => {
                view::stop_render_loop(world);
            }
            PlatformLifecycle::WillEnterForeground | PlatformLifecycle::DidBecomeActive => {
                view::start_render_loop(world);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::window::{WindowFocused, WindowOccluded};
    use pg_engine::view::BcView;

    use super::*;
    use PlatformLifecycle::*;

    /// The lifecycle systems' world once `hideLoadingScreen` completed the startup and the render
    /// loop runs.
    fn started_game() -> World {
        let mut world = World::new();
        world.init_resource::<Messages<WindowEvent>>();
        world.init_resource::<Messages<PlatformLifecycle>>();
        world.init_resource::<PlatformSettings>();
        world.init_resource::<LifecycleTracker>();
        world.init_resource::<FrameDriver>();
        world.init_resource::<BcView>();
        let requests = PlatformRequests {
            loading_screen_hidden: true,
            ..default()
        };
        world.insert_resource(requests);
        view::start_render_loop(&mut world);
        world
    }

    /// One frame with `events`: the transitions it reports.
    fn run_frame(world: &mut World, events: &[WindowEvent]) -> Vec<PlatformLifecycle> {
        {
            let mut window_events = world.resource_mut::<Messages<WindowEvent>>();
            window_events.update();
            for event in events {
                window_events.write(event.clone());
            }
        }
        world.resource_mut::<Messages<PlatformLifecycle>>().update();

        world.run_system_cached(track_lifecycle).expect("runs");
        world.run_system_cached(apply_lifecycle).expect("runs");

        let lifecycle = world.resource::<Messages<PlatformLifecycle>>();
        lifecycle.iter_current_update_messages().copied().collect()
    }

    fn occluded(occluded: bool) -> WindowEvent {
        let occlusion = WindowOccluded {
            window: Entity::PLACEHOLDER,
            occluded,
        };
        WindowEvent::WindowOccluded(occlusion)
    }

    fn focused(focused: bool) -> WindowEvent {
        let focus = WindowFocused {
            window: Entity::PLACEHOLDER,
            focused,
        };
        WindowEvent::WindowFocused(focus)
    }

    fn render_loop_active(world: &World) -> bool {
        world.resource::<BcView>().render_loop_active
    }

    fn display_link_paused(world: &World) -> bool {
        world.resource::<FrameDriver>().is_paused()
    }

    #[test]
    fn a_hidden_window_pauses_the_game_without_entering_the_background() {
        let mut world = started_game();

        let hidden = run_frame(&mut world, &[occluded(true)]);
        assert_eq!(hidden, vec![WillResignActive]);
        assert!(display_link_paused(&world));
        assert!(!render_loop_active(&world));

        let shown = run_frame(&mut world, &[occluded(false)]);
        assert_eq!(shown, vec![DidBecomeActive]);
        assert!(!display_link_paused(&world));
        let view = world.resource::<BcView>();
        assert!(view.render_loop_active && view.render_loop_restarted);
    }

    #[test]
    fn an_os_suspension_is_the_background() {
        let mut world = started_game();

        let suspending = run_frame(&mut world, &[WindowEvent::AppLifecycle(AppLifecycle::WillSuspend)]);
        assert_eq!(suspending, vec![WillResignActive, DidEnterBackground]);
        assert!(display_link_paused(&world));
        assert!(!render_loop_active(&world));

        let suspended = run_frame(&mut world, &[WindowEvent::AppLifecycle(AppLifecycle::Suspended)]);
        assert!(suspended.is_empty());

        let resuming = [
            WindowEvent::AppLifecycle(AppLifecycle::WillResume),
            WindowEvent::AppLifecycle(AppLifecycle::Running),
        ];
        let resumed = run_frame(&mut world, &resuming);
        assert_eq!(resumed, vec![WillEnterForeground, DidBecomeActive]);
        assert!(!display_link_paused(&world));
        assert!(render_loop_active(&world));
    }

    #[test]
    fn an_unfocused_window_stops_the_render_loop_but_not_the_display_link() {
        let mut world = started_game();

        let unfocused = run_frame(&mut world, &[focused(false)]);
        assert_eq!(unfocused, vec![WillResignActive]);
        assert!(!render_loop_active(&world));
        assert!(!display_link_paused(&world));

        let refocused = run_frame(&mut world, &[focused(true)]);
        assert_eq!(refocused, vec![DidBecomeActive]);
        assert!(render_loop_active(&world));
    }

    #[test]
    fn the_render_loop_is_left_alone_until_the_startup_completed() {
        let mut world = started_game();
        view::stop_render_loop(&mut world);
        world.resource_mut::<PlatformRequests>().loading_screen_hidden = false;

        let transitions = run_frame(&mut world, &[focused(false), focused(true)]);
        assert_eq!(transitions, vec![WillResignActive, DidBecomeActive]);
        assert!(!render_loop_active(&world));
    }

    #[test]
    fn focus_changes_resign_and_regain_active() {
        let mut tracker = LifecycleTracker::default();
        let mut transitions = Vec::new();
        tracker.set_focused(false, &mut transitions);
        tracker.set_focused(false, &mut transitions);
        assert!(tracker.is_visible());
        tracker.set_focused(true, &mut transitions);
        assert_eq!(transitions, vec![WillResignActive, DidBecomeActive]);
    }

    #[test]
    fn suspension_is_the_background() {
        let mut tracker = LifecycleTracker::default();
        let mut transitions = Vec::new();
        tracker.set_suspended(true, &mut transitions);
        tracker.set_suspended(true, &mut transitions);
        assert!(!tracker.is_foreground() && !tracker.is_visible());
        tracker.set_suspended(false, &mut transitions);
        assert_eq!(transitions, vec![WillResignActive, DidEnterBackground, WillEnterForeground, DidBecomeActive]);
    }

    #[test]
    fn a_hidden_window_only_resigns_active() {
        let mut tracker = LifecycleTracker::default();
        let mut transitions = Vec::new();
        tracker.set_occluded(true, &mut transitions);
        assert!(tracker.is_foreground());
        assert!(!tracker.is_visible());

        tracker.set_occluded(false, &mut transitions);
        assert_eq!(transitions, vec![WillResignActive, DidBecomeActive]);
        assert!(tracker.is_active());
    }

    #[test]
    fn an_unfocused_window_returns_inactive() {
        let mut tracker = LifecycleTracker::default();
        let mut transitions = Vec::new();
        tracker.set_focused(false, &mut transitions);
        tracker.set_occluded(true, &mut transitions);
        tracker.set_occluded(false, &mut transitions);
        assert_eq!(transitions, vec![WillResignActive]);
        assert!(tracker.is_visible() && !tracker.is_active());

        transitions.clear();
        tracker.set_focused(true, &mut transitions);
        assert_eq!(transitions, vec![DidBecomeActive]);
    }

    #[test]
    fn suspension_of_a_hidden_window_enters_the_background_once() {
        let mut tracker = LifecycleTracker::default();
        let mut transitions = Vec::new();
        tracker.set_occluded(true, &mut transitions);
        tracker.set_suspended(true, &mut transitions);
        tracker.set_occluded(false, &mut transitions);
        assert!(!tracker.is_visible());
        tracker.set_suspended(false, &mut transitions);
        assert_eq!(transitions, vec![WillResignActive, DidEnterBackground, WillEnterForeground, DidBecomeActive]);
    }
}
