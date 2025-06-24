mod walk;
mod config;
mod args;
mod file_system;

use std::env;

use anyhow::{anyhow, Result};
use clap::{Parser};
use args::Args;
use regex::bytes::{RegexBuilder};
use walk::Walker;

use crate::config::Config;

fn main() {
    let args = Args::parse();

    let search_paths = match args.get_search_paths() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error getting search paths: {}", e);
            std::process::exit(1)
        }
    };
    
    let regexp = match regex_builder(&args) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error building regex: {}", e);
            std::process::exit(1)
        }
    };

    let config = Config::build(args);

    let walker = Walker::new(config);

    walker.scan(search_paths, regexp).unwrap_or_else(|e| {
        eprintln!("Error when starting scan: {}", e);
        std::process::exit(1)
    });
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
