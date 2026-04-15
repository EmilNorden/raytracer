use nalgebra::{Point3, Vector3};
use crate::acceleration::bounds::AABB;
use crate::content::triangle::{Triangle, TriangleIntersection};
use crate::core::Ray;
use crate::static_stack::StaticStack;

#[derive(Clone, Copy)]
enum Axis {
    X, Y, Z
}

impl Axis {
    fn as_index(&self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }
}

#[derive(Copy, Clone)]
struct NodeSearchData<'a> {
    node: &'a TreeNode,
    tmin: f32,
    tmax: f32,
}

impl<'a> NodeSearchData<'a> {
    fn new(node: &'a TreeNode, tmin: f32, tmax: f32) -> Self {
        Self {
            node,
            tmin,
            tmax,
        }
    }
}

pub struct KDTree {
    root: TreeNode,
    bounds: AABB,
}

impl KDTree {
    const MAX_BUILD_DEPTH: usize = 64;
    const MIN_LEAF_TRIANGLE_COUNT: usize = 128;

    pub fn new(items: Vec<Triangle>) -> Self {
       let mut bounds = AABB::from_points(items.iter()
           .flat_map(|x| [x.v0().position, x.v1().position, x.v2().position]));

        bounds.inflate(0.001);
        bounds.ensure_minimum_dimensions(0.001);
        Self {
            root: TreeNode::build_node(items, Axis::X, 0),
            bounds,
        }
    }

    pub fn bounds(&self) -> AABB {
        self.bounds
    }
    
    pub fn triangle_count_from_node(&self, node: &TreeNode) -> u32 {
        node.items.len() as u32 +   
            node.left.as_ref().map_or(0, |x| self.triangle_count_from_node(x.as_ref())) + 
            node.right.as_ref().map_or(0, |x| self.triangle_count_from_node(x.as_ref()))
    }
    
    pub fn triangle_count(&self) -> u32 {
        self.triangle_count_from_node(&self.root)
    }

    pub fn intersects(&self, ray: &Ray) -> Option<TriangleIntersection> {

        let (global_tmin, global_tmax) = if let Some(hit) = self.bounds.intersect(ray) {
            (hit.tmin, hit.tmax)
        }
        else {
            return None;
        };

        if global_tmax.is_nan() || global_tmin.is_nan() || global_tmax.is_infinite() || global_tmin.is_infinite() {
            panic!("Oh no something is wrong");
        }

        let mut nodes = StaticStack::<NodeSearchData, 256>::new_with_default(
            NodeSearchData::new(&self.root, global_tmin, global_tmax));

        let mut closest_hit = None;
        let mut closest_hit_dist = global_tmax;

        while !nodes.is_empty() {
            let current = nodes.pop();
            let node = current.node;
            let tmin = current.tmin;
            let tmax = current.tmax;

            if tmin > closest_hit_dist {
                continue;
            }

            if tmin.is_nan() || tmax.is_nan() || tmin.is_infinite() || tmax.is_infinite() {
                println!("Oh no something is wrong");
            }

            if let Some(hit) = Self::intersects_mesh(self, ray, node, tmax.min(closest_hit_dist)) {
                if hit.dist < closest_hit_dist {
                    closest_hit_dist = hit.dist;
                    closest_hit = Some(hit);
                }
            }

            if !node.is_leaf() {
                let a = node.splitting_axis.as_index();
                let thit = (node.splitting_value - ray.origin()[a]) * ray.direction_inv()[a];

                match Self::compare_range_with_plane(ray, tmin, tmax, node) {
                    RangePlaneComparison::AbovePlane => nodes.push(NodeSearchData::new(node.right.as_ref().unwrap().as_ref(), tmin, tmax)),
                    RangePlaneComparison::BelowPlane => nodes.push(NodeSearchData::new(node.left.as_ref().unwrap().as_ref(), tmin, tmax)),
                    RangePlaneComparison::BelowToAbove => {
                        nodes.push(NodeSearchData::new(node.right.as_ref().unwrap().as_ref(), thit, tmax));
                        nodes.push(NodeSearchData::new(node.left.as_ref().unwrap().as_ref(), tmin, thit));
                    }
                    RangePlaneComparison::AboveToBelow => {
                        nodes.push(NodeSearchData::new(node.left.as_ref().unwrap().as_ref(), thit, tmax));
                        nodes.push(NodeSearchData::new(node.right.as_ref().unwrap().as_ref(), tmin, thit));
                    }
                }
            }
        }

        closest_hit
    }

