mod walk;
mod config;
mod args;
mod file_system;
mod tui;
mod input;

use std::{sync::mpsc, thread};

use anyhow::{anyhow, Result};
use clap::{Parser};
use args::Args;
use regex::bytes::{RegexBuilder};

use crate::{config::Config, tui::{App, AppEvent}};

fn main() {
    // let args = Args::parse();

    // let search_paths = match args.get_search_paths() {
    //     Ok(s) => s,
    //     Err(e) => {
    //         eprintln!("Error getting search paths: {}", e);
    //         std::process::exit(1)
    //     }
    // };
    
    // let regexp = match regex_builder(&args) {
    //     Ok(r) => r,
    //     Err(e) => {
    //         eprintln!("Error building regex: {}", e);
    //         std::process::exit(1)
    //     }
    // };

    // let config = Config::build(args);
    // let walker = Walker::new(config);

    let (tx, rx) = mpsc::channel::<AppEvent>();

    // let search_thread = {
    //     thread::spawn(move || {
    //         walker.scan(search_paths, regexp, tx).unwrap_or_else(|e| {
    //             eprintln!("Error when starting scan: {}", e);
    //         });
    //     })
    // };

    let mut terminal = ratatui::init();
    let _ = App::new(rx).run(&mut terminal);
    ratatui::restore();

    // search_thread.join().unwrap();
}

fn regex_builder(args: &Args) -> Result<regex::bytes::Regex> {
    RegexBuilder::new(&args.pattern)
        .case_insensitive(!&args.case_sensitive)
        .dot_matches_new_line(true)
        .build()
        .map_err(|e| {
            anyhow!("{}", e)
        })
}
