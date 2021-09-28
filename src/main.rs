mod frame_counter;
use frame_counter::FrameCounter;
use structopt::StructOpt;
use bosrender::offscreen::OffScreen;
use bosrender::settings::Settings;
use anyhow::{Result, Context};
use std::io::BufWriter;
use std::fs::File;
use std::collections::VecDeque;

fn main() -> Result<()> {
    let cfg = Settings::from_args();

    let frames_in_flight = 3;

    let mut engine = OffScreen::new(cfg.clone())?;

    let (tile_width, tile_height) = bosrender::offscreen::calc_tile_dims(&cfg);

    let work_order: Vec<((usize, usize), f32, usize)> = (cfg.first_frame..).take(cfg.frames).map(|frame_idx| {
        let time = cfg.rate * (frame_idx + cfg.first_frame) as f32;
        bosrender::tiles::tiles(
            (cfg.width as _, cfg.height as _),
            (tile_width as _, tile_height as _),
        )
        .into_iter()
        .map(move |pos| (pos, time, frame_idx))
    })
    .flatten()
    .collect();

    let mut work_order = work_order.into_iter();

    // Submit `frames_in_flight` frames to prime the engine
    let mut tile_tracker = VecDeque::new();
    for (pos, time, frame_idx) in work_order.by_ref().take(cfg.frames_in_flight) {
        engine.submit_tile(time, pos.0 as _, pos.1 as _)?;
        tile_tracker.push_back((pos, frame_idx));
    }

    // Download each frame
    let mut last_frame_idx = cfg.first_frame;
    let mut current_image = vec![0; (cfg.width * cfg.height * 3) as usize];

    loop {
        // Download the most recently rendered tile
        let tile_info = tile_tracker.pop_front();

        dbg!(tile_info, last_frame_idx);

        let finish_frame = match tile_info {
            Some((_, frame_idx)) => {
                if frame_idx != last_frame_idx {
                    let finish_frame = last_frame_idx;
                    last_frame_idx = frame_idx;
                    Some(finish_frame)
                } else {
                    None
                }
            },
            None => Some(last_frame_idx),
        };

        let pos = tile_info.map(|(pos, _)| pos);

        // If we've finished a frame, save it
        if let Some(frame_idx) = finish_frame {
            let path = format!("{}_{:04}.png", cfg.name, frame_idx);
            write_rgb_png(cfg.width, cfg.height, &current_image, &path).context("Writing image")?;
        }

        // If we have tile data, blit it
        if let Some(pos) = pos {
            let tile_data = engine.download_frame().context("Downloading frame")?;
            bosrender::tiles::blit_rgb(
                &tile_data,
                &mut current_image,
                pos,
                (cfg.width as _, cfg.height as _),
                (tile_width as _, tile_height as _),
            )
        }

        // Submit new work, if any
        if let Some((pos, time, frame_idx)) = work_order.next() {
            tile_tracker.push_back((pos, frame_idx));
            engine.submit_tile(time, pos.0 as _, pos.1 as _)?;
        } else {
            // There are no frames in flight and we've got no more tiles to submit. Finish!
            if finish_frame.is_some() && tile_tracker.is_empty() {
                break;
            }
        }
    }

    println!("Finished!");

    Ok(())
}

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