    fn intersects_mesh(&self, ray: &Ray, node: &TreeNode, tmax: f32) -> Option<TriangleIntersection> {
        let mut closest_hit = None;
        let mut closest_hit_dist = tmax;

        for item in &node.items {
            if let Some(hit) = item.intersect(ray) {
                if hit.dist < closest_hit_dist {
                    closest_hit_dist = hit.dist;
                    closest_hit = Some(hit);
                }
            }
        }

        closest_hit
    }

    fn compare_range_with_plane(ray: &Ray, tmin: f32, tmax: f32, node: &TreeNode) -> RangePlaneComparison {
        let axis = node.splitting_axis.as_index();
        let range_start = ray.origin()[axis] + (ray.direction()[axis] * tmin);
        let range_end = ray.origin()[axis] + (ray.direction()[axis] * tmax);

        let splitting_value = node.splitting_value;

        if range_start < splitting_value && range_end < splitting_value {
            RangePlaneComparison::BelowPlane
        } else if range_start >= splitting_value && range_end >= splitting_value {
            RangePlaneComparison::AbovePlane
        } else if range_start < splitting_value && range_end >= splitting_value {
            RangePlaneComparison::BelowToAbove
        } else if range_start >= splitting_value && range_end < splitting_value {
            RangePlaneComparison::AboveToBelow
        } else {
            panic!("This should never happen. range_start {} range_end {} splitting_value {}!", range_start, range_end, splitting_value);
        }
    }
}

enum RangePlaneComparison {
    BelowPlane,
    AbovePlane,
    BelowToAbove,
    AboveToBelow,
}

struct TreeNode {
    splitting_axis: Axis,
    splitting_value: f32,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
    items: Vec<Triangle>,
}

impl TreeNode
{
    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    fn make_leaf(items: Vec<Triangle>, splitting_axis: Axis) -> TreeNode {
        TreeNode {
            splitting_axis,
            splitting_value: 0.0,
            left: None,
            right: None,
            items,
        }
    }

    pub fn build_node(mut items: Vec<Triangle>, fallback_axis: Axis, depth: usize) -> TreeNode {
        if items.len() < KDTree::MIN_LEAF_TRIANGLE_COUNT || depth >= KDTree::MAX_BUILD_DEPTH {
            return Self::make_leaf(items, fallback_axis);
        }

        let splitting_axis = Self::choose_split_axis(&items, fallback_axis);

        items.sort_by(|a, b| {
            // Sort each face by comparing the center of the triangles.
            // Previously I used the first vertex of each face but that didnt work out well.

            let a_mid_point = Self::get_triangle_center(&a.v0().position, &a.v1().position, &a.v2().position);
            let b_mid_point = Self::get_triangle_center(&b.v0().position, &b.v1().position, &b.v2().position);

            a_mid_point[splitting_axis.as_index()].total_cmp(&b_mid_point[splitting_axis.as_index()])
        });

        let half_size = items.len() / 2;
        let median_point = Self::get_triangle_center(&items[half_size].v0().position, &items[half_size].v1().position, &items[half_size].v2().position);
        let splitting_value = median_point[splitting_axis.as_index()];

        let mut left_side = Vec::with_capacity(half_size);
        let mut right_side = Vec::with_capacity(half_size);
        let mut local_items = Vec::new();

        let axis = splitting_axis.as_index();

        for item in items {
            let v0 = item.v0().position;
            let v1 = item.v1().position;
            let v2 = item.v2().position;

            let tri_min = v0[axis].min(v1[axis].min(v2[axis]));
            let tri_max = v0[axis].max(v1[axis].max(v2[axis]));

            if tri_max < splitting_value {
                left_side.push(item);
            } else if tri_min >= splitting_value {
                right_side.push(item);
            } else {
                // Keep straddling triangles in the current node to guarantee child recursion shrinks.
                local_items.push(item);
            }
        }

        // No useful split, keep this node as leaf to avoid unchanged recursion.
        if left_side.is_empty() || right_side.is_empty() {
            local_items.extend(left_side);
            local_items.extend(right_side);
            return Self::make_leaf(local_items, splitting_axis);
        }

        Self {
            splitting_axis,
            splitting_value,
            left: Some(Box::new(Self::build_node(left_side, splitting_axis, depth + 1))),
            right: Some(Box::new(Self::build_node(right_side, splitting_axis, depth + 1))),
            items: local_items,
        }
    }

