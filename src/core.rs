use nalgebra::{Matrix4, Point3, Vector3};

pub struct Ray {
    origin: Point3<f32>,
    direction: Vector3<f32>,
}

impl Ray {
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>) -> Ray {
        Ray { origin, direction }
    }

    pub fn origin(&self) -> Point3<f32> { self.origin }
    pub fn direction(&self) -> Vector3<f32> { self.direction }

    pub fn transform(&self, matrix: Matrix4<f32>) -> Ray {
        let origin = matrix.transform_point(&self.origin);
        let direction = matrix.transform_vector(&self.direction);
        Ray { origin, direction }
    }
}