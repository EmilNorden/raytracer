use crate::acceleration::bounds::AABB;
use crate::scene::Intersectable;

struct BVH4 {

}

enum BVHNode {
    Leaf(LeafNode),
    Internal(InternalNode),
}

struct LeafNode {
    objects: Vec<Box<dyn Intersectable>>,
}

struct InternalNode {
    bounds: AABB,
    children: [Option<Box<BVHNode>>; 4],
}