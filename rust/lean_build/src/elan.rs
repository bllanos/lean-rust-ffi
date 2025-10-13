use std::error::Error;
use std::sync::Arc;

use crate::elan_fork::elan::{
    Cfg as ElanCfg, Notification, OverrideReason, notify::NotificationLevel,
};

mod toolchain;

use toolchain::LeanToolchainVersion;

fn create_elan_cfg() -> Result<ElanCfg, Box<dyn Error>> {
    Ok(ElanCfg::from_env(Arc::new(
        move |n: Notification<'_>| match n.level() {
            NotificationLevel::Warn => {
                println!("cargo:warning={n}");
            }
        },
    ))?)
}

fn rerun_build_if_elan_environment_variables_change() {
    println!("cargo:rerun-if-env-changed={}", crate::ELAN_TOOLCHAIN);
}

fn rerun_build_if_elan_settings_change(elan_cfg: &ElanCfg) {
    println!(
        "cargo:rerun-if-changed={}",
        elan_cfg.elan_dir.join("settings.toml").display()
    );
}

fn rerun_build_if_lean_toolchain_override_changes(
    elan_cfg: &ElanCfg,
    override_reason: &OverrideReason,
) -> Result<(), Box<dyn Error>> {
    match override_reason {
        OverrideReason::Environment => {
            rerun_build_if_elan_environment_variables_change();
            Ok(())
        }
        OverrideReason::InToolchainDirectory(_) => {
            Err(format!("unexpected toolchain override_reason reason: {override_reason}").into())
        }
        OverrideReason::LeanpkgFile(path) => {
            println!("cargo:rerun-if-changed={}", path.display());
            Ok(())
        }
        OverrideReason::OverrideDB(_) => {
            rerun_build_if_elan_settings_change(elan_cfg);
            Ok(())
        }
        OverrideReason::ToolchainFile(path) => {
            println!("cargo:rerun-if-changed={}", path.display());
            Ok(())
        }
    }
}

pub fn rerun_build_if_lean_version_changes() -> Result<(), Box<dyn Error>> {
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
        println!("cargo:warning=Lean toolchain: {lean_toolchain_version}");
        println!(
            "cargo:warning=specifying a floating version of the Lean toolchain (i.e. a channel) will slow down Cargo builds because the entire Elan toolchains directory (\"{}\") must be monitored for changes to detect a change in the latest Lean toolchain version",
            elan_toolchain_directory.display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            elan_toolchain_directory.display()
        );
    }
    Ok(())
}
