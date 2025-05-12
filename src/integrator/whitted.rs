use crate::frame::Frame;
use crate::integrator::integrator::Integrator;
use crate::scene::scene::Scene;

struct WhittedIntegrator;

impl Integrator for WhittedIntegrator {
    fn integrate(scene: &Scene, frame: &mut Frame) {
        
    }
}
