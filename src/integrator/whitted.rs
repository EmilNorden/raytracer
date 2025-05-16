
use nalgebra::Vector3;
use rayon::prelude::*;
use crate::camera::viewpoint::Viewpoint;
use crate::frame::Frame;
use crate::integrator::integrator::Integrator;
use crate::scene::scene::Scene;

pub struct WhittedIntegrator;

impl WhittedIntegrator {
    pub fn new() -> Self {
        Self
    }
}
impl Integrator for WhittedIntegrator {
    fn integrate(&self, scene: &Scene, frame: &mut Frame) {
        let width = frame.width() as usize;
        let height = frame.height() as usize;
        println!("Rendering start");
        let scanlines = (0..height).into_par_iter().map(|y| {
            let mut pixels = vec![Vector3::new(0.0, 0.0, 0.0); width as usize];
            let v = y as f32 / height as f32;
            for x in 0..width {
                let u = x as f32 / width as f32;

                let ray = scene.camera.generate_ray(1.0-u, 1.0-v);

                let mut result = Vector3::new(1.0, 1.0, 1.0);
                if let Some(hit) = scene.intersect(&ray) {
                    result = hit.material.sample_color_bilinear(hit.intersection.tex_coord.x, hit.intersection.tex_coord.y);
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
