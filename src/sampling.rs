use std::f32::consts::PI;
use ultraviolet::Vec3;

pub fn cosine_weighted_sample_on_hemisphere(u: f32, v: f32) -> Vec3 {
    return Vec3 {
        x: (2f32 * PI * v).cos() * u.sqrt(),
        y: (2f32 * PI * v).sin() * u.sqrt(),
        z: (1f32 - u).sqrt(),
    };
}
