mod ray;
mod sampling;
mod specular;
mod sphere;
mod utils;

use chrono;
use image;
use rand::Rng;
use std::time::Instant;
use ultraviolet::Vec3;

use crate::ray::Ray;
use crate::sampling::cosine_weighted_sample_on_hemisphere;
use crate::specular::{ideal_specular_reflect, ideal_specular_transmit};
use crate::sphere::{MaterialType, Sphere, EPSILON_SPHERE};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use std::sync::{Arc, Mutex};

/*
test : 1600x1200, 16SPP, 3/20 Bounce    : 5s/5s
test : 1600x1200, 32SPP, 3/20 Bounce    : 10s/11s
test : 1600x1200, 64SPP, 3/20 Bounce    : 20s/22s
test : 1600x1200, 128SPP, 3/20 Bounce   : 47s/46s/45s/42s
test : 1600x1200, 256SPP, 3/20 Bounce   : 91s/94s/85s/91s
test : 1600x1200, 512SPP, 3/20 Bounce   : 196s
test : 1600x1200, 1024SPP, 3/20 Bounce  : 373s
test : 1600x1200, 2048SPP, 3/20 Bounce  : 762s
test : 1600x1200, 4096SPP, 3/20 Bounce  : 1541s
test : 1600x1200, 8192SPP, 3/20 Bounce  : 3191s

test : 1600x1200, 128SPP, 3/20 Bounce   : 47s/46s/45s/42s
test : 1600x1200, 128SPP, 3/200 Bounce  : 53s/49s/46s/47s/49s
test : 1600x1200, 128SPP, 3/2000 Bounce : 58s/58s/64s/59s

test : 1600x1200, 128SPP, 30/200 Bounce : 226s
 */

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const TEXTURE_SIZE: usize = WIDTH * HEIGHT;

const SAMPLE: usize = 128; //the most important for quality

const MIN_BOUNCE: usize = 3; //can have a major impact on perf.
const MAX_BOUNCE: usize = 20; //that too, but minor compared to MIN

const FOV: f32 = 0.5135;
const REFRACTIVE_INDEX_OUT: f32 = 1.0;
const REFRACTIVE_INDEX_IN: f32 = 1.52;

fn main() {
    let now = Instant::now();
    println!("Starting, at {}", chrono::offset::Local::now());

    let eye = Vec3::new(50.0, 52.0, 295.6);
    let gaze = Vec3::new(0.0, -0.042612, -1.0).normalized();
    let cx = Vec3::new((FOV * (WIDTH as f32)) / (HEIGHT as f32), 0.0, 0.0);
    let cy = cx.cross(gaze).normalized() * FOV;

    let sphere_list = create_scene();

    let texture = Arc::new(Mutex::new(std::vec![Vec3::zero(); TEXTURE_SIZE]));
    (0..HEIGHT)
        .into_par_iter()
        .for_each(|i| render(i, eye, gaze, cx, cy, Arc::clone(&texture), &sphere_list));

    let tex = texture.lock().unwrap();

    let mut buffer = vec![0 as u8; WIDTH * HEIGHT * 3];
    for i in 0..tex.len() {
        buffer[i * 3 + 0] = fast_srgb8::f32_to_srgb8(tex[i].x);
        buffer[i * 3 + 1] = fast_srgb8::f32_to_srgb8(tex[i].y);
        buffer[i * 3 + 2] = fast_srgb8::f32_to_srgb8(tex[i].z);
    }

    save_image("image.png", buffer);

    println!("Render time : {}s", now.elapsed().as_secs())
}

fn render(
    column: usize,
    eye: Vec3,
    gaze: Vec3,
    cx: Vec3,
    cy: Vec3,
    texture: Arc<Mutex<Vec<Vec3>>>,
    sph: &Vec<Sphere>,
) {
    println!("{}", column);

    let mut rng = rand::thread_rng();
    let mut rngbuff = [0f32;SAMPLE];
    //let mut rngbuff2 = [0f32;SAMPLE];
    let mut luminance: Vec3;

    for rowindex in 0..WIDTH {
        let i = (HEIGHT - 1 - column) * WIDTH + rowindex;
        luminance = Vec3::zero();
        rng.fill(&mut rngbuff);
        //rng.fill(&mut rngbuff2);

        for _ in 0..SAMPLE {
            let dx = rng.gen::<f32>() - 0.5;
            let dy = rng.gen::<f32>() - 0.5;
            let d: Vec3 = cx * ((dx + rowindex as f32) / WIDTH as f32 - 0.5)
                + cy * ((dy + column as f32) / HEIGHT as f32 - 0.5)
                + gaze;

            luminance += radiance(
                &mut Ray {
                    origin: eye + d * 130f32,
                    direction: d.normalized(),
                    tmin: EPSILON_SPHERE,
                    tmax: f32::INFINITY,
                    depth: 0,
                },
                &sph,
            ) * (1f32 / SAMPLE as f32);
        }
        let mut texture = texture.lock().unwrap();
        texture[i] += luminance.clamped(
            Vec3 {
                x: 0f32,
                y: 0f32,
                z: 0f32,
            },
            Vec3 {
                x: 1f32,
                y: 1f32,
                z: 1f32,
            },
        );
    }
}

