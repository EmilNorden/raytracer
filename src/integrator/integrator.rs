use crate::context::Context;
use crate::frame::Frame;
use crate::integrator::debug::DebugIntegrator;
use crate::integrator::pathtracing::PathTracingIntegrator;
use crate::options::RenderOptions;
use crate::scene::scene::Scene;

pub trait Integrator {
    fn integrate(&self, scene: &Scene, frame: &mut Frame, samples: u32, ctx: &Context);
}

pub enum IntegratorImpl {
    Debug(DebugIntegrator),
    Pathtracing(PathTracingIntegrator),
}

impl Integrator for IntegratorImpl {
    fn integrate(&self, scene: &Scene, frame: &mut Frame, samples: u32, ctx: &Context) {
        match self {
            IntegratorImpl::Debug(i) => {
                i.integrate(scene, frame, samples, ctx);
            }
            IntegratorImpl::Pathtracing(i) => {
                i.integrate(scene, frame, samples, ctx);
            }
        }
    }
}

pub fn create(options: &RenderOptions) -> IntegratorImpl {
    if options.debug {
        IntegratorImpl::Debug(DebugIntegrator::new())
    }
    else {
        IntegratorImpl::Pathtracing(PathTracingIntegrator::new())
    }
}