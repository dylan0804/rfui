use std::{env, path::PathBuf, sync::atomic::{AtomicU32, Ordering}};
use ignore::{WalkBuilder, DirEntry};
use colored::*;

use crate::args::{Args, Type};

pub struct Walker {
    pattern: String,
    kind: Type,
    show_hidden: bool,
    max_depth: Option<usize>,
    count_enabled: bool
}

impl Walker {
    pub fn new(args: Args) -> Self {
        Self {
            pattern: args.pattern, 
            kind: args.kind,
            show_hidden: args.show_hidden,
            max_depth: args.max_depth,
            count_enabled: args.count_enabled
        }
    }

    pub fn scan(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pattern = &self.pattern;
        let kind = &self.kind;
        
        let count = if self.count_enabled {
            Some(AtomicU32::new(0))
        } else {
            None
        };
        let count_ref = count.as_ref();
    
        WalkBuilder::new(get_home_dir()?)
            .hidden(!self.show_hidden)
            .max_depth(self.max_depth)
            .build_parallel()
            .run(|| {
                Box::new(move |entry| {
                    if let Ok(entry) = entry {
                        if should_process_entry(&entry, kind) {
                            let search_text = get_search_text(&entry, kind);
                            if search_text.contains(pattern) {
                                if let Some(counter) = count_ref {
                                    counter.fetch_add(1, Ordering::Relaxed);
                                }
                                if !self.count_enabled {
                                    print_highlighted_match(&entry, &pattern);
                                }
                            }
                        }
                    }
                    ignore::WalkState::Continue
                })
            });
        
        if let Some(counter) = count {
            println!("{}", counter.load(Ordering::Relaxed));
        }
        Ok(())
    }
}

fn get_home_dir() -> Result<PathBuf, &'static str> {
    env::home_dir().ok_or_else(|| "Could not find home directory")
}

fn print_highlighted_match(entry: &DirEntry, pattern: &str) {
    let path_str = entry.path().to_string_lossy();
    let highlighted = path_str.replace(pattern, &pattern.bright_yellow().to_string());
    println!("{}", highlighted);
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

fn get_search_text(entry: &DirEntry, kind: &Type) -> String {
    match kind {
        Type::File => entry.path()
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default(),
        Type::Directory => entry.path()
            .to_string_lossy()
            .to_lowercase(),
    }
}