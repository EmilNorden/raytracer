
#[cfg(feature = "open_image_denoise")]
mod oidn;
mod passthrough;

use std::io::Write;
use crate::context::Context;
use crate::frame::Frame;
use crate::integrator::albedo::AlbedoIntegrator;
use crate::integrator::integrator::Integrator;
use crate::integrator::normal::NormalIntegrator;
use crate::options::{DenoiseAlgorithm, DenoiseSettings, RenderOptions};
use crate::scene::scene::Scene;

pub enum DenoiseImpl {
    OpenImageDenoise(oidn::Oidn),
    None(passthrough::Passthrough),
}

impl DenoiseFilter for DenoiseImpl {
    fn denoise(&self, frame: &Frame, albedo: &Option<Frame>, normal: &Option<Frame>) -> Frame {
        match self {
            DenoiseImpl::OpenImageDenoise(i) => {
                i.denoise(frame, albedo, normal)
            }
            DenoiseImpl::None(i) => {
                i.denoise(frame, albedo, normal)
            }
        }
    }

    fn supports_auxiliary_albedo(&self) -> bool {
        match self {
            DenoiseImpl::OpenImageDenoise(i) => i.supports_auxiliary_albedo(),
            DenoiseImpl::None(i) => i.supports_auxiliary_albedo(),
        }
    }

    fn supports_auxiliary_normal(&self) -> bool {
        match self {
            DenoiseImpl::OpenImageDenoise(i) => i.supports_auxiliary_normal(),
            DenoiseImpl::None(i) => i.supports_auxiliary_normal(),
        }
    }
}

pub enum AuxiliaryImage {
    Albedo(Frame),
    Normal(Frame),
}

pub trait DenoiseFilter {
    fn denoise(&self, frame: &Frame, albedo: &Option<Frame>, normal: &Option<Frame>) -> Frame;

    fn supports_auxiliary_albedo(&self) -> bool;
    fn supports_auxiliary_normal(&self) -> bool;
}

pub struct DenoiseResult {
    pub denoised_frame: Frame,
    pub auxiliary_albedo: Option<Frame>,
    pub auxiliary_normal: Option<Frame>,
}

pub struct Denoiser {
    denoise_filter: DenoiseImpl,
    settings: DenoiseSettings,
}

impl Denoiser {
    pub fn denoise(&self, frame: &Frame, scene: &Scene, samples: u32, options: &RenderOptions, ctx: &Context) -> DenoiseResult {

        let albedo = if self.settings.auxiliary_albedo {
            if self.denoise_filter.supports_auxiliary_albedo() {
                let albedo_integrator = AlbedoIntegrator {};
                print!("Creating auxiliary albedo frame for denoising...");
                let mut albedo_frame = Frame::new(frame.width(), frame.height());
                for _ in 0..samples {
                    albedo_integrator.integrate(&scene, &mut albedo_frame, samples, options, &ctx);
                }
                println!("Done.");
                Some(albedo_frame)
            }
            else {
                println!("Warning: Denoiser does not support auxiliary albedo, but it was requested in settings. Ignoring.");
                None
            }
        } else {
            None
        };

        let normal = if self.settings.auxiliary_normal {
            if self.denoise_filter.supports_auxiliary_normal() {
                let normal_integrator = NormalIntegrator {};
                print!("Creating auxiliary normal frame for denoising...");
                std::io::stdout().flush().unwrap();
                let mut normal_frame = Frame::new(frame.width(), frame.height());
                for _ in 0..samples {
                    normal_integrator.integrate(&scene, &mut normal_frame, samples, options, &ctx);
                }

                println!("Done.");
                Some(normal_frame)
            }
            else {
                println!("Warning: Denoiser does not support auxiliary normal, but it was requested in settings. Ignoring.");
                None
            }
        } else {
            None
        };

        print!("Denoising frame...");
        std::io::stdout().flush().unwrap();
        let result = self.denoise_filter.denoise(frame, &albedo, &normal);
        println!("Done.");

        DenoiseResult { denoised_frame: result, auxiliary_albedo: albedo, auxiliary_normal: normal }
    }
}

pub fn create_denoiser(denoise: &DenoiseAlgorithm) -> Denoiser {
    let (denoise_impl, settings) = match denoise {
        DenoiseAlgorithm::OpenImageDenoise(x) => (DenoiseImpl::OpenImageDenoise(oidn::Oidn::new()), x.clone()),
        DenoiseAlgorithm::None => (DenoiseImpl::None(passthrough::Passthrough::new()), DenoiseSettings::default()),
    };

    Denoiser { denoise_filter: denoise_impl, settings }
}