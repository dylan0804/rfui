use std::sync::mpsc::Sender;
use std::{path::PathBuf};
use anyhow::{anyhow, Result};
use std::result::Result::Ok;
use ignore::{DirEntry, WalkBuilder, WalkParallel};
use ignore::WalkState;
use regex::bytes::Regex;

use crate::args::{Type};
use crate::config::Config;
use crate::exit_codes::ExitCode;
use crate::file_system::{self};
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
        if paths.is_empty() {
            return Err(anyhow!("No paths provided for search"))
        }

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
    ) -> Result<ExitCode> {
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

                    if !regexp.is_match(&file_system::osstr_to_bytes(entry.file_name())) { 
                        return WalkState::Continue
                    }
                    
                    let full_path = entry.path();
                    let relative_path = file_system::get_relative_path(full_path)
                        .unwrap_or_else(|| full_path.to_string_lossy().to_string());
                
                    tx_clone.send(AppEvent::SearchResult(relative_path)).unwrap();
                }
                WalkState::Continue
            })
        });

        tx.send(AppEvent::SearchComplete).unwrap();

        Ok(ExitCode::Success)
    }
}

// fn highlight_match(relative_path: &str, regexp: &Regex) -> Result<String> {
//     let highlighted = regexp.replace_all(relative_path.as_bytes(), |caps: &regex::bytes::Captures| {
//         let matched = String::from_utf8_lossy(&caps[0]);
//         matched.bright_yellow().bold().to_string()
//     });

//     Ok(String::from_utf8(highlighted.to_vec())?)
// }

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