mod sphere;
mod ray;
mod sampling;
mod specular;

use std::time::{Instant};
use chrono;
use ultraviolet::Vec3;
use rand::Rng;
use image;
//use rayon::prelude::*;

use crate::ray::Ray;
use crate::sphere::{Sphere, MaterialType, EPSILON_SPHERE};
use crate::specular::{ideal_specular_reflect, ideal_specular_transmit};
use crate::sampling::cosine_weighted_sample_on_hemisphere;


const WIDTH: usize = 512;
const HEIGHT: usize = 512;
const TEXTURE_SIZE: usize = WIDTH * HEIGHT;
const SAMPLE: usize = 256;

const FOV: f32 = 0.5135;
const REFRACTIVE_INDEX_OUT: f32 = 1.0;
const REFRACTIVE_INDEX_IN: f32 = 1.5;
const MIN_BOUNCE: usize = 5;
const MAX_BOUNCE: usize = 30;


fn main() {
    let now = Instant::now();   //     println!("{}", now.elapsed().as_secs());
    println!("Starting, at {}", chrono::offset::Local::now());

    let eye = Vec3::new(50.0, 52.0, 295.6);
    let gaze = Vec3::new(0.0, -0.042612, -1.0).normalized();
    let cx = Vec3::new((FOV * (WIDTH as f32)) / (HEIGHT as f32), 0.0, 0.0);
    let cy = cx.cross(gaze).normalized() * FOV;

    let mut texture = std::vec![Vec3::zero(); TEXTURE_SIZE];
    let mut sphere_list: Vec<Sphere> = Vec::new();

    //LIGHT
    sphere_list.push(sphere::Sphere {
        position: Vec3 { x: 50.0, y: -3.0, z: 81.0 },
        radius: 6.5,
        emission: Vec3 { x: 30.0, y: 30.0, z: 30.0 },
        color: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        material: MaterialType::Diffuse,
    });

    //LEFT
    sphere_list.push(sphere::Sphere {
        position: Vec3 { x: 1e3 + 1.0, y: 40.8, z: 81.6 },
        radius: 1e3,
        emission: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        color: Vec3 { x: 0.75, y: 0.25, z: 0.25 },
        material: MaterialType::Diffuse,
    });

    //RIGHT
    sphere_list.push(sphere::Sphere {
        position: Vec3 { x: -1e3 + 99.0, y: 40.8, z: 81.6 },
        radius: 1e3,
        emission: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        color: Vec3 { x: 0.25, y: 0.25, z: 0.75 },
        material: MaterialType::Diffuse,
    });

    //BACK
    sphere_list.push(sphere::Sphere {
        position: Vec3 { x: 50.0, y: 40.8, z: 1e3 },
        radius: 1e3,
        emission: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        color: Vec3 { x: 0.25, y: 0.75, z: 0.25 },
        material: MaterialType::Diffuse,
    });

    //FRONT
    sphere_list.push(sphere::Sphere {
        position: Vec3 { x: 50.0, y: 40.8, z: -1e3 + 170.0},
        radius: 1e3,
        emission: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        color: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        material: MaterialType::Diffuse,
    });

    //BOTTOM
    sphere_list.push(sphere::Sphere {
        position: Vec3 { x: 50.0, y: 1e3, z: 81.6},
        radius: 1e3,
        emission: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        color: Vec3 { x: 0.75, y: 0.75, z: 0.75 },
        material: MaterialType::Diffuse,
    });

    //TOP
    sphere_list.push(sphere::Sphere {
        position: Vec3 { x: 50.0, y: -1e3 + 81.6, z: 81.6},
        radius: 1e3,
        emission: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        color: Vec3 { x: 0.75, y: 0.75, z: 0.75 },
        material: MaterialType::Diffuse,
    });

    //MIRROR
    sphere_list.push(sphere::Sphere {
        position: Vec3 { x: 27.0, y: 16.5, z: 47.0},
        radius: 16.5,
        emission: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        color: Vec3 { x: 0.5, y: 0.5, z: 0.999 },
        material: MaterialType::Specular,
    });

    //BLUE GLASS
    sphere_list.push(sphere::Sphere {
        position: Vec3 { x: 50.0, y: 12.5, z: 81.0},
        radius: 6.5,
        emission: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        color: Vec3 { x: 0.5, y: 0.5, z: 0.999 },
        material: MaterialType::Refractive,
    });

    //GLASS
    sphere_list.push(sphere::Sphere {
        position: Vec3 { x: 73.0, y: 16.5, z: 78.0},
        radius: 16.5,
        emission: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        color: Vec3 { x: 0.999, y: 0.99, z: 0.999 },
        material: MaterialType::Refractive,
    });


    for i in 0..HEIGHT {
        render(i, WIDTH, HEIGHT, SAMPLE, eye, gaze, cx, cy, &mut texture, &sphere_list);
    }
//        (0..HEIGHT).into_par_iter().for_each(|i| render(i, WIDTH, HEIGHT, SAMPLE, eye, gaze, cx, cy, &mut texture, &spherelist));

    let mut buffer: [u8;WIDTH*HEIGHT*3] = [0 as u8;WIDTH*HEIGHT*3];
    for i in 0..texture.len() {
        buffer[i*3] = (texture[i].x.powf(1.0/2.2) * 255.0).clamp(0.0,255.0) as u8;
        buffer[i*3+1] = (texture[i].y.powf(1.0/2.2) * 255.0).clamp(0.0,255.0) as u8;
        buffer[i*3+2] = (texture[i].z.powf(1.0/2.2) * 255.0).clamp(0.0,255.0) as u8;
    }
    image::save_buffer("image.png", &buffer, WIDTH as u32, HEIGHT as u32, image::ColorType::Rgb8).unwrap();

    println!("Render time : {}s", now.elapsed().as_secs())
}

