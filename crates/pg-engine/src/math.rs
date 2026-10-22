//! Vector and matrix helpers shared by the engine and the game, matching the original's
//! float32 arithmetic.

pub use bevy_math::{Mat4, Vec2, Vec3, Vec4};

/// The original's `VECTOR4` default: `(0, 0, 0, 1)`.
pub const VEC4_ORIGIN: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);

/// `(1, 1, 1, 1)`, the default scale.
pub const VEC4_ONE: Vec4 = Vec4::ONE;

/// `FUN_1001a15d4`: `sqrtf(x² + y² + z²)` of a vec4 (w ignored).
pub fn length3(v: Vec4) -> f32 {
    let squared = v.x * v.x + v.y * v.y + v.z * v.z;
    squared.sqrt()
}

/// `FUN_1001a157c`: all four components scaled by the inverse xyz length. The squares are summed
/// in f32, the square root and reciprocal taken in f64 (`fcvt d2, s2; fsqrt; fdiv`) and the
/// factor rounded back to f32; a zero vector becomes inf/NaN exactly like the original.
pub fn normalize3(v: Vec4) -> Vec4 {
    let squared = v.x * v.x + v.y * v.y + v.z * v.z;
    let inverse_length = (1.0 / f64::from(squared).sqrt()) as f32;
    v * inverse_length
}

/// Degrees to radians the way the original writes it (`deg * 3.1415927 / 180`).
pub fn deg_to_rad(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}

/// Radians to degrees (`rad * 180 / 3.1415927`).
pub fn rad_to_deg(radians: f32) -> f32 {
    radians * 180.0 / std::f32::consts::PI
}

/// C `fcvtzs` float → int conversion: truncates toward zero, saturates, NaN → 0.
pub fn trunc_i32(value: f32) -> i32 {
    value as i32
}
