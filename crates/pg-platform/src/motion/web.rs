//! `DeviceMotionEvent.accelerationIncludingGravity` in the browser.
//!
//! Readings are in the device's natural axes and m/s². Chrome and Firefox report the reaction
//! to gravity like Android's sensor (+9.81 along an axis that points up); Safari on iOS passes
//! Core Motion's gravity through scaled to m/s², i.e. already in the iOS convention. The screen
//! orientation angle maps the axes to the landscape-right device. iOS Safari only sends events
//! after `DeviceMotionEvent.requestPermission()`, which must run inside a user gesture: the
//! first touch or click asks.

use std::cell::{Cell, RefCell};

use bevy::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{DeviceMotionEvent, Event};

use super::{DisplayRotation, STANDARD_GRAVITY, SensorReading, natural_to_landscape_right, reaction_to_ios};
use crate::PlatformSystems;

/// A JavaScript event listener.
type Listener = Closure<dyn FnMut(Event)>;

thread_local! {
    /// The latest reading in the device's natural axes, m/s².
    static LATEST_READING: Cell<Option<[f64; 3]>> = const { Cell::new(None) };

    /// Whether the permission prompt was already requested.
    static PERMISSION_REQUESTED: Cell<bool> = const { Cell::new(false) };

    /// The listeners, kept alive for the whole run.
    static LISTENERS: RefCell<Vec<Listener>> = const { RefCell::new(Vec::new()) };
}

pub(super) fn build(app: &mut App) {
    app.add_systems(Startup, install_listeners);

    let poll = poll_device_motion
        .in_set(PlatformSystems::Input)
        .after(crate::lifecycle::track_lifecycle)
        .before(super::update_motion);
    app.add_systems(Update, poll);
}

fn on_device_motion(event: Event) {
    let Some(motion) = event.dyn_ref::<DeviceMotionEvent>() else {
        return;
    };
    let Some(acceleration) = motion.acceleration_including_gravity() else {
        return;
    };
    let (Some(x), Some(y), Some(z)) = (acceleration.x(), acceleration.y(), acceleration.z()) else {
        return;
    };
    LATEST_READING.with(|latest| latest.set(Some([x, y, z])));
}

/// `DeviceMotionEvent.requestPermission()` where it exists (iOS Safari), once, from a gesture.
fn on_first_gesture(_event: Event) {
    if PERMISSION_REQUESTED.with(|requested| requested.replace(true)) {
        return;
    }
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(constructor) = js_sys::Reflect::get(&window, &"DeviceMotionEvent".into()) else {
        return;
    };
    let Ok(request) = js_sys::Reflect::get(&constructor, &"requestPermission".into()) else {
        return;
    };
    let Some(request) = request.dyn_ref::<js_sys::Function>() else {
        return;
    };
    // The promise settles later; events simply start arriving once the user allows it.
    let _ = request.call0(&constructor);
}

fn add_listener(window: &web_sys::Window, event_name: &str, handler: fn(Event)) {
    let closure = Listener::new(handler);
    let callback = closure.as_ref().unchecked_ref();
    if window.add_event_listener_with_callback(event_name, callback).is_err() {
        warn!("could not listen to {event_name}");
    }
    LISTENERS.with(|listeners| listeners.borrow_mut().push(closure));
}

fn install_listeners() {
    let Some(window) = web_sys::window() else {
        return;
    };
    add_listener(&window, "devicemotion", on_device_motion);
    add_listener(&window, "touchend", on_first_gesture);
    add_listener(&window, "click", on_first_gesture);
}

/// The screen orientation angle (`screen.orientation.angle`, else the old `window.orientation`).
fn screen_rotation(window: &web_sys::Window) -> DisplayRotation {
    if let Ok(screen) = window.screen() {
        let orientation = screen.orientation();
        if let Ok(angle) = orientation.angle() {
            return DisplayRotation::from_degrees(i32::from(angle));
        }
    }

    let legacy = js_sys::Reflect::get(window, &"orientation".into()).ok();
    let legacy_degrees = legacy.and_then(|value| value.as_f64());
    match legacy_degrees {
        Some(degrees) => DisplayRotation::from_degrees(degrees as i32),
        None => DisplayRotation::Rotation0,
    }
}

/// Safari on iOS and iPadOS (which presents itself as a Mac with a touch screen).
fn reports_ios_convention(window: &web_sys::Window) -> bool {
    let navigator = window.navigator();
    let user_agent = navigator.user_agent().unwrap_or_default();
    let mobile_apple = ["iPhone", "iPad", "iPod"].iter().any(|device| user_agent.contains(device));
    let touch_mac = user_agent.contains("Macintosh") && navigator.max_touch_points() > 1;
    mobile_apple || touch_mac
}

fn poll_device_motion(mut reading: ResMut<SensorReading>, mut ios_convention: Local<Option<bool>>) {
    let Some([x, y, z]) = LATEST_READING.with(Cell::get) else {
        return;
    };
    let Some(window) = web_sys::window() else {
        return;
    };
    let ios = *ios_convention.get_or_insert_with(|| reports_ios_convention(&window));

    let natural = Vec3::new(x as f32, y as f32, z as f32);
    let rotation = screen_rotation(&window);
    let landscape = natural_to_landscape_right(natural, rotation);
    let acceleration = if ios { landscape / STANDARD_GRAVITY } else { reaction_to_ios(landscape) };
    reading.acceleration = Some(acceleration);
}
