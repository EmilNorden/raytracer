use crate::content::mesh::{SceneLoader};
use russimp::scene::{PostProcess};
use std::path::Path;
use crate::content::assimp::converter::{AssimpSceneConverter};
use crate::scene::scene::Scene;

pub struct AssimpLoader;

impl SceneLoader for AssimpLoader {
    fn load_scene<P: AsRef<Path>>(path: P) -> anyhow::Result<Scene> {
        let scene = russimp::scene::Scene::from_file(
            path.as_ref().to_str().unwrap(),
            vec![
                PostProcess::CalculateTangentSpace,
                PostProcess::Triangulate,
                PostProcess::JoinIdenticalVertices,
                PostProcess::SortByPrimitiveType,
            ],
        )?;


        AssimpSceneConverter::convert_scene(scene)
    }
}
