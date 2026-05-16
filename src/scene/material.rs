use std::f32::consts::PI;
use nalgebra::{Matrix3, Vector2, Vector3, Vector4};
use rand::Rng;
use crate::context::Context;
use crate::math::lerp;
use crate::scene::coordinate_system::CoordinateSystem;
use crate::scene::texture::{Texture};
use crate::static_stack::StaticStack;
use crate::consts::ETA_STACK_SIZE;

pub const IOR_AIR: f32 = 1.000277;

pub struct Material {
    /*
    - Some BRDF should be attached => Determines how it interacts with light
    - Also, some way to get albedo (color) at specific point. A reference to a texture?
     */

    color: Vector3<f32>,
    texture: Option<Texture>,
    normal_map: Option<Texture>,
    emissive_texture: Option<Texture>,
    metallic_roughness_texture: Option<Texture>,
    normal_scale: f32,
    emissive: Vector3<f32>,
    roughness: f32,

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
    invert_albedo: bool,
}

pub struct CachedTextureLookups<'a> {
    material: &'a Material,
    tex_coords: Vector2<f32>,
    albedo: Vector3<f32>,
    emissive: Option<Vector3<f32>>,
    metallic_roughness: Option<(f32, f32)>,
}

impl<'a> CachedTextureLookups<'a> {
    pub fn new(material: &'a Material, tex_coords: Vector2<f32>) -> Self {
        let albedo = material.sample_color(tex_coords.x, tex_coords.y);
        Self {
            material,
            tex_coords,
            albedo,
            emissive: None,
            metallic_roughness: None,
        }
    }

    pub fn albedo(&self) -> Vector3<f32> { self.albedo }
    pub fn emissive(&mut self) -> Vector3<f32> {
        *self.emissive.get_or_insert_with(|| self.material.sample_emissive(self.tex_coords.x, self.tex_coords.y))
    }
    pub fn metallic(&mut self) -> f32 {
        self.metallic_roughness.get_or_insert_with(|| self.material.sample_metallic_roughness(self.tex_coords)).0
    }
    pub fn roughness(&mut self) -> f32 {

        self.metallic_roughness.get_or_insert_with(|| self.material.sample_metallic_roughness(self.tex_coords)).1
    }
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
    pub fn new(color: Vector3<f32>, texture: Option<Texture>, normal_map: Option<Texture>, emissive_texture: Option<Texture>, metallic_roughness_texture: Option<Texture>, normal_scale: f32, emissive: Vector3<f32>, roughness: f32, metallic: f32, transmission_factor: f32, ior: f32, invert_albedo: bool) -> Self {
        Self {
            color,
            texture,
            normal_map,
            emissive_texture,
            metallic_roughness_texture,
            normal_scale,
            emissive,
            roughness,
            metallic,
            transmission_factor,
            ior,
            invert_albedo
        }
    }

    pub fn sample_metallic_roughness(&self, tex_coords: Vector2<f32>) -> (f32, f32) {
        self.metallic_roughness_texture.as_ref().map(|t| {
            let color = t.sample_color(tex_coords.x, tex_coords.y);
            (color.z, color.y)
        }).unwrap_or((self.metallic, self.roughness))
    }

    pub fn apply_normal_map(&self, normal: Vector3<f32>, tangent: Vector4<f32>, tex_coords: Vector2<f32>) -> Vector3<f32> {
        if let Some(normal_map) = &self.normal_map {
            let shading_normal = normal.normalize();
            let tangent_xyz = tangent.xyz();

            if tangent_xyz.norm_squared() <= 1e-12 {
                return shading_normal;
            }

            let tangent_dir =
                (tangent_xyz - shading_normal * shading_normal.dot(&tangent_xyz)).normalize();
            let handedness = if tangent.w < 0.0 { -1.0 } else { 1.0 };
            let bitangent = shading_normal.cross(&tangent_dir).normalize() * handedness;
            let tbn = Matrix3::from_columns(&[tangent_dir, bitangent, shading_normal]);

            let mut normal_tangent_space =
                normal_map.sample_color(tex_coords.x, tex_coords.y) * 2.0 - Vector3::new(1.0, 1.0, 1.0);
            normal_tangent_space.x *= self.normal_scale;
            normal_tangent_space.y *= self.normal_scale;
            normal_tangent_space = normal_tangent_space.normalize();

            (tbn * normal_tangent_space).normalize()
        }
        else {
            normal.normalize()
        }
    }

    pub fn emissive_factor(&self) -> Vector3<f32> { self.emissive }
    pub fn transmission_factor(&self) -> f32 { self.transmission_factor }
    pub fn ior(&self) -> f32 { self.ior }

