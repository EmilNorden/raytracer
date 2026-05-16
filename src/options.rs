use std::fmt::{Display, Formatter};
use serde::Deserialize;
/*
#[derive(Parser, Debug, Deserialize)]
pub struct PartialRenderOptions {
    #[arg(long, short)]
    pub scene_file: Option<String>,
    #[arg(long, short)]
    pub output_folder: Option<String>,
    #[arg(long)]
    pub width: Option<u32>,
    #[arg(long)]
    pub height: Option<u32>,
    #[arg(long, short = 'x')]
    pub samples: Option<u32>,
    #[arg(short = 'd')]
    pub debug: Option<bool>,
    #[arg(short = 'b')]
    pub max_bounces: Option<u32>,
    #[arg(short = 'v')]
    pub video: Option<bool>,
    #[arg(short = 'f')]
    pub frame_rate: Option<u32>,
}
*/
#[derive(Debug, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Display for Resolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum FocalDistance {
    Fixed(f32),
    Auto(f32, f32)
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct DofSettings {
    pub focal_distance: FocalDistance,
    pub aperture_size: f32,
}

impl Default for DofSettings {
    fn default() -> Self {
        Self {
            focal_distance: FocalDistance::Fixed(1.0),
            aperture_size: 0.0,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RenderOptions {
    pub scene_file: String,
    pub output_folder: String,
    pub resolution: Resolution,
    pub samples: u32,
    pub max_bounces: u32,
    pub video: bool,
    pub frame_rate: u32,
    pub denoise: DenoiseAlgorithm,
    pub integrator: Integrator,
    pub depth_of_field: Option<DofSettings>,
}

#[derive(Debug, Deserialize)]
pub enum Integrator {
    Pathtracing,
    Albedo,
    Debug
}

impl Display for Integrator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Integrator::Pathtracing => write!(f, "Pathtracing"),
            Integrator::Albedo => write!(f, "Albedo"),
            Integrator::Debug => write!(f, "Debug"),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Default)]
pub struct DenoiseSettings {
    pub auxiliary_albedo: bool,
    pub auxiliary_normal: bool,
}

#[derive(Debug, Deserialize)]
pub enum DenoiseAlgorithm {
    OpenImageDenoise(DenoiseSettings),
    None,
}

impl Display for DenoiseAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DenoiseAlgorithm::OpenImageDenoise(_) => write!(f, "OpenImageDenoise"),
            DenoiseAlgorithm::None => write!(f, "None"),
        }
    }
}

impl Display for RenderOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "RenderOptions:")?;
        writeln!(f, "  scene_file: {}", self.scene_file)?;
        writeln!(f, "  output_folder: {}", self.output_folder)?;
        writeln!(f, "  resolution: {}", self.resolution)?;
        writeln!(f, "  samples: {}", self.samples)?;
        writeln!(f, "  max_bounces: {}", self.max_bounces)?;
        writeln!(f, "  video: {}", self.video)?;
        writeln!(f, "  frame_rate: {}", self.frame_rate)?;
        writeln!(f, "  denoise: {}", self.denoise)?;
        write!(f, "  integrator: {}", self.integrator)
    }
}
