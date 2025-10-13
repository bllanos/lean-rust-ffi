use std::path::PathBuf;

use super::super::elan_dist::{self, dist::ToolchainDesc};
use super::super::elan_utils;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(
        "no default toolchain configured. run `elan default stable` to install and configure the latest Lean 4 stable release."
    )]
    NoDefaultToolchain,
    #[error("no local Lean toolchains found")]
    NoLocalToolchains,
    #[error(transparent)]
    LocalToolchainDoesNotMatchRemote(#[from] Box<LocalToolchainDoesNotMatchRemote>),
    #[error("invalid toolchain name: '{0}'")]
    InvalidToolchainName(String),
    #[error("override toolchain '{toolchain}' is not installed: {reason_err}")]
    OverrideToolchainNotInstalled {
        toolchain: ToolchainDesc,
        reason_err: String,
        source: Box<Self>,
    },
    #[error("this crate does not install toolchains, toolchain in question is '{0}'")]
    ToolchainInstallForbidden(ToolchainDesc),
    #[error("error parsing settings")]
    ParsingSettings(toml::de::Error),
    #[error("unknown metadata version '{0}'")]
    UnknownMetadataVersion(String),
    #[error("empty toolchain file '{}'", .path.display())]
    EmptyToolchainFile { path: PathBuf },
    #[error("could not parse '{}' as a Leanpkg file", .path.display())]
    InvalidLeanpkgFile {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("invalid 'package.lean_version' value in '{}': expected string instead of {type_str}", .path.display())]
    InvalidLeanVersion {
        path: PathBuf,
        type_str: &'static str,
    },
    #[error(transparent)]
    Utils(#[from] elan_utils::Error),
    #[error(transparent)]
    Dist(#[from] elan_dist::Error),
    #[error("failed to obtain the current directory")]
    CurrentDirectory { source: std::io::Error },
}

#[derive(thiserror::Error, Debug)]
#[error(
    "the latest local toolchain, '{local}', does not match the toolchain to resolve, '{unresolved}'"
)]
pub struct LocalToolchainDoesNotMatchRemote {
    pub local: ToolchainDesc,
    pub unresolved: ToolchainDesc,
}

pub type Result<T> = std::result::Result<T, Error>;
