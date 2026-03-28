use std::f32::consts::PI;
use nalgebra::Vector3;
use rand::Rng;
use crate::scene::coordinate_system::CoordinateSystem;
use crate::scene::Intersection;
use crate::scene::texture::Texture;

pub struct Material {
    /*
    - Some BRDF should be attached => Determines how it interacts with light
    - Also, some way to get albedo (color) at specific point. A reference to a texture?
     */

    color: Vector3<f32>,
    texture: Option<Texture>,
    emissive_texture: Option<Texture>,
    emissive: Vector3<f32>,
    roughness: f32,
    f0: Vector3<f32>,

    /*
    metallic is a scalar value in
    [0.0,1.0] that indicates whether a material behaves like:

    A dielectric (non-metal, e.g., wood, plastic) → metallic = 0.0
    A metal (e.g., gold, copper) → metallic = 1.0
    Or something in between (blended material) → metallic = 0.5
    In glTF, it’s part of the baseColorTexture + metallicRoughnessTexture bundle.
     */
    metallic: f32,
}

pub struct BsdfSample {
    pub direction: Vector3<f32>,
    pub bsdf_value: Vector3<f32>,
    pub pdf: f32,
    pub is_reflection: bool,
    pub albedo: Vector3<f32>,
}

impl Material {
    pub fn new(color: Vector3<f32>, texture: Option<Texture>, emissive_texture: Option<Texture>, emissive: Vector3<f32>, roughness: f32) -> Self {
        Self { color, texture, emissive_texture, emissive, roughness, f0: Vector3::new(0.4, 0.4, 0.4), metallic: 0.0 }
    }

    pub fn color(&self) -> Vector3<f32> { self.color }
    pub fn roughness(&self) -> f32 { self.roughness }
    pub fn emissive_factor(&self) -> Vector3<f32> { self.emissive }

    pub fn sample_color(&self, u: f32, v: f32) -> Vector3<f32> {
        self.texture.as_ref().map(|t| t.sample_color(u, v)).unwrap_or(self.color)
    }

    pub fn sample_emissive(&self, u: f32, v: f32) -> Vector3<f32> {
        self.emissive
        //self.emissive_texture.as_ref().map(|t| {t.sample_color(u, v).component_mul(&self.emissive)}).unwrap_or(self.emissive)
    }

    fn build_orthonormal_basis(normal: &Vector3<f32>) -> (Vector3<f32>, Vector3<f32>) {
        // Choose arbitrary vector not parallel to normal
        let up = if normal.abs().x < 0.9 {
            Vector3::new(1.0, 0.0, 0.0)
        } else {
            Vector3::new(0.0, 1.0, 0.0)
        };

        let tangent = up.cross(normal).normalize();
        let bitangent = normal.cross(&tangent);

        (tangent, bitangent)
    }

    /*fn cosine_sample_hemisphere(normal: &Vector3<f32>, rng: &mut impl Rng) -> (Vector3<f32>, f32) {
        // Generate two uniform random numbers
        let u1: f32 = rng.random();
        let u2: f32 = rng.random();

        // Cosine-weighted sampling in polar coordinates
        let r = u1.sqrt();
        let theta = 2.0 * std::f32::consts::PI * u2;

        // Convert to Cartesian in tangent space (z = normal direction)
        let x = r * theta.cos();
        let y = r * theta.sin();
        let z = (1.0 - u1).sqrt();  // This ensures cosine weighting

        // Build orthonormal basis from normal
        let (tangent, bitangent) = Self::build_orthonormal_basis(&normal);

        // Transform from tangent space to world space
        let dir = tangent * x + bitangent * y + normal * z;

        // Return direction AND the PDF value (cos(θ)/π)
        let pdf = z / std::f32::consts::PI;

        (dir, pdf)
    }*/

