use std::f32::consts::PI;
use nalgebra::Vector3;
use rand::Rng;
use crate::scene::coordinate_system::CoordinateSystem;
use crate::scene::texture::Texture;

pub struct Material {
    /*
    - Some BRDF should be attached => Determines how it interacts with light
    - Also, some way to get albedo (color) at specific point. A reference to a texture?
     */

    color: Vector3<f32>,
    texture: Option<Texture>,
    normal_map: Option<Texture>,
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
    In glTF, it's part of the baseColorTexture + metallicRoughnessTexture bundle.
     */
    metallic: f32,

    // Transmission/refraction properties
    transmission_factor: f32,  // 0.0 = opaque, 1.0 = fully transparent
    ior: f32,                  // Index of refraction (1.5 for glass, 1.33 for water)
}

pub struct BsdfSample {
    pub direction: Vector3<f32>,
    pub bsdf_value: Vector3<f32>,
    pub pdf: f32,
    pub is_reflection: bool,
    pub is_transmission: bool,
    pub albedo: Vector3<f32>,
}

impl Material {
    pub fn new(color: Vector3<f32>, texture: Option<Texture>, normal_map: Option<Texture>, emissive_texture: Option<Texture>, emissive: Vector3<f32>, roughness: f32, metallic: f32, transmission_factor: f32, ior: f32) -> Self {
        Self {
            color,
            texture,
            normal_map,
            emissive_texture,
            emissive,
            roughness,
            f0: Vector3::new(0.04, 0.04, 0.04),  // Standard dielectric F0
            metallic,
            transmission_factor,
            ior,
        }
    }

    pub fn color(&self) -> Vector3<f32> { self.color }
    pub fn roughness(&self) -> f32 { self.roughness }
    pub fn emissive_factor(&self) -> Vector3<f32> { self.emissive }
    pub fn transmission_factor(&self) -> f32 { self.transmission_factor }
    pub fn ior(&self) -> f32 { self.ior }

    pub fn set_transmission(&mut self, transmission_factor: f32, ior: f32) {
        self.transmission_factor = transmission_factor.clamp(0.0, 1.0);
        self.ior = ior.max(1.0);  // IOR must be >= 1.0
    }

    pub fn sample_color(&self, u: f32, v: f32) -> Vector3<f32> {
        self.texture.as_ref().map(|t| t.sample_color(u, v)).unwrap_or(self.color)
    }

