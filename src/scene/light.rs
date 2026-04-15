use nalgebra::{Point3, Vector3};
use crate::content::mesh::MeshInstance;

pub enum LightSource {
    Point(PointLight),
    Mesh(MeshInstance),
}

pub struct PointLight {
    pub color: Vector3<f32>,
    pub intensity: f32,
    pub position: Point3<f32>,
    pub radius: f32,
}

impl PointLight {
    pub fn new(position: Point3<f32>, color: Vector3<f32>, intensity: f32, radius: f32) -> Self {
        Self {
            color,
            intensity,
            position,
            radius,
        }
    }
}
