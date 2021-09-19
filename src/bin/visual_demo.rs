use anyhow::Result;
use bosrender::{visualize, settings::Settings};
use structopt::StructOpt;

fn main() -> Result<()> {
    let cfg = Settings::from_args();
    visualize(cfg, false)
}
