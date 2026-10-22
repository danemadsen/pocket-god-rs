//! Keyboard emulation of the accelerometer (desktop, and browsers without a sensor).
//!
//! The emulated device rests upright in landscape-right, exactly (-1, 0, 0), and tilts while
//! keys are held, turning at [`TILT_SPEED`]:
//!
//! - ← / →: roll it like a steering wheel by [`TILT_ANGLE`] (left / right edge down): pygmies
//!   tip, the laser target slides sideways, things roll downhill;
//! - ↑ / ↓: lean its top edge away / toward you by [`TILT_ANGLE`] (the laser target rises);
//! - Shift with an arrow: tilt by [`STEEP_TILT_ANGLE`] instead (pygmies slide off, the laser
//!   reaches the top);
//! - F: turn it upside down (pygmies hang or fall);
//! - S: shake it for [`SHAKE_DURATION`] (earthquake).
//!
//! `SpriteLogic::isAccelEnabled` ignores the accelerometer when any raw component is exactly
//! zero (or the device lies flat), and so does the earthquake check of `PygmyLogic`. A pure
//! roll keeps z at zero, so while the device is rolled it also leans back by [`HOLD_LEAN`], as a
//! hand-held device does; the shake moves all three axes. At rest the readers that are gated
//! fall back to straight-down gravity, which is what the upright device gives anyway.

use bevy::prelude::*;

use super::RESTING_ACCELERATION;

/// Degrees an arrow key tilts the device.
pub const TILT_ANGLE: f32 = 30.0;

/// Degrees with Shift held.
pub const STEEP_TILT_ANGLE: f32 = 70.0;

/// Degrees the device leans back while it is rolled and no pitch key is held.
pub const HOLD_LEAN: f32 = 10.0;

/// Degrees per second the emulated device turns toward the held keys' pose.
pub const TILT_SPEED: f32 = 240.0;

/// Seconds a press of the shake key keeps shaking (holding it shakes on).
pub const SHAKE_DURATION: f32 = 0.6;

/// Shakes per second and their size in g. The engine's raw filter passes ~90 % at this rate,
/// so the total force swings well past the earthquake check's 0.9 and 1.1.
pub const SHAKE_FREQUENCY: f32 = 6.0;
pub const SHAKE_AMPLITUDE: f32 = 0.8;

/// The shake's direction in device axes: mostly along the device's long vertical axis, with
/// some of the other two so no raw component stays at zero.
const SHAKE_DIRECTION: Vec3 = Vec3::new(0.88, 0.38, 0.28);

/// The emulation keys that are held.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TiltKeys {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub steep: bool,
    pub flip: bool,
    pub shake: bool,
}

impl TiltKeys {
    pub fn from_keyboard(keys: &ButtonInput<KeyCode>) -> Self {
        let steep_keys = [KeyCode::ShiftLeft, KeyCode::ShiftRight];
        Self {
            left: keys.pressed(KeyCode::ArrowLeft),
            right: keys.pressed(KeyCode::ArrowRight),
            up: keys.pressed(KeyCode::ArrowUp),
            down: keys.pressed(KeyCode::ArrowDown),
            steep: keys.any_pressed(steep_keys),
            flip: keys.pressed(KeyCode::KeyF),
            shake: keys.pressed(KeyCode::KeyS),
        }
    }

    fn tilt_angle(&self) -> f32 {
        if self.steep { STEEP_TILT_ANGLE } else { TILT_ANGLE }
    }

    /// Target roll in degrees, positive with the left edge down.
    fn roll_target(&self) -> f32 {
        if self.flip {
            return 180.0;
        }
        let angle = self.tilt_angle();
        match (self.left, self.right) {
            (true, false) => angle,
            (false, true) => -angle,
            _ => 0.0,
        }
    }

    /// Target pitch in degrees, positive with the top edge leaning away (screen facing up).
    fn pitch_target(&self, roll_target: f32) -> f32 {
        let angle = self.tilt_angle();
        match (self.up, self.down) {
            (true, false) => angle,
            (false, true) => -angle,
            _ if roll_target != 0.0 => HOLD_LEAN,
            _ => 0.0,
        }
    }
}

