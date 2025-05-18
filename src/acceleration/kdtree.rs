use nalgebra::Point3;
use crate::acceleration::bounds::AABB;
use crate::content::triangle::{Triangle, TriangleIntersection};
use crate::core::Ray;
use crate::static_stack::StaticStack;

#[derive(Clone, Copy)]
enum Axis {
    X, Y, Z
}

impl Axis {
    fn next(&self) -> Self {
        match self {
            Axis::X => Axis::Y,
            Axis::Y => Axis::Z,
            Axis::Z => Axis::X,
        }
    }

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
    pub fn new(items: Vec<Triangle>) -> Self {
       let mut bounds = AABB::from_points(items.iter()
           .flat_map(|x| [x.v0().position, x.v1().position, x.v2().position]));
        bounds.ensure_minimum_dimensions(0.001);
        Self {
            root: TreeNode::build_node(items, Axis::X),
            bounds,
        }
    }

    pub fn bounds(&self) -> AABB {
        self.bounds
    }

    pub fn intersects(&self, ray: &Ray) -> Option<TriangleIntersection> {

        let (global_tmin, global_tmax) = if let Some(hit) = self.bounds.intersect(ray) {
            (hit.tmin, hit.tmax)
        }
        else {
            return None;
        };

        let mut nodes = StaticStack::<NodeSearchData, 100>::new_with_default(
            NodeSearchData::new(&self.root, global_tmin, global_tmax));

        while !nodes.is_empty() {
            let current = nodes.pop();
            let node = current.node;
            let tmin = current.tmin;
            let tmax = current.tmax;


            if node.is_leaf() {
                if let Some(hit) = Self::intersects_mesh(self, ray, node, tmax) {
                    // TODO: Här sätter jag hit.dist = tmax, men det borde väl intersects_mesh göra?
                    return Some(hit);
                }
            } else {
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

        None
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
            panic!("This should never happen!");
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
        self.items.len() > 0
    }

    pub fn build_node(mut items: Vec<Triangle>, splitting_axis: Axis) -> TreeNode {
        if items.len() < 128 {
            return TreeNode {
                splitting_axis,
                splitting_value: 0.0,
                left: None,
                right: None,
                items,
            };
        }

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

        for item in items {
            let v0 = item.v0().position;
            let v1 = item.v1().position;
            let v2 = item.v2().position;

            if v0[splitting_axis.as_index()] >= splitting_value ||
                v1[splitting_axis.as_index()] >= splitting_value ||
                v2[splitting_axis.as_index()] >= splitting_value {
                right_side.push(item.clone());
            }

            if v0[splitting_axis.as_index()] < splitting_value ||
                v1[splitting_axis.as_index()] < splitting_value ||
                v2[splitting_axis.as_index()] < splitting_value {
                left_side.push(item.clone());
            }
        }

        Self {
            splitting_axis,
            splitting_value,
            left: Some(Box::new(Self::build_node(left_side, splitting_axis.next()))),
            right: Some(Box::new(Self::build_node(right_side, splitting_axis.next()))),
            items: Vec::new(),
        }
    }

    fn get_triangle_center(p0: &Point3<f32>, p1: &Point3<f32>, p2: &Point3<f32>) -> Point3<f32> {
        ((p0.coords + p1.coords + p2.coords) / 3.0).into()
    }
}

#[cfg(test)]
mod tests {
    use rand::{random, Rng};
    use super::*;


}