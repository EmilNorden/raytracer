use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, Size};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};

use crate::content::gltf::loader::GltfLoader;
use crate::content::scene_loader::SceneLoader;
use crate::integrator::integrator::create;
use crate::options::RenderOptions;
use crate::render_controller::{RenderController, RenderNotification};

mod core;
mod camera;
mod scene;
mod acceleration;
mod content;
mod integrator;
mod frame;
mod options;
mod static_stack;
mod animation;
mod render_controller;

struct App {
    width: u32,
    height: u32,
    total_samples: u32,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    render_controller: RenderController,
    latest_rgba: Vec<u8>,
    current_sample: u32,
    current_frame: u32,
    is_done: bool,
}

impl App {
    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            if self.is_done {
                window.set_title("Pathtracer - done");
            } else {
                window.set_title(&format!(
                    "Pathtracer - sample {}/{} - frame {}",
                    self.current_sample, self.total_samples, self.current_frame
                ));
            }
        }
    }

    fn pull_render_updates(&mut self) {
        if let Some(update) = self.render_controller.latest_update() {
            self.current_sample = update.sample;
            self.current_frame = update.frame;
            self.latest_rgba = update.rgba;

            if update.is_done && !self.is_done {
                println!("Render time: {:?}", update.elapsed);
                if let Some(path) = update.output_path {
                    println!("Saved frame to {}", path.display());
                }
            }

            self.is_done = update.is_done;

            self.update_window_title();
        }
    }

    fn draw_latest_frame(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(pixels) = self.pixels.as_mut() {
            if pixels.frame_mut().len() == self.latest_rgba.len() {
                pixels.frame_mut().copy_from_slice(&self.latest_rgba);
            }

            if let Err(err) = pixels.render() {
                eprintln!("Failed to render to window: {err}");
                event_loop.exit();
            }
        }
    }
}

impl ApplicationHandler<RenderNotification> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Pathtracer")
                        .with_inner_size(Size::Logical(LogicalSize::new(
                            self.width as f64,
                            self.height as f64,
                        ))),
                )
                .unwrap(),
        );

        let surface = SurfaceTexture::new(
            self.width,
            self.height,
            window.clone(),
        );
        let pixels = Pixels::new(self.width, self.height, surface)
        .expect("Failed to create pixel surface");

        self.window = Some(window);
        self.pixels = Some(pixels);
        self.update_window_title();

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: RenderNotification) {
        self.pull_render_updates();

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if self.window.as_ref().map(|w| w.id()) != Some(window_id) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(pixels) = self.pixels.as_mut() {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        eprintln!("Failed to resize surface: {err}");
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.draw_latest_frame(event_loop);
            }
            _ => {}
        }
    }
}

fn read_options() -> anyhow::Result<RenderOptions> {
    let launch_file: RenderOptions = ron::de::from_reader(std::fs::File::open("launch.ron")?)?;

    Ok(launch_file)
}

fn main() {
    let options = read_options().unwrap();
    println!("Using the following options:\n{}", options);
    let (scene, animation_controller) = GltfLoader::load_scene(&options.scene_file, &options).unwrap();

    if scene.lights().is_empty() {
        println!("No light sources found in scene. Aborting");
        return;
    }

    let integrator = create(&options);
    let width = options.resolution.width;
    let height = options.resolution.height;
    let total_samples = options.samples;

    let event_loop = EventLoop::<RenderNotification>::with_user_event()
        .build()
        .unwrap();
    let proxy: EventLoopProxy<RenderNotification> = event_loop.create_proxy();

    let render_controller = RenderController::start(options, scene, animation_controller, integrator, proxy);

    let mut app = App {
        width,
        height,
        total_samples,
        window: None,
        pixels: None,
        render_controller,
        latest_rgba: vec![0; (width * height * 4) as usize],
        current_sample: 0,
        current_frame: 0,
        is_done: false,
    };

    event_loop.run_app(&mut app).expect("Failed to run app");
}
