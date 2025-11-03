use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const LEAN_SYS_ROOT_MODULE: &[u8; 565] = include_bytes!("lean_sys_root_module.rs");

pub fn create_lean_sys_root_module<P: AsRef<Path>>(
    lean_sys_root_module_path: P,
) -> Result<(), Box<dyn Error>> {
    let path = lean_sys_root_module_path.as_ref();
    let mut lean_sys_root_module_file = File::create(path)
        .map_err(|err| format!("failed to create file \"{}\": {}", path.display(), err))?;

    lean_sys_root_module_file
        .write_all(LEAN_SYS_ROOT_MODULE)
        .map_err(|err| format!("failed to write to file \"{}\": {}", path.display(), err))?;

    println!(
        "cargo::rustc-env=LEAN_SYS_ROOT_MODULE_INCLUDE={}",
        path.display()
    );

    Ok(())
}
