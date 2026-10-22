//! The accelerometer, in the iOS `UIAcceleration` convention the engine reads
//! ([`MotionInput`], `docs/re/rendering.md` §10): g units, the device's portrait axes (x to the
//! right edge, y to the top edge, z out of the screen), gravity included. Flat face up reads
//! (0, 0, -1); held upright in landscape-right, the only orientation the original supports,
//! (-1, 0, 0).
//!
//! There is never a (0, 0, 0) "no sensor" reading: readers that `SpriteLogic::isAccelEnabled`
//! does not gate (new pygmies, monkeys, the bonfire, …) would take it for a device lying flat
//! or upside down. Without data the device rests in landscape-right.
//!
//! - Android: the accelerometer through the NDK sensor API, remapped for the display rotation
//!   (the activity is `sensorLandscape`, so both landscape orientations occur).
//! - Web: `DeviceMotionEvent.accelerationIncludingGravity`, remapped for the screen orientation;
//!   the keyboard emulation stands in until a reading arrives (desktop browsers).
//! - Desktop: the keyboard emulation of [`keyboard`].

pub mod keyboard;

#[cfg(target_os = "android")]
mod android;

#[cfg(target_arch = "wasm32")]
mod web;

use bevy::prelude::*;
use pg_engine::platform::TextField;
use pg_engine::view::MotionInput;

use keyboard::{TiltEmulator, TiltKeys};

/// The device at rest, upright in landscape-right (home button on the right).
pub const RESTING_ACCELERATION: Vec3 = Vec3::new(-1.0, 0.0, 0.0);

/// Standard gravity, for sensors that report m/s².
pub const STANDARD_GRAVITY: f32 = 9.80665;

/// How the screen content is rotated from the device's natural orientation (Android
/// `Surface.ROTATION_*`, web `screen.orientation.angle`): the content turns the opposite way
/// to the device.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisplayRotation {
    #[default]
    Rotation0,
    Rotation90,
    Rotation180,
    Rotation270,
}

impl DisplayRotation {
    /// From a quarter-turn count (Android's `Display.getRotation()` values 0–3).
    pub fn from_quarter_turns(turns: i32) -> Self {
        match turns.rem_euclid(4) {
            1 => Self::Rotation90,
            2 => Self::Rotation180,
            3 => Self::Rotation270,
            _ => Self::Rotation0,
        }
    }

    /// From an angle in degrees (`screen.orientation.angle`, or the old `window.orientation`,
    /// which says -90 for 270).
    pub fn from_degrees(degrees: i32) -> Self {
        let turns = (degrees as f32 / 90.0).round() as i32;
        Self::from_quarter_turns(turns)
    }
}

/// A reading in the device's natural axes (x right, y up, z out of the screen, as Android and
/// the W3C define them) expressed in the axes of a virtual iPhone held in landscape-right
/// relative to the screen content, whose x points up the screen and y to its left.
pub fn natural_to_landscape_right(natural: Vec3, rotation: DisplayRotation) -> Vec3 {
    let screen = match rotation {
        DisplayRotation::Rotation0 => Vec2::new(natural.x, natural.y),
        DisplayRotation::Rotation90 => Vec2::new(-natural.y, natural.x),
        DisplayRotation::Rotation180 => Vec2::new(-natural.x, -natural.y),
        DisplayRotation::Rotation270 => Vec2::new(natural.y, -natural.x),
    };
    Vec3::new(screen.y, -screen.x, natural.z)
}

/// An Android (or W3C) accelerometer reading in m/s²: the reaction to gravity, +9.81 along an
/// axis that points up. iOS reports gravity itself in g, -1 along that axis.
pub fn reaction_to_ios(reaction: Vec3) -> Vec3 {
    -reaction / STANDARD_GRAVITY
}

/// The latest reading of the platform's accelerometer, already in the iOS convention.
#[derive(Resource, Default)]
pub struct SensorReading {
    pub(crate) acceleration: Option<Vec3>,
}

/// The keyboard tilt emulation (desktop, and the web without a sensor).
#[derive(Resource, Default)]
pub struct KeyboardMotion {
    emulator: TiltEmulator,
}

pub(crate) fn build(app: &mut App) {
    app.insert_resource(MotionInput {
        acceleration: RESTING_ACCELERATION,
    });
    app.init_resource::<SensorReading>();
    app.init_resource::<KeyboardMotion>();

    #[cfg(target_os = "android")]
    android::build(app);

    #[cfg(target_arch = "wasm32")]
    web::build(app);
}

/// The keys the emulation reads, unless the game's text field has the keyboard.
fn tilt_keys(keys: &ButtonInput<KeyCode>, text_field: Option<&TextField>) -> TiltKeys {
    let typing = text_field.is_some_and(|field| field.first_responder);
    if typing {
        return TiltKeys::default();
    }
    TiltKeys::from_keyboard(keys)
}

