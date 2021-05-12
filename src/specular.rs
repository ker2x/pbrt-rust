use rand::Rng;
use ultraviolet::Vec3;

fn reflectance0(u: f32, v: f32) -> f32 {
    return (u - v) / (u + v) * ((u - v) / (u + v));
}

fn schlick_reflectance(u: f32, v: f32, c: f32) -> f32 {
    let r0 = reflectance0(u, v);
    return r0 + (1f32 - r0) * c * c * c * c * c;
}

pub fn ideal_specular_reflect(d: Vec3, n: Vec3) -> Vec3 {
    return d - 2f32 * n.dot(d) * n;
}

pub fn ideal_specular_transmit(d: Vec3, n: Vec3, n_out: f32, n_in: f32, pr: &mut f32) -> Vec3 {
    let d_re = ideal_specular_reflect(d, n);

    let nl = if n.dot(d) < 0f32 { n } else { -n };

    let nn = if n.dot(d) < 0f32 {
        n_out / n_in
    } else {
        n_in / n_out
    };

    let cos_theta = d.dot(nl);
    let cos2phi = 1.0 - nn * nn * (1.0 - cos_theta * cos_theta);
    let d_tr = (nn * d - nl * (nn * cos_theta + cos2phi.sqrt())).normalized();

    let c = 1.0
        - (if n.dot(d) < 0f32 {
            -cos_theta
        } else {
            d_tr.dot(n)
        });

    let reflectance = schlick_reflectance(n_out, n_in, c);
    let p_re = 0.25 + 0.5 * reflectance;

    // Total Internal Reflection
    if cos2phi < 0f32 {
        *pr = 1.0;
        return d_re;
    }

    let mut rng = rand::thread_rng();
    if rng.gen::<f32>() < p_re {
        *pr = reflectance / p_re;
        return d_re;
    }

    let transmittance = 1.0 - reflectance;
    let p_transmittance = 1.0 - p_re;
    *pr = transmittance / p_transmittance;

    return d_tr;
}
