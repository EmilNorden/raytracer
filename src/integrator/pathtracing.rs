use crate::core::Ray;
use crate::frame::Frame;
use crate::integrator::integrator::Integrator;
use crate::camera::viewpoint::Viewpoint;
use crate::scene::scene::Scene;
use crate::scene::ShadingContext;
use nalgebra::{Vector2, Vector3};
use rand::Rng;
use rayon::prelude::*;
use crate::scene::material::CachedTextureLookups;

pub struct PathTracingIntegrator {}

const MAX_BOUNCES: u32 = 32;
const RR_WARMUP_BOUNCES: u32 = 3;

impl PathTracingIntegrator {
    pub fn new() -> Self {
        Self {}
    }

    fn shade(
        hit: &ShadingContext,
        ray: &Ray,
        scene: &Scene,
        remaining_depth: u32,
        bounce_index: u32,
        rng: &mut impl Rng,
    ) -> Vector3<f32> {
        let u = hit.intersection.tex_coord.x.rem_euclid(1.0);
        let v = hit.intersection.tex_coord.y.rem_euclid(1.0);
        let tex_coords = Vector2::new(u, v);
        let mut cached_textures = CachedTextureLookups::new(&hit.material, tex_coords);
        let albedo = cached_textures.albedo();
        let hit_point = ray.origin() + ray.direction() * hit.intersection.dist;
        let surface_point = hit_point + hit.intersection.normal * 0.001; // Offset along normal, not ray direction

        let normal = hit.material.apply_normal_map(hit.intersection.normal, hit.intersection.tangent, tex_coords);

        // Direct lighting: explicitly sample light sources
        let mut direct_light = Vector3::zeros();
        if let Some((light_point, light_normal, light_emissive, light_pdf)) =
            scene.sample_light(rng)
        {
            // Direction and distance to light
            let to_light = light_point - surface_point;
            let distance_sq = to_light.magnitude_squared();
            let light_dir = to_light.normalize();

            // Cosine terms
            let cos_theta = normal.dot(&light_dir).max(0.0);
            let cos_theta_light = light_normal.dot(&(-light_dir)).max(0.0);
            if cos_theta > 0.0 && cos_theta_light > 0.0 {
                // Cast shadow ray to check visibility
                if scene.is_visible(surface_point, light_point) {
                    let view_dir = -ray.direction();
                    let brdf =
                        hit.material
                            .evaluate_bsdf(&light_dir, &view_dir, &normal, &albedo, &mut cached_textures);
                    direct_light = (light_emissive * (cos_theta_light / (distance_sq * light_pdf)))
                        .component_mul(&brdf)
                        * cos_theta;
                }
            }
        }

        // Indirect lighting: BSDF sampling for next bounce
        let sample =
            hit.material
                .sample_bsdf(ray.direction(), normal, albedo, &mut cached_textures, rng);

        // Offset based on outgoing hemisphere relative to the geometric normal.
        // This works for reflection and for both entering/exiting transmission.
        let n = normal;
        let offset_sign = if sample.direction.dot(&n) >= 0.0 {
            1.0
        } else {
            -1.0
        };

        let cos_theta = if sample.is_transmission {
            sample.direction.dot(&n).abs()
        } else {
            sample.direction.dot(&n).max(0.0)
        };

        const MIN_PDF: f32 = 1e-5;

        let weighted_contribution = if sample.pdf > MIN_PDF && cos_theta > 0.0 {
            let indirect_origin = hit_point + n * (0.001 * offset_sign);
            let new_ray = Ray::new(indirect_origin, sample.direction);

            // Compute survival probability for Russian roulette
            // Use max component of (BSDF * cos_theta) as a proxy for path importance
            let bsdf_weighted = sample.bsdf_value.component_mul(&Vector3::new(cos_theta, cos_theta, cos_theta));
            let max_component = bsdf_weighted.x.max(bsdf_weighted.y).max(bsdf_weighted.z);
            let survival_prob = if bounce_index < RR_WARMUP_BOUNCES {
                1.0 // Continue with probability 1.0 for early bounces
            } else {
                max_component.min(1.0) // Clamp to [0,1]
            };

            // Russian roulette termination
            if rng.random::<f32>() > survival_prob {
                Vector3::zeros() // Path terminated
            } else {
                let indirect_light = Self::trace(&new_ray, scene, remaining_depth - 1, bounce_index + 1, rng);
                // Re-weight by survival probability to maintain unbiasedness
                (indirect_light.component_mul(&sample.bsdf_value) * cos_theta) / (sample.pdf * survival_prob)
            }
        } else {
            Vector3::zeros()
        };

        let emissive = cached_textures.emissive();

        // Combine: emissive + direct*albedo + indirect*albedo
        // Direct light gets modulated by albedo here
        // Indirect light gets modulated by albedo because it represents incoming radiance that needs to be reflected
        emissive + direct_light + weighted_contribution
    }

    fn trace(
        ray: &Ray,
        scene: &Scene,
        remaining_depth: u32,
        bounce_index: u32,
        rng: &mut impl Rng,
    ) -> Vector3<f32> {
        // Safety cap: avoid pathological recursion (Russian roulette handles practical termination)
        if remaining_depth == 0 {
            return Vector3::zeros();
        }

        scene
            .intersect(ray)
            .map(|hit| Self::shade(&hit, ray, scene, remaining_depth, bounce_index, rng))
            .unwrap_or_else(|| scene.environment(&ray))
    }
}

impl Integrator for PathTracingIntegrator {
    fn integrate(&self, scene: &Scene, frame: &mut Frame, samples: u32) {
        // TODO: Can this "threading boilerplate" be moved outside the integrator, so every dont have to do the same thing?
        let width = frame.width() as usize;
        let height = frame.height() as usize;

        let height_inv = 1.0 / height as f32;
        let width_inv = 1.0 / width as f32;
        let samples_inv = 1.0 / samples as f32;

        frame.pixels_mut().par_chunks_mut(width).enumerate().for_each(|(y, row)| {
            let mut rng = rand::rng();
            let v = y as f32 * height_inv;
            for x in 0..width {
                let u = x as f32 * width_inv;

                let ray = scene.active_camera().generate_ray(1.0 - u, 1.0 - v);
                //let ray = scene.camera.generate_offset_ray(1.0 - u, 1.0 - v, 0.4, 16.0, &mut rng);

                let result = Self::trace(&ray, scene, MAX_BOUNCES, 0, &mut rng);

                row[x] += result * samples_inv;
            }

        });
    }
}
