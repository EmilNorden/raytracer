
use std::path::Path;
use gltf::camera::Projection;
use nalgebra::{Matrix4, Point3, Vector2, Vector3};
use crate::camera::perspective_camera::PerspectiveCamera;
use crate::content::scene_loader::{SceneError, SceneLoader};
use crate::scene::scene::Scene;
use std::sync::Arc;
use gltf::buffer::Data;
use gltf::Image;
use gltf::image::Source;
use gltf::mesh::Mode;
use crate::content::content_cache::ContentCache;
use crate::content::gltf::material::create_material;
use crate::content::mesh::{Mesh, MeshData};
use crate::content::triangle::{Triangle, Vertex};
use crate::scene::material::Material;

pub struct GltfLoader{}

impl GltfLoader {

    fn extract_directions(transform: &Matrix4<f32>) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
        let right = Vector3::new(transform[(0, 0)], transform[(1, 0)], transform[(2, 0)]);
        let up = Vector3::new(transform[(0, 1)], transform[(1, 1)], transform[(2, 1)]);
        let forward = -Vector3::new(transform[(0, 2)], transform[(1, 2)], transform[(2, 2)]);

        (right, up, forward)
    }

    fn extract_translation(transform: &Matrix4<f32>) -> Point3<f32> {
        Point3::new(transform[(0, 3)], transform[(1, 3)], transform[(2, 3)])
    }

    fn create_camera(scene: &gltf::scene::Scene) -> anyhow::Result<PerspectiveCamera> {
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

        Ok(PerspectiveCamera::new(origin, forward, up, perspective.aspect_ratio().unwrap(), perspective.yfov()))
    }

    fn create_meshes(scene: &gltf::scene::Scene, buffers: &Vec<Data>, total_mesh_count: usize, total_material_count: usize, folder: &Path) -> anyhow::Result<Vec<Mesh>> {
        let mut byte_size = 0;
        let mut mesh_data_map :Vec<Option<Vec<Arc<MeshData>>>> = vec![None; total_mesh_count];
        let mesh_nodes = scene.nodes()
            .filter(|n| n.mesh().is_some());

        let mut meshes = Vec::new();

        for mesh_node in mesh_nodes {

            let mesh = mesh_node.mesh().unwrap();

            let mesh_data = if mesh_data_map[mesh.index()].is_some() {
                mesh_data_map[mesh.index()].clone().unwrap()
            } else {
                let data = Self::create_mesh_data(&buffers, &mesh, total_material_count, folder)?;
                mesh_data_map[mesh.index()] = Some(data.clone());
                data
            };

            let inverse_transform = Matrix4::from(mesh_node.transform().matrix()).try_inverse()
                .ok_or( SceneError::UnsupportedFormat("Could not invert mesh transform".to_string()))?;

            for data in mesh_data {
                meshes.push(Mesh::new(data, inverse_transform));
            }
        }

        println!("Total mesh data size: {} bytes", byte_size);

        Ok(meshes)
    }

    fn create_mesh_data(buffers: &Vec<Data>, mesh: &gltf::mesh::Mesh, total_material_count: usize, folder: &Path) -> anyhow::Result<Vec<Arc<MeshData>>> {
        let mut material_map : Vec<Option<Arc<Material>>> = vec![None; total_material_count];
        //let mut texture_cache = ContentCache::new();

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

            let tex_coords = reader.read_tex_coords(0).map(|tex_coords| {
                tex_coords.into_f32()
                    .map(|t| Vector2::new(t[0], t[1]))
                    .collect::<Vec<_>>()
            })
                .unwrap_or_else(|| vec![Vector2::new(0.0, 0.0); positions.len()]);

            let indices = reader.read_indices().map(|indices| {
                indices.into_u32().collect::<Vec<_>>()
            }).ok_or_else(|| SceneError::UnsupportedFormat("No indices found in mesh".to_string()))?;


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

                let vertex0 = Vertex {
                    position: pos0,
                    normal: normal0,
                    uv: tex_coord0,
                };

                let vertex1 = Vertex {
                    position: pos1,
                    normal: normal1,
                    uv: tex_coord1,
                };

                let vertex2 = Vertex {
                    position: pos2,
                    normal: normal2,
                    uv: tex_coord2,
                };

                triangles.push(Triangle::new([vertex0, vertex1, vertex2], 0));
            }

            meshes.push(Arc::new(MeshData::new(triangles, material)));
        }

        Ok(meshes)
    }
}
impl SceneLoader for GltfLoader {
    fn load_scene<P: AsRef<Path>>(path: P) -> anyhow::Result<Scene> {
        let path = path.as_ref();
        let parent_folder = path.parent().unwrap();

        let (document, buffers , images) = gltf::import(path)?;

        if let Some(scene) = document.default_scene() {

            let camera = Self::create_camera(&scene)?;

            let meshes = Self::create_meshes(&scene, &buffers, document.meshes().len(), document.materials().len(), parent_folder)?;

            Ok(Scene::new(camera, meshes))
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