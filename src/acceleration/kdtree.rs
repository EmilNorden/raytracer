use nalgebra::Point3;

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

struct TreeNode<T> {
    splitting_axis: Axis,
    splitting_value: f32,
    left: Option<Box<TreeNode<T>>>,
    right: Option<Box<TreeNode<T>>>,
    items: Vec<T>,
}

trait HasPosition {
    fn position(&self) -> Point3<f32>;
}

enum PartitionResult {
    Left,
    Right,
    Split,
}
trait Partitionable {
    fn partition(&self, axis: Axis, value: f32) -> PartitionResult;
}


impl<T> TreeNode<T> where T : HasPosition + Partitionable + Clone
{
    fn build_node(mut items: Vec<T>, splitting_axis: Axis) -> TreeNode<T> {
        if items.len() < 100 {
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

            let a_mid_point = a.position();
            let b_mid_point = b.position();

            a_mid_point[splitting_axis.as_index()].total_cmp(&b_mid_point[splitting_axis.as_index()])
        });

        let half_size = items.len() / 2;
        let median_point = items[half_size].position();
        let splitting_value = median_point[splitting_axis.as_index()];

        let mut left_side = Vec::with_capacity(half_size);
        let mut right_side = Vec::with_capacity(half_size);

        for item in items {
            match item.partition(splitting_axis, splitting_value) {
                PartitionResult::Left => left_side.push(item),
                PartitionResult::Right => right_side.push(item),
                PartitionResult::Split => {
                    left_side.push(item.clone());
                    right_side.push(item);
                }
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
    pub fn new(items: Vec<T>) -> Self {
        Self::build_node(items, Axis::X)
    }
}

#[cfg(test)]
mod tests {
    use rand::{random, Rng};
    use super::*;
    struct TestItem {
        pub position: Point3<f32>,
    }

    impl HasPosition for TestItem {
        fn position(&self) -> Point3<f32> {
            self.position
        }
    }

    impl Partitionable for TestItem {
        fn partition(&self, axis: Axis, value: f32) -> PartitionResult {
            if self.position[axis.as_index()] < value {
                PartitionResult::Left
            } else {
                PartitionResult::Right
            }
        }
    }
    #[test]
    fn test_build_node() {


    }
}