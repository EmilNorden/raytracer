use clap::Parser;

#[derive(Parser, Debug)]
pub struct RenderOptions {
    #[arg(long, short)]
    pub scene_file: String,
    #[arg(long, short, default_value = "output.png")]
    pub output_file: String,
    #[arg(long, short, default_value = "800")]
    pub width: u32,
    #[arg(long, short, default_value = "600")]
    pub height: u32,
}