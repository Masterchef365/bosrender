mod frame_counter;
use frame_counter::FrameCounter;
use structopt::StructOpt;
use bosrender::offscreen::OffScreen;
use bosrender::settings::Settings;
use anyhow::{Result, Context};
use std::io::BufWriter;
use std::fs::File;

fn main() -> Result<()> {
    todo!()
}

/*
fn main() -> Result<()> {
    let cfg = Settings::from_args();

    let frames_in_flight = 3;

    let mut engine = OffScreen::new(cfg.clone())?;

    let frame_idx_to_time = |frame_idx| cfg.rate * (frame_idx + cfg.first_frame) as f32;

    // Submit `frames_in_flight` frames for rendering
    for frame_idx in 0..frames_in_flight {
        engine.submit(frame_idx_to_time(frame_idx))?;
    }

    // Download each frame
    for frame_idx in FrameCounter::new(cfg.frames) {
        let image = engine.download_frame().with_context(|| format!("Downloading frame {}", frame_idx))?;

        let path = format!("{}_{:04}.png", cfg.name, frame_idx + cfg.first_frame);

        write_rgb_png(cfg.width, cfg.height, &image, &path).context("Writing image")?;

        let next_frame_idx = frame_idx + frames_in_flight;

        // If we're not going to be finished soon, queue up another frame
        if cfg.frames - frame_idx > frames_in_flight {
            engine.submit(frame_idx_to_time(next_frame_idx))?;
        }
    }

    println!("Finished!");

    Ok(())
}
*/

fn write_rgb_png(width: u32, height: u32, data: &[u8], path: &str) -> Result<()> {
    let file = File::create(&path).with_context(|| format!("Failed to create image {}", path))?;
    let ref mut w = BufWriter::new(file);
    
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(data)?;

    /*
    encoder.set_trns(vec!(0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8));
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));     // 1.0 / 2.2, unscaled, but rounded
    let source_chromaticities = png::SourceChromaticities::new(     // Using unscaled instantiation here
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000)
    );
    encoder.set_source_chromaticities(source_chromaticities);
    */

    Ok(())
}
