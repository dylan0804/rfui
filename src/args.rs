use std::{
    env,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::mpsc::Sender,
    vec,
};

use anyhow::{Context, Result, anyhow};
use clap::{ArgAction, Parser};
use regex::bytes::RegexBuilder;

use crate::{
    config::Config,
    exit_codes::ExitCode,
    file_system::{self},
    tui::AppEvent,
    walk::Walker,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, disable_help_flag = true)]
pub struct Args {
    #[arg(help = "Pattern to search")]
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
        help = "Filter by type: file (f/file) or directory (d/dir)"
    )]
    pub kind: Option<Type>,

    #[arg(short = 'H', long = "hidden", default_value_t = false)]
    pub show_hidden: bool,

    #[arg(short = 'd', long = "max-depth", help = "Set maximum depth search")]
    pub max_depth: Option<usize>,

    #[arg(short = 's', long = "case-sensitive")]
    pub case_sensitive: bool,

    #[arg(short = 't', long)]
    pub threads: Option<NonZeroUsize>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Type {
    #[value(alias = "d", alias = "dir")]
    Directory,

    #[value(alias = "f", alias = "file")]
    File,
}

fn is_valid_directory(path: &Path) -> Result<()> {
    if file_system::is_existing_dir(path) {
        Ok(())
    } else {
        Err(anyhow!("Could not retrieve current directory"))
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
        return Ok(vec![normalize_path(&current_dir)]);
    };

    Ok(paths
        .iter()
        .filter_map(|path| {
            let expanded_path = expand_tilde(path.to_string_lossy().to_string());
            let path = Path::new(&expanded_path);

            if file_system::is_existing_dir(path) {
                Some(normalize_path(path))
            } else {
                None
            }
        })
        .collect())
}

pub fn expand_tilde(path: String) -> String {
    if path.starts_with("~") {
        if let Ok(home) = env::var("HOME") {
            path.replacen("~", &home, 1)
        } else {
            path.to_string()
        }
    } else {
        path.to_string()
    }
}

pub fn parse_input_args(input: &str) -> Result<Args, clap::Error> {
    let args = input.split_whitespace().collect::<Vec<&str>>();

    let mut full_args = vec!["rfui"];
    full_args.extend(args);

    Args::try_parse_from(full_args)
}

pub fn build_and_scan(args: Args, tx: Sender<AppEvent>) -> Result<ExitCode> {
    let search_paths =
        get_search_paths(&args.path).with_context(|| "Failed to get search paths")?;

    let regexp = regex_builder(&args).with_context(|| "Failed building regex pattern")?;

    let config = Config::build(args);
    let walker = Walker::new(config);

    walker.scan(search_paths, regexp, tx)
}

fn regex_builder(args: &Args) -> Result<regex::bytes::Regex> {
    RegexBuilder::new(&args.pattern)
        .case_insensitive(!&args.case_sensitive)
        .dot_matches_new_line(true)
        .build()
        .map_err(|e| anyhow!("{}", e))
}
