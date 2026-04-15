use std::ops::Sub;
use nalgebra::{Point3, Vector2, Vector3, Vector4};
use crate::core::Ray;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>, // TODO: Hmmm
    pub tangent: Vector4<f32>,
    pub uv: Vector2<f32>,
}

impl Vertex {
    pub fn transform(&self, transform: &nalgebra::Matrix4<f32>) -> Vertex {
        let normal_matrix = transform
            .fixed_view::<3, 3>(0, 0)
            .into_owned()
            .try_inverse()
            .unwrap()
            .transpose();
        let orientation_sign = if transform.fixed_view::<3, 3>(0, 0).into_owned().determinant() < 0.0 {
            -1.0
        } else {
            1.0
        };

        let normal = (normal_matrix * self.normal).normalize();
        let tangent_dir = normal_matrix * self.tangent.xyz();
        let tangent = if tangent_dir.norm_squared() <= 1e-12 {
            nalgebra::Vector4::new(0.0, 0.0, 0.0, self.tangent.w * orientation_sign)
        } else {
            tangent_dir.normalize().insert_row(3, self.tangent.w * orientation_sign)
        };

        Vertex {
            position: transform.transform_point(&self.position),
            normal,
            tangent,
            uv: self.uv,
        }
    }
}

#[derive(Clone)]
pub struct Triangle {
    vertices: [Vertex; 3],
}

pub struct TriangleIntersection {
    pub triangle: Triangle,
    pub dist: f32,
    pub barycentric: Vector2<f32>,
}

impl Triangle {
    pub fn new(vertices: [Vertex; 3],) -> Self {
        Self {
            vertices,
        }
    }

    pub fn v0(&self) -> Vertex { self.vertices[0] }
    pub fn v1(&self) -> Vertex { self.vertices[1] }
    pub fn v2(&self) -> Vertex { self.vertices[2] }

    pub fn intersect(&self, ray: &Ray) -> Option<TriangleIntersection> {
        let epsilon = 1e-8;

        // TODO: Performance test precomputed edges for larger scenes
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

    pub fn transform(&self, transform: &nalgebra::Matrix4<f32>) -> Triangle {
        Triangle::new([
            self.v0().transform(transform),
            self.v1().transform(transform),
            self.v2().transform(transform),
        ])
    }

    pub fn sample_uniform_point(&self, rng: &mut impl rand::Rng) -> (Point3<f32>, Vector3<f32>) {
        let u: f32 = rng.random();
        let v: f32 = rng.random();

        // This transformation ensures uniform distribution across the triangle
        let sqrt_u = u.sqrt();
        let bary_u = 1.0 - sqrt_u;
        let bary_v = v * sqrt_u;
        let bary_w = 1.0 - bary_u - bary_v;

        // Interpolate position
        let point = self.vertices[0].position.coords * bary_u
            + self.vertices[1].position.coords * bary_v
            + self.vertices[2].position.coords * bary_w;

        // Interpolate normal
        let normal = self.vertices[0].normal * bary_u
            + self.vertices[1].normal * bary_v
            + self.vertices[2].normal * bary_w;

        (<Point3<f32>>::from(point), normal)
    }
}

/*



 */