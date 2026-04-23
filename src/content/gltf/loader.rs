use crate::animation::controller::AnimationController;
use crate::animation::{Animation, AnimationChannel, AnimationOutputs};
use crate::camera::perspective_camera::PerspectiveCamera;
use crate::content::gltf::material::create_material;
use crate::content::mesh::{MeshData, MeshInstance};
use crate::content::scene_loader::{SceneError, SceneLoader};
use crate::content::triangle::Vertex;
use crate::options::RenderOptions;
use crate::scene::light::{LightSource, PointLight};
use crate::scene::material::Material;
use crate::scene::node_graph::{NodeGraph, NodeTransform, SceneNode};
use crate::scene::scene::Scene;
use gltf::animation::util::{ReadOutputs, Rotations};
use gltf::animation::Interpolation;
use gltf::buffer::Data;
use gltf::camera::Projection;
use gltf::khr_lights_punctual::Kind;
use gltf::mesh::Mode;
use gltf::Node;
use nalgebra::{Matrix4, Point3, Quaternion, UnitQuaternion, Vector2, Vector3, Vector4};
use std::path::Path;
use std::sync::Arc;

pub struct GltfLoader{}

impl GltfLoader {

    fn build_fallback_tangents(
        positions: &[Point3<f32>],
        normals: &[Vector3<f32>],
        tex_coords: &[Vector2<f32>],
        indices: &[u32],
    ) -> Vec<Vector4<f32>> {
        let mut accumulated_tangent = vec![Vector3::zeros(); positions.len()];
        let mut accumulated_bitangent = vec![Vector3::zeros(); positions.len()];

        for i in (0..indices.len()).step_by(3) {
            let i0 = indices[i] as usize;
            let i1 = indices[i + 1] as usize;
            let i2 = indices[i + 2] as usize;

            let p0 = positions[i0];
            let p1 = positions[i1];
            let p2 = positions[i2];

            let uv0 = tex_coords[i0];
            let uv1 = tex_coords[i1];
            let uv2 = tex_coords[i2];

            let edge1 = p1 - p0;
            let edge2 = p2 - p0;
            let duv1 = uv1 - uv0;
            let duv2 = uv2 - uv0;

            let determinant = duv1.x * duv2.y - duv1.y * duv2.x;
            if determinant.abs() <= 1e-8 {
                continue;
            }

            let inv_det = 1.0 / determinant;
            let tangent = (edge1 * duv2.y - edge2 * duv1.y) * inv_det;
            let bitangent = (edge2 * duv1.x - edge1 * duv2.x) * inv_det;
            if tangent.norm_squared() <= 1e-12 || bitangent.norm_squared() <= 1e-12 {
                continue;
            }

            accumulated_tangent[i0] += tangent;
            accumulated_tangent[i1] += tangent;
            accumulated_tangent[i2] += tangent;
            accumulated_bitangent[i0] += bitangent;
            accumulated_bitangent[i1] += bitangent;
            accumulated_bitangent[i2] += bitangent;
        }

        accumulated_tangent
            .into_iter()
            .zip(accumulated_bitangent)
            .zip(normals.iter())
            .map(|((tangent, bitangent), normal)| {
                if tangent.norm_squared() <= 1e-12 {
                    Vector4::new(1.0, 0.0, 0.0, 1.0)
                } else {
                    let tangent_dir = (tangent - *normal * normal.dot(&tangent)).normalize();
                    let handedness = if normal.cross(&tangent_dir).dot(&bitangent) < 0.0 {
                        -1.0
                    } else {
                        1.0
                    };
                    tangent_dir.insert_row(3, handedness)
                }
            })
            .collect()
    }

    fn extract_directions(transform: &Matrix4<f32>) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
        let right = Vector3::new(transform[(0, 0)], transform[(1, 0)], transform[(2, 0)]);
        let up = Vector3::new(transform[(0, 1)], transform[(1, 1)], transform[(2, 1)]);
        let forward = -Vector3::new(transform[(0, 2)], transform[(1, 2)], transform[(2, 2)]);

