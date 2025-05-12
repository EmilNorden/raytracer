use std::time::Instant;
use crate::content::scene_loader::SceneLoader;
use image::ColorType::Rgb32F;
use image::Rgb;
use nalgebra::{Matrix4, Point3, Vector2, Vector3};
use crate::camera::perspective_camera::PerspectiveCamera;
use crate::camera::viewpoint::Viewpoint;
//use crate::content::assimp::loader::AssimpLoader;
use crate::scene::{Intersectable, Shadeable, Sphere};
use crate::scene::material::Material;
use crate::content::triangle::{Triangle, Vertex};
use crate::scene::scene::{Scene, SceneObject};
use rayon::prelude::*;

mod core;
mod camera;
mod scene;
mod acceleration;
mod content;
mod integrator;
mod frame;

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
    let scene = crate::content::gltf::loader::GltfLoader::load_scene("/Users/emilnorden/models/gltf/apples4.gltf").unwrap();

    let height = 512;
    let width = (height as f32 * 1.7777777f32) as i32;

    let start = Instant::now();
    let foo = (0..height).into_par_iter().map(|y| {
        let mut pixels = vec![Vector3::new(0.0, 0.0, 0.0); width as usize];
        let v = y as f32 / height as f32;
        for x in 0..width {
            let u = x as f32 / width as f32;

            let ray = scene.camera.generate_ray(1.0-u, 1.0-v);

            let mut result = Vector3::new(0.0, 0.0, 0.0);
            if let Some(hit) = scene.intersect(&ray) {
                result = Vector3::new(1.0, 1.0, 1.0);
            }

            pixels[x as usize] = result;
        }

        pixels
    })
    .collect::<Vec<Vec<Vector3<f32>>>>();

    for y in 0..height {
        for x in 0..width {
            let color = foo[y as usize][x as usize];
            image.put_pixel(x as u32, y as u32, Rgb([(color.x * 255.0) as u8, (color.y * 255.0) as u8, (color.z * 255.0) as u8]));
        }
    }

    let elapsed = start.elapsed();
    println!("Elapsed: {:?}", elapsed);

    image.save("image.png").unwrap();
}
