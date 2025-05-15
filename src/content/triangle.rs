use std::ops::Sub;
use nalgebra::{Point3, Vector2, Vector3};
use crate::core::Ray;
use crate::scene::Intersection;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>, // TODO: Hmmm
    pub uv: Vector2<f32>,
}

#[derive(Clone)]
pub struct Triangle {
    vertices: [Vertex; 3],
    material_id: usize,
}

pub struct TriangleIntersection {
    pub triangle: Triangle,
    pub dist: f32,
    pub barycentric: Vector2<f32>,
}

impl Triangle {
    pub fn new(vertices: [Vertex; 3], material_id: usize) -> Self {
        Self { vertices, material_id }
    }

    pub fn v0(&self) -> Vertex { self.vertices[0] }
    pub fn v1(&self) -> Vertex { self.vertices[1] }
    pub fn v2(&self) -> Vertex { self.vertices[2] }

    pub fn intersect(&self, ray: &Ray) -> Option<TriangleIntersection> {
        let epsilon = 1e-8;

        let edge1 = self.vertices[1].position.sub(self.vertices[0].position);
        let edge2 = self.vertices[2].position.sub(self.vertices[0].position);

        let h = ray.direction().cross(&edge2);
        let a = edge1.dot(&h);
        if a > -epsilon && a < epsilon {
            return None; // This ray is parallel to this triangle.
        }

        let f = 1.0 / a;
        let s = ray.origin().sub(self.vertices[0].position);
        let u = f * s.dot(&h);
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = s.cross(&edge1);
        let v = f * ray.direction().dot(&q);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        // At this stage we can compute t to find out where the intersection point is on the line.
        let t = f * edge2.dot(&q);
        if t > epsilon {
            Some(TriangleIntersection { triangle: self.clone(), dist: t, barycentric: Vector2::new(u, v)}) // intersection at t along the ray with barycentric coords u,v
        } else {
            None // Line intersection but not a ray intersection.
        }
    }
}

/*



 */