use std::env;
use std::path::{Path, PathBuf};

use lean_build::library_build::LakeLibraryDescription;

const LEAN_MODULE_PARENT_DIRECTORY_NAME: &str = "lean";
const LEAN_MODULE_DIRECTORY_NAME: &str = "map_array";

#[derive(thiserror::Error, Debug)]
#[error("failed to access the level {level} parent directory of the Cargo manifest directory \"{}\"", .manifest_directory.display())]
struct CargoManifestParentPathError {
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

fn get_lake_package_path() -> Result<PathBuf, CargoManifestParentPathError> {
    let manifest_directory = Path::new(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest_directory
        .parent()
        .ok_or_else(|| CargoManifestParentPathError::new(1, manifest_directory))?
        .parent()
        .ok_or_else(|| CargoManifestParentPathError::new(2, manifest_directory))?
        .join(LEAN_MODULE_PARENT_DIRECTORY_NAME)
        .join(LEAN_MODULE_DIRECTORY_NAME))
}

fn main() -> anyhow::Result<()> {
    let lake_package_path = get_lake_package_path()?;
    let c_files_directory = lake_package_path.join(".lake").join("build").join("ir");
    lean_build::library_build::build(
        &LakeLibraryDescription {
            lake_package_path,
            lake_executable_path: None::<PathBuf>,
            target_name: "MapArray",
            source_directory: None::<PathBuf>,
            c_files_directory: Some(c_files_directory),
        },
        Default::default(),
    )?;
    Ok(())
}
