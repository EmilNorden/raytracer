use nalgebra::{Matrix4, Vector3};
use crate::camera::perspective_camera::PerspectiveCamera;
use crate::content::mesh::Mesh;
use crate::core::Ray;
use crate::scene::{Intersectable, Shadeable, Intersection, ShadingContext};

pub struct SceneNode {
    pub transform: Matrix4<f32>,
    pub meshes: Vec<u32>,
    pub children: Vec<SceneNode>,
}
pub struct Scene {
    pub camera: PerspectiveCamera, // TODO: Replace with camera trait
    meshes: Vec<Mesh>,
    emissive_meshes: Vec<Mesh>,
}

impl Scene {
    pub fn new(camera: PerspectiveCamera, meshes: Vec<Mesh>) -> Self {
        let mut emissive_meshes = Vec::new();
        for mesh in &meshes {
            if mesh.material().emissive_factor() != Vector3::zeros() {
                emissive_meshes.push(mesh.clone());
            }
        }
        Self {
            camera,
            meshes,
            emissive_meshes,
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<ShadingContext> {
        let mut best_dist = f32::MAX;
        let mut best_hit = None;
        for object in &self.meshes {

            if let Some(hit) = object.intersect(ray, 0.0, f32::MAX) {
                if hit.dist < best_dist {
                    best_dist = hit.dist;
                    best_hit = Some(ShadingContext {
                        ray: ray.clone(),
                        intersection: hit,
                        material: object.material(),
                        mesh_index: object.mesh_index()
                    });
                }
            }
        }

        best_hit
    }

    pub fn emissive_meshes(&self) -> &Vec<Mesh> {
        &self.emissive_meshes
    }
}

pub struct SceneObject {
    pub inverse_world: Matrix4<f32>,
    pub geometry: Box<dyn Intersectable>,
}