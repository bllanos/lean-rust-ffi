use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::FileOutputError;

const LEAN_SYS_ROOT_MODULE: &[u8; 565] = include_bytes!("lean_sys_root_module.rs");

#[derive(thiserror::Error, Debug)]
#[error("error generating file \"{}\"", .path.display())]
pub struct LeanSysRootModuleGenerationError {
    pub path: PathBuf,
    pub source: FileOutputError,
}

pub fn create_lean_sys_root_module<P: AsRef<Path>>(
    lean_sys_root_module_path: P,
) -> Result<(), LeanSysRootModuleGenerationError> {
    let path = lean_sys_root_module_path.as_ref();
    File::create(path)
        .map_err(|err| FileOutputError::Create { source: err })
        .and_then(|mut lean_sys_root_module_file| {
            lean_sys_root_module_file
                .write_all(LEAN_SYS_ROOT_MODULE)
                .map_err(|err| FileOutputError::Write { source: err })
        })
        .map_err(|error| LeanSysRootModuleGenerationError {
            path: path.to_path_buf(),
            source: error,
        })?;

    println!(
        "cargo::rustc-env=LEAN_SYS_ROOT_MODULE_INCLUDE={}",
        path.display()
    );

    Ok(())
}