    pub fn set_transmission(&mut self, transmission_factor: f32, ior: f32) {
        self.transmission_factor = transmission_factor.clamp(0.0, 1.0);
        self.ior = ior.max(1.0);  // IOR must be >= 1.0
    }

    pub fn sample_color(&self, u: f32, v: f32) -> Vector3<f32> {
        let albedo = self.texture.as_ref().map(|t| t.sample_color(u, v)).unwrap_or(self.color);
        if self.invert_albedo {
            Vector3::repeat(1.0) - albedo
        }
        else {
            albedo
        }
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

    /*fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }*/

    // Refracts using Snell's law. eta_ratio is the ratio of indices of refraction (eta_i / eta_t)
    // Returns None if total internal reflection.
    fn refract(incident: Vector3<f32>, mut normal: Vector3<f32>, eta_ratio: f32) -> Option<Vector3<f32>> {
        let mut cos_i = incident.dot(&normal);

        if cos_i > 0.0 {
            normal = -normal;
        } else {
            cos_i = -cos_i;
        }

        let k = 1.0 - eta_ratio * eta_ratio * (1.0 - cos_i * cos_i);
        if k < 0.0 {
            None
        }
        else {
            let refracted = eta_ratio * incident + (eta_ratio * cos_i - k.sqrt()) * normal;
            Some(refracted.normalize())
        }
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


    /// Sample one BSDF lobe and return the sampled direction together with the
    /// corresponding BSDF value and the PDF of generating that sample.
    ///
    /// Note: `bsdf_value` here is the contribution of the sampled lobe, not a
    /// full evaluation of all lobes. That is intentional because `pdf` is also
    /// branch-conditioned (e.g. `specular_prob * pdf_spec`).
    pub fn sample_bsdf(&self, incoming: Vector3<f32>, normal: Vector3<f32>, albedo: Vector3<f32>, cached_textures: &mut CachedTextureLookups, rng: &mut impl Rng, eta_stack: &mut StaticStack<f32, ETA_STACK_SIZE>, ctx: &Context) -> BsdfSample {
        let n = normal;
        let v = (-incoming).normalize();
        let n_dot_v = n.dot(&v);
        let n_dot_v_max = n_dot_v.max(0.0);
        let exiting_material = n_dot_v < 0.0;
        let transmissive = self.transmission_factor > 0.0;

        // Handle transmission (refraction) for transparent materials
        if self.transmission_factor > 0.0 && rng.random::<f32>() < self.transmission_factor {
            let eta_ratio = if exiting_material { // Refracting from inside. Assuming we are leaving material
               /*
               It seems that sometimes we find ourselves leaving materials we never entered.
               I suspect this occurs when a transmissive object such as a glass sphere is located
               next to another reflective object. When a ray intersection occurs on the reflective object just on the border
               to the transmissive object, the reflected ray is slightly offset along the surface normal so as to not self-intersect,
               however causing the ray to be pushed into the transmissive object.
               For now, the workaround is to fallback to eta_t=IOR_AIR when that happens*/

                ctx.diag.exit_without_enter();

                self.ior / eta_stack.peek_at(1).unwrap_or(IOR_AIR)
            }
            else {
                 eta_stack.peek() / self.ior
            };


            if let Some(refracted_dir) = Self::refract(incoming, normal, eta_ratio) {
                let n_dot_v_abs = n_dot_v.abs();
                let fresnel = Self::fresnel_dielectric(n_dot_v_abs, eta_ratio);

                // Transmission probability (1 - Fresnel reflection)
                let transmission_prob = 1.0 - fresnel;
                if transmission_prob > 1e-6 {
                    if exiting_material {
                        eta_stack.pop();
                        if eta_stack.is_empty() {
                            // eta stack underflow. This shouldn't happen, but could happen
                            // for the same reason as above. Workaround: make sure there's always an eta on the stack
                            ctx.diag.eta_underflow();
                            eta_stack.push(IOR_AIR);
                        }
                    }
                    else {
                        eta_stack.push(self.ior);
                    }
                    // No absorption in the BSDF value for perfect transmission
                    // (absorption would be applied by distance traveled or volume rendering)
                    return BsdfSample {
                        direction: refracted_dir,
                        bsdf_value: Vector3::repeat(self.transmission_factor),
                        pdf: self.transmission_factor * transmission_prob,
                        is_reflection: false,
                        is_transmission: true,
                        albedo,
                    };
                }
            }
            // If total internal reflection, fall through to reflection
        }

        if n_dot_v_max <= 0.0 {
            return BsdfSample {
                direction: n,
                bsdf_value: Vector3::zeros(),
                pdf: 0.0,
                is_reflection: true,
                is_transmission: false,
                albedo,
            };
        }

        let alpha = self.alpha(cached_textures);
        let f0 = self.f0_from_albedo(&albedo, cached_textures);
        let specular_prob = if transmissive {
            1.0
        } else {
            self.specular_sampling_probability(&f0)
        };

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
            let g = Self::smith_geometry(n_dot_v_max, n_dot_l, alpha);
            let f = Self::schlick_fresnel(v_dot_h, f0);
            let bsdf_value = f * (d * g / (4.0 * n_dot_v_max * n_dot_l + 1e-6));
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

        let kd = (1.0 - self.transmission_factor) * (1.0 - cached_textures.metallic());
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

    /// Evaluate the **reflective** lobe (BRDF) for a specific pair of directions.
    ///
    /// Both `light_dir` and `view_dir` must be in the same hemisphere as `normal`
    /// (`n·l > 0` and `n·v > 0`). Use [`evaluate_btdf`] when light arrives from the
    /// opposite side of a transmissive surface.
    pub fn evaluate_bsdf(&self, light_dir: &Vector3<f32>, view_dir: &Vector3<f32>, normal: &Vector3<f32>, albedo: &Vector3<f32>, cached_textures: &mut CachedTextureLookups) -> Vector3<f32> {
        let n_dot_l = normal.dot(&light_dir).max(0.0);
        let n_dot_v = normal.dot(&view_dir).max(0.0);

        if n_dot_l <= 0.0 || n_dot_v <= 0.0 {
            return Vector3::zeros();
        }

        let half_vector = (light_dir + view_dir).normalize();
        let n_dot_h = normal.dot(&half_vector).max(0.0);
        let v_dot_h = view_dir.dot(&half_vector).max(0.0);

        let alpha = self.alpha(cached_textures);
        let f0 = self.f0_from_albedo(albedo, cached_textures);

        let d = Self::ggx_ndf(n_dot_h, alpha);
        let g = Self::smith_geometry(n_dot_v, n_dot_l, alpha);
        let f = Self::schlick_fresnel(v_dot_h, f0);
        let specular = f * (d * g / (4.0 * n_dot_v * n_dot_l + 1e-6));
        let diffuse_weight = (1.0 - self.transmission_factor) * (1.0 - cached_textures.metallic());
        let diffuse = albedo * (diffuse_weight / PI);

        diffuse + specular
    }

    /// Evaluate the **transmissive** lobe (BTDF) for a specific pair of directions.
    ///
    /// Call this during next-event estimation when light arrives from **behind** the
    /// surface (i.e. `n·l < 0`) and the material is transmissive. Uses the GGX
    /// microfacet BTDF (Walter et al. 2007).
    ///
    /// * `light_dir`  – unit vector pointing **toward** the light (below the surface, `n·l < 0`).
    /// * `view_dir`   – unit vector pointing **toward** the viewer (above the surface, `n·v > 0`).
    /// * `eta_i`      – IOR on the viewer's side (use [`IOR_AIR`] if the viewer is in air).
    /// * `eta_t`      – IOR on the light's side (typically `self.ior()` for glass/water).
    pub fn evaluate_btdf(
        &self,
        light_dir: &Vector3<f32>,
        view_dir: &Vector3<f32>,
        normal: &Vector3<f32>,
        albedo: &Vector3<f32>,
        cached_textures: &mut CachedTextureLookups,
        eta_i: f32,
        eta_t: f32,
    ) -> Vector3<f32> {
        if self.transmission_factor <= 0.0 {
            return Vector3::zeros();
        }

        let n_dot_v = normal.dot(view_dir);
        let n_dot_l = normal.dot(light_dir);

        // Viewer must be above the surface, light must be below it.
        if n_dot_v <= 0.0 || n_dot_l >= 0.0 {
            return Vector3::zeros();
        }

        // Refraction half-vector (Walter et al. 2007, eq. 16).
        // Points into the same hemisphere as the normal.
        let h_unnorm = -(eta_i * view_dir + eta_t * light_dir);
        if h_unnorm.norm_squared() < 1e-12 {
            return Vector3::zeros();
        }
        let h = {
            let h = h_unnorm.normalize();
            if normal.dot(&h) < 0.0 { -h } else { h }
        };

        let v_dot_h = view_dir.dot(&h).max(0.0);
        // light_dir points away from the viewer, so negate it to get the
        // cosine of the angle between h and the light-side ray.
        let l_dot_h = (-light_dir).dot(&h).max(0.0);
        let n_dot_h  = normal.dot(&h).max(0.0);

        let alpha  = self.alpha(cached_textures);
        let d      = Self::ggx_ndf(n_dot_h, alpha);
        let g      = Self::smith_geometry(n_dot_v, n_dot_l.abs(), alpha);

        let f0 = self.f0_from_albedo(albedo, cached_textures);
        let f  = Self::schlick_fresnel(v_dot_h, f0);
        let one_minus_f = Vector3::repeat(1.0) - f;

        // Jacobian of the refraction half-vector mapping (Walter et al. eq. 17).
        let denom = eta_i * v_dot_h + eta_t * l_dot_h;
        if denom.abs() < 1e-8 {
            return Vector3::zeros();
        }

        let jacobian = (eta_t * eta_t * l_dot_h * v_dot_h) / (n_dot_v * n_dot_l.abs() * denom * denom);

        one_minus_f.component_mul(&(albedo * d * g * jacobian.abs() * self.transmission_factor))
    }

    fn alpha(&self, cached_textures: &mut CachedTextureLookups) -> f32 {
        let roughness = cached_textures.roughness().clamp(0.02, 1.0);
        (roughness * roughness).max(1e-4)
    }

    fn f0_from_albedo(&self, albedo: &Vector3<f32>, cached_textures: &mut CachedTextureLookups) -> Vector3<f32> {
        let dielectric_f0_scalar = ((self.ior - IOR_AIR) / (self.ior + IOR_AIR)).powi(2);
        let dielectric_f0 = Vector3::repeat(dielectric_f0_scalar);
        dielectric_f0 + (albedo - dielectric_f0) * cached_textures.metallic()
    }

    fn specular_sampling_probability(&self, f0: &Vector3<f32>) -> f32 {
        let max_f0 = f0.x.max(f0.y).max(f0.z);
        lerp(0.08, 0.95, max_f0).clamp(0.08, 0.95)
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

   /*pub  fn sample(&self, incoming: Vector3<f32>, normal: Vector3<f32>, u: f32, v: f32, rng: &mut impl Rng) -> (Vector3<f32>, Vector3<f32>, f32, f32) {
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
    }*/

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

#[cfg(test)]
mod tests {
    use crate::scene::texture::WrapMode;
    use super::*;

    fn solid_texture(rgb: [u8; 3]) -> Texture {
        Texture::new(vec![rgb[0], rgb[1], rgb[2], 255], 1, 1, WrapMode::ClampToEdge)
    }

    fn make_material(normal_map: Option<Texture>, normal_scale: f32) -> Material {
        Material::new(
            Vector3::new(1.0, 1.0, 1.0),
            None,
            normal_map,
            None,
            None,
            normal_scale,
            Vector3::zeros(),
            1.0,
            0.0,
            0.0,
            1.5,
            false,
        )
    }

    #[test]
    fn apply_normal_map_returns_geometric_normal_without_map() {
        let material = make_material(None, 1.0);
        let normal = material.apply_normal_map(
            Vector3::new(0.0, 0.0, 1.0),
            Vector4::new(1.0, 0.0, 0.0, 1.0),
            Vector2::new(0.5, 0.5),
        );

        assert!((normal - Vector3::new(0.0, 0.0, 1.0)).norm() <= 1e-6);
    }

    #[test]
    fn apply_normal_map_respects_tangent_frame() {
        let material = make_material(Some(solid_texture([255, 128, 128])), 1.0);
        let normal = material.apply_normal_map(
            Vector3::new(0.0, 0.0, 1.0),
            Vector4::new(1.0, 0.0, 0.0, 1.0),
            Vector2::new(0.5, 0.5),
        );

        assert!(normal.x > 0.99);
        assert!(normal.z.abs() < 0.02);
    }

    #[test]
    fn apply_normal_map_falls_back_for_missing_tangent() {
        let material = make_material(Some(solid_texture([255, 128, 128])), 1.0);
        let normal = material.apply_normal_map(
            Vector3::new(0.0, 0.0, 1.0),
            Vector4::new(0.0, 0.0, 0.0, 1.0),
            Vector2::new(0.5, 0.5),
        );

        assert!((normal - Vector3::new(0.0, 0.0, 1.0)).norm() <= 1e-6);
    }

    #[test]
    fn evaluate_bsdf_does_not_add_diffuse_tint_for_fully_transmissive_materials() {
        let material = Material::new(
            Vector3::new(0.0, 1.0, 0.0),
            None,
            None,
            None,
            None,
            1.0,
            Vector3::zeros(),
            0.5,
            0.0,
            1.0,
            1.5,
            false,
        );
        let mut cache = CachedTextureLookups::new(&material, Vector2::new(0.5, 0.5));
        let normal = Vector3::new(0.0, 0.0, 1.0);
        let light_dir = Vector3::new(0.0, 0.0, 1.0);
        let view_dir = Vector3::new(0.0, 0.0, 1.0);

        let bsdf = material.evaluate_bsdf(&light_dir, &view_dir, &normal, &Vector3::new(0.0, 1.0, 0.0), &mut cache);

        assert!((bsdf.x - bsdf.y).abs() < 1e-4);
        assert!((bsdf.y - bsdf.z).abs() < 1e-4);
    }
}
