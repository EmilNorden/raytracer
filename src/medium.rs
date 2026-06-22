use crate::acceleration::bounds::AABB;
use crate::core::Ray;

pub mod volume;

pub struct Medium {
    bounds: AABB,
    absorption: f32,
    scattering: f32,
    extinction: f32,
}

pub struct MediumHit {
    pub transmittance: f32,
}

impl Medium {
    pub fn new(bounds: AABB, absorption: f32, scattering: f32) -> Self {
        Self {
            bounds,
            absorption,
            scattering,
            extinction: absorption + scattering
        }
    }

    pub fn sample(&self, ray: &Ray, t_max: f32) -> Option<MediumHit> {
        if let Some(hit) = self.bounds.intersect_closest(ray, t_max) {
            let distance = hit.tmax - hit.tmin;
            let transmittance = (-self.extinction * distance).exp();
            Some(MediumHit { transmittance })
        }
        else {
            None
        }
    }
}