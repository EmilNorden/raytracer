use crate::context::Context;
use crate::frame::Frame;
use crate::integrator::albedo::AlbedoIntegrator;
use crate::integrator::normal::NormalIntegrator;
use crate::integrator::pathtracing::PathTracingIntegrator;
use crate::options::RenderOptions;
use crate::scene::scene::Scene;

pub trait Integrator {
    fn integrate(&self, scene: &Scene, frame: &mut Frame, samples: u32, options: &RenderOptions, ctx: &Context);
}

pub enum IntegratorImpl {
    Normal(NormalIntegrator),
    Pathtracing(PathTracingIntegrator),
    Albedo(AlbedoIntegrator),
}

impl Integrator for IntegratorImpl {
    fn integrate(&self, scene: &Scene, frame: &mut Frame, samples: u32, options: &RenderOptions, ctx: &Context) {
        match self {
            IntegratorImpl::Normal(i) => {
                i.integrate(scene, frame, samples, options, ctx);
            }
            IntegratorImpl::Pathtracing(i) => {
                i.integrate(scene, frame, samples, options, ctx);
            },
            IntegratorImpl::Albedo(i) => {
                i.integrate(scene, frame, samples, options, ctx);
            }
        }
    }
}

pub fn create(options: &RenderOptions) -> IntegratorImpl {
    match options.integrator {
        crate::options::Integrator::Pathtracing => IntegratorImpl::Pathtracing(PathTracingIntegrator::new()),
        crate::options::Integrator::Albedo => IntegratorImpl::Albedo(AlbedoIntegrator {}),
        crate::options::Integrator::Debug => IntegratorImpl::Normal(NormalIntegrator {}),
    }
}