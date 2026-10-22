//! The Android accelerometer through the NDK sensor API.
//!
//! The event queue sits on the main thread's looper (the one android-activity polls) with a
//! callback, so readings are drained as they arrive and the latest one waits for the next
//! frame. The display rotation comes from JNI every frame, since turning the device from one
//! landscape to the other changes no configuration: a stale rotation would read the device as
//! upside down after the display flipped. The sensor is off while the app is in the background,
//! like the suspended original's motion manager.

use std::ffi::{c_int, c_void};
use std::ptr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};

use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use ndk_sys::{
    ALOOPER_POLL_CALLBACK, ALooper_forThread, ASENSOR_TYPE_ACCELEROMETER, ASensor, ASensorEvent, ASensorEventQueue,
    ASensorEventQueue_disableSensor, ASensorEventQueue_enableSensor, ASensorEventQueue_getEvents,
    ASensorEventQueue_setEventRate, ASensorManager_createEventQueue, ASensorManager_getDefaultSensor,
    ASensorManager_getInstanceForPackage,
};

use super::{DisplayRotation, SensorReading, natural_to_landscape_right, reaction_to_ios};
use crate::PlatformSystems;
use crate::lifecycle::{self, LifecycleTracker};

/// The package the sensor manager is asked for (`applicationId` in android/app).
const PACKAGE_NAME: &std::ffi::CStr = c"com.boltcreative.pocketgod.port";

/// Microseconds between readings: 60 Hz, the original's motion update interval.
const SENSOR_PERIOD_US: i32 = 16_667;

/// Readings fetched per `ASensorEventQueue_getEvents` call.
const EVENT_BATCH: usize = 8;

/// The accelerometer's queue and sensor, set once on the main thread.
static EVENT_QUEUE: AtomicPtr<ASensorEventQueue> = AtomicPtr::new(ptr::null_mut());
static SENSOR: AtomicPtr<ASensor> = AtomicPtr::new(ptr::null_mut());

/// The latest reading in the device's natural axes, m/s².
static LATEST_READING: Mutex<Option<[f32; 3]>> = Mutex::new(None);

/// Whether the sensor delivers readings, and the display rotation they are mapped with.
#[derive(Resource, Default)]
struct SensorState {
    enabled: bool,
    rotation: DisplayRotation,

    /// The rotation query failed (JNI): the last rotation stays, without a warning every frame.
    rotation_unavailable: bool,
}

pub(super) fn build(app: &mut App) {
    app.init_resource::<SensorState>();
    app.add_systems(Startup, start_accelerometer);

    // After this frame's lifecycle changes: the last update before a suspension must turn the
    // sensor off.
    let poll = poll_accelerometer
        .in_set(PlatformSystems::Input)
        .after(lifecycle::track_lifecycle)
        .before(super::update_motion);
    app.add_systems(Update, poll);
}

/// Drains the queue (called by the looper on the main thread); keeps the latest reading.
unsafe extern "C" fn on_sensor_events(_fd: c_int, _events: c_int, _data: *mut c_void) -> c_int {
    let queue = EVENT_QUEUE.load(Ordering::Acquire);
    if queue.is_null() {
        return 0;
    }

    let mut latest = None;
    // SAFETY: `ASensorEvent` is plain old data; the queue fills at most `EVENT_BATCH` events.
    let mut events: [ASensorEvent; EVENT_BATCH] = unsafe { std::mem::zeroed() };
    loop {
        // SAFETY: `queue` is the live queue created in `start_accelerometer`.
        let count = unsafe { ASensorEventQueue_getEvents(queue, events.as_mut_ptr(), EVENT_BATCH) };
        if count <= 0 {
            break;
        }
        for event in &events[..count as usize] {
            if event.type_ != ASENSOR_TYPE_ACCELEROMETER {
                continue;
            }
            // SAFETY: accelerometer events carry their vector in `data[0..3]`.
            let data = unsafe { event.__bindgen_anon_1.__bindgen_anon_1.data };
            latest = Some([data[0], data[1], data[2]]);
        }
    }

    if let Some(reading) = latest
        && let Ok(mut stored) = LATEST_READING.lock()
    {
        *stored = Some(reading);
    }
    1
}

fn set_sensor_enabled(enabled: bool) {
    let queue = EVENT_QUEUE.load(Ordering::Acquire);
    let sensor = SENSOR.load(Ordering::Acquire);
    if queue.is_null() || sensor.is_null() {
        return;
    }
    // SAFETY: both were created by `start_accelerometer` and live for the whole run.
    unsafe {
        if enabled {
            ASensorEventQueue_enableSensor(queue, sensor);
            ASensorEventQueue_setEventRate(queue, sensor, SENSOR_PERIOD_US);
        } else {
            ASensorEventQueue_disableSensor(queue, sensor);
        }
    }
}

/// Creates the accelerometer's event queue on the main thread's looper.
fn start_accelerometer(_main_thread: NonSendMarker) {
    // SAFETY: plain NDK calls; every pointer is checked before use.
    unsafe {
        let manager = ASensorManager_getInstanceForPackage(PACKAGE_NAME.as_ptr());
        if manager.is_null() {
            warn!("no sensor manager; the device rests in landscape-right");
            return;
        }
        let sensor = ASensorManager_getDefaultSensor(manager, ASENSOR_TYPE_ACCELEROMETER as c_int);
        if sensor.is_null() {
            warn!("no accelerometer; the device rests in landscape-right");
            return;
        }
        let looper = ALooper_forThread();
        if looper.is_null() {
            warn!("the main thread has no looper; the accelerometer is off");
            return;
        }
        let queue = ASensorManager_createEventQueue(
            manager,
            looper,
            ALOOPER_POLL_CALLBACK as c_int,
            Some(on_sensor_events),
            ptr::null_mut(),
        );
        if queue.is_null() {
            warn!("could not create the accelerometer's event queue");
            return;
        }

        SENSOR.store(sensor.cast_mut(), Ordering::Release);
        EVENT_QUEUE.store(queue, Ordering::Release);
    }
}

/// Turns the sensor off in the background and on in the foreground, reads the display rotation
/// and hands the latest reading to [`super::update_motion`].
fn poll_accelerometer(
    _main_thread: NonSendMarker,
    tracker: Res<LifecycleTracker>,
    mut sensor: ResMut<SensorState>,
    mut reading: ResMut<SensorReading>,
) {
    let foreground = tracker.is_foreground();
    if sensor.enabled != foreground {
        set_sensor_enabled(foreground);
        sensor.enabled = foreground;
    }
    if !foreground {
        return;
    }

    if !sensor.rotation_unavailable {
        match crate::android::display_rotation() {
            Some(turns) => sensor.rotation = DisplayRotation::from_quarter_turns(turns),
            None => {
                warn!("the display rotation is unavailable; readings keep the last one");
                sensor.rotation_unavailable = true;
            }
        }
    }

    let latest = match LATEST_READING.lock() {
        Ok(stored) => *stored,
        Err(_) => None,
    };
    let Some([x, y, z]) = latest else {
        return;
    };
    let natural = Vec3::new(x, y, z);
    let landscape = natural_to_landscape_right(natural, sensor.rotation);
    reading.acceleration = Some(reaction_to_ios(landscape));
}
