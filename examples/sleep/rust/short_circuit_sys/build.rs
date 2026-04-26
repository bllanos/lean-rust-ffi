use std::env;

use examples_build_infrastructure::Config;

fn main() -> anyhow::Result<()> {
    examples_build_infrastructure::build(Config {
        lean_module_directory_name: "short_circuit",
        manifest_directory: env!("CARGO_MANIFEST_DIR"),
        target_name: "ShortCircuit",
    })?;
    Ok(())
}