        (right, up, forward)
    }

    fn extract_translation(transform: &Matrix4<f32>) -> Point3<f32> {
        Point3::new(transform[(0, 3)], transform[(1, 3)], transform[(2, 3)])
    }

    fn create_mesh_data(buffers: &Vec<Data>, mesh: &gltf::mesh::Mesh, material_map: &mut Vec<Option<Arc<Material>>>, folder: &Path) -> anyhow::Result<Vec<Arc<MeshData>>> {
        let mut meshes = Vec::new();

        for primitive in mesh.primitives() {
            if primitive.mode() != Mode::Triangles {
                return Err(SceneError::UnsupportedFormat("Only triangles are supported".to_string()).into());
            }

            let material = create_material(&primitive.material(), folder)?;

            let reader = primitive.reader(|buffer| {
                Some(&buffers[buffer.index()].0)
            });

            let positions = reader.read_positions().map(|positions| {
                positions
                    .map(|p| Point3::new(p[0], p[1], p[2]))
                    .collect::<Vec<_>>()
            }).ok_or_else(|| SceneError::UnsupportedFormat("No positions found in mesh".to_string()))?;

            let normals = reader.read_normals().map(|normals| {
                normals
                    .map(|n| Vector3::new(n[0], n[1], n[2]))
                    .collect::<Vec<_>>()
            }).ok_or_else(|| SceneError::UnsupportedFormat("No normals found in mesh".to_string()))?;

            let tangents = reader.read_tangents().map(|tangents| {
                tangents.map(|t| Vector4::new(t[0], t[1], t[2], t[3])).collect::<Vec<_>>()
            });

            let tex_coords = reader.read_tex_coords(0).map(|tex_coords| {
                tex_coords.into_f32()
                    .map(|t| Vector2::new(t[0], t[1]))
                    .collect::<Vec<_>>()
            })
                .unwrap_or_else(|| vec![Vector2::new(0.0, 0.0); positions.len()]);

            let indices = reader.read_indices().map(|indices| {
                indices.into_u32().collect::<Vec<_>>()
            }).ok_or_else(|| SceneError::UnsupportedFormat("No indices found in mesh".to_string()))?;

            let tangents = tangents.unwrap_or_else(|| Self::build_fallback_tangents(&positions, &normals, &tex_coords, &indices));

            let mut vertices = Vec::new();
            let mut tri_indices = Vec::new();

            for i in 0..positions.len() {
                let position = positions[i];
                let normal = normals[i];
                let tex_coord = tex_coords[i];
                let tangent = tangents[i];

                vertices.push(Vertex {
                    position,
                    normal,
                    tangent,
                    uv: tex_coord,
                });
            }
            for i in (0..indices.len()).step_by(3) {
                let i0 = indices[i];
                let i1 = indices[i + 1];
                let i2 = indices[i + 2];

                tri_indices.push([i0, i1, i2]);
            }

            meshes.push(Arc::new(MeshData::new(vertices, tri_indices, material)));
        }

        Ok(meshes)
    }

    fn create_scene_node(node: &Node, buffers: &Vec<Data>, cameras: &mut Vec<PerspectiveCamera>, lights: &mut Vec<LightSource>, meshes: &mut Vec<MeshInstance>, mesh_data_map: &mut Vec<Option<Vec<Arc<MeshData>>>>, material_data_map: &mut Vec<Option<Arc<Material>>>, folder: &Path, parent_transform: &Matrix4<f32>, options: &RenderOptions) -> anyhow::Result<SceneNode> {
        let transform = parent_transform * Matrix4::from(node.transform().matrix());
        let children = node.children().map(|child|{
            Self::create_scene_node(&child, buffers, cameras, lights, meshes, mesh_data_map, material_data_map, folder, &transform, options)
        }).collect::<anyhow::Result<Vec<SceneNode>>>()?;

        let mut mesh_indices = Vec::new();
        if let Some(mesh) = node.mesh() {
            let mesh_data = if mesh_data_map[mesh.index()].is_some() {
                mesh_data_map[mesh.index()].clone().unwrap()
            } else {
                let data = Self::create_mesh_data(&buffers, &mesh, material_data_map, folder)?;
                mesh_data_map[mesh.index()] = Some(data.clone());
                data
            };

            for data in mesh_data {
                mesh_indices.push(meshes.len());
                meshes.push(MeshInstance::new(data, transform));
            }
        }

        let mut light_index = None;
        if let Some(light) = node.light() {
            match light.kind() {
                Kind::Point => {
                    let position = Self::extract_translation(&transform);
                    let intensity = light.intensity();

                    let color = light.color();
                    light_index = Some(lights.len());
                    lights.push(LightSource::Point(PointLight::new(position, Vector3::new(color[0], color[1], color[2]), intensity, 1.0)))
                }
                _ => {}
            }
        }

        let mut camera_index = None;
        if let Some(camera) = node.camera() {

            let projection = camera.projection();

            let perspective = match projection {
                Projection::Orthographic(_) =>
                    return Err(SceneError::UnsupportedFormat("Orthographic projection not supported".to_string()).into()),
                Projection::Perspective(x) => x
            };

            let (_, up, forward) = Self::extract_directions(&transform);
            let origin = Self::extract_translation(&transform);

            let aspect_ratio = options.resolution.width as f32 / options.resolution.height as f32;

            camera_index = Some(cameras.len());
            cameras.push(PerspectiveCamera::new(origin, forward, up, aspect_ratio, perspective.yfov()))
        }

        let (translation, rotation, scale) = node.transform().decomposed();

        Ok(SceneNode {
            index: node.index(),
            local_transform: NodeTransform::new(
                Vector3::new(translation[0], translation[1], translation[2]),
                UnitQuaternion::from_quaternion(Quaternion::new(rotation[3], rotation[0], rotation[1], rotation[2])),
                Vector3::new(scale[0], scale[1], scale[2]),
            ),
            mesh_indices,
            camera_index,
            light_index,
            children,
        })
    }


    fn load_node_graph(scene: &gltf::scene::Scene, buffers: &Vec<Data>, cameras: &mut Vec<PerspectiveCamera>, lights: &mut Vec<LightSource>, meshes: &mut Vec<MeshInstance>, folder: &Path, total_mesh_count: usize, total_material_count: usize, options: &RenderOptions) -> anyhow::Result<NodeGraph> {
        let mut mesh_data_map :Vec<Option<Vec<Arc<MeshData>>>> = vec![None; total_mesh_count];
        let mut material_map : Vec<Option<Arc<Material>>> = vec![None; total_material_count];

        let nodes = scene.nodes().map(|node|{
            Self::create_scene_node(&node, buffers, cameras, lights, meshes, &mut mesh_data_map, &mut material_map, folder, &Matrix4::identity(), options)
        }).collect::<anyhow::Result<Vec<SceneNode>>>()?;

        Ok(NodeGraph::new(nodes))
    }

    fn load_animations(document: &gltf::Document, buffers: &Vec<Data>) -> anyhow::Result<Vec<Animation>> {
        let mut animations = Vec::new();
        for animation in document.animations() {
            let mut channels = Vec::new();
            for channel in animation.channels() {
                let target = channel.target();
                let node = target.node();
                let interpolation = match channel.sampler().interpolation() {
                    Interpolation::Linear => crate::animation::Interpolation::Linear,
                    Interpolation::Step => crate::animation::Interpolation::Step,
                    Interpolation::CubicSpline => crate::animation::Interpolation::CubicSpline,
                };

                let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));

                let timestamps: Vec<f32> = reader.read_inputs().map(|inputs| inputs.collect())
                    .ok_or_else(|| SceneError::UnsupportedFormat("No animation inputs found".to_string()))?;

                let outputs = reader.read_outputs().map(|outputs| {
                    match outputs {
                        ReadOutputs::Translations(values) => {
                            let vals: Vec<[f32; 3]> = values.collect();
                            AnimationOutputs::Translation(vals.into_iter().map(|v| Point3::new(v[0], v[1], v[2])).collect())
                        }
                        ReadOutputs::Rotations(values) => {
                            match values {
                                Rotations::I8(_) => panic!("I8 rotation!"),
                                Rotations::U8(_) => panic!("U8 rotation!"),
                                Rotations::I16(_) => panic!("I16 rotation!"),
                                Rotations::U16(_) => panic!("U16 rotation!"),
                                Rotations::F32(values_f32) => {
                                    let vals: Vec<[f32; 4]> = values_f32.collect();
                                    AnimationOutputs::Rotation(vals.into_iter().map(|v| UnitQuaternion::from_quaternion(Quaternion::new(v[3], v[0], v[1], v[2]))).collect())
                                }
                            }
                        }
                        ReadOutputs::Scales(values) => {
                            let vals: Vec<[f32; 3]> = values.collect();
                            AnimationOutputs::Scale(vals.into_iter().map(|v| Vector3::new(v[0], v[1], v[2])).collect())
                        }
                        ReadOutputs::MorphTargetWeights(_) => panic!("Morph targets not supported"),
                    }
                }).ok_or_else(|| SceneError::UnsupportedFormat("No animation outputs found".to_string()))?;

                channels.push(AnimationChannel::new(node.index(), timestamps, outputs, interpolation));
            }

            animations.push(Animation::new(channels));

        };

        Ok(animations)
    }
}
impl SceneLoader for GltfLoader {
    fn load_scene<P: AsRef<Path>>(path: P, options: &RenderOptions) -> anyhow::Result<(Scene, AnimationController)> {
        let path = path.as_ref();
        let parent_folder = path.parent().unwrap();

        println!("Loading GLTF file..");
        let (document, buffers , _images) = gltf::import(path)?;

        if let Some(scene) = document.default_scene() {

            let mut cameras = Vec::new();
            let mut lights = Vec::new();
            let mut meshes = Vec::new();
            let node_graph = Self::load_node_graph(&scene, &buffers, &mut cameras, &mut lights, &mut meshes, parent_folder, document.meshes().len(), document.materials().len(), options)?;
            let animations = Self::load_animations(&document, &buffers)?;

            if cameras.is_empty() { return Err(SceneError::NoCameras.into()); }

            println!("Loaded scene with {} meshes, {} cameras, {} lights", meshes.len(), cameras.len(), lights.len());

            Ok((Scene::new(cameras, meshes, lights), AnimationController::new(node_graph, animations)))
        }
        else { Err(SceneError::NoDefaultScene.into()) }
    }
}