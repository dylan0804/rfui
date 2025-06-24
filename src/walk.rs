use std::env;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::{path::PathBuf};
use anyhow::{Result};
use ignore::{DirEntry, WalkBuilder, WalkParallel};
use ignore::WalkState;
use colored::*;
use regex::bytes::Regex;

use crate::args::{Type};
use crate::config::Config;

pub struct Walker {
    config: Config
}

impl Walker {
    pub fn new(config: Config) -> Self {
        Self {
            config
        }
    }

    pub fn build(&self, paths: &Vec<PathBuf>) -> Result<WalkParallel> {
        let first_path = &paths[0];
        let config = &self.config;

        let mut builder = WalkBuilder::new(first_path);
        builder
            .hidden(!config.show_hidden)
            .max_depth(config.max_depth)
            .ignore_case_insensitive(config.case_sensitive)
            .threads(config.threads);
            // add more config here later on if needed

        for path in &paths[1..] {
            builder.add(path);
        }

        let walker = builder.build_parallel();

        Ok(walker)
    }

    pub fn scan(&self, paths: Vec<PathBuf>, regexp: Regex) -> Result<()> {
        let walker = self.build(&paths)?;
        let regexp = &regexp;
        let config: &Config = &self.config;

        walker.run(|| {
            Box::new(move |entry| {
                if let Ok(entry) = entry {
                    if entry.depth() == 0 {
                        return WalkState::Continue;
                    }

                    // check if current entry is the kind we want or not
                    if !should_process_entry(&entry, &config.kind) {
                        return WalkState::Continue
                    };

                    if !regexp.is_match(entry.file_name().as_bytes()) { 
                        return WalkState::Continue
                    }
                    
                    let full_path = entry.path().to_string_lossy();
                    let relative_path = get_relative_path(&full_path)
                        .unwrap_or_else(|| full_path.to_string());

                    print_highlighted_match(&relative_path, &regexp);
                }
                WalkState::Continue
            })
        });

        Ok(())
    }
}

fn get_relative_path(path: &str) -> Option<String> {
    let path: &Path = Path::new(path);
    let current_dir = env::current_dir().ok()?;
    
    current_dir
        .strip_prefix(path)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

fn print_highlighted_match(entry: &str, regexp: &Regex) {
    let highlighted = regexp.replace_all(entry.as_bytes(), |caps: &regex::bytes::Captures| {
        let matched = String::from_utf8_lossy(&caps[0]);
        matched.bright_yellow().bold().to_string()
    });
    let highlighted_result = String::from_utf8_lossy(&highlighted);
    println!("{}", highlighted_result);
}

fn should_process_entry(entry: &DirEntry, kind: &Option<Type>) -> bool {
    if let Some(file_type) = entry.file_type() {
        match kind {
            Some(n) => {
                match n {
                    Type::File => file_type.is_file(),
                    Type::Directory => file_type.is_dir(),
                }
            },
            None => return true
        }
    } else {
        false
    }
}