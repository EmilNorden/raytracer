use nalgebra::{Matrix4, Point3, Vector3};
use crate::core::Ray;

#[derive(Copy, Clone, Debug, Default)]
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

    pub fn compound<I: IntoIterator<Item = AABB>>(aabb_iter: I) -> Self {
        let mut min = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Point3::new(f32::MIN, f32::MIN, f32::MIN);

        for aabb in aabb_iter {
            min.x = min.x.min(aabb.min.x);
            min.y = min.y.min(aabb.min.y);
            min.z = min.z.min(aabb.min.z);
            max.x = max.x.max(aabb.max.x);
            max.y = max.y.max(aabb.max.y);
            max.z = max.z.max(aabb.max.z);
        }

        AABB { min, max }
    }

    pub fn union(&mut self, other: &AABB) {
        self.min.x = self.min.x.min(other.min.x);
        self.min.y = self.min.y.min(other.min.y);
        self.min.z = self.min.z.min(other.min.z);
        self.max.x = self.max.x.max(other.max.x);
        self.max.y = self.max.y.max(other.max.y);
        self.max.z = self.max.z.max(other.max.z);
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

    pub fn centroid(&self) -> Point3<f32> {
        nalgebra::center(&self.min, &self.max)
    }

    pub fn surface_area(&self) -> f32 {
        let d = self.max - self.min;
        let dx = d.x.max(0.0);
        let dy = d.y.max(0.0);
        let dz = d.z.max(0.0);

        2.0 * (dx * dy + dy * dz + dz * dx)
    }

    pub fn expand(&mut self, p: Point3<f32>) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.min.z = self.min.z.min(p.z);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
        self.max.z = self.max.z.max(p.z);
    }

    pub fn inflate(&mut self, offset: f32) {
        self.max = self.max + Vector3::new(offset, offset, offset);
        self.min = self.min - Vector3::new(offset, offset, offset);
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

    pub fn intersect_closest(&self, ray: &Ray, max_t: f32) -> Option<AABBIntersection> {
        let mut tmin = 0.0f32;
        let mut tmax = max_t;

        // Slab test on each axis, with explicit handling for parallel rays.
        for axis in 0..3 {
            let origin = ray.origin()[axis];
            let dir = ray.direction()[axis];
            let bmin = self.min[axis];
            let bmax = self.max[axis];

            if dir.abs() < 1e-8 {
                // Parallel to slab: if outside the slab, no hit.
                if origin < bmin || origin > bmax {
                    return None;
                }
                continue;
            }

            let inv_dir = 1.0 / dir;
            let mut t1 = (bmin - origin) * inv_dir;
            let mut t2 = (bmax - origin) * inv_dir;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }

            tmin = tmin.max(t1);
            tmax = tmax.min(t2);

            // Early exits: empty interval or hit only beyond current max_t.
            if tmax < tmin || tmin > max_t {
                return None;
            }
        }

        Some(AABBIntersection { tmin, tmax })
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

    pub fn corners(&self) -> [Point3<f32>; 8] {
        [self.min,
         Point3::new(self.max.x, self.min.y, self.min.z),
         Point3::new(self.min.x, self.max.y, self.min.z),
         Point3::new(self.max.x, self.max.y, self.min.z),
         Point3::new(self.min.x, self.min.y, self.max.z),
         Point3::new(self.max.x, self.min.y, self.max.z),
         Point3::new(self.min.x, self.max.y, self.max.z),
         self.max]
    }

    pub fn transform(&self, m: &Matrix4<f32>) -> AABB {
        let mut corners = self.corners();
        corners.iter_mut().for_each(|c| *c = m.transform_point(c));
        AABB::from_points(corners)
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_point3_approx_eq(actual: Point3<f32>, expected: Point3<f32>) {
        let eps = 1e-5;
        assert!((actual.x - expected.x).abs() <= eps, "x mismatch: actual={}, expected={}", actual.x, expected.x);
        assert!((actual.y - expected.y).abs() <= eps, "y mismatch: actual={}, expected={}", actual.y, expected.y);
        assert!((actual.z - expected.z).abs() <= eps, "z mismatch: actual={}, expected={}", actual.z, expected.z);
    }

    #[test]
    fn test_transform() {
        let aabb = AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 20.0, 30.0));

        // Translate
        let m = Matrix4::new_translation(&Vector3::new(10.0, 20.0, 30.0));
        let transformed = aabb.transform(&m);
        assert_point3_approx_eq(transformed.min(), Point3::new(10.0, 20.0, 30.0));
        assert_point3_approx_eq(transformed.max(), Point3::new(20.0, 40.0, 60.0));

        // Scale
        let m = Matrix4::new_scaling(3.0);
        let transformed = aabb.transform(&m);
        assert_point3_approx_eq(transformed.min(), Point3::new(0.0, 0.0, 0.0));
        assert_point3_approx_eq(transformed.max(), Point3::new(30.0, 60.0, 90.0));

        // Non-uniform scaling
        let m = Matrix4::new_nonuniform_scaling(&Vector3::new(8.0, 6.0, 3.0));
        let transformed = aabb.transform(&m);
        assert_point3_approx_eq(transformed.min(), Point3::new(0.0, 0.0, 0.0));
        assert_point3_approx_eq(transformed.max(), Point3::new(80.0, 120.0, 90.0));

        let m = Matrix4::new_rotation(Vector3::new(0.0, 0.0, std::f32::consts::FRAC_PI_2));
        let transformed = aabb.transform(&m);
        assert_point3_approx_eq(transformed.min(), Point3::new(-20.0, 0.0, 0.0));
        assert_point3_approx_eq(transformed.max(), Point3::new(0.0, 10.0, 30.0));
    }

    #[test]
    fn test_transform_negative_scale() {
        let aabb = AABB::new(Point3::new(1.0, 2.0, 3.0), Point3::new(4.0, 6.0, 8.0));

        let m = Matrix4::new_nonuniform_scaling(&Vector3::new(-2.0, 3.0, -0.5));
        let transformed = aabb.transform(&m);

        assert_point3_approx_eq(transformed.min(), Point3::new(-8.0, 6.0, -4.0));
        assert_point3_approx_eq(transformed.max(), Point3::new(-2.0, 18.0, -1.5));
    }

    #[test]
    fn test_transform_shear() {
        let aabb = AABB::new(Point3::new(-1.0, -2.0, 0.0), Point3::new(3.0, 4.0, 5.0));

        let m = Matrix4::new(
            1.0, 2.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        let transformed = aabb.transform(&m);

        assert_point3_approx_eq(transformed.min(), Point3::new(-5.0, -2.0, 0.0));
        assert_point3_approx_eq(transformed.max(), Point3::new(11.0, 4.0, 5.0));
    }

    #[test]
    fn test_transform_combined_affine() {
        let aabb = AABB::new(Point3::new(1.0, -1.0, 0.0), Point3::new(4.0, 2.0, 5.0));

        // x' = -2x + y + 10
        // y' =  x + 3z - 4
        // z' = -y + z + 2
        let m = Matrix4::new(
            -2.0, 1.0, 0.0, 10.0,
             1.0, 0.0, 3.0, -4.0,
             0.0,-1.0, 1.0,  2.0,
             0.0, 0.0, 0.0,  1.0,
        );
        let transformed = aabb.transform(&m);

        assert_point3_approx_eq(transformed.min(), Point3::new(1.0, -3.0, 0.0));
        assert_point3_approx_eq(transformed.max(), Point3::new(10.0, 15.0, 8.0));
    }

    #[test]
    fn test_transform_origin_crossing_mirror_rotate() {
        let aabb = AABB::new(Point3::new(-2.0, -1.0, -3.0), Point3::new(1.0, 4.0, 2.0));

        let mirror_x = Matrix4::new_nonuniform_scaling(&Vector3::new(-1.0, 1.0, 1.0));
        let rot_z_90 = Matrix4::new_rotation(Vector3::new(0.0, 0.0, std::f32::consts::FRAC_PI_2));
        let m = rot_z_90 * mirror_x;
        let transformed = aabb.transform(&m);

        assert_point3_approx_eq(transformed.min(), Point3::new(-4.0, -1.0, -3.0));
        assert_point3_approx_eq(transformed.max(), Point3::new(1.0, 2.0, 2.0));
    }


    #[test]
    fn test_corners() {
        let aabb = AABB::new(Point3::new(-1.0, -2.0, -3.0), Point3::new(1.0, 2.0, 3.0));
        let corners = aabb.corners();
        assert_eq!(corners, [
            Point3::new(-1.0, -2.0, -3.0),
            Point3::new(1.0, -2.0, -3.0),
            Point3::new(-1.0, 2.0, -3.0),
            Point3::new(1.0, 2.0, -3.0),
            Point3::new(-1.0, -2.0, 3.0),
            Point3::new(1.0, -2.0, 3.0),
            Point3::new(-1.0, 2.0,  3.0),
            Point3::new(1.0, 2.0, 3.0),
        ])
    }

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

    #[test]
    fn test_surface_area_box() {
        let aabb = AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 3.0, 4.0));
        assert!((aabb.surface_area() - 52.0).abs() <= 1e-5);
    }

    #[test]
    fn test_surface_area_degenerate() {
        let point = AABB::new(Point3::new(1.0, 2.0, 3.0), Point3::new(1.0, 2.0, 3.0));
        let flat = AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 0.0, 5.0));

        assert!((point.surface_area() - 0.0).abs() <= 1e-5);
        assert!((flat.surface_area() - 20.0).abs() <= 1e-5);
    }

    #[test]
    fn test_intersect_closest_prunes_beyond_max_t() {
        let aabb = AABB::new(Point3::new(5.0, -1.0, -1.0), Point3::new(6.0, 1.0, 1.0));
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));

        assert!(aabb.intersect_closest(&ray, 4.0).is_none());

        let hit = aabb.intersect_closest(&ray, 6.0).expect("expected hit within max_t");
        assert!((hit.tmin - 5.0).abs() <= 1e-5);
        assert!((hit.tmax - 6.0).abs() <= 1e-5);
    }

    #[test]
    fn test_intersect_closest_parallel_axis() {
        let aabb = AABB::new(Point3::new(-1.0, -1.0, -1.0), Point3::new(1.0, 1.0, 1.0));

        // Parallel to X slabs, outside in X => no hit.
        let miss = Ray::new(Point3::new(2.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));
        assert!(aabb.intersect_closest(&miss, 100.0).is_none());

        // Parallel to X slabs, inside in X => can still hit by Y traversal.
        let hit_ray = Ray::new(Point3::new(0.5, -2.0, 0.0), Vector3::new(0.0, 1.0, 0.0));
        let hit = aabb.intersect_closest(&hit_ray, 100.0).expect("expected hit");
        assert!((hit.tmin - 1.0).abs() <= 1e-5);
        assert!((hit.tmax - 3.0).abs() <= 1e-5);
    }
}