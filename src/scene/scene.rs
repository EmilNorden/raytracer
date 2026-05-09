use std::fmt::Display;
use crate::acceleration::bvh::BVH;
use crate::camera::perspective_camera::PerspectiveCamera;
use crate::content::mesh::MeshInstance;
use crate::core::Ray;
use crate::scene::light::LightSource;
use crate::scene::{Intersectable, Shadeable, ShadingContext};
use nalgebra::{Point3, Vector3};
use crate::context::Context;
use crate::scene::material::Material;

pub struct Scene {
    cameras : Vec<PerspectiveCamera>,
    meshes: Vec<MeshInstance>,
    bvh: BVH,
    lights: Vec<LightSource>,
    materials: Vec<Material>,
}

pub struct LightSample {
    pub wi: Vector3<f32>,
    pub radiance: Vector3<f32>,
    pub pdf: f32,
    pub is_delta: bool,
    pub position: Option<Point3<f32>>,
}

impl Display for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Scene with {} cameras, {} meshes, {} lights. Total triangles: {}", self.cameras.len(), self.meshes.len(), self.lights.len(), self.triangle_count())
    }
}

impl Scene {
    pub fn new(cameras: Vec<PerspectiveCamera>, mut meshes: Vec<MeshInstance>, materials: Vec<Material>, mut lights: Vec<LightSource>) -> Self {
        for mesh in &meshes {
            let material = &materials[mesh.material_index() as usize];
            if material.emissive_factor().x > 0.0 || material.emissive_factor().y > 0.0 || material.emissive_factor().z > 0.0 {
                lights.push(LightSource::Mesh(mesh.clone()));
            }
        }

        let bvh = BVH::new(&mut meshes);


        Self {
            cameras,
            meshes,
            bvh,
            lights,
            materials,
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

    pub fn materials(&self) -> &[Material] { &self.materials }

    pub fn lights_mut(&mut self) -> &mut [LightSource] {
        &mut self.lights
    }

    pub fn intersect(&'_ self, ray: &Ray, ctx: &Context) -> Option<ShadingContext> {
       self.bvh.intersect(self.meshes.as_slice(), ray, ctx).map(|(mesh_index, hit)| {
            ShadingContext {
                intersection: hit,
                material_index: self.meshes[mesh_index as usize].material_index(),
            }
        })
    }

    pub fn lights(&self) -> &Vec<LightSource> {
        &self.lights
    }

    pub fn environment(&self, _: &Ray) -> Vector3<f32> {
        Vector3::zeros()
    }

    /// Sample a random point on a random emissive surface
    /// Returns (point, normal, emissive_color, pdf)
    pub fn sample_light(&self, rng: &mut impl rand::Rng) -> Option<LightSample> {
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

                let radiance = point_light.color * point_light.intensity;

                Some(LightSample {
                    wi: normal,
                    radiance,
                    pdf,
                    is_delta: false,
                    position: Some(point),
                })
                //Some((point, normal, emissive, pdf))

            },
            LightSource::Directional(directional_light) => {
                let normal = -directional_light.direction.normalize(); // Light comes from this direction

                let pdf = 1.0;

                let radiance = directional_light.color * directional_light.intensity;

                Some(LightSample {
                    wi: normal,
                    radiance,
                    pdf,
                    is_delta: true,
                    position: None
                })
                //Some((point, normal, emissive, pdf))
            },
            LightSource::Mesh(mesh) => {
                // Sample a random point on the mesh by sampling a random triangle
                let triangle_index = rng.random_range(..mesh.triangle_count() as usize);

                let triangle = mesh.triangle_at(triangle_index);

                let (point, normal) = triangle.sample_uniform_point(rng);

                let material = &self.materials[mesh.material_index() as usize];
                let radiance = material.emissive_factor();

                // PDF is 1/area. For now, use a rough estimate
                let bounds = mesh.bounds();
                let area = (bounds.max().x - bounds.min().x) * (bounds.max().z - bounds.min().z);
                let pdf = 1.0 / area;

                Some(LightSample {
                    wi: normal,
                    radiance,
                    pdf,
                    is_delta: false,
                    position: Some(point),
                })
                //Some((point, normal, emissive, pdf))
            }
        }
    }

    pub fn transmission_along_path(&self, p1: Point3<f32>, p2: Point3<f32>, ctx: &Context) -> f32 {
        let direction = p2 - p1;
        let distance = direction.norm();
        if distance <= 1e-5 {
            return 1.0;
        }

        let ray = Ray::new(p1.into(), direction / distance);
        let t_min = 0.001;
        let t_max = (distance - 0.001).max(0.0);

        if let Some((mesh_index, hit)) =
            self.bvh.intersect_with_limits(self.meshes.as_slice(), &ray, t_min, t_max, ctx) {

            let mesh = &self.meshes[mesh_index as usize];
            let material = &self.materials[mesh.material_index() as usize];
            if material.transmission_factor() > 0.0 {
                let transformed_bounds = mesh.bounds().transform(mesh.transform());
                let new_p1 = ray.origin() + (ray.direction() * transformed_bounds.intersect(&ray).unwrap().tmax);

                let subpath = self.transmission_along_path(new_p1, p2, ctx);

                return material.transmission_factor() * subpath;
            }

            return 0.0;
        }

        1.0
    }

    fn transmission_along_path_inner(&self, p1: Point3<f32>, p2: Point3<f32>, ctx: &Context) -> Option<f32> {
        unimplemented!()
    }

    /// Check if there's an unoccluded path between two points
    pub fn is_visible(&self, p1: Point3<f32>, p2: Point3<f32>, ctx: &Context) -> bool {
        let direction = p2 - p1;
        let distance = direction.norm();
        if distance <= 1e-5 {
            return true;
        }

        let ray = Ray::new(p1.into(), direction / distance);
        let t_min = 0.001;
        let t_max = (distance - 0.001).max(0.0);

        self.bvh
            .intersect_with_limits(self.meshes.as_slice(), &ray, t_min, t_max, ctx)
            .is_none()
    }

    pub fn triangle_count(&self) -> usize {
        self.meshes.iter().map(|mesh| mesh.triangle_count()).sum()
    }
}