use std::path::{Path, PathBuf};

use itertools::Itertools;
use regex::Regex;

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

pub fn lookup_unresolved_toolchain_desc(cfg: &Cfg, name: &str) -> Result<UnresolvedToolchainDesc> {
    let pattern = r"^(?:([a-zA-Z0-9-_]+[/][a-zA-Z0-9-_]+)[:])?([a-zA-Z0-9-.]+)$";

    let re = Regex::new(pattern).unwrap();
    if let Some(c) = re.captures(name) {
        let mut release = c.get(2).unwrap().as_str().to_owned();
        let local_tc = Toolchain::from(
            cfg,
            &ToolchainDesc::Local {
                name: release.clone(),
            },
        );
        if local_tc.exists() && local_tc.is_custom() {
            return Ok(UnresolvedToolchainDesc(ToolchainDesc::Local {
                name: release,
            }));
        }
        let mut origin = c
            .get(1)
            .map(|s| s.as_str())
            .unwrap_or(DEFAULT_ORIGIN)
            .to_owned();
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
    } else {
        Err(Error::InvalidToolchainName(name.to_string()))
    }
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
                Err(Error::NoLocalToolchains)
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
        lookup_unresolved_toolchain_desc(cfg, toolchain_name)
    } else {
        Err(Error::EmptyToolchainFile {
            path: toolchain_file.to_path_buf(),
        })
    }
}

pub fn lookup_toolchain_desc(cfg: &Cfg, name: &str) -> Result<ToolchainDesc> {
    resolve_toolchain_desc(cfg, &lookup_unresolved_toolchain_desc(cfg, name)?)
}
