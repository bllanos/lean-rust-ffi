use std::env;
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum FileOutputError {
    #[error("error creating file")]
    Create { source: std::io::Error },
    #[error("error writing to file")]
    Write { source: std::io::Error },
}

/// The environment variable providing the build output directory
const OUT_DIR: &str = "OUT_DIR";

#[derive(thiserror::Error, Debug)]
#[error("error accessing the \"{}\" environment variable", OUT_DIR)]
pub struct OutDirError(#[from] env::VarError);

pub fn get_out_dir() -> Result<PathBuf, OutDirError> {
    Ok(PathBuf::from(env::var(OUT_DIR)?))
}
