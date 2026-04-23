use rayon::iter::ParallelIterator;
use nalgebra::{Vector2, Vector3};
use rayon::iter::IntoParallelIterator;
use crate::camera::viewpoint::Viewpoint;
use crate::frame::Frame;
use crate::integrator::integrator::Integrator;
use crate::scene::scene::Scene;

pub struct DebugIntegrator {}

impl DebugIntegrator {
    pub fn new() -> Self {
        Self{}
    }
}

impl Integrator for DebugIntegrator {
    fn integrate(&self, scene: &Scene, frame: &mut Frame, _samples: u32) {
        let width = frame.width() as usize;
        let height = frame.height() as usize;

        let scanlines = (0..height).into_par_iter().map(|y| {
            let mut pixels = vec![Vector3::new(0.0, 0.0, 0.0); width];
            let v = y as f32 / height as f32;
            for x in 0..width {
                let u = x as f32 / width as f32;

                    let ray = scene.active_camera().generate_ray(1.0 - u, 1.0 - v);

                    if let Some(hit) = scene.intersect(&ray) {
                        let u = hit.intersection.tex_coord.x.rem_euclid(1.0);
                        let v = hit.intersection.tex_coord.y.rem_euclid(1.0);
                        let tex_coords = Vector2::new(u, v);
                        let normal = hit.material.apply_normal_map(hit.intersection.normal, hit.intersection.tangent, tex_coords);

                        pixels[x] += normal;
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