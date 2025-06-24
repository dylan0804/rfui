use std::path::Path;

use normpath::PathExt;

pub fn is_existing_dir(path: &Path) -> bool {
    path.is_dir() && (path.file_name().is_some() || path.normalize().is_ok())
}