    pub fn sample_lambertian_bsdf(&self, _incoming: Vector3<f32>, normal: Vector3<f32>, rng: &mut impl Rng) -> BsdfSample {

        let local_system = CoordinateSystem::from_normal(&normal);
        let local_dir = Self::cosine_sample_hemisphere(rng);

        let direction = local_system.u * local_dir.x + local_system.v * local_dir.y + local_system.w * local_dir.z;

        // Cosine-weighted PDF
        let pdf = direction.dot(&normal).max(0.0) / PI;

        let bsdf_value = self.color / PI;


        BsdfSample {
            direction,
            bsdf_value,
            pdf,
            is_reflection: true,
            albedo: self.color,
        }
    }

    fn cosine_sample_hemisphere(rng: &mut impl Rng) -> Vector3<f32> {
        let phi: f32 = 2.0 * PI * rng.random::<f32>();  // Random angle around Z
        let cos_theta = rng.random::<f32>().sqrt();  // Cosine of polar angle
        let sin_theta = (1.0f32 - cos_theta * cos_theta).sqrt();

        Vector3::new(
            sin_theta * phi.cos(),
            sin_theta * phi.sin(),
            cos_theta
        )
    }

    // GGX BRDF
    pub fn brdf(&self, light_dir: &Vector3<f32>, view_dir: &Vector3<f32>, normal: &Vector3<f32>) -> Vector3<f32> {
        let half_vector = (light_dir + view_dir).normalize();
        let n_dot_l = normal.dot(&light_dir).max(0.0);
        let n_dot_v = normal.dot(&view_dir).max(0.0);
        let n_dot_h = normal.dot(&half_vector).max(0.0);
        let v_dot_h = view_dir.dot(&half_vector).max(0.0);
        let alpha = self.roughness * self.roughness;

        let d = self.ggx_distribution(n_dot_h, alpha);
        //let g = self.geometry_smith(&normal, &view_dir, &light_dir, alpha);
        let g = self.schlick_masking_term(n_dot_v, n_dot_l, alpha);
        let f = self.fresnel_schlick(v_dot_h, Vector3::new(0.04, 0.04, 0.04));

        //f * (d * g / (4.0 * n_dot_v * n_dot_l + 1e-5))
        d * g * f / (4.0 * n_dot_v * n_dot_l + 1e-5)
    }

    fn ggx_distribution(&self, n_dot_h: f32, alpha: f32) -> f32 {
        // AI suggested this:
        //let denom = n_dot_h * n_dot_h * (alpha * alpha) + 1.0;
        let denom = (n_dot_h * alpha - n_dot_h) * n_dot_h + 1.0;
        alpha / (std::f32::consts::PI * denom * denom)
    }

    // Smith’s separable masking-shadowing approximation:
    fn geometry_schlick_ggx(&self, n_dot_v: f32, alpha: f32) -> f32 {
        // AI suggested this:
        let k = (alpha + 1.0).powi(2) / 8.0;

        n_dot_v / (n_dot_v * (1.0 - k) + k)
    }

    fn geometry_smith(&self, n: &Vector3<f32>, v: &Vector3<f32>, l: &Vector3<f32>, alpha: f32) -> f32 {
        // AI suggested this:
        //let k = (alpha + 1.0).powi(2) / 8.0;

        let n_dot_v = n.dot(v).max(0.0);
        let n_dot_l = n.dot(l).max(0.0);
        let ggx1 = self.geometry_schlick_ggx(n_dot_v, alpha);
        let ggx2 = self.geometry_schlick_ggx(n_dot_l, alpha);

        ggx1 * ggx2
    }

    fn schlick_masking_term(&self, n_dot_v: f32, n_dot_l: f32, alpha: f32) -> f32 {
        let k = alpha / 2.0;

        let g_v = n_dot_v / (n_dot_v * (1.0 - k) + k);
        let g_l = n_dot_l / (n_dot_l * (1.0 - k) + k);
        g_v * g_l
    }

