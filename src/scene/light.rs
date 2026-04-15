use nalgebra::Point3;
use crate::content::mesh::MeshInstance;

enum LightSource {
    Point(PointLight),
    Mesh(MeshInstance),
}

struct PointLight {
    color: Point3<f32>,
    intensity: f32,
    position: Point3<f32>,
    radius: f32,
}