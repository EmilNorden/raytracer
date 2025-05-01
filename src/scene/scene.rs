use nalgebra::Matrix4;
use crate::content::mesh::Mesh;
use crate::core::Ray;
use crate::scene::{Intersectable, Intersection};

pub struct SceneNode {
    pub transform: Matrix4<f32>,
    pub meshes: Vec<u32>,
    pub children: Vec<SceneNode>,
}
pub struct Scene {
    meshes: Vec<Mesh>,
    root: SceneNode,
}

impl Scene {
    pub fn new(meshes: Vec<Mesh>, root: SceneNode) -> Self {
        Self {
            meshes,
            root,
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        /*let mut best_dist = f32::MAX;
        let mut best_hit = None;
        for object in &self.objects {
            let world_ray = ray.transform(object.inverse_world);

            if let Some(hit) = object.geometry.intersect(&world_ray, 0.0, f32::MAX) {
                if hit.dist < best_dist {
                    best_dist = hit.dist;
                    best_hit = Some(hit);
                }
            }
        }
        best_hit

         */
        None
    }
}

pub struct SceneObject {
    pub inverse_world: Matrix4<f32>,
    pub geometry: Box<dyn Intersectable>,
}