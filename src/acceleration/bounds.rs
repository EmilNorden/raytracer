use nalgebra::Point3;
use crate::core::Ray;

#[derive(Copy, Clone, Debug)]
pub struct AABB {
    min: Point3<f32>,
    max: Point3<f32>,
}

pub struct AABBIntersection {
    pub tmin: f32,
    pub tmax: f32,
}

impl AABB {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        Self { min, max }
    }

    pub fn from_points<I: IntoIterator<Item = Point3<f32>>>(points: I) -> Self {
        let mut bounds = AABB::new(Point3::new(f32::MAX, f32::MAX, f32::MAX), Point3::new(f32::MIN, f32::MIN, f32::MIN));

        for p in points {
            bounds.expand(p);
        }

        bounds
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

    pub fn ensure_minimum_dimensions(&mut self, min_length: f32) {
        let size = self.max - self.min;
        let half_length = min_length / 2.0;
        if size.x < min_length {
            self.max.x = self.max.x + half_length;
            self.min.x = self.min.x - half_length;
        }
        if size.y < min_length {
            self.max.y = self.max.y + half_length;
            self.min.y = self.min.y - half_length;
        }
        if size.z < min_length {
            self.max.z = self.max.z + half_length;
            self.min.z = self.min.z - half_length;
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<AABBIntersection> {
        let tx1 = (self.min.x - ray.origin().x) / ray.direction().x; //*r.n_inv.x
        let tx2 = (self.max.x - ray.origin().x) / ray.direction().x; //*r.n_inv.x
        
        let mut tmin = tx1.min(tx2);
        let mut tmax = tx1.max(tx2);
        
        let ty1 = (self.min.y - ray.origin().y) / ray.direction().y; //*r.n_inv.y;
        let ty2 = (self.max.y - ray.origin().y) / ray.direction().y; //*r.n_inv.y;
        
        tmin = tmin.max(ty1.min(ty2));
        tmax = tmax.min(ty1.max(ty2));
        
        let tz1 = (self.min.z - ray.origin().z) / ray.direction().z;
        let tz2 = (self.max.z - ray.origin().z) / ray.direction().z;
        
        tmin = tmin.max(tz1.min(tz2));
        tmax = tmax.min(tz1.max(tz2));
        
        if tmax >= tmin { Some(AABBIntersection { tmin, tmax }) } else { None }
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