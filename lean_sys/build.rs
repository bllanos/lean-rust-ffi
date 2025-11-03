use std::error::Error;
use std::path::PathBuf;

use lean_build::LakeEnvironmentDescription;

fn main() -> Result<(), Box<dyn Error>> {
    lean_build::runtime_build::build(
        LakeEnvironmentDescription {
            lake_executable_path: None::<PathBuf>,
        },
        Default::default(),
    )
}
