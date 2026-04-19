use nalgebra::{Point3, UnitQuaternion, Vector3};

pub struct Animation {
    name: String,
    channels: Vec<AnimationChannel>,
}

impl Animation {
    pub fn new(name: String, channels: Vec<AnimationChannel>) -> Self {
        Self {
            name,
            channels,
        }
    }
}

pub struct AnimationChannel {
    node_index: usize,
    timestamps: Vec<f32>,
    outputs: AnimationOutputs,
    interpolation: Interpolation,
}

impl AnimationChannel {
    pub fn new(node_index: usize, timestamps: Vec<f32>, outputs: AnimationOutputs, interpolation: Interpolation) -> Self {
        Self {
            node_index,
            timestamps,
            outputs,
            interpolation,
        }
    }
}

pub enum AnimationOutputs {
    Translation(Vec<Point3<f32>>),
    Rotation(Vec<UnitQuaternion<f32>>),
    Scale(Vec<Vector3<f32>>),
}

#[derive(Copy, Clone, Debug)]
pub enum Interpolation {
    Linear,
    Step,
    CubicSpline
}