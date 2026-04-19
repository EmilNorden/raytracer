use std::collections::HashSet;
use nalgebra::Vector3;
use crate::animation::{Animation, AnimationOutputs, Interpolation};
use crate::scene::node_graph::{NodeGraph, SceneNode};
use crate::scene::scene::Scene;

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum AnimationState {
    Playing,
    Finished,
}

pub struct AnimationController {
    time: f32,
    last_timestamp: f32,
    node_graph: NodeGraph,
    animations: Vec<Animation>,
}

impl AnimationController {
    pub fn new(node_graph: NodeGraph, animations: Vec<Animation>) -> Self {
        let mut last_timestamp = 0.0;
        for animation in &animations {
            for channel in &animation.channels {
                last_timestamp = channel.timestamps.last().unwrap().max(last_timestamp);
            }
        }

        Self {
            time: 0.0,
            last_timestamp,
            node_graph,
            animations,
        }
    }

    fn get_node(nodes: &mut Vec<SceneNode>, node_index: usize) -> Option<&mut SceneNode> {
        for node in nodes {
            if node.index == node_index {
                return Some(node);
            }

            if let Some(child_node) = Self::get_node(&mut node.children, node_index) {
                return Some(child_node);
            }
        }

        None
    }

    pub fn step(&mut self, delta_time: f32, scene: &mut Scene) -> AnimationState {
        self.time += delta_time;

        for animation in &self.animations {
            for channel in &animation.channels {
                let node = Self::get_node(&mut self.node_graph.roots, channel.node_index).unwrap();
                match channel.interpolation {
                    Interpolation::Linear => {
                        let to_index = channel.timestamps
                            .partition_point(|t| *t <= self.time);
                        let from_index = to_index.saturating_sub(1);

                        match &channel.outputs {
                            AnimationOutputs::Translation(translations) => {
                                if to_index == channel.timestamps.len() {
                                    let last_trans = translations.last().unwrap();
                                    node.local_transform.translation = Vector3::new(last_trans.x, last_trans.y, last_trans.z);
                                }
                                else if to_index == 0 {
                                    let first_trans = translations.first().unwrap();
                                    node.local_transform.translation = Vector3::new(first_trans.x, first_trans.y, first_trans.z);
                                }
                                else {
                                    let t_start = channel.timestamps[from_index];
                                    let t_end = channel.timestamps[to_index];
                                    let t_delta = t_end - t_start;
                                    let t_ratio = ((self.time - t_start) / t_delta).clamp(0.0, 1.0);

                                    let trans_start = translations[from_index];
                                    let trans_end = translations[to_index];
                                    let trans_interp = trans_start.lerp(&trans_end, t_ratio);
                                    node.local_transform.translation = Vector3::new(trans_interp.x, trans_interp.y, trans_interp.z);
                                }
                            }
                            AnimationOutputs::Rotation(rotations) => {
                                if to_index == channel.timestamps.len() {
                                    let last_rot = rotations.last().unwrap();
                                    node.local_transform.rotation = *last_rot;
                                }
                                else if to_index == 0 {
                                    let first_rot = rotations.first().unwrap();
                                    node.local_transform.rotation = *first_rot;
                                }
                                else {
                                    let t_start = channel.timestamps[from_index];
                                    let t_end = channel.timestamps[to_index];
                                    let t_delta = t_end - t_start;
                                    let t_ratio = ((self.time - t_start) / t_delta).clamp(0.0, 1.0);

                                    let rot_start = rotations[from_index];
                                    let rot_end = rotations[to_index];
                                    let rot_interp = rot_start.slerp(&rot_end, t_ratio);
                                    node.local_transform.rotation = rot_interp;
                                }
                            }
                            AnimationOutputs::Scale(scales) => {
                                if to_index == channel.timestamps.len() {
                                    let last_scale = scales.last().unwrap();
                                    node.local_transform.scale = *last_scale;
                                }
                                else if to_index == 0 {
                                    let first_scale = scales.first().unwrap();
                                    node.local_transform.scale = *first_scale;
                                }
                                else {
                                    let t_start = channel.timestamps[from_index];
                                    let t_end = channel.timestamps[to_index];
                                    let t_delta = t_end - t_start;
                                    let t_ratio = ((self.time - t_start) / t_delta).clamp(0.0, 1.0);

                                    let scale_start = scales[from_index];
                                    let scale_end = scales[to_index];
                                    let scale_interp = scale_start.lerp(&scale_end, t_ratio);
                                    node.local_transform.scale = scale_interp;
                                }
                            }
                        }

                    }
                    Interpolation::Step => {}
                    Interpolation::CubicSpline => {
                        panic!("Cubic spline interpolation is not yet implemented");
                    }
                }
            }
        }

        Self::update_scene(self.node_graph.roots.as_mut_slice(), &nalgebra::Matrix4::identity(), scene);
        scene.rebuild_bvh();

        if self.time >= self.last_timestamp {
            AnimationState::Finished
        } else {
            AnimationState::Playing
        }
    }

    fn update_scene(nodes: &mut [SceneNode], transform: &nalgebra::Matrix4<f32>, scene: &mut Scene) {
        for node in nodes {
            let node_transform = transform * node.local_transform.to_matrix();

            if let Some(camera_index) = node.camera_index {
                scene.cameras_mut()[camera_index].update_transform(node_transform);
            }

            if let Some(light_index) = node.light_index {
                scene.lights_mut()[light_index].update_transform(node_transform);
            }

            for mesh_index in &node.mesh_indices {
                scene.meshes_mut()[*mesh_index].update_transform(node_transform);
            }

            Self::update_scene(&mut node.children, &node_transform, scene);
        }
    }


}