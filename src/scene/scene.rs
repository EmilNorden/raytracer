use nalgebra::{Matrix4, Point3, Vector3};
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

    pub fn environment(&self, ray: &Ray) -> Vector3<f32> {
        Vector3::zeros()
    }

    /// Sample a random point on a random emissive surface
    /// Returns (point, normal, emissive_color, pdf)
    pub fn sample_light(&self, rng: &mut impl rand::Rng) -> Option<(Point3<f32>, Vector3<f32>, Vector3<f32>, f32)> {
        if self.emissive_meshes.is_empty() {
            return None;
        }

        let mesh_index = rng.random_range(0..self.emissive_meshes.len());
        let mesh = &self.emissive_meshes[mesh_index];

        // Sample a random point on the mesh by sampling a random triangle
        let triangle_index = rng.random_range(..mesh.triangle_count() as usize);

        let triangle = mesh.triangle_at(triangle_index);

        let (point, normal) = triangle.sample_uniform_point(rng);


        let emissive = mesh.material().emissive_factor();

        // PDF is 1/area. For now, use a rough estimate 
        let bounds = mesh.bounds();
        let area = (bounds.max().x - bounds.min().x) * (bounds.max().y - bounds.min().y);
        let pdf = 1.0 / area;

        Some((point, normal, emissive, pdf))
    }

    /// Check if there's an unoccluded path between two points
    pub fn is_visible(&self, p1: Point3<f32>, p2: Point3<f32>) -> bool {
        let direction = p2 - p1;
        let distance = direction.norm();
        let ray = Ray::new(p1.into(), direction / distance);

        // Cast shadow ray
        if let Some(hit) = self.intersect(&ray) {
            // If we hit something before reaching the light, it's occluded
            hit.intersection.dist >= distance - 0.001
        } else {
            true
        }
    }
}

pub struct SceneObject {
    pub inverse_world: Matrix4<f32>,
    pub geometry: Box<dyn Intersectable>,
}