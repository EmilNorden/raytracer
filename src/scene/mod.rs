
use nalgebra::{Matrix, Matrix4, Point3, Vector3};
use crate::acceleration::bounds::AABB;
use crate::core::Ray;
use crate::scene::material::Material;

pub mod material;
pub mod scene;
mod transform;

pub struct Intersection {
    pub dist: f32,
    pub u: f32,
    pub v: f32,
}

pub trait Intersectable {
    fn bounds(&self) -> AABB;
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection>;
}

pub trait Shadeable {
    fn material(&self) -> Material;
}

pub struct SceneObject {
    world: Matrix4<f32>,
    inverse_world: Matrix4<f32>,
}


impl SceneObject {

}

pub struct Sphere {
    pub position: Point3<f32>,
    pub radius: f32,
    pub material: Material,
}

impl Intersectable for Sphere {
    fn bounds(&self) -> AABB {
        AABB::new(self.position + Vector3::new(self.radius, self.radius, self.radius), self.position + Vector3::new(self.radius, self.radius, self.radius))
    }

    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        let oc = ray.origin() - self.position;
        let a = ray.direction().dot(&ray.direction());

        let half_b = oc.dot(&ray.direction());
        let c = oc.dot(&oc) - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range
        let mut root = (-half_b - sqrt_d) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrt_d) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }

        Some(Intersection { dist: root, u: 0.0, v: 0.0 })
    }
}

impl Shadeable for Sphere {
    fn material(&self) -> Material {
        self.material
    }
}