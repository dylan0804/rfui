use std::{env, path::PathBuf};
use ignore::{WalkBuilder};
use colored::*;

fn get_home_dir() -> Result<PathBuf, &'static str> {
    env::home_dir().ok_or_else(|| "Could not find home directory")
}

pub fn search(pattern: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = get_home_dir()?;

    WalkBuilder::new(home_dir)
        .build_parallel()
        .run(|| {
            Box::new(move |entry| {
                if let Ok(entry) = entry {
                    if let Some(filename) = entry.path().file_name() {
                        let filename = filename
                            .to_string_lossy()
                            .to_lowercase();

                        if filename.contains(&pattern) {
                            let orig_path = entry.path().to_string_lossy().to_string();
                            let parent = entry.path().parent()
                                .unwrap_or_else(|| std::path::Path::new(""))
                                .to_string_lossy()
                                .to_string();
                            
                            let highlighted = orig_path.replace(&parent, &parent.truecolor(100, 149, 237).to_string());

                            println!("{}", highlighted)
                        }
                    }
                }
                ignore::WalkState::Continue
            })
        });
    
    Ok(())
}