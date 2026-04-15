use std::sync::Arc;
use nalgebra::{Point3, Vector2, Vector3, Vector4};
use crate::acceleration::bounds::AABB;
use crate::acceleration::kdtree::KDTree;
use crate::content::triangle::{Triangle, Vertex};
use crate::core::Ray;
use crate::scene::{Intersectable, Intersection, Shadeable};
use crate::scene::material::Material;

pub struct MeshData {
    //bounds: AABB,
    triangles: Vec<Triangle>,
    geometry: KDTree,
    material: Material,
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

        closest_intersection.and_then(|x| {
            if x.dist < t_min || x.dist > t_max {
                return None;
            }

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

            let tangent0 = x.triangle.v0().tangent;
            let tangent1 = x.triangle.v1().tangent;
            let tangent2 = x.triangle.v2().tangent;

            let tangent = tangent0 * w + tangent1 * x.barycentric.x + tangent2 * x.barycentric.y;

            Some(Intersection {
                dist: x.dist,
                tex_coord,
                normal,
                tangent,
            })
        })
    }

    /*pub fn triangles(&self) -> &[Triangle] {
        self.triangles.as_slice()
    }*/

    pub fn bounds(&self) -> AABB {
        self.geometry.bounds()
    }
}

fn transform_normal_and_tangent(
    transform: &nalgebra::Matrix4<f32>,
    normal: Vector3<f32>,
    tangent: Vector4<f32>,
) -> (Vector3<f32>, Vector4<f32>) {
    let linear = transform.fixed_view::<3, 3>(0, 0).into_owned();
    let normal_matrix = transform
        .fixed_view::<3, 3>(0, 0)
        .into_owned()
        .try_inverse()
        .unwrap()
        .transpose();
    let orientation_sign = if linear.determinant() < 0.0 { -1.0 } else { 1.0 };

    let world_normal = (normal_matrix * normal).normalize();
    let world_tangent_xyz = normal_matrix * tangent.xyz();
    let world_tangent = if world_tangent_xyz.norm_squared() <= 1e-12 {
        Vector4::new(0.0, 0.0, 0.0, tangent.w * orientation_sign)
    } else {
        world_tangent_xyz
            .normalize()
            .insert_row(3, tangent.w * orientation_sign)
    };

    (world_normal, world_tangent)
}

#[derive(Clone)]
pub struct MeshInstance {
    mesh_index: usize,
    data: Arc<MeshData>,
    position: Point3<f32>,
    transform: nalgebra::Matrix4<f32>,
    inverse_transform: nalgebra::Matrix4<f32>,
}

impl MeshInstance {
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

impl Intersectable for MeshInstance {
    fn bounds(&self) -> AABB {
        self.data.bounds()
    }

    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        let object_space_ray = ray.transform(self.inverse_transform);

        self.data.intersect(&object_space_ray, t_min, t_max).map(|x| {
            let (normal, tangent) = transform_normal_and_tangent(&self.transform, x.normal, x.tangent);
            Intersection {
                dist: x.dist,
                tex_coord: x.tex_coord,
                normal,
                tangent,
            }
        })
    }

    fn transform(&self) -> &nalgebra::Matrix4<f32> {
        &self.transform
    }
}

