use clap::Parser;

#[derive(Parser, Debug)]
pub struct RenderOptions {
    #[arg(long, short)]
    pub scene_file: String,
    #[arg(long, short, default_value = "output.png")]
    pub output_file: String,
    #[arg(long, default_value = "800")]
    pub width: u32,
    #[arg(long, default_value = "600")]
    pub height: u32,
    #[arg(long, short = 'x', default_value = "10")]
    pub samples: u32,
    #[arg(short = 'd')]
    pub debug: bool,
}