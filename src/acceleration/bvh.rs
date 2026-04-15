use crate::acceleration::bounds::AABB;
use crate::content::mesh::MeshInstance;
use crate::core::Ray;
use crate::scene::{Intersectable, Intersection};

pub struct BVH {
    nodes: Vec<BVHNode>,
    bounds: AABB,
}

#[derive(Default)]
struct BVHNode {
    bbox: AABB,
    left: u32,
    right: u32,
    first: u32,
    count: u32,
}

impl BVHNode {
    fn is_leaf(&self) -> bool {
        self.left == 0 && self.right == 0
    }
}

struct Split {
    axis: usize,
    position: usize,
    min_c: f32,
    extent: f32,
}

const BIN_COUNT: usize = 16;
#[derive(Copy, Clone, Default)]
struct Bin {
    bounds: AABB,
    count: u32,
}

impl BVH {
    pub fn new(items: &mut [MeshInstance]) -> Self {
        let bbox = AABB::compound(items.iter().map(|m| {
            m.bounds().transform(m.transform())
        }));

        let mut nodes = Vec::new();
        Self::build_node(items, 0, &mut nodes);

        Self {
            nodes,
            bounds: bbox,
        }
    }

    pub fn intersect(&self, items: &[MeshInstance], ray: &Ray) -> Option<(u32, Intersection)> {
        self.intersect_with_limits(items, ray, 0.0, f32::INFINITY)
    }

