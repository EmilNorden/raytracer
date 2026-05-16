use crate::animation::controller::{AnimationController, AnimationState};
use crate::context::Context;
use crate::denoise::{Denoiser};
use crate::frame::Frame;
use crate::integrator::integrator::{Integrator, IntegratorImpl};
use crate::options::{FocalDistance, RenderOptions};
use crate::scene::scene::Scene;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use nalgebra::Point3;
use crate::camera::perspective_camera::PerspectiveCamera;
use crate::camera::viewpoint::Viewpoint;
use crate::scene::node_graph::{NodeGraph, SceneNode};

pub struct RenderUpdate {
    pub sample: u32,
    pub rgba: Vec<u8>,
    pub is_done: bool,
    pub elapsed: Duration,
    pub output_path: Option<PathBuf>,
}

enum RenderCommand {
    Stop,
}

pub struct RenderController {
    update_rx: Receiver<RenderUpdate>,
    command_tx: Sender<RenderCommand>,
    worker: Option<JoinHandle<()>>,
}

impl RenderController {
    pub fn start(
        options: RenderOptions,
        mut scene: Scene,
        mut node_graph: NodeGraph,
        mut animation_controller: AnimationController,
        integrator: IntegratorImpl,
        denoiser: Denoiser,
        ctx: Context,
    ) -> Self {
        let (update_tx, update_rx) = mpsc::channel();
        let (command_tx, command_rx) = mpsc::channel();

        let worker = thread::spawn(move || {
            let mut frame = Frame::new(options.resolution.width, options.resolution.height);
            let render_start = Instant::now();

            let frame_duration = 1.0 / options.frame_rate as f32;
            let mut stop_video = false;
            let mut frame_index = 0;

            let mut camera = scene.active_camera().clone();

            Self::update_depth_of_field(&options, &mut scene, &mut node_graph, &ctx, &mut camera);

            loop {
                for sample in 1..=options.samples {
                    if Self::should_stop(&command_rx) {
                        break;
                    }

                    integrator.integrate(&scene, &camera, &mut frame, options.samples, &options, &ctx);

                    let mut rgba = vec![0_u8; (frame.width() * frame.height() * 4) as usize];
                    frame.write_rgba(&mut rgba);

                    let is_done = sample == options.samples;
                    let output_path = if is_done {
                        let folder = std::path::Path::new(&options.output_folder);
                        println!("Writing to output folder {:?}", folder);
                        if !folder.exists() {
                            std::fs::create_dir_all(folder)
                                .expect("failed to create output folder");
                        }

                        let path = std::path::Path::new(&options.output_folder)
                            .join(format!("out{:04}.png", frame_index));
                        frame.save(path.clone());

                        let denoise_result = denoiser.denoise(&frame, &scene, &camera, options.samples, &options, &ctx);

                        let path = std::path::Path::new(&options.output_folder)
                            .join(format!("out{:04}_denoised.png", frame_index));
                        denoise_result.denoised_frame.save(path.clone());

                        if let Some(auxiliary_albedo) = denoise_result.auxiliary_albedo {
                            let path = std::path::Path::new(&options.output_folder)
                                .join(format!("out{:04}_albedo.png", frame_index));
                            auxiliary_albedo.save(path.clone());
                        }

                        if let Some(auxiliary_normal) = denoise_result.auxiliary_normal {
                            let path = std::path::Path::new(&options.output_folder)
                                .join(format!("out{:04}_normal.png", frame_index));
                            auxiliary_normal.save(path.clone());
                        }


                        frame.clear();
                        frame_index += 1;

                        Some(path)
                    } else {
                        None
                    };

                    let update = RenderUpdate {
                        sample,
                        rgba,
                        is_done,
                        elapsed: render_start.elapsed(),
                        output_path,
                    };

                    if update_tx.send(update).is_err() {
                        break;
                    }

                }

                ctx.finalize();

                if !options.video {
                    break;
                }

                if stop_video {
                    let _ = Command::new("ffmpeg")
                        .current_dir("output") // 👈 only ffmpeg runs here
                        .args([
                            "-framerate", "30",
                            "-i", "out%04d.png",
                            "-pix_fmt", "yuv420p",
                            "out.mp4",
                        ])
                        .status()
                        .expect("failed to run ffmpeg");

                    break;
                }

                if animation_controller.step(frame_duration, &mut node_graph, &mut scene) == AnimationState::Finished {
                    stop_video = true;
                }
            }
        });

        Self {
            update_rx,
            command_tx,
            worker: Some(worker),
        }
    }

    fn update_depth_of_field(options: &RenderOptions, mut scene: &mut Scene, mut node_graph: &mut NodeGraph, ctx: &Context, camera: &mut PerspectiveCamera) {
        if let Some(dof) = &options.depth_of_field {
            let focal_distance = match dof.focal_distance.clone() {
                FocalDistance::Fixed(val) => Some(val),
                FocalDistance::Auto(u, v) => {
                    let focus_ray = scene.active_camera().generate_ray(u, v);
                    scene.intersect(&focus_ray, &ctx).map(|hit| hit.intersection.dist)
                },
                FocalDistance::Object(name) => {
                    match node_graph.get_node_by_name(&name) {
                        None => {
                            eprintln!("Warning: could not find object '{}' for focal distance", name);
                            None
                        },
                        Some(node) => {
                            Some((Point3::from(node.local_transform.translation) - camera.origin()).norm())
                        }
                    }
                }
            };

            if let Some(focal_distance) = focal_distance {
                camera.set_focal_distance(focal_distance);
            }
        }
    }

    pub fn latest_update(&self) -> Option<RenderUpdate> {
        let mut latest = None;
        while let Ok(update) = self.update_rx.try_recv() {
            latest = Some(update);
        }
        latest
    }

    pub fn stop(&mut self) {
        let _ = self.command_tx.send(RenderCommand::Stop);

        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }

    fn should_stop(command_rx: &Receiver<RenderCommand>) -> bool {
        match command_rx.try_recv() {
            Ok(RenderCommand::Stop) => true,
            Err(TryRecvError::Disconnected) => true,
            Err(TryRecvError::Empty) => false,
        }
    }
}

impl Drop for RenderController {
    fn drop(&mut self) {
        self.stop();
    }
}
