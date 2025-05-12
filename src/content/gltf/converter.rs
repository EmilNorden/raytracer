use gltf::camera::Projection;
use gltf::Document;
use crate::camera::perspective_camera::PerspectiveCamera;
use crate::content::scene_loader::SceneError;

pub fn convert_camera(document: Document) -> anyhow::Result<PerspectiveCamera> {
   /* let camera = document.cameras().next()
        .ok_or_else(|| { return SceneError::NoCameras.into() })?;

    let (aspect_ratio, fov) = match camera.projection() {
        Projection::Orthographic(_) =>
            return Err(SceneError::UnsupportedFormat("Orthographic projection not currently supported".to_string()).into()),
        Projection::Perspective(perspective) =>
            (perspective.aspect_ratio(), perspective.yfov()),
    };*/



    unimplemented!()
}