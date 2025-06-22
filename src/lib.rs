use std::{env, path::PathBuf};
use ignore::{WalkBuilder, DirEntry};
use colored::*;

pub mod args;
pub use args::{Args, Type};

fn get_home_dir() -> Result<PathBuf, &'static str> {
    env::home_dir().ok_or_else(|| "Could not find home directory")
}

fn print_highlighted_match(entry: &DirEntry, pattern: &str) {
    let path_str = entry.path().to_string_lossy();
    let highlighted = path_str.replace(pattern, &pattern.bright_yellow().to_string());
    println!("{}", highlighted);
}

fn should_process_entry(entry: &DirEntry, kind: &Type) -> bool {
    match kind {
        Type::File => entry.path().is_file(),
        Type::Folder => entry.path().is_dir(),
    }
}

fn get_search_text(entry: &DirEntry, kind: &Type) -> String {
    match kind {
        Type::File => entry.path()
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default(),
        Type::Folder => entry.path()
            .to_string_lossy()
            .to_lowercase(),
    }
}

pub fn search(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = get_home_dir()?;
    let pattern = &args.file.to_lowercase();

    WalkBuilder::new(home_dir)
        .build_parallel()
        .run(|| {
            Box::new(move |entry| {
                if let Ok(entry) = entry {
                    if should_process_entry(&entry, &args.kind) {
                        let search_text = get_search_text(&entry, &args.kind);
                        if search_text.contains(pattern) {
                            print_highlighted_match(&entry, &pattern);
                        }
                    }
                }
                ignore::WalkState::Continue
            })
        });
    
    Ok(())
}