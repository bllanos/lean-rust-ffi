use std::error::Error;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

mod env;
mod package;

pub use env::get_lake_environment;
pub use package::{LakeBuildOutputTraversalEvent, LakeBuildOutputTraverser, find_c_files};

fn display_slice(slice: &[u8]) -> &str {
    str::from_utf8(slice).unwrap_or("[Non-UTF8]")
}

fn get_lake_executable_path(lake_executable_path: Option<&OsStr>) -> &OsStr {
    match lake_executable_path {
        Some(lake_executable_path) => lake_executable_path,
        None => OsStr::new("lake"),
    }
}

pub trait LakeEnvironmentDescriber {
    fn get_lake_executable_path(&self) -> &OsStr;
}

impl<T: LakeEnvironmentDescriber> LakeEnvironmentDescriber for &T {
    fn get_lake_executable_path(&self) -> &OsStr {
        (*self).get_lake_executable_path()
    }
}

pub struct LakeEnvironmentDescription<T: AsRef<OsStr> = PathBuf> {
    /// The path to the Lake executable. Defaults to `"lake"`, which requires
    /// the executable to be on the executable search path.
    pub lake_executable_path: Option<T>,
}

impl<T: AsRef<OsStr>> LakeEnvironmentDescriber for LakeEnvironmentDescription<T> {
    fn get_lake_executable_path(&self) -> &OsStr {
        get_lake_executable_path(
            self.lake_executable_path
                .as_ref()
                .map(<T as AsRef<OsStr>>::as_ref),
        )
    }
}

pub struct LakeLibraryDescription<
    'a,
    P: AsRef<Path>,
    Q: AsRef<OsStr> = PathBuf,
    R: AsRef<Path> = PathBuf,
    S: AsRef<Path> = PathBuf,
> {
    /// The path to the Lake package containing the library
    pub lake_package_path: P,
    /// The path to the Lake executable. Defaults to `"lake"`, which requires
    /// the executable to be on the executable search path.
    pub lake_executable_path: Option<Q>,
    pub target_name: &'a str,
    /// The directory containing the library's Lean source files, used for
    /// change detection. Defaults to `lake_package_path`
    pub source_directory: Option<R>,
    /// The directory containing the library's build C files, which is useful in
    /// cases where only a subtree of the directory hierarchy of build C files
    /// is of interest. Defaults to `lake_package_path`
    pub c_files_directory: Option<S>,
}

impl<'a, P: AsRef<Path>, Q: AsRef<OsStr>, R: AsRef<Path>, S: AsRef<Path>> LakeEnvironmentDescriber
    for LakeLibraryDescription<'a, P, Q, R, S>
{
    fn get_lake_executable_path(&self) -> &OsStr {
        get_lake_executable_path(
            self.lake_executable_path
                .as_ref()
                .map(<Q as AsRef<OsStr>>::as_ref),
        )
    }
}

impl<'a, P: AsRef<Path>, Q: AsRef<OsStr>, R: AsRef<Path>, S: AsRef<Path>>
    LakeLibraryDescription<'a, P, Q, R, S>
{
    fn get_lake_package_path(&self) -> &Path {
        self.lake_package_path.as_ref()
    }

    fn get_source_directory(&self) -> &Path {
        match self.source_directory.as_ref() {
            Some(source_directory) => source_directory.as_ref(),
            None => self.get_lake_package_path(),
        }
    }

    fn get_c_files_directory(&self) -> &Path {
        match self.c_files_directory.as_ref() {
            Some(c_files_directory) => c_files_directory.as_ref(),
            None => self.get_lake_package_path(),
        }
    }
}

fn run_lake_command_and_retrieve_stdout<'a, P: AsRef<OsStr>>(
    lake_executable_path: P,
    args: &'a [&'a OsStr],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let output = Command::new(&lake_executable_path)
        .args(args)
        .output()
        .map_err(|err| {
            format!(
                "Failed to invoke Lake executable with path \"{}\", error: {err}",
                display_slice(lake_executable_path.as_ref().as_encoded_bytes()),
            )
        })?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(format!(
            "Lake invocation (path \"{}\") with arguments {args:?} failed with status {}, stdout:{}, stderr: {}",
            display_slice(lake_executable_path.as_ref().as_encoded_bytes()),
            output.status,
            display_slice(&output.stdout),
            display_slice(&output.stderr),
        )
        .into())
    }
}

/// Assumes there is only a single target path
fn get_lake_target_path_from_lake_query_output(
    lean_build_output: &[u8],
) -> Result<PathBuf, Box<dyn Error>> {
    let output_str = str::from_utf8(lean_build_output)
        .map_err(|err| format!("Lake query output is not valid UTF8: {err}"))?;
    Ok(Path::new(output_str).to_path_buf())
}

fn rerun_build_if_lake_package_changes<
    P: AsRef<Path>,
    Q: AsRef<OsStr>,
    R: AsRef<Path>,
    S: AsRef<Path>,
>(
    lake_library_description: &LakeLibraryDescription<P, Q, R, S>,
) {
    println!(
        "cargo::rerun-if-changed={}",
        lake_library_description.get_source_directory().display()
    );
}

pub fn build_and_link_static_lean_library<
    P: AsRef<Path>,
    Q: AsRef<OsStr>,
    R: AsRef<Path>,
    S: AsRef<Path>,
>(
    lake_library_description: &LakeLibraryDescription<P, Q, R, S>,
) -> Result<(), Box<dyn Error>> {
    let lake_package_path = lake_library_description.get_lake_package_path();
    let build_target = format!("@/{}:static", lake_library_description.target_name);
    let args = [
        OsStr::new("--dir"),
        lake_package_path.as_os_str(),
        OsStr::new("query"),
        OsStr::new("--text"),
        OsStr::new(&build_target),
    ];
    let stdout = run_lake_command_and_retrieve_stdout(
        lake_library_description.get_lake_executable_path(),
        &args,
    )?;
    let library_path = get_lake_target_path_from_lake_query_output(&stdout)?;
    if let Some(library_directory) = library_path.parent() {
        println!("cargo::rustc-link-search={}", library_directory.display());
    }
    println!(
        "cargo::rustc-link-lib=static={}",
        lake_library_description.target_name
    );

    rerun_build_if_lake_package_changes(lake_library_description);
    Ok(())
}
