use std::path::{Path, PathBuf};

use itertools::Itertools;

use super::super::elan_dist::{dist::ToolchainDesc, manifestation::DEFAULT_ORIGIN};
use super::super::elan_utils::utils;
use super::config::Cfg;
use super::errors::*;

/// A fully resolved reference to a toolchain which may or may not exist
pub struct Toolchain {
    pub desc: ToolchainDesc,
    path: PathBuf,
}

impl Toolchain {
    pub fn exists(&self) -> bool {
        // HACK: linked toolchains are symlinks, and, contrary to what std docs
        // lead me to believe `fs::metadata`, used by `is_directory` does not
        // seem to follow symlinks on windows.
        utils::is_directory(&self.path) || self.is_symlink()
    }

    pub fn is_custom(&self) -> bool {
        assert!(self.exists());
        self.is_symlink()
    }

    pub fn install_from_dist(&self) -> Result<()> {
        Err(Error::ToolchainInstallForbidden(self.desc.clone()))
    }

    fn is_symlink(&self) -> bool {
        use std::fs;
        fs::symlink_metadata(&self.path)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    }

    pub fn from(cfg: &Cfg, desc: &ToolchainDesc) -> Self {
        // We need to replace ":" and "/" with "-" in the toolchain name in
        // order to make a name which is a valid name for a directory.
        let dir_name = desc.to_string().replace("/", "--").replace(":", "---");

        let path = cfg.toolchains_dir.join(&dir_name[..]);

        Toolchain {
            desc: desc.clone(),
            path: path.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnresolvedToolchainDesc(pub ToolchainDesc);

pub fn lookup_unresolved_toolchain_desc(
    cfg: &Cfg,
    name: &str,
    base_dir: Option<&Path>,
) -> Result<UnresolvedToolchainDesc> {
    // Try parsing as a relative file path first (needs base_dir context)
    let base_dir = base_dir.unwrap_or_else(|| Path::new("."));
    if let Some(path_desc) = try_parse_path_toolchain(name, base_dir)? {
        return Ok(UnresolvedToolchainDesc(path_desc));
    }

    // Parse the base descriptor (handles absolute paths, origin:release, and bare names)
    let desc = ToolchainDesc::from_resolved_str(name)?;

    // Path toolchains (absolute paths) are returned as-is
    if matches!(desc, ToolchainDesc::Path { .. }) {
        return Ok(UnresolvedToolchainDesc(desc));
    }

    // Extract the release/name portion for local toolchain check
    let release_name = match &desc {
        ToolchainDesc::Local { name } => name.clone(),
        ToolchainDesc::Remote { release, .. } => release.clone(),
        _ => unreachable!(),
    };

    // Check if it matches an existing linked (custom) toolchain
    let local_tc = Toolchain::from(
        cfg,
        &ToolchainDesc::Local {
            name: release_name.clone(),
        },
    );
    if local_tc.exists() && local_tc.is_custom() {
        return Ok(UnresolvedToolchainDesc(ToolchainDesc::Local {
            name: release_name,
        }));
    }

    // Build a Remote descriptor with unresolved-specific transformations
    let (mut origin, mut release) = match desc {
        ToolchainDesc::Remote {
            origin, release, ..
        } => (origin, release),
        ToolchainDesc::Local { name } => (DEFAULT_ORIGIN.to_owned(), name),
        _ => unreachable!(),
    };

    if release.starts_with("nightly") && !origin.ends_with("-nightly") {
        origin = format!("{origin}-nightly");
    }
    let mut from_channel = None;
    if release == "lean-toolchain"
        || release == "stable"
        || release == "beta"
        || release == "nightly"
    {
        from_channel = Some(release.to_string());
    }
    if release.starts_with(char::is_numeric) {
        release = format!("v{release}")
    }
    Ok(UnresolvedToolchainDesc(ToolchainDesc::Remote {
        origin,
        release,
        from_channel,
    }))
}

fn find_latest_local_toolchain(cfg: &Cfg, channel: &str) -> Option<ToolchainDesc> {
    let toolchains = cfg.list_toolchains().ok()?;
    let toolchains = toolchains.into_iter().filter_map(|tc| match tc {
        ToolchainDesc::Remote { release: ref r, .. } => Some((tc.to_owned(), r.to_string())),
        _ => None,
    });
    let toolchains: Vec<_> = match channel {
        "nightly" => toolchains
            .filter(|t| t.1.starts_with("nightly-"))
            .sorted_by_key(|t| t.1.to_string())
            .map(|t| t.0)
            .collect(),
        _ => toolchains
            .filter_map(|t| {
                semver::Version::parse(t.1.trim_start_matches("v"))
                    .ok()
                    .filter(|v| (channel == "stable") == v.pre.is_empty())
                    .map(|v| (t.0, v))
            })
            .sorted_by_key(|t| t.1.clone())
            .map(|t| t.0)
            .collect(),
    };
    toolchains.into_iter().last()
}

pub fn resolve_toolchain_desc_ext(
    cfg: &Cfg,
    unresolved_tc: &UnresolvedToolchainDesc,
) -> Result<ToolchainDesc> {
    if let ToolchainDesc::Remote {
        ref origin,
        ref release,
        ref from_channel,
    } = unresolved_tc.0
    {
        if release == "lean-toolchain"
            || release == "stable"
            || release == "beta"
            || release == "nightly"
        {
            if let Some(tc) = find_latest_local_toolchain(cfg, release) {
                let resolved_toolchain = ToolchainDesc::from_resolved_str(&tc.to_string())?;
                if let ToolchainDesc::Remote {
                    origin: resolved_origin,
                    release: resolved_release,
                    ..
                } = resolved_toolchain
                    && origin == &resolved_origin
                // Note: Local toolchains are missing channel fields
                {
                    Ok(ToolchainDesc::Remote {
                        origin: resolved_origin,
                        release: resolved_release,
                        from_channel: from_channel.clone(),
                    })
                } else {
                    Err(Box::new(LocalToolchainDoesNotMatchRemote {
                        local: tc,
                        unresolved: unresolved_tc.0.clone(),
                    })
                    .into())
                }
            } else {
                Err(Error::NoLocalToolchains {
                    release: release.to_owned(),
                })
            }
        } else {
            Ok(unresolved_tc.0.clone())
        }
    } else {
        Ok(unresolved_tc.0.clone())
    }
}

pub fn resolve_toolchain_desc(
    cfg: &Cfg,
    unresolved_tc: &UnresolvedToolchainDesc,
) -> Result<ToolchainDesc> {
    resolve_toolchain_desc_ext(cfg, unresolved_tc)
}

pub fn read_unresolved_toolchain_desc_from_file(
    cfg: &Cfg,
    toolchain_file: &Path,
) -> Result<UnresolvedToolchainDesc> {
    let s = utils::read_file("toolchain file", toolchain_file)?;
    if let Some(s) = s.lines().next() {
        let toolchain_name = s.trim();

        let toolchain_file_dir = toolchain_file.parent().unwrap(); // Every file should have a parent

        lookup_unresolved_toolchain_desc(cfg, toolchain_name, Some(toolchain_file_dir))
    } else {
        Err(Error::EmptyToolchainFile {
            path: toolchain_file.to_path_buf(),
        })
    }
}

pub fn lookup_toolchain_desc(cfg: &Cfg, name: &str) -> Result<ToolchainDesc> {
    resolve_toolchain_desc(cfg, &lookup_unresolved_toolchain_desc(cfg, name, None)?)
}

/// Try to parse a string as a file path, validating it contains a Lean toolchain
fn try_parse_path_toolchain(
    path_str: &str,
    toolchain_file_dir: &Path,
) -> Result<Option<ToolchainDesc>> {
    // Try to resolve the path relative to the lean-toolchain file's directory
    let path = if Path::new(path_str).is_absolute() {
        PathBuf::from(path_str)
    } else {
        toolchain_file_dir.join(path_str)
    };

    // Validate that bin/lean exists
    let lean_binary = path
        .join("bin")
        .join(format!("lean{}", std::env::consts::EXE_SUFFIX));
    if lean_binary.is_file() {
        return Ok(Some(ToolchainDesc::Path { path }));
    }

    // Error on path-like input rather than falling through to toolchain-name parsing, which would
    // silently attempt to download a default toolchain.
    if looks_like_explicit_path(path_str) {
        return Err(Error::MissingLeanBinary {
            path,
            lean_binary_path: lean_binary,
        });
    }

    Ok(None)
}

fn looks_like_explicit_path(s: &str) -> bool {
    Path::new(s).is_absolute()
        || s.starts_with("./")
        || s.starts_with("../")
        || s.starts_with(".\\")
        || s.starts_with("..\\")
}

// Unit tests in Elan's code that access the filesystem have been removed
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolchain_name_not_mistaken_for_path() {
        // Standard toolchain names must not be treated as paths
        for name in &[
            "leanprover/lean4:v4.3.0",
            "v4.3.0",
            "nightly-2024-01-01",
            "stable",
        ] {
            let result = try_parse_path_toolchain(name, Path::new("/irrelevant")).unwrap();
            assert!(result.is_none(), "{name} should not parse as a path");
        }
    }
}
