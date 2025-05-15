use std::path::Path;
use std::sync::Arc;
use nalgebra::{Point3, Vector2};
use crate::acceleration::bounds::AABB;
use crate::content::triangle::Triangle;
use crate::core::Ray;
use crate::scene::{Intersectable, Intersection, Shadeable};
use crate::scene::material::Material;
use crate::scene::scene::Scene;

pub struct MeshData {
    bounds: AABB,
    triangles: Vec<Triangle>,
    material: Material,
}
pub struct Mesh {
    data: Arc<MeshData>,
    inverse_transform: nalgebra::Matrix4<f32>,
}

impl MeshData {
    pub fn new<I: IntoIterator<Item = Triangle>>(triangle_iter: I, material: Material) -> Self {
        let triangles: Vec<Triangle> = triangle_iter.into_iter().collect();
        let mut bounds = AABB::new(Point3::new(f32::MAX, f32::MAX, f32::MAX), Point3::new(f32::MIN, f32::MIN, f32::MIN));

        for tri in &triangles {
            bounds.expand(tri.v0().position);
            bounds.expand(tri.v1().position);
            bounds.expand(tri.v2().position);
        }

        Self {
            bounds,
            triangles: triangles.into_iter().collect(),
            material
        }
    }

    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        let mut best_dist = f32::MAX;
        let mut closest_intersection = None;
        for triangle in &self.triangles {
            if let Some(intersection) = triangle.intersect(ray) {
                if intersection.dist < best_dist {
                    best_dist = intersection.dist;
                    closest_intersection = Some(intersection);
                }
            }
        }

        closest_intersection.map(|x| {

            let tex_coord0 = x.triangle.v0().uv;
            let tex_coord1 = x.triangle.v1().uv;
            let tex_coord2 = x.triangle.v2().uv;

            let w = 1.0 - x.barycentric.x - x.barycentric.y;

            let tex_coord = tex_coord0 * x.barycentric.x + tex_coord1 * x.barycentric.y + tex_coord2 * w;

            Intersection {
                dist: x.dist,
                tex_coord,
            }
        })
    }

    pub fn triangles(&self) -> &[Triangle] {
        self.triangles.as_slice()
    }
    pub fn bounds(&self) -> AABB {
        self.bounds
    }
}


impl Mesh {
    pub fn new(data: Arc<MeshData>, inverse_transform: nalgebra::Matrix4<f32>) -> Self {
        Self {
            data,
            inverse_transform,
        }
    }
}

impl Intersectable for Mesh {
    fn bounds(&self) -> AABB {
        self.data.bounds()
    }

    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        let object_space_ray = ray.transform(self.inverse_transform);

        self.data.bounds.intersect(&object_space_ray)
            .and_then(|_| { self.data.intersect(&object_space_ray, t_min, t_max) })
        // TODO: Have to re-calculate intersection point and tdist.
    }
}

impl Shadeable for Mesh {
    fn material(&self) -> &Material {
        &self.data.material
    }
}