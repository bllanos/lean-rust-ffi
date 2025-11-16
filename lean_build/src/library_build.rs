use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use bindgen::{BindgenError, builder};

use crate::lake::{
    self, LakeBuildOutputTraversalEvent, LakeBuildOutputTraverser, ModuleNameCreationError,
};
pub use crate::lake::{EnvironmentError, LakeLibraryBuildError, LakeLibraryDescription};
use crate::{NotUnicodeString, OutDirError};

pub struct OutputFilesConfig<'a> {
    /// The name of the file containing Rust bindings to the Lean library that
    /// will be generated in the build output directory
    ///
    /// The filename extension should be included in the string.
    ///
    /// The full path to the file will be exported as the
    /// `LEAN_LIBRARY_RUST_BINDINGS` environment variable.
    pub library_bindings_filename: &'a str,
}

impl Default for OutputFilesConfig<'static> {
    fn default() -> Self {
        Self {
            library_bindings_filename: "bindings.rs",
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BindingsGenerationError {
    #[error(transparent)]
    OutDir(#[from] OutDirError),
    #[error("error creating file \"{}\"", .path.display())]
    Create {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("error writing to file \"{}\"", .path.display())]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("error generating Rust bindings for \"{}\"", .path.display())]
    Bindgen { path: PathBuf, source: BindgenError },
    #[error("error creating Rust bindings module hierarchy")]
    ModuleNameCreation(#[from] ModuleNameCreationError),
}

#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    #[error("error retrieving Lake environment")]
    LakeEnvironmentError(#[from] EnvironmentError),
    #[error("invalid Lean include directory path")]
    LeanIncludeDirectoryNotUnicode(#[from] NotUnicodeString),
    #[error("error building and linking Lean module library")]
    LakeLibraryBuild(#[from] LakeLibraryBuildError),
    #[error("error creating Lean module Rust bindings")]
    BindingsGenerationError(#[from] BindingsGenerationError),
}

pub fn build<P: AsRef<Path>, Q: AsRef<OsStr>, R: AsRef<Path>, S: AsRef<Path>>(
    lake_library_description: &LakeLibraryDescription<P, Q, R, S>,
    output_files_config: OutputFilesConfig,
) -> Result<(), BuildError> {
    // Ensure the Lean toolchain is installed first
    let lake_environment = lake::get_lake_environment(lake_library_description)?;

    lake::build_and_link_static_lean_library(lake_library_description)?;

    let lean_include_directory = lake_environment.lean_include_directory();
    let lean_include_directory_str = lean_include_directory
        .to_str()
        .ok_or_else(|| NotUnicodeString(lean_include_directory.clone().into()))?;

    let out_dir = crate::get_out_dir().map_err(BindingsGenerationError::OutDir)?;

    let lean_c_files_traverser = lake::find_c_files(lake_library_description)
        .map_err(BindingsGenerationError::ModuleNameCreation)?;

    let bindings_out_filename = out_dir.join(output_files_config.library_bindings_filename);
    let mut bindings_out_file =
        File::create(&bindings_out_filename).map_err(|err| BindingsGenerationError::Create {
            path: bindings_out_filename.clone(),
            source: err,
        })?;

    let mut callback = |event| -> Result<(), BindingsGenerationError> {
        (match event {
            LakeBuildOutputTraversalEvent::PushDirectory { module_name } => writeln!(
                &mut bindings_out_file,
                "#[allow(non_snake_case)]
pub mod {module_name} {{"
            ),
            LakeBuildOutputTraversalEvent::CFile { path, module_name } => {
                let bindings = builder()
                    .clang_args(&["-I", lean_include_directory_str])
                    .header(path)
                    .blocklist_file(".*\\.h")
                    // This function is not defined in user-created Lean modules
                    .blocklist_function("initialize_Init")
                    .wrap_unsafe_ops(true)
                    .wrap_static_fns(false)
                    .must_use_type("lean_obj_res")
                    .must_use_type("b_lean_obj_res")
                    .use_core()
                    .merge_extern_blocks(true)
                    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
                    .generate()
                    .map_err(|err| BindingsGenerationError::Bindgen {
                        path: Path::new(path).to_path_buf(),
                        source: err,
                    })?;
                writeln!(&mut bindings_out_file, "pub mod {module_name} {{")
                    .and_then(|_| crate::write_warning_allow_directives(&mut bindings_out_file))
                    .and_then(|_| writeln!(&mut bindings_out_file, "use lean_sys::*;"))
                    .and_then(|_| bindings.write(Box::new(&bindings_out_file)))
                    .and_then(|_| writeln!(&mut bindings_out_file, "}}"))
            }
            LakeBuildOutputTraversalEvent::PopDirectory => writeln!(&mut bindings_out_file, "}}"),
        })
        .map_err(|err| BindingsGenerationError::Write {
            path: bindings_out_filename.clone(),
            source: err,
        })
    };

    lean_c_files_traverser.visit(&mut callback)?;

    println!(
        "cargo::rustc-env=LEAN_LIBRARY_RUST_BINDINGS={}",
        bindings_out_filename.display()
    );

    Ok(())
}
