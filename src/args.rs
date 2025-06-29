use std::{env, num::NonZeroUsize, path::{Path, PathBuf}, sync::mpsc::Sender, vec};

use anyhow::{anyhow, Result};
use clap::{ArgAction, Parser};
use regex::bytes::RegexBuilder;

use crate::{config::Config, file_system::{self}, tui::AppEvent, walk::Walker};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(
        help = "Pattern to search"
    )]
    pub pattern: String,

    #[arg(
        action = ArgAction::Append,
        value_name = "path",
        help = "the root directories for the filesystem search (optional)",
    )]
    path: Vec<PathBuf>,

    #[arg(
        short = 'k', 
        long = "kind", 
        help = "Filter by type: file (f/file) or directory (d/dir)",
    )]
    pub kind: Option<Type>,

    #[arg(
        short = 'H', 
        long = "hidden", 
        default_value_t = false
    )]
    pub show_hidden: bool,

    #[arg(
        short = 'd',
        long = "max-depth",
        help = "Set maximum depth search"
    )]
    pub max_depth: Option<usize>,

    #[arg(
        short = 's',
        long = "case-sensitive"
    )]
    pub case_sensitive: bool,

    #[arg(
        short = 't',
        long,
    )]
    pub threads: Option<NonZeroUsize>,

    #[arg(
        short = 'm',
        long = "max-results",
    )]
    pub max_results: Option<usize>,

    #[arg(
        short = 'j',
        long = "json",
        help = "Output results in JSON format"
    )]
    pub json: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Type {
    #[value(alias = "d", alias = "dir")]
    Directory,
    
    #[value(alias = "f", alias = "file")]
    File
}

fn is_valid_directory(path: &Path) -> Result<()> {
    if file_system::is_existing_dir(path) {
        Ok(())
    } else {
        return Err(anyhow!("Could not retrieve current directory"))
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    // handle current dir
    if path == Path::new(".") {
        PathBuf::from("./")
    } else {
        // everything else (abs dir n whatnot)
        path.to_path_buf()
    }
}

fn get_search_paths(path: &Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    let paths = if !path.is_empty() {
        path
    } else {
        let current_dir = env::current_dir()?;
        is_valid_directory(&current_dir)?;
        return Ok(vec![normalize_path(&current_dir)])
    };

    Ok(paths
        .iter()
        .filter_map(|path| {
            if file_system::is_existing_dir(path) {
                Some(normalize_path(path))
            } else {
                eprintln!("Path {:?} is not a valid directory", path);
                None
            }
        }).collect()
    )
}

pub fn parse_input_args(input: &str) -> Result<Args, clap::Error> {
    let args = input.split_whitespace().collect::<Vec<&str>>();

    let mut full_args = vec!["rfd"];
    full_args.extend(args);    

    Args::try_parse_from(full_args)
}

pub fn build_and_scan(args: Args, tx: Sender<AppEvent>) {
    let search_paths = match get_search_paths(&args.path) {
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
    
    walker.scan(search_paths, regexp, tx).unwrap_or_else(|e| {
        eprintln!("Error when starting scan: {}", e);
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