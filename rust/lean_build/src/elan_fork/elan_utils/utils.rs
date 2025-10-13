use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::errors::*;
use super::notifications::Notification;
use super::raw;

pub use raw::{if_not_empty, is_directory, is_file};

const ELAN_HOME: &str = "ELAN_HOME";

pub fn read_file(name: &'static str, path: &Path) -> Result<String> {
    raw::read_file(path).map_err(|err| Error::ReadingFile {
        name,
        path: PathBuf::from(path),
        source: err,
    })
}

pub fn ensure_dir_exists(
    name: &'static str,
    path: &Path,
    _notify_handler: &dyn Fn(Notification<'_>),
) -> Result<bool> {
    if !raw::is_directory(path) {
        Err(Error::CreateDirectoryForbidden {
            name,
            path: path.to_path_buf(),
        })
    } else {
        Ok(false)
    }
}

pub fn canonicalize_path(path: &Path, notify_handler: &dyn Fn(Notification<'_>)) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| {
        notify_handler(Notification::NoCanonicalPath(path));
        PathBuf::from(path)
    })
}

pub fn read_dir(name: &'static str, path: &Path) -> Result<fs::ReadDir> {
    fs::read_dir(path).map_err(|error| Error::ReadingDirectory {
        name,
        path: PathBuf::from(path),
        source: error,
    })
}

pub fn toolchain_sort<T: AsRef<str>>(v: &mut [T]) {
    fn special_version(ord: u64, s: &str) -> String {
        format!("0.0.0-{ord}.{s}")
    }

    fn toolchain_sort_key(s: &str) -> String {
        if s.starts_with("stable") {
            special_version(0, s)
        } else if s.starts_with("beta") {
            special_version(1, s)
        } else if s.starts_with("nightly") {
            special_version(2, s)
        } else {
            s.replace("_", "-")
        }
    }

    v.sort_by(|a, b| {
        let a_str: &str = a.as_ref();
        let b_str: &str = b.as_ref();
        let a_key = toolchain_sort_key(a_str);
        let b_key = toolchain_sort_key(b_str);
        a_key.cmp(&b_key)
    });
}

pub fn elan_home() -> Result<PathBuf> {
    let env_var = env::var_os(ELAN_HOME);

    let cwd = env::current_dir().map_err(Error::CurrentDirectory)?;
    let elan_home = env_var.clone().map(|home| cwd.join(home));
    let user_home = home_dir().map(|p| p.join(".elan"));
    elan_home.or(user_home).ok_or(Error::ElanHome)
}

pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}
