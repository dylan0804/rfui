use std::error::Error;
use clap::Parser;
use rfd::args::Args;

fn main() {
    let args = Args::parse();
    
    rfd::search(&args).unwrap_or_else(|e: Box<dyn Error + 'static>| {
        eprintln!("Error: {}", e);
        std::process::exit(1)
    })
}
