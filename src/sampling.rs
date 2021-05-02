use ultraviolet::Vec3;
use std::f32::consts::PI;

/*
pub fn uniform_sample_on_hemisphere(u: f32, v:f32) -> Vec3 {
        let sintheta = f32::max(0f32,1f32 - u * u).sqrt();
        let phi = 2f32 * PI * v;
        return Vec3 {
            x: phi.cos() * sintheta,
            y: phi.sin() * sintheta,
            z: u
        }
}
 */

pub fn cosine_weighted_sample_on_hemisphere(u:f32, v:f32) -> Vec3 {
        //let costheta = (1f32 - u).sqrt();
        //let sintheta = u.sqrt();
        let phi = 2f32 * PI * v;
        return Vec3 {
            x: phi.cos() * u.sqrt(),
            y: phi.sin() * u.sqrt(),
            z: (1f32 - u).sqrt()
        }
}
