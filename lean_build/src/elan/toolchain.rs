use std::fmt;
use std::path::PathBuf;

use super::ToolchainResolutionError;
use crate::elan_fork::elan::{Cfg as ElanCfg, OverrideReason, Toolchain};
use crate::elan_fork::elan_dist::dist::ToolchainDesc;

#[derive(PartialEq, Eq)]
pub enum LeanToolchainVersion {
    // A linked toolchain
    Local {
        release: String,
    },
    Remote {
        // The GitHub source repository
        origin: String,
        // The release name, usually a Git tag
        resolved_release: String,
        // The channel name the release was resolved from, if any
        channel: Option<String>,
    },
    // A toolchain specified by file path in a `lean-toolchain` file
    Path {
        path: PathBuf,
    },
}

impl LeanToolchainVersion {
    pub fn from_elan_environment(
        elan_cfg: &ElanCfg,
    ) -> Result<(Self, Option<OverrideReason>), ToolchainResolutionError> {
        let (toolchain, override_reason) = elan_cfg
            .toolchain_for_current_directory()
            .map_err(ToolchainResolutionError::from_elan_error)?;
        Ok((toolchain.into(), override_reason))
    }

    pub fn is_floating_version(&self) -> bool {
        matches!(
            self,
            Self::Remote {
                channel: Some(_),
                ..
            }
        )
    }
}

impl From<Toolchain> for LeanToolchainVersion {
    fn from(toolchain: Toolchain) -> Self {
        let desc = toolchain.desc;
        match desc {
            ToolchainDesc::Local { name } => Self::Local { release: name },
            ToolchainDesc::Remote {
                origin,
                release,
                from_channel,
            } => Self::Remote {
                origin,
                resolved_release: release,
                channel: from_channel,
            },
            ToolchainDesc::Path { path } => Self::Path { path },
        }
    }
}

impl fmt::Display for LeanToolchainVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Remote {
                origin,
                resolved_release,
                channel: Some(channel),
            } => write!(
                f,
                "latest release on channel \"{channel}\" from origin \"{origin}\", resolved using the latest installed toolchain version \"{resolved_release}\""
            ),
            Self::Remote {
                origin,
                resolved_release,
                channel: None,
            } => write!(f, "release \"{resolved_release}\" from origin \"{origin}\""),
            Self::Local { release } => write!(f, "local release \"{release}\""),
            Self::Path { path } => write!(f, "local directory \"{}\"", path.display()),
        }
    }
}
