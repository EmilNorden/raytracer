use nalgebra::{Matrix4, Point3, Vector3};

#[derive(Debug, Clone)]
pub struct Ray {
    origin: Point3<f32>,
    direction: Vector3<f32>,
    direction_inv: Vector3<f32>,
}

impl Ray {
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>) -> Self {
        Self {
            origin,
            direction,
            direction_inv: Vector3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z),
        }
    }

    pub fn origin(&self) -> Point3<f32> { self.origin }
    pub fn direction(&self) -> Vector3<f32> { self.direction }
    pub fn direction_inv(&self) -> Vector3<f32> { self.direction_inv }

    pub fn transform(&self, matrix: Matrix4<f32>) -> Ray {
        let origin = matrix.transform_point(&self.origin);
        let direction = matrix.transform_vector(&self.direction);
        Ray::new(origin, direction)
    }
}