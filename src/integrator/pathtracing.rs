use std::ops::Mul;
use nalgebra::Vector3;
use rand::Rng;
use rayon::iter::IntoParallelIterator;
use crate::camera::viewpoint::Viewpoint;
use crate::core::Ray;
use crate::frame::Frame;
use crate::integrator::integrator::Integrator;
use crate::scene::scene::Scene;
use rayon::prelude::*;
use crate::scene::{Shadeable, ShadingContext};

pub struct PathTracingIntegrator {}

impl PathTracingIntegrator {
    pub fn new() -> Self {
        Self {}
    }

    fn shade(hit: &ShadingContext, ray: &Ray, scene: &Scene, depth: u32, rng: &mut impl Rng) -> Vector3<f32> {
        let u = hit.intersection.tex_coord.x.rem_euclid(1.0);
        let v = hit.intersection.tex_coord.y.rem_euclid(1.0);
        let emissive = hit.material.sample_emissive(u, v).component_mul(&Vector3::repeat(10.0));
        let albedo = hit.material.sample_color(u, v);
        let hit_point = ray.origin() + ray.direction() * hit.intersection.dist;
        let surface_point = hit_point + hit.intersection.normal * 0.001; // Offset along normal, not ray direction

        // Direct lighting: explicitly sample light sources
        let mut direct_light = Vector3::zeros();
        if let Some((light_point, light_normal, light_emissive, light_pdf)) = scene.sample_light(rng) {
            // Direction and distance to light
            let to_light =  light_point - surface_point;
            let distance_sq = to_light.magnitude_squared();
            let light_dir = to_light.normalize();

            // Cosine terms
            let cos_theta = hit.intersection.normal.dot(&light_dir).max(0.0);
            let cos_theta_light = light_normal.dot(&(-light_dir)).max(0.0);
            if cos_theta > 0.0 && cos_theta_light > 0.0 {
                // Cast shadow ray to check visibility
                if scene.is_visible(surface_point, light_point) {
                    // Direct lighting: incoming radiance from light, without albedo yet
                    // L_direct = Le * (1/π) * cos_theta * cos_theta_light / (distance^2 * pdf)
                    direct_light = (light_emissive * cos_theta * cos_theta_light) / (distance_sq * light_pdf);
                }
            }
        }

        // Indirect lighting: BSDF sampling for next bounce
        let sample = hit.material.sample_lambertian_bsdf(ray.direction(), hit.intersection.normal, rng);
        let new_ray = Ray::new(surface_point, sample.direction);
        let indirect_light = Self::trace(&new_ray, scene, depth-1, rng);

        // Combine: emissive + direct*albedo + indirect*albedo
        // Direct light gets modulated by albedo here
        // Indirect light gets modulated by albedo because it represents incoming radiance that needs to be reflected
        emissive + (direct_light + indirect_light).component_mul(&albedo)
    }

    fn trace(ray: &Ray, scene: &Scene, depth: u32, rng: &mut impl Rng) -> Vector3<f32> {
        if depth == 0 {
            return Vector3::zeros()
        }

        scene.intersect(ray).map(|hit| {
            Self::shade(&hit, ray, scene, depth, rng)
        }).unwrap_or_else(|| scene.environment(&ray))
    }
}

impl Integrator for PathTracingIntegrator {
    fn integrate(&self, scene: &Scene, frame: &mut Frame, samples: u32) {
        // TODO: Can this "threading boilerplate" be moved outside the integrator, so every dont have to do the same thing?
        let width = frame.width() as usize;
        let height = frame.height() as usize;

        println!("Rendering {} samples", samples);
        let scanlines = (0..height).into_par_iter().map(|y| {
            let mut rng = rand::rng();
            let mut pixels = vec![Vector3::new(0.0, 0.0, 0.0); width];
            let v = y as f32 / height as f32;
            for x in 0..width {
                let u = x as f32 / width as f32;

                for _ in 0..samples {
                    let ray = scene.camera.generate_ray(1.0 - u, 1.0 - v);

                    let result = Self::trace(&ray, scene, 4, &mut rng);

                    pixels[x] += result * (1.0 / samples as f32);
                }
            }

            pixels
        }).collect::<Vec<Vec<Vector3<f32>>>>();

        for y in 0..height {
            for x in 0..width {
                frame.add_sample(x, y, scanlines[y][x])
            }
        }
    }
}