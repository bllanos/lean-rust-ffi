use std::path::{Path, PathBuf};

use lean_build::library_build::{BuildError, LakeLibraryDescription};

const LEAN_MODULE_PARENT_DIRECTORY_NAME: &str = "lean";

pub struct Config<'a> {
    pub lean_module_directory_name: &'a str,
    pub manifest_directory: &'a str,
    pub target_name: &'a str,
}

#[derive(thiserror::Error, Debug)]
#[error("failed to access the level {level} parent directory of the Cargo manifest directory \"{}\"", .manifest_directory.display())]
pub struct CargoManifestParentPathError {
    level: usize,
    manifest_directory: PathBuf,
}

impl CargoManifestParentPathError {
    fn new(level: usize, manifest_directory: &Path) -> Self {
        Self {
            level,
            manifest_directory: manifest_directory.into(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ExampleBuildError {
    #[error(transparent)]
    CargoManifestParentPath(#[from] CargoManifestParentPathError),
    #[error(transparent)]
    Build(#[from] BuildError),
}

fn get_lake_package_path(config: &Config) -> Result<PathBuf, CargoManifestParentPathError> {
    let manifest_directory = Path::new(config.manifest_directory);
    Ok(manifest_directory
        .parent()
        .ok_or_else(|| CargoManifestParentPathError::new(1, manifest_directory))?
        .parent()
        .ok_or_else(|| CargoManifestParentPathError::new(2, manifest_directory))?
        .join(LEAN_MODULE_PARENT_DIRECTORY_NAME)
        .join(config.lean_module_directory_name))
}

pub fn build(config: Config) -> Result<(), ExampleBuildError> {
    let lake_package_path = get_lake_package_path(&config)?;
    let c_files_directory = lake_package_path.join(".lake").join("build").join("ir");
    lean_build::library_build::build(
        &LakeLibraryDescription {
            lake_package_path,
            lake_executable_path: None::<PathBuf>,
            target_name: config.target_name,
            source_directory: None::<PathBuf>,
            c_files_directory: Some(c_files_directory),
        },
        Default::default(),
    )?;
    Ok(())
}
