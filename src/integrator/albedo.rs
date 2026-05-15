use crate::camera::viewpoint::Viewpoint;
use crate::context::Context;
use crate::frame::Frame;
use crate::integrator::integrator::Integrator;
use crate::scene::scene::Scene;
use rayon::prelude::*;
use crate::options::{FocalDistance, RenderOptions};

pub struct AlbedoIntegrator {}

impl Integrator for AlbedoIntegrator {
    fn integrate(&self, scene: &Scene, frame: &mut Frame, _samples: u32, options: &RenderOptions, ctx: &Context) {
        let width = frame.width() as usize;
        let height = frame.height() as usize;

        let height_inv = 1.0 / height as f32;
        let width_inv = 1.0 / width as f32;

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

                    let ray = camera.generate_offset_ray(1.0 - u, 1.0 - v, &mut rng);
                    if let Some(hit) = scene.intersect(&ray, ctx) {
                        let u = hit.intersection.tex_coord.x;
                        let v = hit.intersection.tex_coord.y;
                        let material = &scene.materials()[hit.material_index as usize];
                        row[x] = material.sample_color(u, v);
                    }
                    else {
                        row[x] = scene.environment(&ray);
                    }
                }
            });
    }
}