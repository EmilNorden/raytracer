use nalgebra::Vector3;
use crate::scene::Intersection;
use crate::scene::texture::Texture;

pub struct Material {
    /*
    - Some BRDF should be attached => Determines how it interacts with light
    - Also, some way to get albedo (color) at specific point. A reference to a texture?
     */

    color: Vector3<f32>,
    texture: Option<Texture>,
}

impl Material {
    pub fn new(color: Vector3<f32>, texture: Option<Texture>) -> Self {
        Self { color, texture}
    }

    pub fn color(&self) -> Vector3<f32> { self.color }
    
    pub fn sample_color_bilinear(&self, u: f32, v: f32) -> Vector3<f32> {
        self.texture.as_ref().map(|t| t.sample_color(u, v)).unwrap_or(self.color)
    }
}