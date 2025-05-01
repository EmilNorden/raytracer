use nalgebra::Point3;

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

impl<T> TreeNode<T> where T : HasPosition
{
    fn new(mut items: Vec<T>) -> Self {
        let axis = Axis::X;

        items.sort_by(|a, b| {
            // Sort each face by comparing the center of the triangles.
            // Previously I used the first vertex of each face but that didnt work out well.

            let a_mid_point = a.position();
            let b_mid_point = b.position();

            a_mid_point[axis.as_index()].total_cmp(&b_mid_point[axis.as_index()])
        });

        let half_size = items.len() / 2;
        let median_point = &items[half_size];
        let splitting_value = median_point[axis.as_index()];

        let mut left_side = Vec::with_capacity(half_size);
        let mut right_side = Vec::with_capacity(half_size);

       /* for item in items {

        }*/

        unimplemented!()


    }
}