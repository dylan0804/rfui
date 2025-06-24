use std::{num::NonZeroUsize, path::{Path, PathBuf}, vec};

use anyhow::{anyhow, Ok, Result};
use clap::{ArgAction, Parser, ValueEnum};

use crate::file_system;

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
        long
    )]
    pub case_sensitive: bool,

    #[arg(
        short = 't',
        long,
    )]
    pub threads: Option<NonZeroUsize>,
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

impl Args {
    pub fn get_search_paths(&self) -> Result<Vec<PathBuf>> {
        let paths = if !self.path.is_empty() {
            &self.path
        } else {
            let current_dir = Path::new("./");
            is_valid_directory(current_dir)?;
            return Ok(vec![normalize_path(current_dir)])
        };

        Ok(paths
            .iter()
            .filter_map(|path| {
                if file_system::is_existing_dir(path) {
                    let mut path_str = String::from(path.to_str()?);
                    path_str.push_str("/");
                    Some(PathBuf::from(path_str))
                } else {
                    eprintln!("Path {:?} is not a valid directory", path);
                    None
                }
            }).collect()
        )
    }
}