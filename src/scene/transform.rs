use nalgebra::{Matrix4, Point3, Quaternion, Rotation3, Scale3, Translation3, Vector3};

pub struct Transform {
    world: Matrix4<f32>,
    inverse_world: Matrix4<f32>,
}

impl Transform {
    pub fn new(world: Matrix4<f32>) -> Self {
        Self {
            world,
            inverse_world: world.try_inverse().unwrap(),
        }
    }
}

pub struct TransformBuilder {
    translation: Translation3<f32>,
    rotation: Rotation3<f32>,
    scale: Scale3<f32>
}

impl Default for TransformBuilder {
    fn default() -> Self {
        Self {
            translation: Translation3::default(),
            rotation: Rotation3::default(),
            scale: Scale3::identity(),
        }
    }
}

impl TransformBuilder {
    pub fn build(self) -> Transform {
        let foo = self.translation * self.rotation;
        //let q = Quaternion::
        unimplemented!()
    }
}