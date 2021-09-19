pub mod visualizer;
mod engine;
pub mod trainer;
pub use visualizer::visualize;

pub struct Render {
    cfg: RenderSettings,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderSettings {
    /// Batch size
    pub batch_size: u32,
    /// Width dimension of the rendered image
    pub output_width: u32,
    /// Height dimension of the rendered image
    pub output_height: u32,
    /// Number of input images
    pub input_images: u32,
    /// Width of input images
    pub input_width: u32,
    /// Height of input images
    pub input_height: u32,
    /// Number of input points
    pub input_points: u32,
}

pub struct Input;
