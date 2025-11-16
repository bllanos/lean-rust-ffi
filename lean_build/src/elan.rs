use std::error::Error;
use std::sync::Arc;

use crate::elan_fork::elan::{
    Cfg as ElanCfg, Notification, OverrideReason, notify::NotificationLevel,
};

mod toolchain;

use toolchain::LeanToolchainVersion;

fn create_elan_cfg() -> Result<ElanCfg, ToolchainResolutionError> {
    ElanCfg::from_env(Arc::new(move |n: Notification<'_>| match n.level() {
        NotificationLevel::Warn => {
            println!("cargo::warning={n}");
        }
    }))
    .map_err(ToolchainResolutionError::from_elan_error)
}

fn rerun_build_if_elan_environment_variables_change() {
    println!("cargo::rerun-if-env-changed={}", crate::ELAN_TOOLCHAIN);
}

fn rerun_build_if_elan_settings_change(elan_cfg: &ElanCfg) {
    println!(
        "cargo::rerun-if-changed={}",
        elan_cfg.elan_dir.join("settings.toml").display()
    );
}

#[derive(thiserror::Error, Debug)]
#[error("unexpected toolchain override reason: {override_reason}")]
pub struct UnexpectedToolchainOverrideReasonError {
    pub override_reason: OverrideReason,
}

impl From<OverrideReason> for UnexpectedToolchainOverrideReasonError {
    fn from(override_reason: OverrideReason) -> Self {
        Self { override_reason }
    }
}

fn rerun_build_if_lean_toolchain_override_changes(
    elan_cfg: &ElanCfg,
    override_reason: &OverrideReason,
) -> Result<(), UnexpectedToolchainOverrideReasonError> {
    match override_reason {
        OverrideReason::Environment => {
            rerun_build_if_elan_environment_variables_change();
            Ok(())
        }
        OverrideReason::InToolchainDirectory(_) => Err(override_reason.clone().into()),
        OverrideReason::LeanpkgFile(path) => {
            println!("cargo::rerun-if-changed={}", path.display());
            Ok(())
        }
        OverrideReason::OverrideDB(_) => {
            rerun_build_if_elan_settings_change(elan_cfg);
            Ok(())
        }
        OverrideReason::ToolchainFile(path) => {
            println!("cargo::rerun-if-changed={}", path.display());
            Ok(())
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("error resolving Lean toolchain")]
pub struct ToolchainResolutionError(#[source] pub Box<dyn Error + Send + Sync + 'static>);

impl ToolchainResolutionError {
    fn from_elan_error(error: crate::elan_fork::elan::Error) -> Self {
        Self(Box::new(error) as Box<dyn Error + Send + Sync + 'static>)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EnvironmentError {
    #[error(transparent)]
    ToolchainResolution(#[from] ToolchainResolutionError),
    #[error(transparent)]
    UnexpectedToolchainOverrideReason(#[from] UnexpectedToolchainOverrideReasonError),
}

pub fn rerun_build_if_lean_version_changes() -> Result<(), EnvironmentError> {
    rerun_build_if_elan_environment_variables_change();
    let elan_cfg = create_elan_cfg()?;
    rerun_build_if_elan_settings_change(&elan_cfg);
    let (lean_toolchain_version, override_reason) =
        LeanToolchainVersion::from_elan_environment(&elan_cfg)?;
    if let Some(override_reason) = override_reason {
        rerun_build_if_lean_toolchain_override_changes(&elan_cfg, &override_reason)?;
    }

    if lean_toolchain_version.is_floating_version() {
        let elan_toolchain_directory = &elan_cfg.toolchains_dir;
        println!("cargo::warning=Lean toolchain: {lean_toolchain_version}");
        println!(
            "cargo::warning=specifying a floating version of the Lean toolchain (i.e. a channel) will slow down Cargo builds because the entire Elan toolchains directory (\"{}\") must be monitored for changes to detect a change in the latest Lean toolchain version",
            elan_toolchain_directory.display()
        );
        println!(
            "cargo::rerun-if-changed={}",
            elan_toolchain_directory.display()
        );
    }
    Ok(())
}
