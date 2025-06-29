use std::{borrow::Cow, env, ffi::OsStr, path::Path};
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

#[cfg(unix)]
pub fn osstr_to_bytes(entry: &OsStr) -> Cow<[u8]> {
    use std::os::unix::ffi::OsStrExt;
    Cow::Borrowed(entry.as_bytes())
}

#[cfg(windows)]
pub fn osstr_to_bytes(entry: &OsStr) -> Cow<[u8]> {
    let string = entry.to_string_lossy();

    match string {
        Cow::Owned(s) => Cow::Owned(s.into_bytes()),
        Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
    }
}