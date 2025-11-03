use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use bindgen::builder;

use crate::lake::{self, LakeEnvironmentDescriber};

pub struct OutputFilesConfig<'a> {
    /// The base of the native library that will be generated to contain functions
    /// that are declared as `inline` in Lean's header files
    ///
    /// For example, `"foo"` will result in `foo.c` and `libfoo.a` in the build
    /// output directory.
    pub inline_functions_library_base_name: &'a str,
    /// The name of the file containing Rust bindings to Lean that will be
    /// generated in the build output directory
    ///
    /// The filename extension should be included in the string.
    pub lean_bindings_filename: &'a str,
    /// The name of the file that can serve as the `lib.rs` file of a Lean sys library
    ///
    /// This file will be output to the build directory with the given name. The
    /// filename extension should be included in the string. It is intended to
    /// be used with `include!()` as follows:
    ///
    /// ```ignore
    /// #![no_std]
    /// include!(env!("LEAN_SYS_ROOT_MODULE_INCLUDE"));
    /// ```
    pub lean_sys_root_module_filename: &'a str,
}

impl Default for OutputFilesConfig<'static> {
    fn default() -> Self {
        Self {
            inline_functions_library_base_name: "lean_sys_inline_functions_wrapper",
            lean_bindings_filename: "bindings.rs",
            lean_sys_root_module_filename: "lean_sys_root_module.rs",
        }
    }
}

pub fn build<T: LakeEnvironmentDescriber>(
    lake_environment_describer: T,
    output_files_config: OutputFilesConfig,
) -> Result<(), Box<dyn Error>> {
    // Ensure the Lean toolchain is installed first
    let lake_environment = lake::get_lake_environment(&lake_environment_describer)?;
    let lean_library_directory = lake_environment.lean_library_directory();
    let lean_sysroot_library_directory = lake_environment.lean_sysroot_library_directory();

    lake_environment.export_rustc_env();
    crate::elan::rerun_build_if_lean_version_changes()?;

    println!(
        "cargo::rustc-link-search={}",
        lean_library_directory.display()
    );
    println!(
        "cargo::rustc-link-search={}",
        lean_sysroot_library_directory.display()
    );

    println!("cargo::rustc-link-lib=static=Init");
    println!("cargo::rustc-link-lib=static=leanrt");
    println!("cargo::rustc-link-lib=static=uv");
    println!("cargo::rustc-link-lib=static=gmp");
    println!("cargo::rustc-link-lib=static=c++");
    println!("cargo::rustc-link-lib=static=c++abi");
    println!("cargo::rustc-link-lib=dylib=m");

    let lean_include_directory = lake_environment.lean_include_directory();
    let lean_include_directory_str =
        lean_include_directory
            .to_str()
            .ok_or_else(|| -> Box<dyn Error> {
                format!(
                    "Lean include directory path is not a valid UTF-8 string, \"{}\"",
                    lean_include_directory.display()
                )
                .into()
            })?;

    let lean_header_path = lake_environment.lean_header_path();
    let lean_header_path_str = lean_header_path.to_str().ok_or_else(|| -> Box<dyn Error> {
        format!(
            "Lean header path is not a valid UTF-8 string, \"{}\"",
            lean_header_path.display()
        )
        .into()
    })?;

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let inline_wrapper_functions_out_file = out_dir.join(format!(
        "{}.c",
        output_files_config.inline_functions_library_base_name
    ));

    let bindings = builder()
        .clang_args(&["-I", lean_include_directory_str])
        .header(lean_header_path_str)
        .wrap_unsafe_ops(true)
        .wrap_static_fns(true)
        .wrap_static_fns_path(&inline_wrapper_functions_out_file)
        // Block functions that produce compilation errors
        .blocklist_function("lean_get_rc_mt_addr")
        .must_use_type("lean_obj_res")
        .must_use_type("b_lean_obj_res")
        .use_core()
        .merge_extern_blocks(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?;

    let bindings_out_filename = out_dir.join(output_files_config.lean_bindings_filename);
    let mut bindings_out_file = File::create(&bindings_out_filename).map_err(|err| {
        format!(
            "failed to create Lean runtime Rust bindings file \"{}\": {}",
            bindings_out_filename.display(),
            err
        )
    })?;
    // Create a module to contain warning allow directives
    writeln!(&mut bindings_out_file, "mod lean_sys {{")?;
    crate::write_warning_allow_directives(&mut bindings_out_file)?;
    bindings.write(Box::new(&bindings_out_file))?;
    writeln!(
        &mut bindings_out_file,
        "}}
pub use lean_sys::*;"
    )?;

    println!(
        "cargo::rustc-env=LEAN_RUST_BINDINGS={}",
        bindings_out_filename.display()
    );

    cc::Build::new()
        .file(&inline_wrapper_functions_out_file)
        .include(lake_environment.lean_include_directory())
        .include(lake_environment.lean_clang_include_directory())
        .include(lake_environment.lean_lean_include_directory())
        .static_flag(true)
        .compiler(lake_environment.lean_clang_path())
        .archiver(lake_environment.lean_ar_path())
        .compile(output_files_config.inline_functions_library_base_name);

    let lean_sys_root_module_path = out_dir.join(output_files_config.lean_sys_root_module_filename);
    super::rust::create_lean_sys_root_module(lean_sys_root_module_path)?;

    Ok(())
}
