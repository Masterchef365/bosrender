use anyhow::{Context, Result};
use bosrender::offscreen::OffScreen;
use bosrender::settings::Settings;
use std::collections::VecDeque;
use std::fs::File;
use std::io::BufWriter;
use structopt::StructOpt;
use std::time::{Duration, Instant};

struct Job {
    pos: (usize, usize),
    time: f32,
    frame_idx: usize,
    tile_idx: usize,
}

fn main() -> Result<()> {
    let cfg = Settings::from_args();

    let mut engine = OffScreen::new(cfg.clone())?;

    let mut line_display = RealtimeDisplay::from_fps(60.);

    let (tile_width, tile_height) = bosrender::offscreen::calc_tile_dims(&cfg);

    let tiles = bosrender::tiles::tiles(
        (cfg.width as _, cfg.height as _),
        (tile_width as _, tile_height as _),
    );

    let work_order: Vec<Job> = (cfg.first_frame..)
        .take(cfg.frames)
        .map(|frame_idx| {
            let time = cfg.rate * (frame_idx + cfg.first_frame) as f32;
            tiles.iter().enumerate().map(move |(tile_idx, &pos)| Job {
                pos,
                time,
                frame_idx,
                tile_idx,
            })
        })
        .flatten()
        .collect();

    let mut work_order = work_order.into_iter();

    // Submit `frames_in_flight` frames to prime the engine
    let mut tile_tracker = VecDeque::new();
    for job in work_order.by_ref().take(cfg.frames_in_flight) {
        let Job { pos, time, .. } = job;
        engine.submit_tile(time, pos.0 as _, pos.1 as _)?;
        tile_tracker.push_back(job);
    }

    // Download each frame
    let mut last_frame_idx = cfg.first_frame;
    let mut current_image = vec![0; (cfg.width * cfg.height * 3) as usize];

    loop {
        // Download the most recently rendered tile
        let tile_info = tile_tracker.pop_front();

        // Display the status line
        if let Some(job) = &tile_info {
            line_display.status_line(format_args!(
                "Frame {}/{}, Tile {}/{}",
                job.frame_idx + 1,
                cfg.frames,
                job.tile_idx + 1,
                tiles.len()
            ));
        }

        let finish_frame = match &tile_info {
            Some(job) => {
                if job.frame_idx != last_frame_idx {
                    let finish_frame = last_frame_idx;
                    last_frame_idx = job.frame_idx;
                    Some(finish_frame)
                } else {
                    None
                }
            }
            None => Some(last_frame_idx),
        };

        // If we've finished a frame, save it
        if let Some(frame_idx) = finish_frame {
            let path = format!("{}_{:04}.png", cfg.name, frame_idx);
            write_rgb_png(cfg.width, cfg.height, &current_image, &path).context("Writing image")?;
        }

        // If we have tile data, blit it
        if let Some(job) = &tile_info {
            let tile_data = engine.download_frame().context("Downloading frame")?;
            bosrender::tiles::blit_rgb(
                &tile_data,
                &mut current_image,
                job.pos,
                (cfg.width as _, cfg.height as _),
                (tile_width as _, tile_height as _),
            )
        }

        // Submit new work, if any
        if let Some(job) = work_order.next() {
            engine.submit_tile(job.time, job.pos.0 as _, job.pos.1 as _)?;
            tile_tracker.push_back(job);
        } else {
            // There are no frames in flight and we've got no more tiles to submit. Finish!
            if finish_frame.is_some() && tile_tracker.is_empty() {
                break;
            }
        }
    }

    println!();
    println!("Finished!");

    Ok(())
}

fn write_rgb_png(width: u32, height: u32, data: &[u8], path: &str) -> Result<()> {
    debug_assert_eq!(data.len() % 3, 0);
    debug_assert_eq!(data.len() % width as usize, 0);
    debug_assert_eq!(data.len() % height as usize, 0);
    debug_assert_eq!(data.len() / width as usize, height as usize);

    let file = File::create(&path).with_context(|| format!("Failed to create image {}", path))?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(data)?;

    Ok(())
}

struct RealtimeDisplay {
    last_update: Instant,
    refresh_interval: Duration,
}

impl RealtimeDisplay {
    pub fn from_fps(fps: f32) -> Self {
        Self::from_interval(Duration::from_secs_f32(1. / fps))
    }

    pub fn from_interval(refresh_interval: Duration) -> Self {
        Self {
            last_update: Instant::now(),
            refresh_interval,
        }
    }

    pub fn needs_update(&mut self) -> bool {
        if self.last_update.elapsed() >= self.refresh_interval {
            self.last_update = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn status_line<T: std::fmt::Display>(&mut self, v: T) {
        self.lazy_status_line(|| v)
    }

    pub fn lazy_status_line<F: FnOnce() -> T, T: std::fmt::Display>(&mut self, f: F) {
        if self.needs_update() {
            // TODO: Make sure this works on Windows?
            print!("\r\x1b[1K{}", f());
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }
    }
}