/// Fills [`MotionInput`] for the next tick: the sensor when it has spoken, else the keyboard
/// emulation (which rests in landscape-right).
pub fn update_motion(
    time: Res<Time<Real>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    text_field: Option<Res<TextField>>,
    sensor: Res<SensorReading>,
    mut keyboard: ResMut<KeyboardMotion>,
    mut motion: ResMut<MotionInput>,
) {
    let acceleration = match sensor.acceleration {
        Some(reading) => reading,
        None => {
            let held = match keys.as_deref() {
                Some(keys) => tilt_keys(keys, text_field.as_deref()),
                None => TiltKeys::default(),
            };
            let dt = time.delta_secs();
            keyboard.emulator.update(held, dt)
        }
    };

    if motion.acceleration != acceleration {
        motion.acceleration = acceleration;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UPRIGHT_REACTION: f32 = STANDARD_GRAVITY;

    fn assert_close(actual: Vec3, expected: Vec3) {
        assert!((actual - expected).length() < 1e-5, "{actual} != {expected}");
    }

    #[test]
    fn android_landscape_readings_rest_in_landscape_right() {
        // ROTATION_90: the phone turned counter-clockwise, its top to the left (home button
        // on the right): its +x points up, so the reaction force is +x.
        let rotation_90 = Vec3::new(UPRIGHT_REACTION, 0.0, 0.0);
        let landscape = natural_to_landscape_right(rotation_90, DisplayRotation::Rotation90);
        assert_close(reaction_to_ios(landscape), RESTING_ACCELERATION);

        // ROTATION_270: turned the other way, +x points down.
        let rotation_270 = Vec3::new(-UPRIGHT_REACTION, 0.0, 0.0);
        let landscape = natural_to_landscape_right(rotation_270, DisplayRotation::Rotation270);
        assert_close(reaction_to_ios(landscape), RESTING_ACCELERATION);

        // A landscape-natural tablet at ROTATION_0: +y points up.
        let tablet = Vec3::new(0.0, UPRIGHT_REACTION, 0.0);
        let landscape = natural_to_landscape_right(tablet, DisplayRotation::Rotation0);
        assert_close(reaction_to_ios(landscape), RESTING_ACCELERATION);
    }

    #[test]
    fn flat_face_up_reads_minus_z_in_every_rotation() {
        let flat = Vec3::new(0.0, 0.0, UPRIGHT_REACTION);
        for turns in 0..4 {
            let rotation = DisplayRotation::from_quarter_turns(turns);
            let landscape = natural_to_landscape_right(flat, rotation);
            assert_close(reaction_to_ios(landscape), Vec3::new(0.0, 0.0, -1.0));
        }
    }

    #[test]
    fn tilting_the_left_edge_down_reads_positive_y_in_both_landscapes() {
        // 30° roll with the screen's left side going down, as iOS would read it.
        let (sin, cos) = 30f32.to_radians().sin_cos();
        let expected = Vec3::new(-cos, sin, 0.0);

        // ROTATION_90: device axes are the virtual iPhone's; the reaction is -gravity.
        let rotation_90 = -expected * STANDARD_GRAVITY;
        let landscape = natural_to_landscape_right(rotation_90, DisplayRotation::Rotation90);
        assert_close(reaction_to_ios(landscape), expected);

        // ROTATION_270: the device is upside down relative to the virtual iPhone.
        let rotation_270 = Vec3::new(-rotation_90.x, -rotation_90.y, rotation_90.z);
        let landscape = natural_to_landscape_right(rotation_270, DisplayRotation::Rotation270);
        assert_close(reaction_to_ios(landscape), expected);
    }

    #[test]
    fn rotations_parse_from_android_and_web_values() {
        assert_eq!(DisplayRotation::from_quarter_turns(1), DisplayRotation::Rotation90);
        assert_eq!(DisplayRotation::from_quarter_turns(3), DisplayRotation::Rotation270);
        assert_eq!(DisplayRotation::from_quarter_turns(4), DisplayRotation::Rotation0);
        assert_eq!(DisplayRotation::from_degrees(90), DisplayRotation::Rotation90);
        assert_eq!(DisplayRotation::from_degrees(-90), DisplayRotation::Rotation270);
        assert_eq!(DisplayRotation::from_degrees(270), DisplayRotation::Rotation270);
        assert_eq!(DisplayRotation::from_degrees(180), DisplayRotation::Rotation180);
    }

    #[test]
    fn typing_in_the_text_field_releases_the_tilt_keys() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::ArrowLeft);
        let free = tilt_keys(&keys, None);
        assert!(free.left);

        let field = TextField {
            first_responder: true,
            ..default()
        };
        let typing = tilt_keys(&keys, Some(&field));
        assert_eq!(typing, TiltKeys::default());
    }
}
