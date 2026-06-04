use crate::content::mesh::MeshInstance;
use crate::scene::cdf::CDF;
use nalgebra::{Point3, Vector3};

pub struct EmissiveMesh {
    pub mesh: MeshInstance,
    pub cdf: CDF,
}

impl EmissiveMesh {
    pub fn new(mesh: &MeshInstance) -> Self {
        let cdf = CDF::new_from_iterator(
            mesh.triangles(),
            |triangle| triangle.area());

        Self { mesh: mesh.clone(), cdf }
    }

    pub fn total_area(&self) -> f32 { self.cdf.total_weight() }
}

pub enum LightSource {
    Point(PointLight),
    Directional(DirectionalLight),
    Mesh(EmissiveMesh),
}

impl LightSource {
    pub fn update_transform(&mut self, transform: nalgebra::Matrix4<f32>) {
        match self {
            LightSource::Point(light) => {
                light.position = transform.transform_point(&Point3::origin());
            },
            LightSource::Directional(_) => {
                // Do nothing
            }
            LightSource::Mesh(emissive_mesh) => {
                // TODO: Should we also update the CDF?
                emissive_mesh.mesh.update_transform(transform);
            },
        }
    }
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

pub struct DirectionalLight {
    pub color: Vector3<f32>,
    pub intensity: f32,
    pub direction: Vector3<f32>,
}

impl DirectionalLight {
    pub fn new(direction: Vector3<f32>, color: Vector3<f32>, intensity: f32) -> Self {
        Self {
            color,
            intensity,
            direction,
        }
    }
}