use std::path::Path;
use gltf::image::Source;
use nalgebra::Vector3;
use crate::scene::material::Material;
use crate::scene::texture::Texture;

pub fn create_material(material: &gltf::Material, folder: &Path) -> anyhow::Result<Material> {
    let texture = material.pbr_metallic_roughness().base_color_texture().map(|albedo| {
        let texture = albedo.texture();
        let source = texture.source();

         match source.source() {
            Source::View { .. } => panic!("Unexpected source: view"),
            Source::Uri { uri, mime_type } => {
                let image_path = folder.join(uri);
                let img = match image::open(&image_path) {
                    Ok(img) => img,
                    Err(e) => panic!("Failed to load image: {}", image_path.display())
                };
                Texture::new(img.to_rgba8().to_vec(), img.width(), img.height())
            }
        }
    });

    let base_color = material.pbr_metallic_roughness().base_color_factor();

    Ok(Material::new(Vector3::new(base_color[0], base_color[1], base_color[2]), texture))
}