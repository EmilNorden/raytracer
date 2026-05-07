use crate::acceleration::bvh::BVH;
use crate::camera::perspective_camera::PerspectiveCamera;
use crate::content::mesh::MeshInstance;
use crate::core::Ray;
use crate::scene::light::LightSource;
use crate::scene::{Intersectable, Shadeable, ShadingContext};
use nalgebra::{Point3, Vector3};

pub struct Scene {
    cameras : Vec<PerspectiveCamera>,
    meshes: Vec<MeshInstance>,
    bvh: BVH,
    lights: Vec<LightSource>,
}

pub struct LightSample {
    pub wi: Vector3<f32>,
    pub radiance: Vector3<f32>,
    pub pdf: f32,
    pub is_delta: bool,
    pub position: Option<Point3<f32>>,
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

    pub fn intersect(&'_ self, ray: &Ray) -> Option<ShadingContext<'_>> {
       self.bvh.intersect(self.meshes.as_slice(), ray).map(|(mesh_index, hit)| {
            ShadingContext {
                intersection: hit,
                material: self.meshes[mesh_index as usize].material(),
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


                let radiance = mesh.material().emissive_factor();

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