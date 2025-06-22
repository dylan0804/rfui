mod walk;
mod args;

use clap::{Parser};
use args::Args;
use walk::Walker;

fn main() {
    let args = Args::parse();

    let walker = Walker::new(args);
    walker.scan().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1)
    });
}
