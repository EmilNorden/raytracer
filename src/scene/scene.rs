use nalgebra::{Matrix4, Point3, Vector3};
use crate::acceleration::bvh::BVH;
use crate::animation::Animation;
use crate::camera::perspective_camera::PerspectiveCamera;
use crate::content::mesh::MeshInstance;
use crate::core::Ray;
use crate::scene::{Intersectable, Shadeable, ShadingContext};
use crate::scene::light::{LightSource, PointLight};
use crate::scene::node_graph::NodeGraph;

pub struct SceneNode {
    pub transform: Matrix4<f32>,
    pub meshes: Vec<u32>,
    pub children: Vec<SceneNode>,
}
pub struct Scene {
    cameras : Vec<PerspectiveCamera>,
    meshes: Vec<MeshInstance>,
    bvh: BVH,
    lights: Vec<LightSource>,
}

impl Scene {
    pub fn new(cameras: Vec<PerspectiveCamera>, mut meshes: Vec<MeshInstance>, mut lights: Vec<LightSource>) -> Self {
        for mesh in &meshes {
            if mesh.material().emissive_factor() != Vector3::zeros() {
                lights.push(LightSource::Mesh(mesh.clone()));
            }
        }

        let bvh = BVH::new(&mut meshes);


        Self {
            cameras,
            meshes,
            bvh,
            lights,
        }
    }

    pub fn rebuild_bvh(&mut self) {
        self.bvh = BVH::new(&mut self.meshes);
    }

    pub fn active_camera(&self) -> &PerspectiveCamera {
        &self.cameras[0]
    }

    pub fn cameras_mut(&mut self) -> &mut [PerspectiveCamera] {
        &mut self.cameras
    }

    pub fn meshes_mut(&mut self) -> &mut [MeshInstance] {
        &mut self.meshes
    }

    pub fn lights_mut(&mut self) -> &mut [LightSource] {
        &mut self.lights
    }

    pub fn intersect(&self, ray: &Ray) -> Option<ShadingContext> {
       self.bvh.intersect(self.meshes.as_slice(), ray).map(|(mesh_index, hit)| {
            ShadingContext {
                ray: ray.clone(),
                intersection: hit,
                material: self.meshes[mesh_index as usize].material(),
                mesh_index: self.meshes[mesh_index as usize].mesh_index(),
            }
        })
        /*let mut best_dist = f32::MAX;
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

        best_hit*/
    }

    pub fn lights(&self) -> &Vec<LightSource> {
        &self.lights
    }

    pub fn environment(&self, ray: &Ray) -> Vector3<f32> {
        Vector3::zeros()
    }

    /// Sample a random point on a random emissive surface
    /// Returns (point, normal, emissive_color, pdf)
    pub fn sample_light(&self, rng: &mut impl rand::Rng) -> Option<(Point3<f32>, Vector3<f32>, Vector3<f32>, f32)> {
        if self.lights.is_empty() {
            return None;
        }

        let light_index = rng.random_range(0..self.lights.len());
        let light = &self.lights[light_index];

        match light {
            LightSource::Point(point_light) => {
                let u = (rng.random::<f32>() * 2.0) - 1.0;
                let v = (rng.random::<f32>() * 2.0) - 1.0;
                let w = (rng.random::<f32>() * 2.0) - 1.0;
                let point = point_light.position + (Vector3::new(u, v, w) * point_light.radius);
                let normal = (point - point_light.position).normalize();

                let area = 4.0 * std::f32::consts::PI * point_light.radius * point_light.radius;
                let pdf = 1.0 / area;

                let emissive = point_light.color * point_light.intensity;

                Some((point, normal, emissive, pdf))

            }
            LightSource::Mesh(mesh) => {
                // Sample a random point on the mesh by sampling a random triangle
                let triangle_index = rng.random_range(..mesh.triangle_count() as usize);

                let triangle = mesh.triangle_at(triangle_index);

                let (point, normal) = triangle.sample_uniform_point(rng);


                let emissive = mesh.material().emissive_factor();

                // PDF is 1/area. For now, use a rough estimate
                let bounds = mesh.bounds();
                let area = (bounds.max().x - bounds.min().x) * (bounds.max().z - bounds.min().z);
                let pdf = 1.0 / area;

                Some((point, normal, emissive, pdf))
            }
        }
    }

    /// Check if there's an unoccluded path between two points
    pub fn is_visible(&self, p1: Point3<f32>, p2: Point3<f32>) -> bool {
        let direction = p2 - p1;
        let distance = direction.norm();
        if distance <= 1e-5 {
            return true;
        }

        let ray = Ray::new(p1.into(), direction / distance);
        let t_min = 0.001;
        let t_max = (distance - 0.001).max(0.0);

        self.bvh
            .intersect_with_limits(self.meshes.as_slice(), &ray, t_min, t_max)
            .is_none()
    }
}

pub struct SceneObject {
    pub inverse_world: Matrix4<f32>,
    pub geometry: Box<dyn Intersectable>,
}