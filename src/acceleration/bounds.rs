use nalgebra::Point3;

#[derive(Copy, Clone, Debug)]
pub struct AABB {
    min: Point3<f32>,
    max: Point3<f32>,
}

impl AABB {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        Self { min, max }
    }

    pub fn min(&self) -> Point3<f32> { self.min }
    pub fn max(&self) -> Point3<f32> { self.max }

    pub fn expand(&mut self, p: Point3<f32>) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.min.z = self.min.z.min(p.z);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
        self.max.z = self.max.z.max(p.z);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_positive_x() {
        let mut aabb = AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));

        aabb.expand(Point3::new(10.0, 0.0, 0.0));

        assert_eq!(aabb.min(), Point3::new(0.0, 0.0, 0.0));
        assert_eq!(aabb.max(), Point3::new(10.0, 1.0, 1.0));
    }

    #[test]
    fn test_expand_negative_x() {
        let mut aabb = AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));

        aabb.expand(Point3::new(-10.0, 0.0, 0.0));

        assert_eq!(aabb.min(), Point3::new(-10.0, 0.0, 0.0));
        assert_eq!(aabb.max(), Point3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_expand_positive_y() {
        let mut aabb = AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));

        aabb.expand(Point3::new(0.0, 10.0, 0.0));

        assert_eq!(aabb.min(), Point3::new(0.0, 0.0, 0.0));
        assert_eq!(aabb.max(), Point3::new(1.0, 10.0, 1.0));
    }

    #[test]
    fn test_expand_negative_y() {
        let mut aabb = AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));

        aabb.expand(Point3::new(0.0, -10.0, 0.0));

        assert_eq!(aabb.min(), Point3::new(0.0, -10.0, 0.0));
        assert_eq!(aabb.max(), Point3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_expand_positive_z() {
        let mut aabb = AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));

        aabb.expand(Point3::new(0.0, 0.0, 10.0));

        assert_eq!(aabb.min(), Point3::new(0.0, 0.0, 0.0));
        assert_eq!(aabb.max(), Point3::new(1.0, 1.0, 10.0));
    }

    #[test]
    fn test_expand_negative_z() {
        let mut aabb = AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));

        aabb.expand(Point3::new(0.0, 0.0, -10.0));

        assert_eq!(aabb.min(), Point3::new(0.0, 0.0, -10.0));
        assert_eq!(aabb.max(), Point3::new(1.0, 1.0, 1.0));
    }
}