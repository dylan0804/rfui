use std::{env, path::Path};
use normpath::PathExt;

pub fn is_existing_dir(path: &Path) -> bool {
    path.is_dir() && (path.file_name().is_some() || path.normalize().is_ok())
}

pub fn get_relative_path(path: &Path) -> Option<String> {
    path
        .strip_prefix(env::current_dir().ok()?)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}