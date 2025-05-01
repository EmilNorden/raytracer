use image::ColorType::Rgb32F;
use image::Rgb;
use nalgebra::{Matrix4, Point3, Vector2, Vector3};
use crate::camera::perspective_camera::PerspectiveCamera;
use crate::camera::viewpoint::Viewpoint;
use crate::content::assimp::loader::AssimpLoader;
use crate::scene::{Intersectable, Shadeable, Sphere};
use crate::scene::material::Material;
use crate::content::mesh::{Mesh, SceneLoader};
use crate::content::triangle::{Triangle, Vertex};
use crate::scene::scene::{Scene, SceneObject};

mod core;
mod camera;
mod scene;
mod acceleration;
mod content;

fn main() {
    let mut image = image::RgbImage::new(1024, 512);

    let camera = PerspectiveCamera::new(
        Point3::origin(),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, 1.0, 0.0),
        0.5,
        70f32.to_radians()
    );
    // /Users/emilnorden/models/apple
    //let mut meshes = AssimpLoader::load_scene("/Users/emilnorden/models/apple/apple.obj").unwrap();
    let mut scene = AssimpLoader::load_scene("/Users/emilnorden/models/gltf/apples.gltf").unwrap();

    /*let spheres = vec![
        Sphere {
            position: Point3::new(0.0, 0.0, -100.0),
            radius: 3.0,
            material: Material::new(Vector3::new(1.0, 0.0, 0.0))
        },
         Sphere {
            position: Point3::new(1.0, 0.0, -50.0),
            radius: 1.0,
            material: Material::new(Vector3::new(0.0, 1.0, 0.0))
        }
    ];

    let normal = Vector3::new(0.0, 0.0, 1.0);
    let uv = Vector2::new(0.0, 0.0);
    let v = [
        Vertex { position: Point3::new(0.0, 0.5, 0.0), normal, uv},
        Vertex { position: Point3::new(0.5, -0.5, 0.0), normal, uv},
        Vertex { position: Point3::new(-0.5, -0.5, 0.0), normal, uv}
    ];

    let mesh = Mesh::new([
        Triangle::new(v, 0),
    ]);*/

    /*let mut scene = Scene::new(vec![
        SceneObject {
            inverse_world: Matrix4::new_translation(&Vector3::new(0.0, 0.0, 50.0)).try_inverse().unwrap(),
            geometry: Box::new(mesh),
        }
    ]);*/

    for y in 0..512 {
        println!("Now doing Y: {}", y);
        let v = y as f32 / 512.0;
        for x in 0..1024 {
            let u = x as f32 / 1024f32;

            let ray = camera.generate_ray(u, 1.0-v);

            /*let mut best = f32::INFINITY;
            let mut result = Vector3::new(0.0, 0.0, 0.0);
            for sphere in &spheres {
                if let Some(hit) = sphere.intersect(&ray, 0.0, f32::INFINITY) {
                    if hit.dist < best {
                        best = hit.dist;
                        result = sphere.material().color();
                    }
                }
            }*/

            let mut result = Vector3::new(0.0, 0.0, 0.0);
            if let Some(hit) = scene.intersect(&ray) {
                result = Vector3::new(1.0, 1.0, 1.0);
            }

            image.put_pixel(x as u32, y as u32, Rgb([(result.x * 255.0) as u8, (result.y * 255.0) as u8, (result.z * 255.0) as u8]));

        }
    }

    image.save("image.png").unwrap();
}
