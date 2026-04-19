use std::f32::consts::PI;
use nalgebra::{Matrix4, Point3, Vector2, Vector3};
use rand::Rng;
use crate::camera::viewpoint::Viewpoint;
use crate::core::Ray;
use crate::scene::node_graph::NodeTransform;

struct ViewPlane {
    base: Point3<f32>,
    size: Vector2<f32>,
    u_dir: Vector3<f32>,
    v_dir: Vector3<f32>,
}

impl ViewPlane {
    pub fn new(camera_origin: Point3<f32>, camera_direction: Vector3<f32>, camera_up: Vector3<f32>, yfov: f32, aspect_ratio: f32) -> Self {
        const PLANE_DISTANCE:f32 = 10.0;
        let plane_height = 2.0 * PLANE_DISTANCE * (yfov / 2.0).tan();
        let plane_width = plane_height * aspect_ratio;

        let n = (camera_direction * -1.0).normalize();

        let u_dir = camera_up.cross(&n).normalize();
        //let u_dir = n.cross(&camera_up).normalize();
        let v_dir = n.cross(&u_dir).normalize();

        let plane_center = camera_origin - (n * PLANE_DISTANCE);

        let base = plane_center +
            (u_dir * (plane_width / 2.0)) -
            (v_dir * (plane_height / 2.0));

        Self {
            base,
            size: Vector2::new(plane_width, plane_height),
            u_dir,
            v_dir
        }
    }

    pub fn get_coordinates_from_uv(&self, u: f32, v: f32) -> Point3<f32> {
        self.base - (self.u_dir * u * self.size.x) + (self.v_dir * v * self.size.y)
    }
}
pub struct PerspectiveCamera {
    origin: Point3<f32>,
    direction: Vector3<f32>,
    up: Vector3<f32>,
    aspect_ratio: f32,
    view_plane: ViewPlane
}

impl PerspectiveCamera {
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>, up: Vector3<f32>, aspect_ratio: f32, yfov: f32) -> Self {
        Self { origin, direction, up, aspect_ratio, view_plane: ViewPlane::new(origin, direction, up, yfov, aspect_ratio) }
    }

    pub fn update_transform(&mut self, transform: Matrix4<f32>) {
        let position = transform.transform_point(&Point3::origin());
        let forward = transform.transform_vector(&Vector3::new(0.0, 0.0, -1.0)).normalize();
        let up = transform.transform_vector(&Vector3::new(0.0, 1.0, 0.0)).normalize();

        self.origin = position;
        self.direction = forward;
        self.up = up;
        self.view_plane = ViewPlane::new(self.origin, self.direction, self.up, self.view_plane.size.y, self.aspect_ratio);
    }
}

impl Viewpoint for PerspectiveCamera {
    fn generate_ray(&self, u: f32, v: f32) -> Ray {
        let direction = self.view_plane.get_coordinates_from_uv(u, v) - self.origin;
        Ray::new(self.origin, direction.normalize())
    }

    fn generate_offset_ray(&self, u: f32, v: f32, radius: f32, focal_distance: f32, rng: &mut impl Rng) -> Ray {
        let direction = self.view_plane.get_coordinates_from_uv(u, v) - self.origin;
        let focal_point = self.origin + direction.normalize() * focal_distance;

        let angle = rng.random::<f32>() * PI * 2.0;
        let length = rng.random::<f32>() * radius;

        let origin = self.origin + (self.view_plane.u_dir * angle.sin() * length) + (self.view_plane.v_dir * angle.cos() * length);
        let direction = (focal_point - origin).normalize();

        Ray::new(origin, direction)
    }
}