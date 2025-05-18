
use nalgebra::{Point3, Vector3};
use rayon::prelude::*;
use crate::camera::viewpoint::Viewpoint;
use crate::core::Ray;
use crate::frame::Frame;
use crate::integrator::integrator::Integrator;
use crate::scene::scene::Scene;
use crate::scene::ShadingContext;

pub struct WhittedIntegrator;

impl WhittedIntegrator {
    pub fn new() -> Self {
        Self
    }

    fn shade(context: &ShadingContext) -> f32 {
        unimplemented!()
    }
}
impl Integrator for WhittedIntegrator {
    fn integrate(&self, scene: &Scene, frame: &mut Frame) {
        let width = frame.width() as usize;
        let height = frame.height() as usize;
        println!("Rendering start");
        let scanlines = (0..height).into_par_iter().map(|y| {
            let mut pixels = vec![Vector3::new(0.0, 0.0, 0.0); width];
            let v = y as f32 / height as f32;
            for x in 0..width {
                let u = x as f32 / width as f32;

                let ray = scene.camera.generate_ray(1.0-u, 1.0-v);

                let mut result = Vector3::new(1.0, 1.0, 1.0);
                if let Some(hit) = scene.intersect(&ray) {
                    let color = hit.material.sample_color(hit.intersection.tex_coord.x, hit.intersection.tex_coord.y);

                    //let mut light = Vector3::new(0.0, 0.0, 0.0);
                    let intersection_point = ray.origin() + ray.direction() * hit.intersection.dist;

                    let mut L_d = Vector3::new(0.0, 0.0, 0.0);
                    for light in scene.emissive_meshes() {
                        let light_dir = (light.position() - intersection_point).normalize();
                        let shadow_ray = Ray::new(intersection_point + light_dir * 0.05, light_dir);

                        if let Some(shadow_hit) = scene.intersect(&shadow_ray) {
                            if light.mesh_index() == shadow_hit.mesh_index {
                                L_d += shadow_hit.material.sample_emissive(shadow_hit.intersection.tex_coord.x, shadow_hit.intersection.tex_coord.y)
                                     * shadow_ray.direction().dot(&hit.intersection.normal).max(0.0);
                            }
                        }
                    }

                    result = hit.material.sample_color(hit.intersection.tex_coord.x, hit.intersection.tex_coord.y).component_mul(&L_d);
                }

                pixels[x] = result;
            }

            pixels
        })
            .collect::<Vec<Vec<Vector3<f32>>>>();

        for y in 0..height {
            for x in 0..width {
                frame.add_sample(x, y, scanlines[y][x])
            }
        }



        println!("Rendering end");
    }
}
