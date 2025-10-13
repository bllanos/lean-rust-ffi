use std::env;
use std::fmt::{self, Display};
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use super::super::elan_dist::dist::ToolchainDesc;
use super::super::elan_utils::utils;
use super::errors::*;
use super::notifications::*;
use super::settings::{Settings, SettingsFile};
use super::toolchain::{
    Toolchain, UnresolvedToolchainDesc, lookup_toolchain_desc, lookup_unresolved_toolchain_desc,
    read_unresolved_toolchain_desc_from_file, resolve_toolchain_desc,
};

#[derive(Debug, Clone)]
pub enum OverrideReason {
    /// `ELAN_TOOLCHAIN` environment variable override
    Environment,
    /// `elan override` override
    OverrideDB(PathBuf),
    /// `lean-toolchain` override
    ToolchainFile(PathBuf),
    /// `leanpkg.toml` override lol
    LeanpkgFile(PathBuf),
    /// inside a toolchain directory
    InToolchainDirectory(PathBuf),
}

impl Display for OverrideReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> ::std::result::Result<(), fmt::Error> {
        match *self {
            OverrideReason::Environment => {
                write!(f, "environment override by {}", crate::ELAN_TOOLCHAIN)
            }
            OverrideReason::OverrideDB(ref path) => {
                write!(f, "directory override for '{}'", path.display())
            }
            OverrideReason::ToolchainFile(ref path) => {
                write!(f, "overridden by '{}'", path.display())
            }
            OverrideReason::InToolchainDirectory(ref path) => {
                write!(
                    f,
                    "override because inside toolchain directory '{}'",
                    path.display()
                )
            }
            OverrideReason::LeanpkgFile(ref path) => {
                write!(f, "overridden by '{}'", path.display())
            }
        }
    }
}

pub struct Cfg {
    pub elan_dir: PathBuf,
    pub settings_file: SettingsFile,
    pub toolchains_dir: PathBuf,
    pub env_override: Option<String>,
    pub notify_handler: Arc<dyn Fn(Notification<'_>)>,
}

impl Cfg {
    pub fn toolchain_for_current_directory(&self) -> Result<(Toolchain, Option<OverrideReason>)> {
        self.find_override_toolchain_or_default()
            .and_then(|r| r.ok_or(Error::NoDefaultToolchain))
    }

    pub fn find_override_toolchain_or_default(
        &self,
    ) -> Result<Option<(Toolchain, Option<OverrideReason>)>> {
        if let Some((toolchain, reason)) = self.find_override()? {
            let toolchain = resolve_toolchain_desc(self, &toolchain)?;
            match self.get_toolchain(&toolchain, false) {
                Ok(toolchain) => {
                    if toolchain.exists() {
                        Ok(Some((toolchain, Some(reason))))
                    } else {
                        toolchain.install_from_dist()?;
                        Ok(Some((toolchain, Some(reason))))
                    }
                }
                Err(e) => {
                    let reason_err = match reason {
                        OverrideReason::Environment => {
                            format!(
                                "the {} environment variable specifies an uninstalled toolchain",
                                crate::ELAN_TOOLCHAIN
                            )
                        }
                        OverrideReason::OverrideDB(ref path) => {
                            format!(
                                "the directory override for '{}' specifies an uninstalled toolchain",
                                path.display()
                            )
                        }
                        OverrideReason::ToolchainFile(ref path) => {
                            format!(
                                "the toolchain file at '{}' specifies an uninstalled toolchain",
                                path.display()
                            )
                        }
                        OverrideReason::LeanpkgFile(ref path) => {
                            format!(
                                "the leanpkg.toml file at '{}' specifies an uninstalled toolchain",
                                path.display()
                            )
                        }
                        OverrideReason::InToolchainDirectory(ref path) => {
                            format!(
                                "could not parse toolchain directory at '{}'",
                                path.display()
                            )
                        }
                    };
                    Err(Error::OverrideToolchainNotInstalled {
                        toolchain,
                        reason_err,
                        source: Box::new(e),
                    })
                }
            }
        } else if let Some(tc) = self.resolve_default()? {
            Ok(Some((self.get_toolchain(&tc, false)?, None)))
        } else {
            Ok(None)
        }
    }

    pub fn find_override(&self) -> Result<Option<(UnresolvedToolchainDesc, OverrideReason)>> {
        // First check ELAN_TOOLCHAIN
        if let Some(ref name) = self.env_override {
            return Ok(Some((
                lookup_unresolved_toolchain_desc(self, name)?,
                OverrideReason::Environment,
            )));
        }

        // Then walk up the directory tree from the current directory, looking
        // for either the directory in override database, a `lean-toolchain`
        // file, or a `leanpkg.toml` file.
        if let Some(res) = self
            .settings_file
            .with(|s| self.find_override_from_dir_walk(s))?
        {
            return Ok(Some(res));
        }
        Ok(None)
    }

