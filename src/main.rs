use std::time::SystemTime;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: String
}

fn main() {
    let args = Args::parse();

    let start = SystemTime::now();
    let _ = rfd::search(&args.file.to_lowercase());
}
