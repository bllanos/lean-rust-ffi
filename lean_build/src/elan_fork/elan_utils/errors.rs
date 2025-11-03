use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("could not read {name} file '{}'", .path.display())]
    ReadingFile {
        name: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not read {name} directory '{}'", .path.display())]
    ReadingDirectory {
        name: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("expected TOML type: '{expected_type}' for key '{path}{key}'")]
    ExpectedType {
        expected_type: &'static str,
        path: String,
        key: String,
    },
    #[error("missing key: '{path}{key}'")]
    MissingKey { path: String, key: String },
    #[error("error accessing current directory")]
    CurrentDirectory(#[from] std::io::Error),
    #[error("could not find an Elan home directory path")]
    ElanHome,
    #[error("this crate does not create directories, directory in question is '{}', named '{name}'", .path.display())]
    CreateDirectoryForbidden { name: &'static str, path: PathBuf },
}

pub type Result<T> = std::result::Result<T, Error>;
