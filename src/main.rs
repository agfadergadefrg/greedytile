//! CLI entry point for the greedy tile generation algorithm

use clap::Parser;
use greedytile::io::cli::{Cli, FileProcessor};

fn main() -> greedytile::Result<()> {
    let cli = Cli::parse();
    let mut processor = FileProcessor::new(cli);
    processor.process()
}
