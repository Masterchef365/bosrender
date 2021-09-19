mod settings;
use structopt::StructOpt;
use std::time::{Instant, Duration};
use std::io::Write;

fn pretty_time(span: Duration) -> String {
    let nanos = span.as_nanos();
    let suffixes = [
        (1, "ns"),
        (1_000, "µs"),
        (1_000_000, "ms"),
        (1_000_000_000, "s"),
        (60 * 1_000_000_000, "minutes"),
    ];
    for (n, suffix) in suffixes {
        if nanos > n {
        }
    }
}

fn main() {
    let cfg = settings::Settings::from_args();

    // TODO: Consider a callback system like GP
    let mut last_end = None;
    for frame_idx in 0..cfg.frames {
        print!("\r");
        print!("Frame {}/{}", frame_idx + 1, cfg.frames);
        std::thread::sleep(std::time::Duration::from_millis(1));
        let end = Instant::now();
        if let Some(last_end) = last_end.take() {
            let last_frame_time = end - last_end;
            println!("Last frame time: {} ms");
        }
        last_end = Some(end);
        println!("\r");
        std::io::stdout().flush().expect("Stdout error");
    }
}
