
use std::time::Instant;
use clap::Parser;
use crate::content::scene_loader::SceneLoader;
use crate::camera::viewpoint::Viewpoint;
use crate::scene::{Intersectable, Shadeable};
use rayon::prelude::*;
use crate::content::gltf::loader::GltfLoader;
use crate::frame::Frame;
use crate::integrator::debug::DebugIntegrator;
use crate::integrator::whitted::WhittedIntegrator;
use crate::options::RenderOptions;
use crate::integrator::integrator::Integrator;
use crate::integrator::pathtracing::PathTracingIntegrator;

mod core;
mod camera;
mod scene;
mod acceleration;
mod content;
mod integrator;
mod frame;
mod options;
mod static_stack;

fn main() {
    let options = RenderOptions::parse();

    let scene = GltfLoader::load_scene(&options.scene_file, &options).unwrap();

    if scene.emissive_meshes().is_empty() {
        println!("No emissive materials found in scene. Aborting");
        return;
    }

    let mut frame = Frame::new(options.width, options.height);
    
    let integrator = integrator::integrator::create(&options);
    let start = Instant::now();
    integrator.integrate(&scene, &mut frame, options.samples);
    println!("Render time: {:?}", start.elapsed());
    frame.save(&options.output_file);
}
