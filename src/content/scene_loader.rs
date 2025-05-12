use std::path::Path;
use crate::scene::scene::Scene;

#[derive(Debug)]
pub enum SceneError {
    NoDefaultScene,
    NoCameras,
    UnsupportedFormat(String)
}

impl std::error::Error for SceneError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }
}

impl std::fmt::Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SceneError::NoDefaultScene => write!(f, "No default scene found"),
            SceneError::NoCameras => write!(f, "No cameras found"),
            SceneError::UnsupportedFormat(message) => write!(f, "Unsupported format: {}", message)
        }
    }
}

pub trait SceneLoader {
    fn load_scene<P: AsRef<Path>>(path: P) -> anyhow::Result<Scene>;
}