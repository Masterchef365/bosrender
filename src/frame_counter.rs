use std::time::{Instant, Duration};
use std::io::Write;

/// Frame counter, an iterator which shows it's progress in stdout
pub struct FrameCounter {
    last_time: Option<Instant>,
    idx: usize,
    n: usize,
}

impl FrameCounter {
    /// Creates a new FrameCounter with the specified number of frames `n`
    pub fn new(n: usize) -> Self {
        Self {
            last_time: None,
            idx: 0,
            n,
        }
    }
}

impl Iterator for FrameCounter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.idx;
        if ret == self.n {
            return None;
        }

        let time = Instant::now();

        print!("\r");
        print!("Frame {}/{}, ", ret + 1, self.n);

        if let Some(last_time) = self.last_time.take() {
            print!("Last frame time: {} ms", time.duration_since(last_time).as_millis());
        }

        std::io::stdout().flush().expect("Stdout error");

        self.last_time = Some(time);

        self.idx += 1;
        Some(ret)
    }
}
