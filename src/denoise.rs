
#[cfg(feature = "open_image_denoise")]
mod oidn;
mod passthrough;

use crate::context::Context;
use crate::frame::Frame;
use crate::integrator::albedo::AlbedoIntegrator;
use crate::integrator::integrator::Integrator;
use crate::integrator::normal::NormalIntegrator;
use crate::options::{DenoiseAlgorithm, DenoiseSettings};
use crate::scene::scene::Scene;

pub enum DenoiseImpl {
    OpenImageDenoise(oidn::Oidn),
    None(passthrough::Passthrough),
}

impl DenoiseFilter for DenoiseImpl {
    fn denoise(&self, frame: &Frame, albedo: Option<Frame>, normal: Option<Frame>) -> Frame {
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
    fn denoise(&self, frame: &Frame, albedo: Option<Frame>, normal: Option<Frame>) -> Frame;

    fn supports_auxiliary_albedo(&self) -> bool;
    fn supports_auxiliary_normal(&self) -> bool;
}

pub struct Denoiser {
    denoise_filter: DenoiseImpl,
    settings: DenoiseSettings,
}

impl Denoiser {
    pub fn denoise(&self, frame: &Frame, scene: &Scene, ctx: &Context) -> Frame {

        let albedo = if self.settings.auxiliary_albedo {
            if self.denoise_filter.supports_auxiliary_albedo() {
                let albedo_integrator = AlbedoIntegrator {};
                print!("Creating auxiliary albedo frame for denoising...");
                let mut albedo_frame = Frame::new(frame.width(), frame.height());
                albedo_integrator.integrate(&scene, &mut albedo_frame, 1, &ctx);
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
                let mut normal_frame = Frame::new(frame.width(), frame.height());
                normal_integrator.integrate(&scene, &mut normal_frame, 1, &ctx);
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
        let result = self.denoise_filter.denoise(frame, albedo, normal);
        println!("Done.");

        result
    }
}

pub fn create_denoiser(denoise: &DenoiseAlgorithm) -> Denoiser {
    let (denoise_impl, settings) = match denoise {
        DenoiseAlgorithm::OpenImageDenoise(x) => (DenoiseImpl::OpenImageDenoise(oidn::Oidn::new()), x.clone()),
        DenoiseAlgorithm::None => (DenoiseImpl::None(passthrough::Passthrough::new()), DenoiseSettings::default()),
    };

    Denoiser { denoise_filter: denoise_impl, settings }
}