impl Shadeable for MeshInstance {
    fn material(&self) -> &Material {
        &self.data.material
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::{Matrix4, Vector4};
    use crate::content::triangle::Vertex;
    use super::*;

    fn create_test_mesh() -> Arc<MeshData> {
        let tangent = Vector4::new(1.0, 0.0, 0.0, 1.0);
        let triangles = vec![
            Triangle::new([
                Vertex { position: Point3::new( -1.0, 1.0, 1.0), uv: Vector2::zeros(), normal: Vector3::new(0.0, 0.0, 1.0), tangent },
                Vertex { position: Point3::new( 1.0, 1.0, 1.0), uv: Vector2::zeros(), normal: Vector3::new(0.0, 0.0, 1.0), tangent },
                Vertex { position: Point3::new( -1.0, -1.0, 1.0), uv: Vector2::zeros(), normal: Vector3::new(0.0, 0.0, 1.0), tangent }
            ]),
            Triangle::new([
                Vertex { position: Point3::new( 1.0, 1.0, 1.0), uv: Vector2::zeros(), normal: Vector3::new(0.0, 0.0, 1.0), tangent },
                Vertex { position: Point3::new( 1.0, -1.0, 1.0), uv: Vector2::zeros(), normal: Vector3::new(0.0, 0.0, 1.0), tangent },
                Vertex { position: Point3::new( -1.0, -1.0, 1.0), uv: Vector2::zeros(), normal: Vector3::new(0.0, 0.0, 1.0), tangent }
            ]),
        ];
        let material = Material::new(Vector3::zeros(), None, None, None, None, 1.0, Vector3::zeros(), 0.0, 0.0, 0.0, 1.5);
        Arc::new(MeshData::new(triangles, material))
    }
    #[test]
    fn intersect_should_return_distance_in_world_space() {
        let mesh_data = create_test_mesh();

        let transform = Matrix4::new_scaling(5.0);
        let mesh = MeshInstance::new(0, mesh_data, Point3::default(), transform);

        let ray = Ray::new(Point3::new(0.0, 0.0, 10.0), Vector3::new(0.0, 0.0, -1.0));

        let intersection = mesh.intersect(&ray, 0.0, 1000.0).unwrap();
        assert_eq!(intersection.dist, 5.0);
    }

    #[test]
    fn intersect_should_return_normal_in_world_space_for_identity_mesh() {
        let mesh_data = create_test_mesh();

        let identity_mesh = MeshInstance::new(0, mesh_data.clone(), Point3::default(), Matrix4::identity());

        let ray = Ray::new(Point3::new(0.0, 0.0, 10.0), Vector3::new(0.0, 0.0, -1.0));
        let intersection = identity_mesh.intersect(&ray, 0.0, 1000.0).unwrap();
        assert_eq!(intersection.normal, Vector3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn intersect_should_return_normal_in_world_space_for_rotated_mesh() {
        let mesh_data = create_test_mesh();

        let transform =
            Matrix4::new_rotation(Vector3::new(-std::f32::consts::FRAC_PI_2, 0.0, 0.0));
        let mesh = MeshInstance::new(0, mesh_data.clone(), Point3::default(), transform);

        let ray = Ray::new(Point3::new(0.0, 10.0, 0.0), Vector3::new(0.0, -1.0, 0.0));
        let intersection = mesh.intersect(&ray, 0.0, 1000.0).unwrap();
        
        let expected_normal = Vector3::new(0.0, 1.0, 0.0);
        for i in 0..3 {
            assert!((intersection.normal[i] - expected_normal[i]).abs() < 0.00001);     
        }
    }

    #[test]
    fn intersect_should_return_tangent_in_world_space_for_rotated_mesh() {
        let mesh_data = create_test_mesh();

        let transform =
            Matrix4::new_rotation(Vector3::new(-std::f32::consts::FRAC_PI_2, 0.0, 0.0));
        let mesh = MeshInstance::new(0, mesh_data, Point3::default(), transform);

        let ray = Ray::new(Point3::new(0.0, 10.0, 0.0), Vector3::new(0.0, -1.0, 0.0));
        let intersection = mesh.intersect(&ray, 0.0, 1000.0).unwrap();

        let expected_tangent = Vector3::new(1.0, 0.0, 0.0);
        for i in 0..3 {
            assert!((intersection.tangent[i] - expected_tangent[i]).abs() < 0.00001);
        }
        assert_eq!(intersection.tangent.w, 1.0);
    }

    #[test]
    fn intersect_should_interpolate_tangent_across_triangle() {
        let triangle = Triangle::new([
            Vertex {
                position: Point3::new(0.0, 0.0, 0.0),
                uv: Vector2::new(0.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 1.0),
                tangent: Vector4::new(1.0, 0.0, 0.0, 1.0),
            },
            Vertex {
                position: Point3::new(1.0, 0.0, 0.0),
                uv: Vector2::new(1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 1.0),
                tangent: Vector4::new(0.0, 1.0, 0.0, 1.0),
            },
            Vertex {
                position: Point3::new(0.0, 1.0, 0.0),
                uv: Vector2::new(0.0, 1.0),
                normal: Vector3::new(0.0, 0.0, 1.0),
                tangent: Vector4::new(1.0, 1.0, 0.0, 1.0),
            },
        ]);

        let material = Material::new(Vector3::zeros(), None, None, None, None, 1.0, Vector3::zeros(), 0.0, 0.0, 0.0, 1.5);
        let mesh = MeshInstance::new(
            0,
            Arc::new(MeshData::new(vec![triangle], material)),
            Point3::default(),
            Matrix4::identity(),
        );

        let ray = Ray::new(Point3::new(0.25, 0.25, 1.0), Vector3::new(0.0, 0.0, -1.0));
        let intersection = mesh.intersect(&ray, 0.0, 10.0).unwrap();

        let expected = Vector3::new(0.75, 0.5, 0.0).normalize();
        for i in 0..3 {
            assert!((intersection.tangent[i] - expected[i]).abs() < 1e-5);
        }
        assert_eq!(intersection.tangent.w, 1.0);
    }

    #[test]
    fn intersect_should_flip_tangent_handedness_for_mirrored_transform() {
        let mesh_data = create_test_mesh();
        let transform = Matrix4::new_nonuniform_scaling(&Vector3::new(-1.0, 1.0, 1.0));
        let mesh = MeshInstance::new(0, mesh_data, Point3::default(), transform);

        let ray = Ray::new(Point3::new(0.0, 0.0, 10.0), Vector3::new(0.0, 0.0, -1.0));
        let intersection = mesh.intersect(&ray, 0.0, 1000.0).unwrap();

        assert_eq!(intersection.tangent.w, -1.0);
    }
}