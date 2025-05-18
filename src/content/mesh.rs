use std::path::Path;
use std::sync::Arc;
use nalgebra::{Point3, Vector2, Vector3};
use crate::acceleration::bounds::AABB;
use crate::acceleration::kdtree::KDTree;
use crate::content::triangle::Triangle;
use crate::core::Ray;
use crate::scene::{Intersectable, Intersection, Shadeable};
use crate::scene::material::Material;
use crate::scene::scene::Scene;

pub struct MeshData {
    //bounds: AABB,
    //triangles: Vec<Triangle>,
    geometry: KDTree,
    material: Material,
}
#[derive(Clone)]
pub struct Mesh {
    mesh_index: usize,
    data: Arc<MeshData>,
    position: Point3<f32>,
    inverse_transform: nalgebra::Matrix4<f32>,
}

impl MeshData {
    pub fn new<I: IntoIterator<Item = Triangle>>(triangle_iter: I, material: Material) -> Self {
        let triangles: Vec<Triangle> = triangle_iter.into_iter().collect();

        /*let bounds = AABB::from_points(
            triangles.iter().map(|tri| [tri.v0().position, tri.v1().position, tri.v2().position]).flatten()
        );*/

        Self {
            //bounds,
            //triangles: triangles.into_iter().collect(),
            geometry: KDTree::new(triangles),
            material
        }
    }

    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        /*let mut best_dist = f32::MAX;
        let mut closest_intersection = None;
        for triangle in &self.triangles {
            if let Some(intersection) = triangle.intersect(ray) {
                if intersection.dist < best_dist {
                    best_dist = intersection.dist;
                    closest_intersection = Some(intersection);
                }
            }
        }*/

       let closest_intersection = self.geometry.intersects(ray);

        closest_intersection.map(|x| {
            // TODO: Should I only return the barycentric UV coordinates and the triangle, and only interpolate these parameters once I have found the true intersection?
            let tex_coord0 = x.triangle.v0().uv;
            let tex_coord1 = x.triangle.v1().uv;
            let tex_coord2 = x.triangle.v2().uv;

            let w = 1.0 - x.barycentric.x - x.barycentric.y;

            let tex_coord = tex_coord0 * w + tex_coord1 * x.barycentric.x + tex_coord2 * x.barycentric.y;

            let normal0 = x.triangle.v0().normal;
            let normal1 = x.triangle.v1().normal;
            let normal2 = x.triangle.v2().normal;

            let normal = normal0 * w + normal1 * x.barycentric.x + normal2 * x.barycentric.y;

            Intersection {
                dist: x.dist,
                tex_coord,
                normal
            }
        })
    }

    /*pub fn triangles(&self) -> &[Triangle] {
        self.triangles.as_slice()
    }*/

    pub fn bounds(&self) -> AABB {
        self.geometry.bounds()
    }
}


impl Mesh {
    pub fn new(mesh_index: usize, data: Arc<MeshData>, position: Point3<f32>, inverse_transform: nalgebra::Matrix4<f32>) -> Self {
        Self {
            mesh_index,
            data,
            position,
            inverse_transform,
        }
    }

    pub fn mesh_index(&self) -> usize {
        self.mesh_index
    }

    pub fn position(&self) -> Point3<f32> {
        self.position
    }
}

impl Intersectable for Mesh {
    fn bounds(&self) -> AABB {
        self.data.bounds()
    }

    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        let object_space_ray = ray.transform(self.inverse_transform);

        /*self.data.bounds.intersect(&object_space_ray)
            .and_then(|_| { self.data.intersect(&object_space_ray, t_min, t_max) })*/

        self.data.intersect(&object_space_ray, t_min, t_max)
        // TODO: Have to re-calculate intersection point and tdist.
    }
}

impl Shadeable for Mesh {
    fn material(&self) -> &Material {
        &self.data.material
    }
}