    pub fn intersect_with_limits(
        &self,
        items: &[MeshInstance],
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(u32, Intersection)> {
        if self.bounds.intersect_closest(ray, t_max).is_none() {
            return None;
        }
        Self::traverse_bvh(&self.nodes, items, ray, t_min, t_max)
    }

    fn traverse_bvh(
        nodes: &[BVHNode],
        prims: &[MeshInstance],
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(u32, Intersection)> {
        let mut stack = [0u32; 64];
        let mut stack_ptr = 0;

        stack[stack_ptr] = 0; // root
        stack_ptr += 1;

        let mut closest_t = t_max;
        let mut hit = None;

        while stack_ptr > 0 {
            stack_ptr -= 1;
            let node_idx = stack[stack_ptr] as usize;
            let node = &nodes[node_idx];

            // 1. AABB test
            if node.bbox.intersect_closest(ray, closest_t).is_some() {
                if node.is_leaf() {
                    // 2. Test primitives
                    for i in node.first..node.first + node.count {
                        let prim = &prims[i as usize];

                        if let Some(h) = prim.intersect(ray, t_min, closest_t) {
                            closest_t = h.dist;
                            hit = Some((i, h));
                        }
                    }
                } else {
                    // 3. Traverse children (near first!)
                    let left = node.left as usize;
                    let right = node.right as usize;

                    let t_left = nodes[left].bbox.intersect_closest(ray, closest_t);
                    let t_right = nodes[right].bbox.intersect_closest(ray, closest_t);

                    match (t_left, t_right) {
                        (Some(tl), Some(tr)) => {
                            if tl.tmin < tr.tmin {
                                stack[stack_ptr] = right as u32;
                                stack_ptr += 1;
                                stack[stack_ptr] = left as u32;
                                stack_ptr += 1;
                            } else {
                                stack[stack_ptr] = left as u32;
                                stack_ptr += 1;
                                stack[stack_ptr] = right as u32;
                                stack_ptr += 1;
                            }
                        }
                        (Some(_), None) => {
                            stack[stack_ptr] = left as u32;
                            stack_ptr += 1;
                        }
                        (None, Some(_)) => {
                            stack[stack_ptr] = right as u32;
                            stack_ptr += 1;
                        }
                        (None, None) => {}
                    }
                }
            }
        }

        hit
    }

    fn build_node(items: &mut [MeshInstance], start: usize, nodes: &mut Vec<BVHNode>) -> usize {
        let node_index = nodes.len();

        nodes.push(BVHNode::default());

        let bounds = AABB::compound(items.iter().map(|m| {
            m.bounds().transform(m.transform())
        }));

        if items.len() < 3 {
            nodes[node_index] = BVHNode {
                bbox: bounds,
                left: 0,
                right: 0,
                first: start as u32,
                count: items.len() as u32,
            };
            return node_index;
        }

        if let Some(split) = Self::find_best_split(items) {
            let mid = Self::partition(items, &split);

            // Degenerate partition: keep node as leaf to avoid empty/unchanged recursion.
            if mid == 0 || mid == items.len() {
                nodes[node_index] = BVHNode {
                    bbox: bounds,
                    left: 0,
                    right: 0,
                    first: start as u32,
                    count: items.len() as u32,
                };
                return node_index;
            }

            let (left, right) = items.split_at_mut(mid);

            let left_index = Self::build_node(left, start, nodes);
            let right_index = Self::build_node(right, start + mid, nodes);

            nodes[node_index] = BVHNode {
                bbox: bounds,
                left: left_index as u32,
                right: right_index as u32,
                first: 0,
                count: 0,
            };
        } else {
            nodes[node_index] = BVHNode {
                bbox: bounds,
                left: 0,
                right: 0,
                first: start as u32,
                count: items.len() as u32,
            };
        }

        node_index
    }

    fn goes_left(item: &MeshInstance, split: &Split) -> bool {
        let c = item.bounds().transform(&item.transform()).centroid()[split.axis];

        let mut bin = ((c - split.min_c) / split.extent * BIN_COUNT as f32) as usize;
        bin = bin.min(BIN_COUNT - 1);

        bin <= split.position
    }

    fn partition(items: &mut [MeshInstance], split: &Split) -> usize {
        let mut swap_index = 0;
        for i in 0..items.len() {
            if Self::goes_left(&items[i], split) {
                items.swap(i, swap_index);
                swap_index += 1;
            }
        }

        swap_index
    }

    fn centroid_bounds(items: &[MeshInstance], axis: usize) -> (f32, f32) {
        let mut min_c = f32::INFINITY;
        let mut max_c = f32::NEG_INFINITY;

        for item in items {
            let c = item.bounds().transform(&item.transform()).centroid()[axis];
            min_c = min_c.min(c);
            max_c = max_c.max(c);
        }

        (min_c, max_c)
    }

    fn find_best_split(items: &mut [MeshInstance]) -> Option<Split> {
        let mut best_cost = f32::INFINITY;
        let mut best_split = None;

        // TODO: THis is already done in calling function. Pass that bbox instead of doing it again?
        let parent_bbox = AABB::compound(items.iter().map(|m| {
            m.bounds().transform(m.transform())
        }));
        let parent_area = parent_bbox.surface_area();

        for axis in 0..3 {
            // 1. Compute centroid bounds
            let (min_c, max_c) = Self::centroid_bounds(items, axis);
            let extent = max_c - min_c;

            if extent <= 1e-5 {
                continue; // can't split along this axis
            }

            // 2. Create bins
            let mut bins = [Bin::default(); BIN_COUNT];

            for item in &mut *items {
                let item_bounds = item.bounds().transform(&item.transform());
                let c = item_bounds.centroid()[axis];
                let mut bin_idx = ((c - min_c) / extent * BIN_COUNT as f32) as usize;
                bin_idx = bin_idx.min(BIN_COUNT - 1);

                bins[bin_idx].count += 1;
                if bins[bin_idx].count > 1 {
                    bins[bin_idx].bounds.union(&item_bounds);
                }
                else {
                    bins[bin_idx].bounds = item_bounds;
                }

            }

            // 3. Prefix sums (left side)
            let mut left_bbox = [AABB::default(); BIN_COUNT];
            let mut left_count = [0u32; BIN_COUNT];

            let mut accum_bbox = AABB::default();
            let mut accum_count = 0;

            for i in 0..BIN_COUNT {
                if i == 0 {
                    accum_bbox = bins[i].bounds;
                }
                else {
                    accum_bbox.union(&bins[i].bounds);
                }
                accum_count += bins[i].count;

                left_bbox[i] = accum_bbox;
                left_count[i] = accum_count;
            }

            // 4. Suffix sums (right side)
            let mut right_bbox = [AABB::default(); BIN_COUNT];
            let mut right_count = [0u32; BIN_COUNT];

            let mut accum_bbox = AABB::default();
            let mut accum_count = 0;

            for i in (0..BIN_COUNT).rev() {
                if i == BIN_COUNT - 1 {
                    accum_bbox = bins[i].bounds;
                }
                else {
                    accum_bbox.union(&bins[i].bounds);
                }
                accum_count += bins[i].count;

                right_bbox[i] = accum_bbox;
                right_count[i] = accum_count;
            }

            // 5. Evaluate splits
            for i in 0..BIN_COUNT - 1 {
                let left_n = left_count[i];
                let right_n = right_count[i + 1];

                if left_n == 0 || right_n == 0 {
                    continue;
                }

                let left_area = left_bbox[i].surface_area();
                let right_area = right_bbox[i + 1].surface_area();

                let cost =
                    (left_area / parent_area) * left_n as f32 +
                        (right_area / parent_area) * right_n as f32;

                if cost < best_cost {
                    best_cost = cost;
                    best_split = Some(Split {
                        axis,
                        position: i,
                        min_c,
                        extent,
                    });
                }
            }
        }

        best_split
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use nalgebra::{Matrix4, Point3, Vector2, Vector3};
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;
    use crate::content::mesh::{MeshData, MeshInstance};
    use crate::content::triangle::{Triangle, Vertex};
    use crate::core::Ray;
    use crate::scene::{Intersectable, Intersection};
    use crate::scene::material::Material;
    use super::BVH;

    fn make_triangle() -> Triangle {
        Triangle::new([
            Vertex {
                position: Point3::new(-1.0, -1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 1.0),
                tangent: nalgebra::Vector4::new(1.0, 0.0, 0.0, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: Point3::new(1.0, -1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 1.0),
                tangent: nalgebra::Vector4::new(1.0, 0.0, 0.0, 1.0),
                uv: Vector2::new(1.0, 0.0),
            },
            Vertex {
                position: Point3::new(-1.0, 1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 1.0),
                tangent: nalgebra::Vector4::new(1.0, 0.0, 0.0, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
        ])
    }

    fn make_mesh(mesh_index: usize, translation: Vector3<f32>) -> MeshInstance {
        let material = Material::new(
            Vector3::zeros(),
            None,
            None,
            None,
            None,
            1.0,
            Vector3::zeros(),
            0.0,
            0.0,
            0.0,
            1.5,
        );

        let data = Arc::new(MeshData::new(vec![make_triangle()], material));
        let transform = Matrix4::new_translation(&translation);

        MeshInstance::new(mesh_index, data, Point3::new(0.0, 0.0, 0.0), transform)
    }

    fn brute_force(
        meshes: &[MeshInstance],
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(u32, Intersection)> {
        let mut closest = t_max;
        let mut hit = None;

        for (i, mesh) in meshes.iter().enumerate() {
            if let Some(h) = mesh.intersect(ray, t_min, closest) {
                closest = h.dist;
                hit = Some((i as u32, h));
            }
        }

        hit
    }

    #[test]
    fn bvh_matches_bruteforce_for_closest_hit() {
        let mut meshes = vec![
            make_mesh(0, Vector3::new(0.0, 0.0, 3.0)),
            make_mesh(1, Vector3::new(0.0, 0.0, 6.0)),
            make_mesh(2, Vector3::new(2.0, 0.0, 4.0)),
        ];

        let bvh = BVH::new(&mut meshes);

        let rays = vec![
            Ray::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            Ray::new(Point3::new(-0.25, 0.25, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            Ray::new(Point3::new(2.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            Ray::new(Point3::new(4.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
        ];

        for ray in rays {
            let bvh_hit = bvh.intersect_with_limits(&meshes, &ray, 0.001, f32::INFINITY);
            let brute_hit = brute_force(&meshes, &ray, 0.001, f32::INFINITY);

            match (bvh_hit, brute_hit) {
                (None, None) => {}
                (Some((bvh_idx, bvh_i)), Some((bf_idx, bf_i))) => {
                    assert_eq!(bvh_idx, bf_idx);
                    assert!((bvh_i.dist - bf_i.dist).abs() <= 1e-5);
                }
                _ => panic!("BVH and brute force disagree for ray"),
            }
        }
    }

    #[test]
    fn intersect_with_limits_respects_t_max_for_shadow_segment() {
        let mut meshes = vec![
            make_mesh(0, Vector3::new(0.0, 0.0, 5.0)),
        ];

        let bvh = BVH::new(&mut meshes);
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0));

        let miss = bvh.intersect_with_limits(&meshes, &ray, 0.001, 4.5);
        assert!(miss.is_none());

        let hit = bvh.intersect_with_limits(&meshes, &ray, 0.001, 5.5);
        assert!(hit.is_some());
    }

    #[test]
    fn bvh_matches_bruteforce_randomized_regression() {
        let mut meshes = vec![
            make_mesh(0, Vector3::new(-2.0, -1.5, 3.0)),
            make_mesh(1, Vector3::new(1.5, -0.25, 4.5)),
            make_mesh(2, Vector3::new(-0.5, 1.75, 6.0)),
            make_mesh(3, Vector3::new(2.25, 2.0, 8.0)),
            make_mesh(4, Vector3::new(0.0, 0.0, 10.0)),
        ];

        let bvh = BVH::new(&mut meshes);
        let mut rng = StdRng::seed_from_u64(0xC0FFEE);

        for ray_index in 0..5000 {
            let origin = Point3::new(
                rng.random_range(-3.0..3.0),
                rng.random_range(-3.0..3.0),
                rng.random_range(-2.0..1.0),
            );

            let mut direction = Vector3::new(
                rng.random_range(-0.75..0.75),
                rng.random_range(-0.75..0.75),
                rng.random_range(0.2..1.0),
            );
            direction = direction.normalize();

            let ray = Ray::new(origin, direction);
            let t_min = 0.001;
            let t_max = 20.0;

            let bvh_hit = bvh.intersect_with_limits(&meshes, &ray, t_min, t_max);
            let brute_hit = brute_force(&meshes, &ray, t_min, t_max);

            match (bvh_hit, brute_hit) {
                (None, None) => {}
                (Some((bvh_idx, bvh_i)), Some((bf_idx, bf_i))) => {
                    assert_eq!(
                        bvh_idx, bf_idx,
                        "mesh index mismatch at ray {}",
                        ray_index
                    );
                    assert!(
                        (bvh_i.dist - bf_i.dist).abs() <= 1e-4,
                        "distance mismatch at ray {}: bvh={} brute={}",
                        ray_index,
                        bvh_i.dist,
                        bf_i.dist
                    );
                }
                _ => panic!("BVH/brute force hit mismatch at ray {}", ray_index),
            }
        }
    }
}

