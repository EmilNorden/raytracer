use nalgebra::{Matrix4, UnitQuaternion, Vector3};

/// Decomposed TRS transform — the native representation for animation.
#[derive(Debug, Clone)]
pub struct NodeTransform {
    pub translation: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

impl NodeTransform {
    pub fn new(translation: Vector3<f32>, rotation: UnitQuaternion<f32>, scale: Vector3<f32>) -> Self {
        Self { translation, rotation, scale }
    }

    /// Reconstruct the 4×4 world-space matrix from TRS.
    pub fn to_matrix(&self) -> Matrix4<f32> {
        let t = Matrix4::new_translation(&self.translation);
        let r = self.rotation.to_homogeneous();
        let s = Matrix4::new_nonuniform_scaling(&self.scale);
        t * r * s
    }
}

impl Default for NodeTransform {
    fn default() -> Self {
        Self {
            translation: Vector3::zeros(),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

/// A single node in the scene graph, mirroring the GLTF node hierarchy.
#[derive(Debug, Clone)]
pub struct SceneNode {
    /// GLTF node index (stable across the lifetime of the document).
    pub index: usize,
    /// Local TRS transform relative to the parent.
    pub local_transform: NodeTransform,
    /// Indices into the scene's mesh list, if this node has a mesh.
    pub mesh_indices: Vec<usize>,
    /// Index into the scene's camera list, if this node has a camera.
    pub camera_index: Option<usize>,
    /// Index into the scene's light list, if this node has a light.
    pub light_index: Option<usize>,
    pub children: Vec<SceneNode>,
}

/// The full scene-graph extracted from the GLTF default scene.
#[derive(Debug, Clone)]
pub struct NodeGraph {
    /// Top-level (root) nodes — the direct children of the GLTF scene object.
    pub roots: Vec<SceneNode>,
}

impl NodeGraph {
    pub fn new(roots: Vec<SceneNode>) -> Self {
        Self { roots }
    }

    #[allow(dead_code)]
    /// Iterate every node in depth-first order.
    pub fn iter(&self) -> impl Iterator<Item = &SceneNode> {
        NodeIter { stack: self.roots.iter().collect() }
    }
}

#[allow(dead_code)]
struct NodeIter<'a> {
    stack: Vec<&'a SceneNode>,
}

impl<'a> Iterator for NodeIter<'a> {
    type Item = &'a SceneNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        self.stack.extend(node.children.iter());
        Some(node)
    }
}

