use crate::scene::material::Material;
use crate::scene::texture::{Texture, WrapMode};
use gltf::image::Source;
use gltf::material::NormalTexture;
use gltf::texture;
use nalgebra::Vector3;
use std::path::Path;
use gltf::texture::WrappingMode;
use serde::Deserialize;
use crate::context::Context;

#[derive(Deserialize)]
struct MaterialExtras {
    invert: Option<bool>,
}

fn extract_invert_albedo_flag(material: &gltf::material::Material) -> bool {
    material
        .extras()
        .as_ref()
        .and_then(|extras| {
            gltf::json::deserialize::from_str::<MaterialExtras>(extras.get()).ok()
        })
        .and_then(|extras| extras.invert)
        .unwrap_or(false)
}

fn create_texture_internal(texture: &texture::Texture, folder: &Path, ctx: &Context) -> Texture {
    let source = texture.source();
    let wrap_mode = match texture.sampler().wrap_s() {
        WrappingMode::ClampToEdge => WrapMode::ClampToEdge,
        WrappingMode::MirroredRepeat => WrapMode::MirroredRepeat,
        WrappingMode::Repeat => WrapMode::Repeat,
    };

    match source.source() {
        Source::View { .. } => panic!("Unexpected source: view"),
        Source::Uri { uri, mime_type: _mime_type } => {
            let uri = uri.replace("%20", " "); // TODO: Quick fix. Probably have to do proper URL decoding.
            let image_path = folder.join(uri);
            let img = match image::open(&image_path) {
                Ok(img) => img,
                Err(e) => panic!("Failed to load image {}. Error: {}", image_path.display(), e)
            };
            ctx.mem.texture_memory_bytes(img.width() as u64 * img.height() as u64 * 4);
            Texture::new(img.to_rgba8().to_vec(), img.width(), img.height(), wrap_mode)
        }
    }
}
fn create_normal_texture(texture: &Option<NormalTexture>, folder: &Path, ctx: &Context) -> Option<Texture> {
    texture.as_ref().map(|x| {
        let texture = x.texture();
        create_texture_internal(&texture, folder, ctx)
    })
}

fn create_texture(texture: &Option<texture::Info<'_>>, folder: &Path, ctx: &Context) -> Option<Texture> {
    texture.as_ref().map(|x| {
        let texture = x.texture();
        create_texture_internal(&texture, folder, ctx)
    })
}

pub fn create_material(material: &gltf::Material, folder: &Path, ctx: &Context) -> anyhow::Result<Material> {
    let albedo_texture = create_texture(&material.pbr_metallic_roughness().base_color_texture(), folder, ctx);
    let emissive_texture = create_texture(&material.emissive_texture(), folder, ctx);
    let normal_texture = material.normal_texture();
    let normal_map = create_normal_texture(&normal_texture, folder, ctx);
    let normal_scale = normal_texture.as_ref().map_or(1.0, |x| x.scale());
    let metallic_roughness_texture = create_texture(&material.pbr_metallic_roughness().metallic_roughness_texture(), folder, ctx);

    let base_color = material.pbr_metallic_roughness().base_color_factor();
    let roughness = material.pbr_metallic_roughness().roughness_factor();
    let metallic = material.pbr_metallic_roughness().metallic_factor();

    let invert_albedo = extract_invert_albedo_flag(material);

    let transmission_factor = material
        .transmission()
        .map(|transmission| transmission.transmission_factor())
        .unwrap_or(0.0);
    let ior = material.ior().unwrap_or(1.5);
    const EMISSIVE_SCALE: f32 = 1.0; // TODO: This is a hack to make emissive materials more visible. Should probably be exposed as a parameter.
    let emissive_strength = material.emissive_strength().unwrap_or(0.0) * EMISSIVE_SCALE;
    let emissive = Vector3::new(material.emissive_factor()[0] * emissive_strength, material.emissive_factor()[1] * emissive_strength, material.emissive_factor()[2] * emissive_strength);
    Ok(Material::new(Vector3::new(base_color[0], base_color[1], base_color[2]), albedo_texture, normal_map, emissive_texture, metallic_roughness_texture, normal_scale, emissive, roughness, metallic, transmission_factor, ior, invert_albedo))
}