fn render(column: usize, width: usize, height: usize, nbsample: usize, eye: Vec3, gaze: Vec3, cx: Vec3, cy: Vec3, texture: &mut Vec<Vec3>, sph: &Vec<Sphere>) {

    let mut rng = rand::thread_rng();
    let mut luminance: Vec3;

    for rowindex in 0..width {

        let mut columnindex = 0;
        let i = (height - 1 - column) * width + rowindex;

        while columnindex < 2 {
            let mut subpixelrow = 0;

            while subpixelrow < 2 {
                luminance = Vec3::zero();

                for _ in 0..nbsample {
                    let u1 = 2f32 * rng.gen::<f32>();
                    let u2 = 2f32 * rng.gen::<f32>();
                    let dx = if u1 < 1.0 { u1.sqrt() - 1.0 } else { 1.0 - (2.0 - u1).sqrt() };
                    let dy = if u2 < 1.0 { u2.sqrt() - 1.0 } else { 1.0 - (2.0 - u2).sqrt() };
                    let d = cx * (((subpixelrow as f32 + 0.5 + dx) / 2f32 + rowindex as f32) / width as f32 - 0.5) +
                        cy * (((columnindex as f32 + 0.5 + dy) / 2f32 + column as f32) / height as f32 - 0.5) + gaze;

                    luminance += radiance(&mut Ray {
                        origin: eye + d * 130f32,
                        direction: d.normalized(),
                        tmin: EPSILON_SPHERE,
                        tmax: f32::INFINITY,
                        depth: 0,
                    }, &sph) * (1f32 / nbsample as f32);
                }
                texture[i] += 0.25 * luminance.clamped(Vec3 { x: 0f32, y: 0f32, z: 0f32 }, Vec3 { x: 1f32, y: 1f32, z: 1f32 });
                subpixelrow += 1;
            }
            columnindex += 1;
        }
    }
}

fn intersect(ray: &mut Ray, id: &mut usize, sphere: &Vec<Sphere>) -> bool {
    *id = 0;
    let mut hit = false;
/*
    let o = sphere.iter().position(|sphere| sphere.intersect(ray));
    if let Some(pos) = o {
        *id = pos;
        hit = true
    }
 */
    for i in 0..sphere.len() {
        if sphere[i].intersect(ray) == false {
            continue;
        } else {
            hit = true;
            *id = i;
        }
    }

    return hit;
}

fn radiance(ray: &mut Ray, sphere: &Vec<Sphere>) -> Vec3 {

    let mut rng = rand::thread_rng();
    let mut luminance: Vec3 = Vec3::zero();
    let mut color: Vec3 = Vec3::one();
    let mut id: usize = 0;

    loop {
        if intersect(ray, &mut id, sphere) == false {
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
            MaterialType::Specular => {     //mirror
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
                let d = ideal_specular_transmit(ray.direction, n, REFRACTIVE_INDEX_OUT, REFRACTIVE_INDEX_IN, &mut pr);
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
                let u = if w.x.abs() > 0.1 { Vec3 { x: 0f32, y: 1f32, z: 0f32 } } else { Vec3 { x: 1f32, y: 0f32, z: 0f32 }.cross(w).normalized() };
                let v = w.cross(u);
                let sample_distance = cosine_weighted_sample_on_hemisphere(rng.gen::<f32>(), rng.gen::<f32>());
                let distance = (sample_distance.x * u + sample_distance.y * v + sample_distance.z * w).normalized();
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