/*use crate::core::Ray;
use crate::frame::Frame;
use crate::integrator::integrator::Integrator;
use crate::camera::viewpoint::Viewpoint;
use crate::scene::scene::Scene;
use crate::scene::ShadingContext;
use nalgebra::{Point3, Vector2, Vector3, Vector4};
use rand::Rng;
use rayon::prelude::*;
use crate::camera::perspective_camera::PerspectiveCamera;
use crate::context::Context;
use crate::scene::material::{CachedTextureLookups, Material, IOR_AIR};
use crate::static_stack::StaticStack;

pub struct BidirectionalPathTracingIntegrator {}

const MAX_BOUNCES: u32 = 32;
const RR_WARMUP_BOUNCES: u32 = 3;

const EYE_SUBPATH_LENGTH: u32 = MAX_BOUNCES;

#[derive(Copy, Clone)]
struct SubpathVertex<'a> {
    position: Point3<f32>,
    normal: Vector3<f32>,
    tex_coords: Vector2<f32>,
    tangent: Vector4<f32>,
    material: &'a Material,
}

struct Subpath {
    vertices: [SubpathVertex; EYE_SUBPATH_LENGTH as usize],
    path_length: u32,
}

impl BidirectionalPathTracingIntegrator {
    pub fn new() -> Self {
        Self {}
    }

    fn create_eye_subpath(&self, camera_ray: Ray, scene: &Scene, ctx: &Context) -> Subpath {
        let mut subpath = Subpath {
            vertices: [SubpathVertex {}; EYE_SUBPATH_LENGTH as usize],
            path_length: 0,
        };

        let mut length = 0;
        let mut current_ray = camera_ray;
        scene.intersect(&current_ray, ctx).map(|hit| {
            subpath.vertices[length].position = current_ray.origin() + (current_ray.direction() * hit.intersection.dist);
            subpath.vertices[length].normal = hit.intersection.normal;
            subpath.vertices[length].tex_coords = hit.intersection.tex_coord;
            subpath.vertices[length].tangent = hit.intersection.tangent;
            length += 1;
        });
    }
}

impl Integrator for BidirectionalPathTracingIntegrator {
    fn integrate(&self, scene: &Scene, frame: &mut Frame, samples: u32, ctx: &Context) {
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

                // Assume initial eta = 1.000277 (Air) for all rays
                let mut eta_stack = StaticStack::<f32, 8>::new_with_default(IOR_AIR);


                row[x] += result * samples_inv;
            }

        });
    }
}*/