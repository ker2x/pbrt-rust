use ultraviolet::Vec3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub tmin: f32,
    pub tmax: f32,
    pub depth: usize,
}

impl Ray {
    pub fn eval(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}
