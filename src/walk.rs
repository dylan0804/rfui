use std::path::Path;
use std::{path::PathBuf};
use anyhow::{Context, Result};
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

        println!("trh {}", config.threads);

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
        let config = &self.config;
        let regexp = &regexp;
        let search_paths = &paths;

        walker.run(|| {
            Box::new(move |entry| {
                if let Ok(entry) = entry {
                    // check if current entry is the kind we want or not
                    if !should_process_entry(&entry, &config.kind) {
                        return WalkState::Continue
                    };

                    let full_path = entry.path().to_string_lossy();

                    if !regexp.is_match(&full_path.as_bytes()) { 
                        return WalkState::Continue
                    }

                    let relative_path =  match get_relative_path(&full_path, &search_paths) {
                        Some(rel) => rel,
                        None => full_path.to_string()
                    };

                    print_highlighted_match(&relative_path, &regexp);

                    if config.prune {
                        return WalkState::Skip;
                    }
                }
                WalkState::Continue
            })
        });

        Ok(())
    }
}

fn get_relative_path(path: &str, search_paths: &Vec<PathBuf>) -> Option<String> {
    let path = Path::new(path);
      search_paths
          .iter()
          .find_map(|search_path| {
              path.strip_prefix(search_path)
                  .ok()
                  .map(|p| p.to_string_lossy().to_string())
          })
}

fn print_highlighted_match(entry: &str, regexp: &Regex) {
    let highlighted = regexp.replace_all(entry.as_bytes(), |caps: &regex::bytes::Captures| {
        let matched = String::from_utf8_lossy(&caps[0]);
        matched.bright_yellow().bold().to_string()
    });
    let result = String::from_utf8_lossy(&highlighted);
    println!("{}", result);
}

fn should_process_entry(entry: &DirEntry, kind: &Type) -> bool {
    if let Some(file_type) = entry.file_type() {
        match kind {
            Type::File => file_type.is_file(),
            Type::Directory => file_type.is_dir(),
        }
    } else {
        false
    }
}