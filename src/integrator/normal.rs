use crate::camera::perspective_camera::PerspectiveCamera;
use crate::camera::viewpoint::Viewpoint;
use crate::context::Context;
use crate::frame::Frame;
use crate::integrator::integrator::Integrator;
use crate::options::RenderOptions;
use crate::scene::scene::Scene;
use nalgebra::{Vector2, Vector3};
use rayon::prelude::ParallelSliceMut;
use rayon::prelude::*;

pub struct NormalIntegrator {}

impl NormalIntegrator {
    pub fn new() -> Self {
        Self{}
    }
}

impl Integrator for NormalIntegrator {
    fn integrate(&self, scene: &Scene, camera: &PerspectiveCamera, frame: &mut Frame, _samples: u32, options: &RenderOptions, ctx: &Context) {
        let width = frame.width() as usize;
        let height = frame.height() as usize;

        let height_inv = 1.0 / height as f32;
        let width_inv = 1.0 / width as f32;

        frame
            .pixels_mut()
            .par_chunks_mut(width)
            .enumerate()
            .for_each(|(y, row)| {
                let v = y as f32 * height_inv;
                for x in 0..width {
                    let mut rng = rand::rng();
                    let u = x as f32 * width_inv;

                    let ray = camera.generate_offset_ray(1.0 - u, 1.0 - v, &mut rng);
                    if let Some(hit) = scene.intersect(&ray, ctx) {
                        let u = hit.intersection.tex_coord.x;
                        let v = hit.intersection.tex_coord.y;
                        let tex_coord = Vector2::new(u, v);
                        let material = &scene.materials()[hit.material_index as usize];
                        let normal = material.apply_normal_map(hit.intersection.normal, hit.intersection.tangent, tex_coord);

                        row[x] = normal;
                    }
                    else {
                        row[x] = Vector3::zeros();
                    }
                }
            });
    }
}