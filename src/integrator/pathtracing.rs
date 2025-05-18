use nalgebra::Vector3;
use rayon::iter::IntoParallelIterator;
use crate::camera::viewpoint::Viewpoint;
use crate::core::Ray;
use crate::frame::Frame;
use crate::integrator::integrator::Integrator;
use crate::scene::scene::Scene;

struct PathTracingIntegrator {}

impl PathTracingIntegrator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn trace(&self, ray: &Ray, scene: &Scene) {
        scene.intersect(ray).map(|hit| {
            //let albedo = hit.material.sample_color(hit.intersection.tex_coord.x, hit.intersection.tex_coord.y);

        });
    }
}

impl Integrator for PathTracingIntegrator {
    fn integrate(&self, scene: &Scene, frame: &mut Frame) {
        let width = frame.width() as usize;
        let height = frame.height() as usize;
        println!("Rendering start");
        /*let scanlines = (0..height).into_par_iter().map(|y| {
            let mut pixels = vec![Vector3::new(0.0, 0.0, 0.0); width];
            let v = y as f32 / height as f32;
            for x in 0..width {
                let u = x as f32 / width as f32;

                let ray = scene.camera.generate_ray(1.0 - u, 1.0 - v);

                let result = Vector3::new(0.0, 0.0, 0.0);
                if let Some(hit) = scene.intersect(&ray) {

                }
            }
        });*/
        unimplemented!()
    }
}