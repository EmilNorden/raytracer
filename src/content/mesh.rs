use std::path::Path;
use std::sync::Arc;
use nalgebra::{Point3, Vector2, Vector3};
use crate::acceleration::bounds::AABB;
use crate::acceleration::kdtree::KDTree;
use crate::content::triangle::{Triangle, Vertex};
use crate::core::Ray;
use crate::scene::{Intersectable, Intersection, Shadeable};
use crate::scene::material::Material;
use crate::scene::scene::Scene;

pub struct MeshData {
    //bounds: AABB,
    triangles: Vec<Triangle>,
    geometry: KDTree,
    material: Material,
}
#[derive(Clone)]
pub struct Mesh {
    mesh_index: usize,
    data: Arc<MeshData>,
    position: Point3<f32>,
    transform: nalgebra::Matrix4<f32>,
    inverse_transform: nalgebra::Matrix4<f32>,
}

impl MeshData {
    pub fn new<I: IntoIterator<Item = Triangle>>(triangle_iter: I, material: Material) -> Self {
        let triangles: Vec<Triangle> = triangle_iter.into_iter().collect();

        Self {
            triangles: triangles.clone(),
            geometry: KDTree::new(triangles),
            material
        }
    }

    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {

       let closest_intersection = self.geometry.intersects(ray);

        closest_intersection.map(|x| {
            // TODO: Should I only return the barycentric UV coordinates and the triangle, and only interpolate these parameters once I have found the true intersection?
            let tex_coord0 = x.triangle.v0().uv;
            let tex_coord1 = x.triangle.v1().uv;
            let tex_coord2 = x.triangle.v2().uv;

            let w = 1.0 - x.barycentric.x - x.barycentric.y;

            let tex_coord = tex_coord0 * w + tex_coord1 * x.barycentric.x + tex_coord2 * x.barycentric.y;

            let normal0 = x.triangle.v0().normal;
            let normal1 = x.triangle.v1().normal;
            let normal2 = x.triangle.v2().normal;

            let normal = normal0 * w + normal1 * x.barycentric.x + normal2 * x.barycentric.y;

            Intersection {
                dist: x.dist,
                tex_coord,
                normal
            }
        })
    }

    /*pub fn triangles(&self) -> &[Triangle] {
        self.triangles.as_slice()
    }*/

    pub fn bounds(&self) -> AABB {
        self.geometry.bounds()
    }
}


impl Mesh {
    pub fn new(mesh_index: usize, data: Arc<MeshData>, position: Point3<f32>, transform: nalgebra::Matrix4<f32>) -> Self {
        Self {
            mesh_index,
            data,
            position,
            transform,
            inverse_transform: transform.try_inverse().unwrap()
        }
    }

    pub fn mesh_index(&self) -> usize {
        self.mesh_index
    }

    pub fn position(&self) -> Point3<f32> {
        self.position
    }
    
    pub fn triangle_count(&self) -> usize {
        self.data.triangles.len()
    }

    pub fn triangle_at(&self, index: usize) -> Triangle {
        let triangle = self.data.triangles.get(index).unwrap();
        triangle.transform(&self.transform)
    }
}

impl Intersectable for Mesh {
    fn bounds(&self) -> AABB {
        self.data.bounds()
    }

    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        let object_space_ray = ray.transform(self.inverse_transform);

        self.data.intersect(&object_space_ray, t_min, t_max).map(|x| {
            Intersection {
                dist: x.dist,
                tex_coord: x.tex_coord,
                normal: self.transform.transform_vector(&x.normal).normalize()
            }
        })
    }
}

impl Shadeable for Mesh {
    fn material(&self) -> &Material {
        &self.data.material
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::{Matrix4};
    use crate::content::triangle::Vertex;
    use super::*;

    fn create_test_mesh() -> Arc<MeshData> {
        let triangles = vec![
            Triangle::new([
                Vertex { position: Point3::new( -1.0, 1.0, 1.0), uv: Vector2::zeros(), normal: Vector3::new(0.0, 0.0, 1.0) },
                Vertex { position: Point3::new( 1.0, 1.0, 1.0), uv: Vector2::zeros(), normal: Vector3::new(0.0, 0.0, 1.0) },
                Vertex { position: Point3::new( -1.0, -1.0, 1.0), uv: Vector2::zeros(), normal: Vector3::new(0.0, 0.0, 1.0) }
            ]),
            Triangle::new([
                Vertex { position: Point3::new( 1.0, 1.0, 1.0), uv: Vector2::zeros(), normal: Vector3::new(0.0, 0.0, 1.0) },
                Vertex { position: Point3::new( 1.0, -1.0, 1.0), uv: Vector2::zeros(), normal: Vector3::new(0.0, 0.0, 1.0) },
                Vertex { position: Point3::new( -1.0, -1.0, 1.0), uv: Vector2::zeros(), normal: Vector3::new(0.0, 0.0, 1.0) }
            ]),
        ];
        let material = Material::new(Vector3::zeros(), None, None, Vector3::zeros(), 0.0);
        Arc::new(MeshData::new(triangles, material))
    }
    #[test]
    fn intersect_should_return_distance_in_world_space() {
        let mesh_data = create_test_mesh();

        let transform = Matrix4::new_scaling(5.0);
        let mesh = Mesh::new(0, mesh_data, Point3::default(), transform);

        let ray = Ray::new(Point3::new(0.0, 0.0, 10.0), Vector3::new(0.0, 0.0, -1.0));

        let intersection = mesh.intersect(&ray, 0.0, 1000.0).unwrap();
        assert_eq!(intersection.dist, 5.0);
    }

    #[test]
    fn intersect_should_return_normal_in_world_space_for_identity_mesh() {
        let mesh_data = create_test_mesh();

        let identity_mesh = Mesh::new(0, mesh_data.clone(), Point3::default(), Matrix4::identity());

        let ray = Ray::new(Point3::new(0.0, 0.0, 10.0), Vector3::new(0.0, 0.0, -1.0));
        let intersection = identity_mesh.intersect(&ray, 0.0, 1000.0).unwrap();
        assert_eq!(intersection.normal, Vector3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn intersect_should_return_normal_in_world_space_for_rotated_mesh() {
        let mesh_data = create_test_mesh();

        let transform =
            Matrix4::new_rotation(Vector3::new(-std::f32::consts::FRAC_PI_2, 0.0, 0.0));
        let mesh = Mesh::new(0, mesh_data.clone(), Point3::default(), transform);

        let ray = Ray::new(Point3::new(0.0, 10.0, 0.0), Vector3::new(0.0, -1.0, 0.0));
        let intersection = mesh.intersect(&ray, 0.0, 1000.0).unwrap();
        
        let expected_normal = Vector3::new(0.0, 1.0, 0.0);
        for i in 0..3 {
            assert!((intersection.normal[i] - expected_normal[i]).abs() < 0.00001);     
        }
    }
}