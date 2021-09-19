mod frame_counter;
use frame_counter::FrameCounter;
mod settings;
use structopt::StructOpt;

fn main() {
    let cfg = settings::Settings::from_args();

    let engine = 

    for frame_idx in FrameCounter::new(cfg.frames as _) {
        std::thread::sleep_ms(frame_idx as _);
    }
}