    pub fn get_toolchain(&self, name: &ToolchainDesc, create_parent: bool) -> Result<Toolchain> {
        if create_parent {
            utils::ensure_dir_exists("toolchains", &self.toolchains_dir, &|n| {
                (self.notify_handler)(n.into())
            })?;
        }

        Ok(Toolchain::from(self, name))
    }

    fn find_override_from_dir_walk(
        &self,
        settings: &Settings,
    ) -> Result<Option<(UnresolvedToolchainDesc, OverrideReason)>> {
        let notify = self.notify_handler.as_ref();
        let dir =
            Some(env::current_dir().map_err(|error| Error::CurrentDirectory { source: error })?);
        let mut dir = dir.as_deref();

        while let Some(d) = dir {
            // First check the override database
            if let Some(name) = settings.dir_override(d, notify) {
                let reason = OverrideReason::OverrideDB(d.to_owned());
                return Ok(Some((UnresolvedToolchainDesc(name), reason)));
            }

            // Then look for 'lean-toolchain'
            let toolchain_file = d.join("lean-toolchain");
            if let Ok(desc) = read_unresolved_toolchain_desc_from_file(self, &toolchain_file) {
                let reason = OverrideReason::ToolchainFile(toolchain_file);
                return Ok(Some((desc, reason)));
            }

            // Then look for 'leanpkg.toml'
            let leanpkg_file = d.join("leanpkg.toml");
            if let Ok(content) = utils::read_file("leanpkg.toml", &leanpkg_file) {
                let value =
                    content
                        .parse::<toml::Value>()
                        .map_err(|error| Error::InvalidLeanpkgFile {
                            path: leanpkg_file.clone(),
                            source: error,
                        })?;
                match value
                    .get("package")
                    .and_then(|package| package.get("lean_version"))
                {
                    None => {}
                    Some(toml::Value::String(s)) => {
                        let desc = lookup_unresolved_toolchain_desc(self, s)?;
                        return Ok(Some((desc, OverrideReason::LeanpkgFile(leanpkg_file))));
                    }
                    Some(a) => {
                        return Err(Error::InvalidLeanVersion {
                            path: leanpkg_file,
                            type_str: a.type_str(),
                        });
                    }
                }
            }

            dir = d.parent();

            if dir == Some(&self.toolchains_dir)
                && let Some(last) = d.file_name()
                && let Some(last) = last.to_str()
            {
                return Ok(Some((
                    UnresolvedToolchainDesc(ToolchainDesc::from_toolchain_dir(last)?),
                    OverrideReason::InToolchainDirectory(d.into()),
                )));
            }
        }

        Ok(None)
    }

    pub fn get_default(&self) -> Result<Option<String>> {
        self.settings_file.with(|s| Ok(s.default_toolchain.clone()))
    }

    pub fn resolve_default(&self) -> Result<Option<ToolchainDesc>> {
        if let Some(name) = self.get_default()? {
            let toolchain = lookup_toolchain_desc(self, &name)?;
            Ok(Some(toolchain))
        } else {
            Ok(None)
        }
    }

    pub fn list_toolchains(&self) -> Result<Vec<ToolchainDesc>> {
        if utils::is_directory(&self.toolchains_dir) {
            let mut toolchains: Vec<_> = utils::read_dir("toolchains", &self.toolchains_dir)?
                .filter_map(io::Result::ok)
                .filter(|e| e.file_type().map(|f| !f.is_file()).unwrap_or(false))
                .filter_map(|e| e.file_name().into_string().ok())
                .map(|n| ToolchainDesc::from_toolchain_dir(&n).map_err(|e| e.into()))
                .collect::<Result<Vec<ToolchainDesc>>>()?
                .into_iter()
                .map(|tc| tc.to_string())
                .collect();

            utils::toolchain_sort(&mut toolchains);

            let toolchains: Vec<_> = toolchains
                .iter()
                .flat_map(|s| ToolchainDesc::from_resolved_str(s))
                .collect();
            Ok(toolchains)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn from_env(notify_handler: Arc<dyn Fn(Notification<'_>)>) -> Result<Self> {
        let elan_dir = utils::elan_home()?;

        utils::ensure_dir_exists("home", &elan_dir, &|n| notify_handler(n.into()))?;

        let settings_file = SettingsFile::new(elan_dir.join("settings.toml"));

        let toolchains_dir = elan_dir.join("toolchains");

        // Environment override
        let env_override = env::var(crate::ELAN_TOOLCHAIN)
            .ok()
            .and_then(utils::if_not_empty);

        Ok(Cfg {
            elan_dir,
            settings_file,
            toolchains_dir,
            notify_handler,
            env_override,
        })
    }
}
