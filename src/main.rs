
use std::time::Instant;
use clap::Parser;
use crate::content::scene_loader::SceneLoader;
use crate::camera::viewpoint::Viewpoint;
use crate::scene::{Intersectable, Shadeable};
use rayon::prelude::*;
use crate::content::gltf::loader::GltfLoader;
use crate::frame::Frame;
use crate::integrator::whitted::WhittedIntegrator;
use crate::options::RenderOptions;
use crate::integrator::integrator::Integrator;

mod core;
mod camera;
mod scene;
mod acceleration;
mod content;
mod integrator;
mod frame;
mod options;


fn main() {
    let options = RenderOptions::parse();

    let scene = GltfLoader::load_scene(&options.scene_file, &options).unwrap();

    let mut frame = Frame::new(options.width, options.height);
    let integrator = WhittedIntegrator::new();
    let start = Instant::now();
    integrator.integrate(&scene, &mut frame);
    println!("Render time: {:?}", start.elapsed());
    frame.save(&options.output_file);
}