    fn choose_split_axis(items: &[Triangle], fallback_axis: Axis) -> Axis {
        let mut min_point = Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max_point = Point3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

        for item in items {
            let center = Self::get_triangle_center(&item.v0().position, &item.v1().position, &item.v2().position);
            for axis in 0..3 {
                min_point[axis] = min_point[axis].min(center[axis]);
                max_point[axis] = max_point[axis].max(center[axis]);
            }
        }

        let extent = max_point - min_point;
        if extent.x >= extent.y && extent.x >= extent.z {
            Axis::X
        } else if extent.y >= extent.z {
            Axis::Y
        } else if extent.z.is_finite() {
            Axis::Z
        } else {
            fallback_axis
        }
    }

    fn get_triangle_center(p0: &Point3<f32>, p1: &Point3<f32>, p2: &Point3<f32>) -> Point3<f32> {
        ((p0.coords + p1.coords + p2.coords) / 3.0).into()
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::Vector2;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use crate::content::triangle::Vertex;
    use super::*;

    fn make_large_straddling_triangle() -> Triangle {
        Triangle::new([
            Vertex {
                position: Point3::new(-1.0, -1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: Point3::new(1.0, -1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 1.0),
                uv: Vector2::new(1.0, 0.0),
            },
            Vertex {
                position: Point3::new(0.0, 1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 1.0),
                uv: Vector2::new(0.5, 1.0),
            },
        ])
    }

    fn make_random_triangle(rng: &mut StdRng) -> Triangle {
        let center = Point3::new(
            rng.random_range(-4.0..4.0),
            rng.random_range(-4.0..4.0),
            rng.random_range(2.0..20.0),
        );
        let size = rng.random_range(0.1..0.8);

        Triangle::new([
            Vertex {
                position: Point3::new(center.x - size, center.y - size, center.z),
                normal: Vector3::new(0.0, 0.0, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: Point3::new(center.x + size, center.y - size, center.z),
                normal: Vector3::new(0.0, 0.0, 1.0),
                uv: Vector2::new(1.0, 0.0),
            },
            Vertex {
                position: Point3::new(center.x, center.y + size, center.z),
                normal: Vector3::new(0.0, 0.0, 1.0),
                uv: Vector2::new(0.5, 1.0),
            },
        ])
    }

    fn brute_force_intersect(triangles: &[Triangle], ray: &Ray) -> Option<TriangleIntersection> {
        let mut closest = None;
        let mut closest_dist = f32::INFINITY;

        for triangle in triangles {
            if let Some(hit) = triangle.intersect(ray) {
                if hit.dist < closest_dist {
                    closest_dist = hit.dist;
                    closest = Some(hit);
                }
            }
        }

        closest
    }

    #[test]
    fn build_does_not_duplicate_straddling_triangles() {
        let triangles = vec![make_large_straddling_triangle(); 1024];
        let tree = KDTree::new(triangles.clone());

        assert_eq!(tree.triangle_count() as usize, triangles.len());
    }

    #[test]
    fn kdtree_matches_bruteforce_randomized_regression() {
        let mut rng = StdRng::seed_from_u64(0xBADC0FFE);

        let mut triangles = Vec::with_capacity(320);
        for _ in 0..320 {
            triangles.push(make_random_triangle(&mut rng));
        }

        let tree = KDTree::new(triangles.clone());

        for ray_index in 0..5000 {
            let origin = Point3::new(
                rng.random_range(-6.0..6.0),
                rng.random_range(-6.0..6.0),
                rng.random_range(-2.0..1.0),
            );

            let mut direction = Vector3::new(
                rng.random_range(-0.75..0.75),
                rng.random_range(-0.75..0.75),
                rng.random_range(0.2..1.0),
            );
            direction = direction.normalize();

            let ray = Ray::new(origin, direction);
            let kd_hit = tree.intersects(&ray);
            let brute_hit = brute_force_intersect(&triangles, &ray);

            match (kd_hit, brute_hit) {
                (None, None) => {}
                (Some(kd), Some(brute)) => {
                    assert!(
                        (kd.dist - brute.dist).abs() <= 1e-4,
                        "distance mismatch at ray {}: kd={} brute={}",
                        ray_index,
                        kd.dist,
                        brute.dist
                    );
                }
                _ => panic!("KDTree/brute force hit mismatch at ray {}", ray_index),
            }
        }
    }

}