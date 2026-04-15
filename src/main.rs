use std::sync::Arc;
use std::time::Instant;
use clap::Parser;
use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize, Size};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use crate::content::gltf::loader::GltfLoader;
use crate::content::scene_loader::SceneLoader;
use crate::frame::Frame;
use crate::options::RenderOptions;
use crate::integrator::integrator::{Integrator, IntegratorImpl};
use crate::scene::scene::Scene;

mod core;
mod camera;
mod scene;
mod acceleration;
mod content;
mod integrator;
mod frame;
mod options;
mod static_stack;

struct App {
    options: RenderOptions,
    scene: Scene,
    frame: Frame,
    integrator: IntegratorImpl,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    current_sample: u32,
    render_start: Instant,
}

impl App {
    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!(
                "Pathtracer - sample {}/{}",
                self.current_sample, self.options.samples
            ));
        }
    }

    fn render_next_sample(&mut self, event_loop: &ActiveEventLoop) {
        if self.current_sample >= self.options.samples {
            return;
        }

        self.integrator
            .integrate(&self.scene, &mut self.frame, self.options.samples);
        self.current_sample += 1;
        self.update_window_title();

        if let Some(pixels) = self.pixels.as_mut() {
            self.frame.write_rgba(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                eprintln!("Failed to render to window: {err}");
                event_loop.exit();
                return;
            }
        }

        if self.current_sample >= self.options.samples {
            println!("Render time: {:?}", self.render_start.elapsed());
            if let Some(window) = self.window.as_ref() {
                window.set_title("Pathtracer - done");
            }
            self.frame.save(&self.options.output_file);
            return;
        }

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()
            .with_title("Pathtracer")
            .with_inner_size(Size::Logical(LogicalSize::new(self.options.width as f64,
                                                              self.options.height as f64)))).unwrap());

        let surface = SurfaceTexture::new(self.options.width, self.options.height, window.clone());
        let pixels = Pixels::new(self.options.width, self.options.height, surface)
            .expect("Failed to create pixel surface");

        window.request_redraw();
        self.window = Some(window);
        self.update_window_title();
        self.pixels = Some(pixels);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if self.window.as_ref().map(|w| w.id()) != Some(window_id) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Resized(size) => {
                if let Some(pixels) = self.pixels.as_mut() {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        eprintln!("Failed to resize surface: {err}");
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.render_next_sample(event_loop);
            },
            _ => ()
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.current_sample < self.options.samples {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }
}

fn main() {
    let options = RenderOptions::parse();

    let scene = GltfLoader::load_scene(&options.scene_file, &options).unwrap();

    if scene.emissive_meshes().is_empty() {
        println!("No emissive materials found in scene. Aborting");
        return;
    }

    let frame = Frame::new(options.width, options.height);

    let integrator = integrator::integrator::create(&options);

    let event_loop = EventLoop::new().unwrap();

    let mut app = App {
        options,
        scene,
        frame,
        integrator,
        window: None,
        pixels: None,
        current_sample: 0,
        render_start: Instant::now(),
    };

    event_loop.run_app(&mut app).expect("TODO: panic message");
}
