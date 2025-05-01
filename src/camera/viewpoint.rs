use crate::core::Ray;

pub trait Viewpoint {
    fn generate_ray(&self, u: f32, v: f32) -> Ray;
}