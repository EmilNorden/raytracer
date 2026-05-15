use crate::camera::viewpoint::Viewpoint;
use crate::context::Context;
use crate::core::Ray;
use crate::frame::Frame;
use crate::integrator::integrator::Integrator;
use crate::math;
use crate::scene::ShadingContext;
use crate::scene::material::{CachedTextureLookups, IOR_AIR};
use crate::scene::scene::Scene;
use crate::static_stack::StaticStack;
use nalgebra::Vector3;
use rand::Rng;
use rayon::prelude::*;
use crate::consts::ETA_STACK_SIZE;
use crate::options::{FocalDistance, RenderOptions};

pub struct PathTracingIntegrator {}

struct ShadeResult {
    radiance: Vector3<f32>,
    next_ray: Option<Ray>,
    throughput: Vector3<f32>,
}

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
        eta_stack: &mut StaticStack<f32, ETA_STACK_SIZE>,
        ctx: &Context,
    ) -> ShadeResult {
        let tex_coords = hit.intersection.tex_coord;
        let material = &scene.materials()[hit.material_index as usize];
        let mut cached_textures = CachedTextureLookups::new(&material, tex_coords);
        let albedo = cached_textures.albedo();
        let hit_point = ray.origin() + ray.direction() * hit.intersection.dist;
        let surface_point = hit_point + hit.intersection.normal * 0.001; // Offset along normal, not ray direction

        let normal = material.apply_normal_map(
            hit.intersection.normal,
            hit.intersection.tangent,
            tex_coords,
        );

        // Direct lighting: explicitly sample light sources
        let mut direct_light = Vector3::zeros();
        if let Some(light_sample) = scene.sample_light(rng) {
            if light_sample.is_delta {
                if let Some(light_point) = light_sample.position {
                    // Delta point light contribution.
                    let to_light = light_point - surface_point;
                    let distance_sq = to_light.magnitude_squared();

                    if distance_sq > 1e-12 {
                        let light_dir = to_light.normalize();
                        let cos_theta = normal.dot(&light_dir).max(0.0);

                        if cos_theta > 0.0 {
                            let transmission =
                                scene.transmissions_along_path_2(surface_point, light_point, ctx);
                            if math::is_greater_than_zero(transmission) {
                                let view_dir = -ray.direction();
                                let brdf = material.evaluate_bsdf(
                                    &light_dir,
                                    &view_dir,
                                    &normal,
                                    &albedo,
                                    &mut cached_textures,
                                );

                                direct_light = (light_sample.radiance / distance_sq)
                                    .component_mul(&brdf)
                                    .component_mul(&transmission)
                                    * cos_theta;
                            }
                        }
                    }
                } else {
                    // Handle delta light (e.g., directional light) contribution
                    let light_dir = light_sample.wi; // Light comes from this direction
                    let cos_theta = normal.dot(&light_dir).max(0.0);
                    if cos_theta > 0.0 {
                        // Cast shadow ray to check visibility
                        let shadow_ray = Ray::new(surface_point, light_dir);
                        if scene.intersect(&shadow_ray, ctx).is_none() {
                            let view_dir = -ray.direction();
                            let brdf = material.evaluate_bsdf(
                                &light_dir,
                                &view_dir,
                                &normal,
                                &albedo,
                                &mut cached_textures,
                            );
                            direct_light = (light_sample.radiance / light_sample.pdf)
                                .component_mul(&brdf)
                                * cos_theta;
                        }
                    }
                }
            } else if let Some(light_point) = light_sample.position {
                // Area light contribution.
                let to_light = light_point - surface_point;
                let distance_sq = to_light.magnitude_squared();
                let light_dir = to_light.normalize();

                // Cosine terms
                let cos_theta = normal.dot(&light_dir).max(0.0);
                let cos_theta_light = light_sample.wi.dot(&(-light_dir)).max(0.0);

                if cos_theta > 0.0 && cos_theta_light > 0.0 {
                    let transmission =
                        scene.transmissions_along_path_2(surface_point, light_point, ctx);
                    if math::is_greater_than_zero(transmission) {
                        let view_dir = -ray.direction();
                        let brdf = material.evaluate_bsdf(
                            &light_dir,
                            &view_dir,
                            &normal,
                            &albedo,
                            &mut cached_textures,
                        );
                        direct_light = (light_sample.radiance
                            * (cos_theta_light / (distance_sq * light_sample.pdf)))
                            .component_mul(&brdf)
                            .component_mul(&transmission)
                            * cos_theta;
                    }
                }
            }
        }

        let emissive = cached_textures.emissive();
        let radiance = emissive + direct_light;

        if remaining_depth <= 1 {
            return ShadeResult {
                radiance,
                next_ray: None,
                throughput: Vector3::zeros(),
            };
        }

        // Indirect lighting: BSDF sampling for next bounce.
        let sample = material.sample_bsdf(
            ray.direction(),
            normal,
            albedo,
            &mut cached_textures,
            rng,
            eta_stack,
            ctx,
        );

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

        if sample.pdf <= MIN_PDF || cos_theta <= 0.0 {
            return ShadeResult {
                radiance,
                next_ray: None,
                throughput: Vector3::zeros(),
            };
        }

        let indirect_origin = hit_point + n * (0.001 * offset_sign);
        let next_ray = Ray::new(indirect_origin, sample.direction);

        // Compute survival probability for Russian roulette.
        // Use max component of (BSDF * cos_theta) as a proxy for path importance.
        let bsdf_weighted = sample.bsdf_value * cos_theta;
        let max_component = bsdf_weighted.x.max(bsdf_weighted.y).max(bsdf_weighted.z);
        let survival_prob = if bounce_index < RR_WARMUP_BOUNCES {
            1.0
        } else {
            max_component.min(1.0)
        };

        if survival_prob <= 0.0 || rng.random::<f32>() > survival_prob {
            return ShadeResult {
                radiance,
                next_ray: None,
                throughput: Vector3::zeros(),
            };
        }

        ShadeResult {
            radiance,
            next_ray: Some(next_ray),
            throughput: sample.bsdf_value * (cos_theta / (sample.pdf * survival_prob)),
        }
    }

    fn trace(
        ray: &Ray,
        scene: &Scene,
        remaining_depth: u32,
        bounce_index: u32,
        rng: &mut impl Rng,
        eta_stack: &mut StaticStack<f32, ETA_STACK_SIZE>,
        ctx: &Context,
    ) -> Vector3<f32> {
        if remaining_depth == 0 {
            return Vector3::zeros();
        }

        let mut ray = ray.clone();
        let mut remaining_depth = remaining_depth;
        let mut bounce_index = bounce_index;
        let mut throughput = Vector3::new(1.0, 1.0, 1.0);
        let mut radiance = Vector3::zeros();

        while remaining_depth > 0 {
            let Some(hit) = scene.intersect(&ray, ctx) else {
                radiance += throughput.component_mul(&scene.environment(&ray));
                break;
            };

            let shade = Self::shade(
                &hit,
                &ray,
                scene,
                remaining_depth,
                bounce_index,
                rng,
                eta_stack,
                ctx,
            );

            radiance += throughput.component_mul(&shade.radiance);

            let Some(next_ray) = shade.next_ray else {
                break;
            };

            throughput = throughput.component_mul(&shade.throughput);
            if !math::is_greater_than_zero(throughput) {
                break;
            }

            ray = next_ray;
            remaining_depth -= 1;
            bounce_index += 1;
        }

        radiance
    }
}

