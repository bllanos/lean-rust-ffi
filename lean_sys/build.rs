use std::path::PathBuf;

use lean_build::LakeEnvironmentDescription;

fn main() -> anyhow::Result<()> {
    lean_build::runtime_build::build(
        LakeEnvironmentDescription {
            lake_executable_path: None::<PathBuf>,
        },
        Default::default(),
    )?;
    Ok(())
}
