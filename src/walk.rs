use std::borrow::Cow;
use std::os::unix::ffi::OsStrExt;
use std::sync::mpsc::Sender;
use std::{path::PathBuf};
use std::sync::atomic::{Ordering};
use anyhow::{Context, Result, Error};
use std::result::Result::Ok;
use ignore::{DirEntry, WalkBuilder, WalkParallel};
use ignore::WalkState;
use colored::*;
use regex::bytes::Regex;

use crate::args::{Type};
use crate::config::Config;
use crate::file_system::get_relative_path;
use crate::tui::AppEvent;

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
            .follow_links(false)
            .same_file_system(true)
            .threads(config.threads);
            // add more config here later on if needed
        
        for path in &paths[1..] {
            builder.add(path);
        }

        let walker = builder.build_parallel();

        Ok(walker)
    }

    pub fn scan(
        &self, 
        paths: Vec<PathBuf>, 
        regexp: Regex, 
        tx: Sender<AppEvent>
    ) -> Result<()> {
        let walker: WalkParallel = self.build(&paths)?;
        let regexp = &regexp;
        let config: &Config = &self.config;
        
        walker.run(|| {
            let tx_clone = tx.clone();
            Box::new(move |entry| {
                if let Ok(entry) = entry {
                    if entry.depth() == 0 {
                        return WalkState::Continue;
                    }

                    if !should_process_entry(&entry, &config.kind) {
                        return WalkState::Continue
                    };

                    if !regexp.is_match(entry.file_name().as_bytes()) { 
                        return WalkState::Continue
                    }

                    let current_count = config.total_results.fetch_add(1, Ordering::Relaxed);
                    if let Some(max_results) = config.max_results {
                        if current_count >= max_results {
                            return WalkState::Quit;
                        };
                    };
                    
                    let full_path = entry.path();
                    let relative_path = get_relative_path(full_path)
                        .unwrap_or_else(|| full_path.to_string_lossy().to_string());
                
                    tx_clone.send(AppEvent::SearchResult(relative_path)).unwrap();


                    // let highlighted_match = highlight_match(&relative_path, &regexp)
                    //     .map_or_else(|_| relative_path, |h| h);

                    // tx_clone.send(highlighted_match).unwrap();
                }
                WalkState::Continue
            })
        });

        // notify receiving end
        tx.send(AppEvent::SearchComplete).unwrap();

        Ok(())
    }
}

fn highlight_match(relative_path: &str, regexp: &Regex) -> Result<String> {
    let highlighted = regexp.replace_all(relative_path.as_bytes(), |caps: &regex::bytes::Captures| {
        let matched = String::from_utf8_lossy(&caps[0]);
        matched.bright_yellow().bold().to_string()
    });

    Ok(String::from_utf8(highlighted.to_vec())?)
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