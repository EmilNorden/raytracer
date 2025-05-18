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
    emissive_texture: Option<Texture>,
    emissive: Vector3<f32>,
    roughness: f32,
}

impl Material {
    pub fn new(color: Vector3<f32>, texture: Option<Texture>, emissive_texture: Option<Texture>, emissive: Vector3<f32>, roughness: f32) -> Self {
        Self { color, texture, emissive_texture, emissive, roughness }
    }

    pub fn color(&self) -> Vector3<f32> { self.color }
    pub fn roughness(&self) -> f32 { self.roughness }
    pub fn emissive_factor(&self) -> Vector3<f32> { self.emissive }

    pub fn sample_color(&self, u: f32, v: f32) -> Vector3<f32> {
        self.texture.as_ref().map(|t| t.sample_color(u, v)).unwrap_or(self.color)
    }

    pub fn sample_emissive(&self, u: f32, v: f32) -> Vector3<f32> {
        self.emissive
        //self.emissive_texture.as_ref().map(|t| {t.sample_color(u, v).component_mul(&self.emissive)}).unwrap_or(self.emissive)
    }
}