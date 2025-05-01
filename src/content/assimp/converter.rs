use std::rc::Rc;
use nalgebra::{Matrix4, Point3, Vector2};
use russimp::Matrix4x4;
use russimp::node::Node;
use crate::content::mesh::Mesh;
use crate::content::triangle::{Triangle, Vertex};
use crate::scene::scene::{Scene, SceneNode};

pub struct AssimpSceneConverter;

impl AssimpSceneConverter {
    fn convert_transform(transform: Matrix4x4) -> Matrix4<f32> {
        Matrix4::new(
            transform.a1, transform.a2, transform.a3, transform.a4,
            transform.b1, transform.b2, transform.b3, transform.b4,
            transform.c1, transform.c2, transform.c3, transform.c4,
            transform.d1, transform.d2, transform.d3, transform.d4
        )
    }

    fn convert_mesh(mesh: &russimp::mesh::Mesh) -> Mesh {
        let texture_coordinates = mesh.texture_coords[0].as_ref();

        //let material = &scene.materials[mesh.material_index as usize];

        let mut triangles = Vec::new();
        for face in &mesh.faces {
            let i0 = face.0[0] as usize;
            let i1 = face.0[1] as usize;
            let i2 = face.0[2] as usize;

            let v0 = Point3::new(
                mesh.vertices[i0].x,
                mesh.vertices[i0].y,
                mesh.vertices[i0].z,
            );
            let v1 = Point3::new(
                mesh.vertices[i1].x,
                mesh.vertices[i1].y,
                mesh.vertices[i1].z,
            );
            let v2 = Point3::new(
                mesh.vertices[i2].x,
                mesh.vertices[i2].y,
                mesh.vertices[i2].z,
            );

            let (uv0, uv1, uv2) = match texture_coordinates {
                None => (Vector2::identity(), Vector2::identity(), Vector2::identity()),
                Some(tex) => (
                    Vector2::new(tex[i0].x, tex[i0].y),
                    Vector2::new(tex[i1].x, tex[i1].y),
                    Vector2::new(tex[i2].x, tex[i2].y),
                ),
            };

            let e1 = v0 - v1;
            let e2 = v1 - v2;
            let normal = e1.cross(&e2).normalize();

            triangles.push(Triangle::new(
                [
                    Vertex {
                        position: v0,
                        normal,
                        uv: uv0,
                    },
                    Vertex {
                        position: v1,
                        normal,
                        uv: uv1,
                    },
                    Vertex {
                        position: v2,
                        normal,
                        uv: uv2,
                    },
                ],
                0,
            ));
        }

        Mesh::new(triangles)
    }

    pub fn convert_scene(scene: russimp::scene::Scene) -> anyhow::Result<Scene> {

        let meshes = scene.meshes.iter().map(|mesh| Self::convert_mesh(mesh)).collect();

        let root = Self::convert_node(scene.root.as_ref().unwrap());

        Ok(Scene::new(meshes, root))
    }

    fn convert_node(assimp_node: &Rc<Node>) -> SceneNode {
        SceneNode {
            transform: AssimpSceneConverter::convert_transform(assimp_node.transformation),
            meshes: assimp_node.meshes.clone(),
            children: assimp_node.children.borrow().iter().map(|child| Self::convert_node(child)).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::Point3;
    use russimp::material::Material;
    use super::*;

    #[test]
    fn test_import_simple_scene() {
        let scene = Scene {
            materials: vec![
                Material { properties: vec![], textures: Default::default(), }
            ],
            meshes: vec![
                russimp::mesh::Mesh {
                    faces: vec![russimp::face::Face(vec![0, 2, 1]), ],
                    vertices: vec![
                        russimp::Vector3D {x: 0.0, y: 0.0, z: 0.0},
                        russimp::Vector3D {x: 1.0, y: 2.0, z: 3.0},
                        russimp::Vector3D {x: 10.0, y: 20.0, z: 30.0}
                    ],
                    texture_coords: vec![None],
                    material_index: 0,
                    ..Default::default()
                }
            ],
            metadata: None,
            animations: vec![],
            cameras: vec![],
            lights: vec![],
            root: None,
            flags: 0,
        };

        let result = AssimpImporter::import(scene).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].triangles()[0].v0().position, Point3::new(0.0, 0.0, 0.0));
        assert_eq!(result[0].triangles()[0].v1().position, Point3::new(10.0, 20.0, 30.0));
        assert_eq!(result[0].triangles()[0].v2().position, Point3::new(1.0, 2.0, 3.0));
    }

}