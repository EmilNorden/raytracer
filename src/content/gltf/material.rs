use std::f32::consts::E;
use std::path::Path;
use gltf::image::Source;
use gltf::material::NormalTexture;
use gltf::texture;
use nalgebra::Vector3;
use crate::scene::material::Material;
use crate::scene::texture::Texture;

fn create_texture_internal(texture: &texture::Texture, folder: &Path) -> Texture {
    let source = texture.source();

    match source.source() {
        Source::View { .. } => panic!("Unexpected source: view"),
        Source::Uri { uri, mime_type } => {
            let uri = uri.replace("%20", " "); // TODO: Quick fix. Probably have to do proper URL decoding.
            let image_path = folder.join(uri);
            let img = match image::open(&image_path) {
                Ok(img) => img,
                Err(e) => panic!("Failed to load image: {}", image_path.display())
            };
            Texture::new(img.to_rgba8().to_vec(), img.width(), img.height())
        }
    }
}
fn create_normal_texture(texture: &Option<NormalTexture>, folder: &Path) -> Option<Texture> {
    texture.as_ref().map(|x| {
        let texture = x.texture();
        create_texture_internal(&texture, folder)
    })
}

fn create_texture(texture: &Option<texture::Info<'_>>, folder: &Path) -> Option<Texture> {
    texture.as_ref().map(|x| {
        let texture = x.texture();
        create_texture_internal(&texture, folder)
    })
}

pub fn create_material(material: &gltf::Material, folder: &Path) -> anyhow::Result<Material> {
    let albedo_texture = create_texture(&material.pbr_metallic_roughness().base_color_texture(), folder);
    let emissive_texture = create_texture(&material.emissive_texture(), folder);
    let normal_map = create_normal_texture(&material.normal_texture(), folder);
    let metallic_roughness_texture = create_texture(&material.pbr_metallic_roughness().metallic_roughness_texture(), folder);

    let base_color = material.pbr_metallic_roughness().base_color_factor();
    let roughness = material.pbr_metallic_roughness().roughness_factor();
    let metallic = material.pbr_metallic_roughness().metallic_factor();

    let transmission_factor = 1.0 - base_color[3];
    let ior = material.ior().unwrap_or(1.5);
    const EMISSIVE_SCALE: f32 = 1.0; // TODO: This is a hack to make emissive materials more visible. Should probably be exposed as a parameter.
    let emissive_strength = material.emissive_strength().unwrap_or(0.0) * EMISSIVE_SCALE;
    let emissive = Vector3::new(material.emissive_factor()[0] * emissive_strength, material.emissive_factor()[1] * emissive_strength, material.emissive_factor()[2] * emissive_strength);
    Ok(Material::new(Vector3::new(base_color[0], base_color[1], base_color[2]), albedo_texture, normal_map, emissive_texture, metallic_roughness_texture, emissive, roughness, metallic, transmission_factor, ior))
}