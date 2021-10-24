use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
pub struct Settings {
    /// Screen width in pixels
    #[structopt(short, long, default_value = "1920")]
    pub width: u32,

    /// Screen height in pixels
    #[structopt(short, long, default_value = "1080")]
    pub height: u32,

    /// First frame to render
    #[structopt(long, default_value = "0")]
    pub first_frame: usize, // TODO: Maybe this should be a list of ranges...

    /// Number of frames to render. Infinite if 0
    #[structopt(short, long, default_value = "1")]
    pub frames: usize, // TODO: Maybe this should be a list of ranges...

    /// Number of frames to render. Infinite if 0
    #[structopt(short = "l", long, default_value = "3")]
    pub frames_in_flight: usize, // TODO: Maybe this should be a list of ranges...

    /// How much to increment `anim` by each frame
    #[structopt(short, long, default_value = "0.01666")]
    pub rate: f32,

    /*
    /// Output format. Replaces $f with the file stem of the shader path and $i with the stem of
    /// the shader path. If $f is not found in the pattern, the pattern is assumed to end with "_%f"
    #[structopt(long, value_name="pattern", default_value = "%i_%f.png")]
    */
    #[structopt(long, short, default_value = "out")]
    pub name: String,

    /// Output directory
    #[structopt(short, long, default_value = "")]
    pub output: PathBuf,

    /// Fragment shader path
    pub shader: PathBuf,

    /// Enable validation layers
    #[structopt(long)]
    pub validation: bool,

    /// Tile width
    #[structopt(long)]
    pub tile_width: Option<u32>,

    /// Tile height
    #[structopt(long)]
    pub tile_height: Option<u32>,
}
