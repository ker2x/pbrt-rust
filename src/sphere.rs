use crate::ray::Ray;
use std::fmt;
use std::fmt::Formatter;
use ultraviolet::Vec3;

pub const EPSILON_SPHERE: f32 = 1e-4;

pub enum MaterialType {
    Diffuse,
    Specular,
    Refractive,
}

pub struct Sphere {
    pub position: ultraviolet::Vec3,
    pub radius: f32,
    pub emission: ultraviolet::Vec3,
    pub color: Vec3,
    pub material: MaterialType,
}

impl Sphere {
    pub fn intersect(&self, ray: &mut Ray) -> bool {
        let op = self.position - ray.origin;
        let dop = ray.direction.dot(op);
        let destination = dop * dop - op.dot(op) + self.radius * self.radius;

        return if destination < 0.0 {
            false
        } else {
            let tmin = dop - destination.sqrt();
            if ray.tmin < tmin && tmin < ray.tmax {
                ray.tmax = tmin;
                true
            } else {
                let tmax = dop + destination.sqrt();
                if ray.tmin < tmax && tmax < ray.tmax {
                    ray.tmax = tmax;
                    true
                } else {
                    false
                }
            }
        };
    }
}

impl fmt::Display for MaterialType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            MaterialType::Diffuse => write!(f, "Diffuse"),
            MaterialType::Refractive => write!(f, "Refractive"),
            MaterialType::Specular => write!(f, "Specular"),
        }
    }
}

impl fmt::Display for Sphere {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(position: ({},{},{}), radius: {}, emission: {}, color: ({},{},{}), material: {})",
            self.position.x,
            self.position.y,
            self.position.z,
            self.radius,
            self.emission.x,
            self.color.x,
            self.color.y,
            self.color.z,
            self.material
        )
    }
}
