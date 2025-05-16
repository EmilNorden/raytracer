use crate::frame::Frame;
use crate::scene::scene::Scene;

pub trait Integrator {
    fn integrate(&self, scene: &Scene, frame: &mut Frame);
}