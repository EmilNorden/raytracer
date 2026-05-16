use std::fmt::Display;
use crate::acceleration::bvh::BVH;
use crate::camera::perspective_camera::PerspectiveCamera;
use crate::content::mesh::MeshInstance;
use crate::core::Ray;
use crate::scene::light::LightSource;
use crate::scene::{Intersectable, Intersection, Shadeable, ShadingContext};
use nalgebra::{Point3, Vector3};
use crate::context::Context;
use crate::math::lerp;
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

pub struct PathIntersection {
    pub mesh_index: u32,
    pub intersection: Intersection,
    pub material_index: u32,
}

pub struct PathIntersectionsIter<'a> {
    scene: &'a Scene,
    ray: Ray,
    t_min: f32,
    t_max: f32,
    ctx: &'a Context,
    done: bool,
    hit_count: usize,
}

impl<'a> Iterator for PathIntersectionsIter<'a> {
    type Item = PathIntersection;

    fn next(&mut self) -> Option<Self::Item> {
        const MAX_INTERSECTIONS: usize = 128;
        const RAY_EPSILON: f32 = 1e-4;

        if self.done || self.hit_count >= MAX_INTERSECTIONS || self.t_min >= self.t_max {
            self.done = true;
            return None;
        }

        let Some((mesh_index, hit)) = self
            .scene
            .bvh
            .intersect_with_limits(self.scene.meshes.as_slice(), &self.ray, self.t_min, self.t_max, self.ctx)
        else {
            self.done = true;
            return None;
        };

        self.hit_count += 1;
        let next_t_min = (hit.dist + RAY_EPSILON).max(self.t_min + RAY_EPSILON);
        if !next_t_min.is_finite() || next_t_min >= self.t_max {
            self.done = true;
        } else {
            self.t_min = next_t_min;
        }

        let material_index = self.scene.meshes[mesh_index as usize].material_index();
        Some(PathIntersection {
            mesh_index,
            intersection: hit,
            material_index,
        })
    }
}

impl Display for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Scene with {} cameras, {} meshes, {} lights. Total triangles: {}", self.cameras.len(), self.meshes.len(), self.lights.len(), self.triangle_count())
    }
}

impl Scene {
    pub fn intersections_along_path<'a>(&'a self, ray: Ray, distance: f32, ctx: &'a Context) -> PathIntersectionsIter<'a> {

        let t_min = 0.001;
        let t_max = (distance - 0.001).max(0.0);
        let done = t_max <= t_min;

        PathIntersectionsIter {
            scene: self,
            ray,
            t_min,
            t_max,
            ctx,
            done,
            hit_count: 0,
        }
    }

    pub fn new(cameras: Vec<PerspectiveCamera>, mut meshes: Vec<MeshInstance>, materials: Vec<Material>, mut lights: Vec<LightSource>) -> Self {
        for mesh in &meshes {
            let material = &materials[mesh.material_index() as usize];
            if material.emissive_factor().x > 0.0 || material.emissive_factor().y > 0.0 || material.emissive_factor().z > 0.0 {
                lights.push(LightSource::Mesh(mesh.clone()));
            }
        }

        let bvh = BVH::new(&mut meshes, &materials);


        Self {
            cameras,
            meshes,
            bvh,
            lights,
            materials,
        }
    }

    pub fn rebuild_bvh(&mut self) {
        self.bvh = BVH::new(&mut self.meshes, &self.materials);
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
                let radiance = point_light.color * point_light.intensity;

                if point_light.radius <= 0.0 {
                    return Some(LightSample {
                        wi: Vector3::zeros(),
                        radiance,
                        pdf: 1.0,
                        is_delta: true,
                        position: Some(point_light.position),
                    });
                }

                let direction = loop {
                    let u = (rng.random::<f32>() * 2.0) - 1.0;
                    let v = (rng.random::<f32>() * 2.0) - 1.0;
                    let w = (rng.random::<f32>() * 2.0) - 1.0;
                    let candidate = Vector3::new(u, v, w);
                    if candidate.norm_squared() > 1e-12 {
                        break candidate.normalize();
                    }
                };

                let point = point_light.position + direction * point_light.radius;
                let normal = direction;
                let area = 4.0 * std::f32::consts::PI * point_light.radius * point_light.radius;
                let pdf = 1.0 / area;

                Some(LightSample {
                    wi: normal,
                    radiance: radiance / area,
                    pdf,
                    is_delta: false,
                    position: Some(point),
                })
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

    pub fn transmissions_along_path_2(&self, start: Point3<f32>, end: Point3<f32>, ctx: &Context) -> Vector3<f32> {
        let mut throughput = Vector3::new(1.0, 1.0, 1.0);

        let direction = end - start;
        let distance = direction.norm();

        if distance <=  1e-5 {
            return throughput;
        }

        let ray = Ray::new(start, direction / distance);
        let t_min = 0.001;
        let t_max = (distance - 0.001).max(0.0);
        if !self.bvh.might_intersect_transparent_objects(&ray, t_min, t_max, ctx) {
            return if self.is_visible(start, end, ctx) {
                Vector3::repeat(1.0)
            } else {
                Vector3::zeros()
            }
        }

        for intersection in self.intersections_along_path(ray, distance, ctx) {
            let mesh = &self.meshes[intersection.mesh_index as usize];
            let material = &self.materials[mesh.material_index() as usize];
            let transmission = material.transmission_factor();
            if transmission <= 1e-5 {
                return Vector3::zeros();
            }

            let tex_coords = intersection.intersection.tex_coord;
            let albedo = lerp(
                material.sample_color(tex_coords.x, tex_coords.y),
                Vector3::new(1.0, 1.0, 1.0),
                transmission,
            );
            throughput = throughput.component_mul(&(albedo * transmission));
        }

        throughput
    }

    pub fn transmission_along_path(&self, p1: Point3<f32>, p2: Point3<f32>, ctx: &Context) -> Vector3<f32> {
        let direction = p2 - p1;
        let distance = direction.norm();
        if distance <= 1e-5 {
            return Vector3::new(1.0, 1.0, 1.0);
        }

        let ray = Ray::new(p1.into(), direction / distance);
        let mut t_min = 0.001;
        let t_max = (distance - 0.001).max(0.0);
        if t_max <= t_min {
            return Vector3::new(1.0, 1.0, 1.0);
        }

        const MAX_TRANSMISSION_HITS: usize = 128;
        const RAY_EPSILON: f32 = 1e-4;

        let mut throughput = Vector3::new(1.0, 1.0, 1.0);
        let mut hit_count = 0usize;

        while t_min < t_max && hit_count < MAX_TRANSMISSION_HITS {
            let Some((mesh_index, hit)) =
                self.bvh.intersect_with_limits(self.meshes.as_slice(), &ray, t_min, t_max, ctx)
            else {
                break;
            };

            hit_count += 1;

            let mesh = &self.meshes[mesh_index as usize];
            let material = &self.materials[mesh.material_index() as usize];
            let transmission = material.transmission_factor().clamp(0.0, 1.0);
            if transmission <= 0.0 {
                return Vector3::zeros();
            }

            let tex_coords = hit.tex_coord;
            let albedo = lerp(
                material.sample_color(tex_coords.x, tex_coords.y),
                Vector3::new(1.0, 1.0, 1.0),
                transmission,
            );
            throughput = throughput.component_mul(&(albedo * transmission));

            let next_t_min = (hit.dist + RAY_EPSILON).max(t_min + RAY_EPSILON);
            if !next_t_min.is_finite() {
                break;
            }
            t_min = next_t_min;
        }

        throughput
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