    fn fresnel_schlick(&self, l_dot_h: f32, f0: Vector3<f32>) -> Vector3<f32> {
        // AI suggested cos_theta instead of l_dot_h and cos_theta = v_dot_h in its example.
        f0 + (Vector3::new(1.0, 1.0, 1.0) - f0) * (1.0 - l_dot_h).powf(5.0)
    }




    // ----------------- 8< --------------

   pub  fn sample(&self, incoming: Vector3<f32>, normal: Vector3<f32>, u: f32, v: f32, rng: &mut impl Rng) -> (Vector3<f32>, Vector3<f32>, f32, f32) {
        let alpha = self.roughness * self.roughness;
       let albedo = self.sample_color(u, v);

        // Step 1: Sample microfacet normal (GGX importance sampling)
        let u1: f32 = rng.random();
        let u2: f32 = rng.random();

        let phi = 2.0 * std::f32::consts::PI * u1;
        let cos_theta = ((1.0 - u2) / (1.0 + (alpha * alpha - 1.0) * u2)).sqrt();
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let h = Vector3::new(
            sin_theta * phi.cos(),
            cos_theta,
            sin_theta * phi.sin(),
        );

        // Step 2: Reflect incoming around h to get outgoing
        let outgoing = Self::reflect(-incoming, h);

        // Step 3: Check if outgoing is above the surface
        if outgoing.dot(&normal) <= 0.0 {
            return (Vector3::zeros(), Vector3::zeros(), 0.0, 0.0);
        }

        // Step 4: Compute components of microfacet BRDF
        let n_dot_l = outgoing.dot(&normal).max(0.0);
        let n_dot_v = incoming.dot(&normal).max(0.0);
        let n_dot_h = h.dot(&normal).max(0.0);
        let v_dot_h = incoming.dot(&h).max(0.0);

        let d = Self::ggx_ndf(n_dot_h, alpha);
        let g = Self::smith_geometry(n_dot_v, n_dot_l, alpha);
        let f = Self::schlick_fresnel(v_dot_h, self.f0); // self.f0 is base reflectivity Vec3

       let kd = (Vector3::new(1.0, 1.0, 1.0) - f) * (1.0 - self.metallic);
       let diffuse_brdf = kd.component_mul(&albedo) / std::f32::consts::PI;
       
       let specular_brdf = f * (d * g / (4.0 * n_dot_v * n_dot_l + 1e-5));


       // let brdf = f * (d * g / (4.0 * n_dot_v * n_dot_l + 1e-5));
       let brdf = diffuse_brdf + specular_brdf;

        // Step 5: Compute PDF for GGX importance sampling
        let pdf = d * n_dot_h / (4.0 * v_dot_h + 1e-5);

        (outgoing, brdf, pdf, n_dot_l)
    }

    fn reflect(v: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32> {
        v - 2.0 * v.dot(&n) * n
    }

    fn ggx_ndf(n_dot_h: f32, alpha: f32) -> f32 {
        let a2 = alpha * alpha;
        let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
        a2 / (std::f32::consts::PI * denom * denom + 1e-5)
    }

    fn smith_geometry(n_dot_v: f32, n_dot_l: f32, alpha: f32) -> f32 {
        let ggx1 = Self::smith_g1(n_dot_v, alpha);
        let ggx2 = Self::smith_g1(n_dot_l, alpha);
        ggx1 * ggx2
    }

    fn smith_g1(n_dot_x: f32, alpha: f32) -> f32 {
        let tan2 = (1.0 - n_dot_x * n_dot_x) / (n_dot_x * n_dot_x + 1e-5);
        2.0 / (1.0 + (1.0 + alpha * alpha * tan2).sqrt())
    }

    fn schlick_fresnel(cos_theta: f32, f0: Vector3<f32>) -> Vector3<f32> {
        f0 + (Vector3::new(1.0, 1.0, 1.0) - f0) * (1.0 - cos_theta).powf(5.0)
    }
}