/// The gravity reading of a device rolled by `roll` and pitched by `pitch` (degrees) from the
/// upright landscape-right pose; exactly [`RESTING_ACCELERATION`] at rest.
pub fn tilted_gravity(roll: f32, pitch: f32) -> Vec3 {
    if roll == 0.0 && pitch == 0.0 {
        return RESTING_ACCELERATION;
    }
    let (sin_roll, cos_roll) = roll.to_radians().sin_cos();
    let (sin_pitch, cos_pitch) = pitch.to_radians().sin_cos();
    Vec3::new(-cos_pitch * cos_roll, cos_pitch * sin_roll, -sin_pitch)
}

/// Moves `value` toward `target` by at most `step`, landing on it exactly.
fn approach(value: f32, target: f32, step: f32) -> f32 {
    let difference = target - value;
    if difference.abs() <= step {
        target
    } else {
        value + step.copysign(difference)
    }
}

/// The emulated device's pose and shake.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TiltEmulator {
    roll: f32,
    pitch: f32,
    shake_left: f32,
    shake_clock: f32,
}

impl TiltEmulator {
    /// Advances the device by `dt` seconds toward the held keys' pose and returns its reading.
    pub fn update(&mut self, keys: TiltKeys, dt: f32) -> Vec3 {
        let step = TILT_SPEED * dt.max(0.0);
        let roll_target = keys.roll_target();
        let pitch_target = keys.pitch_target(roll_target);
        self.roll = approach(self.roll, roll_target, step);
        self.pitch = approach(self.pitch, pitch_target, step);

        if keys.shake {
            if self.shake_left <= 0.0 {
                self.shake_clock = 0.0;
            }
            self.shake_left = SHAKE_DURATION;
        }

        let gravity = tilted_gravity(self.roll, self.pitch);
        if self.shake_left <= 0.0 {
            return gravity;
        }

        let phase = std::f32::consts::TAU * SHAKE_FREQUENCY * self.shake_clock;
        let swing = SHAKE_AMPLITUDE * phase.cos();
        self.shake_clock += dt;
        self.shake_left -= dt;
        gravity + SHAKE_DIRECTION * swing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: f32 = 1.0 / 60.0;

    /// `SpriteLogic::isAccelEnabled` @ 0x100199358 on a reading.
    fn accel_enabled(raw: Vec3) -> bool {
        let rot_x = raw.z.atan2(raw.y).to_degrees();
        let rot_y = raw.z.atan2(raw.x).to_degrees();
        let flat = rot_x > -100.0 && rot_x < -80.0 && rot_y > -100.0 && rot_y < -80.0;
        !flat && raw.x != 0.0 && raw.y != 0.0 && raw.z != 0.0
    }

    /// `PygmyLogic::refreshTiltState` @ 0x10012be50's angle: `atan2(raw.y, raw.x)` in degrees.
    fn tilt_degrees(raw: Vec3) -> f32 {
        raw.y.atan2(raw.x).to_degrees()
    }

    fn hold(emulator: &mut TiltEmulator, keys: TiltKeys, seconds: f32) -> Vec3 {
        let frames = (seconds / FRAME).round() as usize;
        let mut reading = Vec3::ZERO;
        for _ in 0..frames {
            reading = emulator.update(keys, FRAME);
        }
        reading
    }

    #[test]
    fn at_rest_the_device_is_exactly_upright_in_landscape_right() {
        let mut emulator = TiltEmulator::default();
        let reading = hold(&mut emulator, TiltKeys::default(), 1.0);
        assert_eq!(reading, Vec3::new(-1.0, 0.0, 0.0));
        assert_eq!(tilt_degrees(reading).abs(), 180.0);
    }

    #[test]
    fn rolling_enables_tilt_and_tips_the_island() {
        let mut emulator = TiltEmulator::default();
        let left = TiltKeys {
            left: true,
            ..default()
        };
        let reading = hold(&mut emulator, left, 0.5);
        assert!(accel_enabled(reading), "{reading}");
        // State 2 of the tilt table: (130, 155].
        let angle = tilt_degrees(reading);
        assert!(angle > 130.0 && angle <= 155.0, "{angle}");
        // Gravity of the tilt physics, normalize(-a.y, a.x), points left and down.
        let gravity = Vec2::new(-reading.y, reading.x).normalize();
        assert!(gravity.x < -0.4 && gravity.y < -0.8, "{gravity}");

        let right = TiltKeys {
            right: true,
            steep: true,
            ..default()
        };
        let reading = hold(&mut emulator, right, 1.0);
        assert!(accel_enabled(reading));
        // State 3, sliding: [-130, -90).
        let angle = tilt_degrees(reading);
        assert!((-130.0..-90.0).contains(&angle), "{angle}");

        let reading = hold(&mut emulator, TiltKeys::default(), 1.0);
        assert_eq!(reading, Vec3::new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn flipping_turns_the_device_upside_down() {
        let mut emulator = TiltEmulator::default();
        let flip = TiltKeys {
            flip: true,
            ..default()
        };
        let reading = hold(&mut emulator, flip, 1.0);
        let angle = tilt_degrees(reading);
        assert!(angle > -90.0 && angle < 90.0, "{angle}");
        assert!(reading.x > 0.9);
    }

    #[test]
    fn leaning_back_raises_the_laser_target() {
        let mut emulator = TiltEmulator::default();
        let up = TiltKeys {
            up: true,
            steep: true,
            ..default()
        };
        let reading = hold(&mut emulator, up, 1.0);
        // `LaserTargetLogic::updateTargetPos` aims at the top once raw x reaches -0.5.
        assert!(reading.x >= -0.5, "{reading}");
        assert!(reading.z < 0.0);

        let gentle = TiltKeys {
            up: true,
            ..default()
        };
        let mut emulator = TiltEmulator::default();
        let reading = hold(&mut emulator, gentle, 1.0);
        assert!((reading.z + 0.5).abs() < 1e-5, "{reading}");
    }

    #[test]
    fn shaking_swings_the_total_force_through_the_earthquake_thresholds() {
        let mut emulator = TiltEmulator::default();
        let shake = TiltKeys {
            shake: true,
            ..default()
        };
        // The engine's raw filter (`BCView::readAccelerometer`) and the earthquake counter
        // (`PygmyLogic::checkForEarthquake:`).
        let mut raw = RESTING_ACCELERATION;
        let mut shake_up = false;
        let mut swings = 0;
        let mut all_axes_moving = true;
        for frame in 0..36 {
            let keys = if frame == 0 { shake } else { TiltKeys::default() };
            let reading = emulator.update(keys, FRAME);
            raw += (reading - raw) * 0.7;
            if frame > 0 {
                all_axes_moving &= raw.x != 0.0 && raw.y != 0.0 && raw.z != 0.0;
            }
            let total = raw.length();
            if total > 1.1 && !shake_up {
                swings += 1;
                shake_up = true;
            } else if total < 0.9 && shake_up {
                swings += 1;
                shake_up = false;
            }
        }
        assert!(swings >= 5, "{swings}");
        assert!(all_axes_moving);

        let settled = hold(&mut emulator, TiltKeys::default(), 0.2);
        assert_eq!(settled, Vec3::new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn poses_change_at_the_tilt_speed() {
        let mut emulator = TiltEmulator::default();
        let left = TiltKeys {
            left: true,
            ..default()
        };
        emulator.update(left, 0.05);
        assert_eq!(emulator.roll, 12.0);
        assert_eq!(emulator.pitch, HOLD_LEAN);
        assert_eq!(approach(29.0, 30.0, 4.0), 30.0);
        assert_eq!(approach(10.0, -30.0, 4.0), 6.0);
    }
}
