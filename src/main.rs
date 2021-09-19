mod frame_counter;
use frame_counter::FrameCounter;
use structopt::StructOpt;
use bosrender::offscreen::OffScreen;
use bosrender::settings::Settings;
use anyhow::Result;

fn main() -> Result<()> {
    let cfg = Settings::from_args();

    let engine = OffScreen::new(cfg.clone())?;

    for frame_idx in FrameCounter::new(cfg.frames as _) {
        std::thread::sleep_ms(frame_idx as _);
    }

    Ok(())
}