fn intersect(ray: &mut Ray, sphere: &Vec<Sphere>) -> (usize, bool) {
    let mut id: usize = 0;
    let mut hit = false;

    for i in 0..sphere.len() {
        if sphere[i].intersect(ray) == false {
            continue;
        } else {
            hit = true;
            id = i;
        }
    }
    return (id, hit);
}

fn radiance(ray: &mut Ray, sphere: &Vec<Sphere>) -> Vec3 {
    let mut rng = rand::thread_rng();
    let mut luminance: Vec3 = Vec3::zero();
    let mut color: Vec3 = Vec3::one();

    loop {
        let (id, hit) = intersect(ray, sphere);
        if hit == false {
            return luminance;
        }

        //let shape :Sphere = sphere[id];
        let p = ray.eval(ray.tmax);
        let n = (p - sphere[id].position).normalized();
        luminance += color * sphere[id].emission;
        color *= sphere[id].color;

        if ray.depth > MIN_BOUNCE {
            let continue_probability = sphere[id].color.component_max();
            if rng.gen::<f32>() >= continue_probability {
                return luminance;
            }
            color /= continue_probability;
        }

        if ray.depth > MAX_BOUNCE {
            return luminance;
        }

        match sphere[id].material {
            MaterialType::Specular => {
                //mirror
                let d = ideal_specular_reflect(ray.direction, n);
                *ray = Ray {
                    origin: p,
                    direction: d,
                    tmin: EPSILON_SPHERE,
                    tmax: f32::INFINITY,
                    depth: ray.depth + 1,
                };
            }
            MaterialType::Refractive => {
                let mut pr = 0f32;
                let d = ideal_specular_transmit(
                    ray.direction,
                    n,
                    REFRACTIVE_INDEX_OUT,
                    REFRACTIVE_INDEX_IN,
                    &mut pr,
                );
                color *= pr;
                *ray = Ray {
                    origin: p,
                    direction: d,
                    tmin: EPSILON_SPHERE,
                    tmax: f32::INFINITY,
                    depth: ray.depth + 1,
                };
            }
            MaterialType::Diffuse => {
                let w = if n.dot(ray.direction) < 0f32 { n } else { -n };
                let u = if w.x.abs() > 0.1 {
                    Vec3 {
                        x: 0f32,
                        y: 1f32,
                        z: 0f32,
                    }
                } else {
                    Vec3 {
                        x: 1f32,
                        y: 0f32,
                        z: 0f32,
                    }
                    .cross(w)
                    .normalized()
                };
                let v = w.cross(u);
                let sample_distance =
                    cosine_weighted_sample_on_hemisphere(rng.gen::<f32>(), rng.gen::<f32>());
                let distance =
                    (sample_distance.x * u + sample_distance.y * v + sample_distance.z * w)
                        .normalized();
                *ray = Ray {
                    origin: p,
                    direction: distance,
                    tmin: EPSILON_SPHERE,
                    tmax: f32::INFINITY,
                    depth: ray.depth + 1,
                }
            }
        }
    }
}

fn save_image(filename : &str, buf :Vec<u8>) {
    /*
Pixel is 8-bit luminance : L8,
Pixel is 8-bit luminance with an alpha channel : La8,
Pixel is 16-bit luminance : L16,
Pixel is 16-bit luminance with an alpha channel : La16,

Pixel contains 8-bit R, G and B channels : Rgb8,
Pixel is 8-bit RGB with an alpha channel : Rgba8,

Pixel is 16-bit RGB : Rgb16,
Pixel is 16-bit RGBA : Rgba16,

Pixel contains 8-bit B, G and R channels : Bgr8,
Pixel is 8-bit BGR with an alpha channel : Bgra8,
 */
    image::save_buffer(
        filename,
        &buf,
        WIDTH as u32,
        HEIGHT as u32,
        image::ColorType::Rgb8,
    )
        .unwrap();
}

