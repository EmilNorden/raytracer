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

#[derive(Debug, Deserialize)]
pub struct RenderOptions {
    pub scene_file: String,
    pub output_folder: String,
    pub resolution: Resolution,
    pub samples: u32,
    pub debug: bool,
    pub max_bounces: u32,
    pub video: bool,
    pub frame_rate: u32,
}

impl Display for RenderOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "RenderOptions:")?;
        writeln!(f, "  scene_file: {}", self.scene_file)?;
        writeln!(f, "  output_folder: {}", self.output_folder)?;
        writeln!(f, "  resolution: {}", self.resolution)?;
        writeln!(f, "  samples: {}", self.samples)?;
        writeln!(f, "  debug: {}", self.debug)?;
        writeln!(f, "  max_bounces: {}", self.max_bounces)?;
        writeln!(f, "  video: {}", self.video)?;
        write!(f, "  frame_rate: {}", self.frame_rate)
    }
}
