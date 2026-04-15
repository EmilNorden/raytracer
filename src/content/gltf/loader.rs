
use gltf::khr_lights_punctual::Kind;
use std::path::Path;
use gltf::camera::Projection;
use nalgebra::{Matrix4, Point3, Vector2, Vector3, Vector4};
use crate::camera::perspective_camera::PerspectiveCamera;
use crate::content::scene_loader::{SceneError, SceneLoader};
use crate::scene::scene::Scene;
use std::sync::Arc;
use gltf::buffer::Data;
use gltf::mesh::Mode;
use gltf::Node;
use gltf::scene::iter;
use crate::content::gltf::material::create_material;
use crate::content::mesh::{MeshInstance, MeshData};
use crate::content::triangle::{Triangle, Vertex};
use crate::options::RenderOptions;
use crate::scene::light::PointLight;
use crate::scene::material::Material;

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

    fn create_camera(scene: &gltf::scene::Scene, options: &RenderOptions) -> anyhow::Result<PerspectiveCamera> {
        let camera_node = scene.nodes()
            .find(|n| n.camera().is_some())
            .ok_or_else(|| SceneError::NoCameras)?;

        let cam = camera_node.camera().unwrap();
        let projection = cam.projection();

        let perspective = match projection {
            Projection::Orthographic(_) =>
                return Err(SceneError::UnsupportedFormat("Orthographic projection not supported".to_string()).into()),
            Projection::Perspective(x) => x
        };

        let camera_transform = Matrix4::from(camera_node.transform().matrix());
        let (_, up, forward) = Self::extract_directions(&camera_transform);
        let origin = Self::extract_translation(&camera_transform);

        let aspect_ratio = options.width as f32 / options.height as f32;

        Ok(PerspectiveCamera::new(origin, forward, up, aspect_ratio, perspective.yfov()))
    }

    fn create_meshes_in_children(parent: &Node, current_transform: Matrix4<f32>, buffers: &Vec<Data>, total_mesh_count: usize, total_material_count: usize, folder: &Path) -> anyhow::Result<Vec<MeshInstance>> {
        let mut mesh_data_map :Vec<Option<Vec<Arc<MeshData>>>> = vec![None; total_mesh_count];

        let child_transform = current_transform * Matrix4::from(parent.transform().matrix());

        let mut meshes = Vec::new();

        for node in parent.children() {
            if let Some(mesh) = node.mesh() {
                let mesh_data = if mesh_data_map[mesh.index()].is_some() {
                    mesh_data_map[mesh.index()].clone().unwrap()
                } else {
                    let data = Self::create_mesh_data(&buffers, &mesh, total_material_count, folder)?;
                    mesh_data_map[mesh.index()] = Some(data.clone());
                    data
                };

                let transform = child_transform * Matrix4::from(node.transform().matrix());

                for data in mesh_data {
                    meshes.push(MeshInstance::new(mesh.index(), data, Self::extract_translation(&transform), transform));
                }
            }

            meshes.append(&mut Self::create_meshes_in_children(&node, child_transform, buffers, total_mesh_count, total_material_count, folder)?);
        }



        Ok(meshes)
    }

    fn create_meshes(scene: &gltf::scene::Scene, buffers: &Vec<Data>, total_mesh_count: usize, total_material_count: usize, folder: &Path) -> anyhow::Result<Vec<MeshInstance>> {
        let mut mesh_data_map :Vec<Option<Vec<Arc<MeshData>>>> = vec![None; total_mesh_count];

        let mut meshes = Vec::new();

        for node in scene.nodes() {
            if let Some(mesh) = node.mesh() {
                let mesh_data = if mesh_data_map[mesh.index()].is_some() {
                    mesh_data_map[mesh.index()].clone().unwrap()
                } else {
                    let data = Self::create_mesh_data(&buffers, &mesh, total_material_count, folder)?;
                    mesh_data_map[mesh.index()] = Some(data.clone());
                    data
                };

                let transform = Matrix4::from(node.transform().matrix());

                for data in mesh_data {
                    meshes.push(MeshInstance::new(mesh.index(), data, Self::extract_translation(&transform), transform));
                }
            }

            meshes.append(&mut Self::create_meshes_in_children(&node, Matrix4::identity(), buffers, total_mesh_count, total_material_count, folder)?);
        }


        Ok(meshes)
    }

    fn create_mesh_data(buffers: &Vec<Data>, mesh: &gltf::mesh::Mesh, total_material_count: usize, folder: &Path) -> anyhow::Result<Vec<Arc<MeshData>>> {
        let mut material_map : Vec<Option<Arc<Material>>> = vec![None; total_material_count];

        let mut meshes = Vec::new();

        for primitive in mesh.primitives() {
            let mut triangles = Vec::new();
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


            for i in (0..indices.len()).step_by(3) {
                let i0 = indices[i];
                let i1 = indices[i + 1];
                let i2 = indices[i + 2];

                let pos0 = positions[i0 as usize];
                let pos1 = positions[i1 as usize];
                let pos2 = positions[i2 as usize];

                let normal0 = normals[i0 as usize];
                let normal1 = normals[i1 as usize];
                let normal2 = normals[i2 as usize];

                let tex_coord0 = tex_coords[i0 as usize];
                let tex_coord1 = tex_coords[i1 as usize];
                let tex_coord2 = tex_coords[i2 as usize];

                let tangent0 = tangents[i0 as usize];
                let tangent1 = tangents[i1 as usize];
                let tangent2 = tangents[i2 as usize];

                let vertex0 = Vertex {
                    position: pos0,
                    normal: normal0,
                    tangent: tangent0,
                    uv: tex_coord0,
                };

                let vertex1 = Vertex {
                    position: pos1,
                    normal: normal1,
                    tangent: tangent1,
                    uv: tex_coord1,
                };

                let vertex2 = Vertex {
                    position: pos2,
                    normal: normal2,
                    tangent: tangent2,
                    uv: tex_coord2,
                };

                triangles.push(Triangle::new([vertex0, vertex1, vertex2]));
            }

            meshes.push(Arc::new(MeshData::new(triangles, material)));
        }

        Ok(meshes)
    }

    fn create_point_lights(scene: &gltf::scene::Scene) -> Vec<PointLight> {
        let mut lights = Vec::new();
        for node in scene.nodes() {
            if let Some(light) = node.light() {
                match light.kind() {
                    Kind::Point => {
                        let transform = Matrix4::from(node.transform().matrix());
                        let position = Self::extract_translation(&transform);
                        let intensity = light.intensity();

                        let color = light.color();
                        lights.push(PointLight::new(position, Vector3::new(color[0], color[1], color[2]), intensity, 1.0))
                    }
                    _ => {}
                }
            }
        }

        lights
    }
}
impl SceneLoader for GltfLoader {
    fn load_scene<P: AsRef<Path>>(path: P, options: &RenderOptions) -> anyhow::Result<Scene> {
        let path = path.as_ref();
        let parent_folder = path.parent().unwrap();

        println!("Loading GLTF file..");
        let (document, buffers , images) = gltf::import(path)?;

        if let Some(scene) = document.default_scene() {

            println!("Setting up camera..");;
            let camera = Self::create_camera(&scene, options)?;

            println!("Processing meshes..");
            let meshes = Self::create_meshes(&scene, &buffers, document.meshes().len(), document.materials().len(), parent_folder)?;

            let lights = Self::create_point_lights(&scene);
            let triangles: usize = meshes.iter().map(|x| x.triangle_count()).sum();
            println!("Loaded {} meshes with {} triangles", meshes.len(), triangles);



            Ok(Scene::new(camera, meshes, lights))
        }
        else { Err(SceneError::NoDefaultScene.into()) }
    }
}

/*
fn traverse_nodes(nodes: Nodes) {
    for n in nodes {
        println!("node: {:?}", n.name().unwrap_or_default());
        if let Some(camera) = n.camera() {
            println!("camera: {:?}", camera.name().unwrap_or_default());
        }

        traverse_children(n.children(), 0);
    }
}

fn traverse_children(children: Children, indent: usize) {
    for c in children {
        for i in 0..indent {
            print!("-");
        }
        println!("child: {:?}", c.name().unwrap_or_default());

        traverse_children(c.children(), indent + 1);
    }
}*/