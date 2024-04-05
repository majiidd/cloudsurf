mod args;
mod logger;

use crate::args::Args;
use crate::logger::init_logging;
use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    init_logging(&args.log_level);

    Ok(())
}
