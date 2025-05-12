use crate::frame::Frame;
use crate::scene::scene::Scene;

pub trait Integrator {
    fn integrate(scene: &Scene, frame: &mut Frame);
}