fn create_scene() -> Vec<Sphere> {
    let mut sphere_list: Vec<Sphere> = Vec::new();
    //LIGHT bottom
    sphere_list.push(sphere::Sphere {
        position: Vec3 {
            x: 50.0,
            y: -3.0,
            z: 81.0,
        },
        radius: 6.5,
        emission: Vec3 {
            x: 40.0,
            y: 30.0,
            z: 30.0,
        },
        color: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        material: MaterialType::Diffuse,
    });

    //LIGHT top
    sphere_list.push(sphere::Sphere {
        position: Vec3 {
            x: 50.0,
            y: 85.0,
            z: 81.0,
        },
        radius: 6.5,
        emission: Vec3 {
            x: 25.0,
            y: 20.0,
            z: 20.0,
        },
        color: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        material: MaterialType::Diffuse,
    });

    //LEFT
    sphere_list.push(sphere::Sphere {
        position: Vec3 {
            x: 1e3 + 1.0,
            y: 40.8,
            z: 81.6,
        },
        radius: 1e3,
        emission: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        color: Vec3 {
            x: 0.75,
            y: 0.25,
            z: 0.25,
        },
        material: MaterialType::Diffuse,
    });

    //RIGHT
    sphere_list.push(sphere::Sphere {
        position: Vec3 {
            x: -1e3 + 99.0,
            y: 40.8,
            z: 81.6,
        },
        radius: 1e3,
        emission: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        color: Vec3 {
            x: 0.25,
            y: 0.25,
            z: 0.75,
        },
        material: MaterialType::Diffuse,
    });

    //BACK

    sphere_list.push(sphere::Sphere {
        position: Vec3 {
            x: 50.0,
            y: 40.8,
            z: 1e3,
        },
        radius: 1e3,
        emission: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        color: Vec3 {
            x: 0.25,
            y: 0.75,
            z: 0.25,
        },
        material: MaterialType::Diffuse,
    });

    //FRONT

    sphere_list.push(sphere::Sphere {
        position: Vec3 {
            x: 50.0,
            y: 40.8,
            z: -1e3 + 170.0,
        },
        radius: 1e3,
        emission: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        color: Vec3 {
            x: 0.3,
            y: 0.3,
            z: 0.05,
        },
        material: MaterialType::Diffuse,
    });

    //BOTTOM
    sphere_list.push(sphere::Sphere {
        position: Vec3 {
            x: 50.0,
            y: 1e3,
            z: 81.6,
        },
        radius: 1e3,
        emission: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        color: Vec3 {
            x: 0.75,
            y: 0.75,
            z: 0.75,
        },
        material: MaterialType::Diffuse,
    });

    //TOP
    sphere_list.push(sphere::Sphere {
        position: Vec3 {
            x: 50.0,
            y: -2e3 + 81.6,
            z: 81.6,
        },
        radius: 2e3,
        emission: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        color: Vec3 {
            x: 0.75,
            y: 0.75,
            z: 0.75,
        },
        material: MaterialType::Diffuse,
    });

    //MIRROR
    sphere_list.push(sphere::Sphere {
        position: Vec3 {
            x: 27.0,
            y: 16.5,
            z: 47.0,
        },
        radius: 16.5,
        emission: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        color: Vec3 {
            x: 0.5,
            y: 0.5,
            z: 0.5,
        },
        material: MaterialType::Specular,
    });

    //BLUE mirror
    sphere_list.push(sphere::Sphere {
        position: Vec3 {
            x: 50.0,
            y: 42.5,
            z: 81.0,
        },
        radius: 12.5,
        emission: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        color: Vec3 {
            x: 0.5,
            y: 0.5,
            z: 0.999,
        },
        material: MaterialType::Specular,
    });

    //glass light
    sphere_list.push(sphere::Sphere {
        position: Vec3 {
            x: 50.0,
            y: 14.5,
            z: 81.0,
        },
        radius: 6.5,
        emission: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        color: Vec3 {
            x: 0.999,
            y: 0.999,
            z: 0.999,
        },
        material: MaterialType::Refractive,
    });

    //GLASS
    sphere_list.push(sphere::Sphere {
        position: Vec3 {
            x: 73.0,
            y: 16.5,
            z: 98.0,
        },
        radius: 16.5,
        emission: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        color: Vec3 {
            x: 0.999,
            y: 0.99,
            z: 0.999,
        },
        material: MaterialType::Refractive,
    });
    return sphere_list;
}