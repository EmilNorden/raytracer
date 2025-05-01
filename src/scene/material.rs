use nalgebra::Vector3;

#[derive(Copy, Clone)]
pub struct Material {
    /*
    - Some BRDF should be attached => Determines how it interacts with light
    - Also, some way to get albedo (color) at specific point. A reference to a texture?
     */

    color: Vector3<f32>
}

impl Material {
    pub fn new(color: Vector3<f32>) -> Self {
        Self { color}
    }

    pub fn color(&self) -> Vector3<f32> { self.color }
}