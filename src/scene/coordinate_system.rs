use nalgebra::Vector3;

pub struct CoordinateSystem {
    pub u: Vector3<f32>,
    pub v: Vector3<f32>,
    pub w: Vector3<f32>,
}

impl CoordinateSystem {
    /// Creates an orthonormal basis from a normal vector using Gram-Schmidt
    pub fn from_normal(normal: &Vector3<f32>) -> CoordinateSystem {
        let w = normal.normalize();

        let ref_vec = if w.abs().y < 0.99999 {
            Vector3::new(0.0, 1.0, 0.0)
        } else {
            Vector3::new(1.0, 0.0, 0.0)
        };

        let proj_len = ref_vec.dot(&w);
        let u_unnormalized = Vector3::new(
            ref_vec.x - (proj_len * w.x),
            ref_vec.y - (proj_len * w.y),
            ref_vec.z - (proj_len * w.z),
        );

        let u = u_unnormalized.normalize();
        let v = w.cross(&u);

        CoordinateSystem {
            u,
            v,
            w
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_from_normal() {
        let normal = Vector3::new(0.0, 1.0, 0.0);
        let system = CoordinateSystem::from_normal(&normal);
        assert_eq!(system.u, Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(system.v, Vector3::new(0.0, 0.0, -1.0));
        assert_eq!(system.w, Vector3::new(0.0, 1.0, 0.0));
    }
}