impl Integrator for PathTracingIntegrator {
    fn integrate(&self, scene: &Scene, frame: &mut Frame, samples: u32, options: &RenderOptions, ctx: &Context) {
        // TODO: Can this "threading boilerplate" be moved outside the integrator, so every dont have to do the same thing?
        let width = frame.width() as usize;
        let height = frame.height() as usize;

        let height_inv = 1.0 / height as f32;
        let width_inv = 1.0 / width as f32;
        let samples_inv = 1.0 / samples as f32;

        let mut camera = scene.active_camera().clone();
        if let Some(dof) = &options.depth_of_field {
            match dof.focal_distance {
                FocalDistance::Fixed(val) => camera.set_focal_distance(val),
                FocalDistance::Auto(u, v) => {
                    let focus_ray = scene.active_camera().generate_ray(u, v);
                    if let Some(focus_hit) = scene.intersect(&focus_ray, ctx) {
                        camera.set_focal_distance(focus_hit.intersection.dist)
                    }
                }
            }
        }


        frame
            .pixels_mut()
            .par_chunks_mut(width)
            .enumerate()
            .for_each(|(y, row)| {
                let mut rng = rand::rng();
                let v = y as f32 * height_inv;
                for x in 0..width {
                    let u = x as f32 * width_inv;

                    //let ray = scene.active_camera().generate_ray(1.0 - u, 1.0 - v);
                    let ray = camera.generate_offset_ray(1.0 - u, 1.0 - v, &mut rng);

                    // Assume initial eta = 1.000277 (Air) for all rays
                    let mut eta_stack = StaticStack::<f32, ETA_STACK_SIZE>::new_with_default(IOR_AIR);

                    let result =
                        Self::trace(&ray, scene, MAX_BOUNCES, 0, &mut rng, &mut eta_stack, ctx);

                    row[x] += result * samples_inv;
                }
            });
    }
}