    pub fn sample_emissive(&self, _u: f32, _v: f32) -> Vector3<f32> {
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

    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    /// Refract a ray using Snell's law
    /// Returns (refracted_direction, eta_ratio) where eta_ratio = eta_t / eta_i
    fn refract(incoming: Vector3<f32>, normal: Vector3<f32>, eta_ratio: f32) -> Option<Vector3<f32>> {
        let v = (-incoming).normalize();
        let n = normal.normalize();
        let n_dot_v = n.dot(&v);

        // If ray hits from inside, we need to flip normal and adjust eta_ratio
        let (n_final, n_dot_v_final, eta_final) = if n_dot_v < 0.0 {
            (-n, -n_dot_v, 1.0 / eta_ratio)
        } else {
            (n, n_dot_v, eta_ratio)
        };

        let discriminant = 1.0 - eta_final * eta_final * (1.0 - n_dot_v_final * n_dot_v_final);
        if discriminant < 0.0 {
            return None;  // Total internal reflection
        }

        let refracted = eta_final * (-v) + (eta_final * n_dot_v_final - discriminant.sqrt()) * n_final;
        Some(refracted.normalize())
    }

    /// Compute Fresnel reflectance for dielectrics (unpolarized light)
    /// Uses the dielectric Fresnel equation
    fn fresnel_dielectric(cos_theta: f32, eta_ratio: f32) -> f32 {
        let cos_theta = cos_theta.abs().clamp(0.0, 1.0);

        // Snell's law to find transmission angle
        let sin2_theta_t = eta_ratio * eta_ratio * (1.0 - cos_theta * cos_theta);
        if sin2_theta_t > 1.0 {
            return 1.0;  // Total internal reflection
        }

        let cos_theta_t = (1.0 - sin2_theta_t).sqrt();

        // Fresnel equations for unpolarized light
        let r_s = (cos_theta - eta_ratio * cos_theta_t) / (cos_theta + eta_ratio * cos_theta_t);
        let r_p = (eta_ratio * cos_theta - cos_theta_t) / (eta_ratio * cos_theta + cos_theta_t);

        (r_s * r_s + r_p * r_p) / 2.0
    }


    pub fn sample_bsdf(&self, incoming: Vector3<f32>, normal: Vector3<f32>, albedo: Vector3<f32>, rng: &mut impl Rng) -> BsdfSample {
        let n = normal.normalize();
        let v = (-incoming).normalize();
        let n_dot_v = n.dot(&v).max(0.0);

        // Handle transmission (refraction) for transparent materials
        if self.transmission_factor > 0.0 && rng.random::<f32>() < self.transmission_factor {
            let eta_ratio = self.ior;  // Assume coming from air (eta=1.0)

            if let Some(refracted_dir) = Self::refract(incoming, normal, eta_ratio) {
                let v_dot_n = v.dot(&n).abs();
                let fresnel = Self::fresnel_dielectric(v_dot_n, eta_ratio);

                // Transmission probability (1 - Fresnel reflection)
                let transmission_prob = 1.0 - fresnel;
                if transmission_prob > 1e-6 {
                    // No absorption in the BSDF value for perfect transmission
                    // (absorption would be applied by distance traveled or volume rendering)
                    return BsdfSample {
                        direction: refracted_dir,
                        bsdf_value: Vector3::new(1.0, 1.0, 1.0),
                        pdf: self.transmission_factor * transmission_prob,
                        is_reflection: false,
                        is_transmission: true,
                        albedo,
                    };
                }
            }
            // If total internal reflection, fall through to reflection
        }

        if n_dot_v <= 0.0 {
            return BsdfSample {
                direction: n,
                bsdf_value: Vector3::zeros(),
                pdf: 0.0,
                is_reflection: true,
                is_transmission: false,
                albedo,
            };
        }

        let alpha = self.alpha();
        let f0 = self.f0_from_albedo(&albedo);
        let specular_prob = self.specular_sampling_probability(&f0);

        if rng.random::<f32>() < specular_prob {
            let h = self.sample_ggx_half_vector(&n, alpha, rng);
            let v_dot_h = v.dot(&h).max(0.0);
            if v_dot_h <= 1e-6 {
                return BsdfSample {
                    direction: n,
                    bsdf_value: Vector3::zeros(),
                    pdf: 0.0,
                    is_reflection: true,
                    is_transmission: false,
                    albedo,
                };
            }

            let l = Self::reflect(-v, h).normalize();
            let n_dot_l = n.dot(&l).max(0.0);
            if n_dot_l <= 0.0 {
                return BsdfSample {
                    direction: l,
                    bsdf_value: Vector3::zeros(),
                    pdf: 0.0,
                    is_reflection: true,
                    is_transmission: false,
                    albedo,
                };
            }

            let n_dot_h = n.dot(&h).max(0.0);
            let d = Self::ggx_ndf(n_dot_h, alpha);
            let g = Self::smith_geometry(n_dot_v, n_dot_l, alpha);
            let f = Self::schlick_fresnel(v_dot_h, f0);
            let bsdf_value = f * (d * g / (4.0 * n_dot_v * n_dot_l + 1e-6));
            let pdf_spec = d * n_dot_h / (4.0 * v_dot_h + 1e-6);

            return BsdfSample {
                direction: l,
                bsdf_value,
                pdf: specular_prob * pdf_spec,
                is_reflection: true,
                is_transmission: false,
                albedo,
            };
        }

        let local_system = CoordinateSystem::from_normal(&n);
        let local_dir = Self::cosine_sample_hemisphere(rng);
        let direction = (local_system.u * local_dir.x + local_system.v * local_dir.y + local_system.w * local_dir.z).normalize();
        let n_dot_l = n.dot(&direction).max(0.0);
        if n_dot_l <= 0.0 {
            return BsdfSample {
                direction,
                bsdf_value: Vector3::zeros(),
                pdf: 0.0,
                is_reflection: true,
                is_transmission: false,
                albedo,
            };
        }

        let kd = 1.0 - self.metallic;
        let bsdf_value = albedo * (kd / PI);
        let pdf_diffuse = n_dot_l / PI;

        BsdfSample {
            direction,
            bsdf_value,
            pdf: (1.0 - specular_prob) * pdf_diffuse,
            is_reflection: true,
            is_transmission: false,
            albedo,
        }
    }

pub fn sample_lambertian_bsdf(&self, _incoming: Vector3<f32>, normal: Vector3<f32>, albedo: Vector3<f32>, rng: &mut impl Rng) -> BsdfSample {

        let local_system = CoordinateSystem::from_normal(&normal);
        let local_dir = Self::cosine_sample_hemisphere(rng);

        let direction = local_system.u * local_dir.x + local_system.v * local_dir.y + local_system.w * local_dir.z;

        // Cosine-weighted PDF
        let pdf = direction.dot(&normal).max(0.0) / PI;

        let bsdf_value = albedo / PI;


        BsdfSample {
            direction,
            bsdf_value,
            pdf,
            is_reflection: true,
            is_transmission: false,
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
    pub fn brdf(&self, light_dir: &Vector3<f32>, view_dir: &Vector3<f32>, normal: &Vector3<f32>, albedo: &Vector3<f32>) -> Vector3<f32> {
        let half_vector = (light_dir + view_dir).normalize();
        let n_dot_l = normal.dot(&light_dir).max(0.0);
        let n_dot_v = normal.dot(&view_dir).max(0.0);
        let n_dot_h = normal.dot(&half_vector).max(0.0);
        let v_dot_h = view_dir.dot(&half_vector).max(0.0);
        if n_dot_l <= 0.0 || n_dot_v <= 0.0 {
            return Vector3::zeros();
        }

        let alpha = self.alpha();
        let f0 = self.f0_from_albedo(albedo);

        let d = Self::ggx_ndf(n_dot_h, alpha);
        let g = Self::smith_geometry(n_dot_v, n_dot_l, alpha);
        let f = Self::schlick_fresnel(v_dot_h, f0);
        let specular = f * (d * g / (4.0 * n_dot_v * n_dot_l + 1e-6));
        let diffuse = albedo * ((1.0 - self.metallic) / PI);

        diffuse + specular
    }

    fn alpha(&self) -> f32 {
        let roughness = self.roughness.clamp(0.02, 1.0);
        (roughness * roughness).max(1e-4)
    }

    fn f0_from_albedo(&self, albedo: &Vector3<f32>) -> Vector3<f32> {
        let dielectric_f0 = Vector3::new(0.04, 0.04, 0.04);
        dielectric_f0 + (albedo - dielectric_f0) * self.metallic
    }

    fn specular_sampling_probability(&self, f0: &Vector3<f32>) -> f32 {
        let max_f0 = f0.x.max(f0.y).max(f0.z);
        Self::lerp(0.08, 0.95, max_f0).clamp(0.08, 0.95)
    }

    fn sample_ggx_half_vector(&self, normal: &Vector3<f32>, alpha: f32, rng: &mut impl Rng) -> Vector3<f32> {
        let u1: f32 = rng.random();
        let u2: f32 = rng.random();

        let phi = 2.0 * PI * u1;
        let a2 = alpha * alpha;
        let cos_theta = ((1.0 - u2) / (1.0 + (a2 - 1.0) * u2)).sqrt();
        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();

        let h_local = Vector3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);
        let basis = CoordinateSystem::from_normal(normal);
        (basis.u * h_local.x + basis.v * h_local.y + basis.w * h_local.z).normalize()
    }

    fn fresnel_schlick(&self, l_dot_h: f32, f0: Vector3<f32>) -> Vector3<f32> {
        Self::schlick_fresnel(l_dot_h, f0)
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