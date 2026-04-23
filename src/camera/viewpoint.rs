use rand::Rng;
use crate::core::Ray;

pub trait Viewpoint {
    fn generate_ray(&self, u: f32, v: f32) -> Ray;

    #[allow(dead_code)]
    fn generate_offset_ray(&self, u: f32, v: f32, radius: f32, focal_distance: f32, rng: &mut impl Rng) -> Ray;
}