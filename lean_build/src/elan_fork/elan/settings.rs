use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::super::elan_dist::dist::ToolchainDesc;
use super::super::elan_utils::{toml_utils::*, utils};
use super::errors::*;
use super::notifications::*;

pub const SUPPORTED_METADATA_VERSIONS: [&str; 2] = ["2", "12"];
pub const DEFAULT_METADATA_VERSION: &str = "12";

#[derive(Clone, Debug, PartialEq)]
pub struct SettingsFile {
    path: PathBuf,
    cache: RefCell<Option<Settings>>,
}

impl SettingsFile {
    pub fn new(path: PathBuf) -> Self {
        SettingsFile {
            path,
            cache: RefCell::new(None),
        }
    }

    fn read_settings(&self) -> Result<()> {
        let mut b = self.cache.borrow_mut();
        if b.is_none() {
            *b = Some(if utils::is_file(&self.path) {
                let content = utils::read_file("settings", &self.path)?;
                Settings::parse(&content)?
            } else {
                Default::default()
            });
        }
        Ok(())
    }

    pub fn with<T, F: FnOnce(&Settings) -> Result<T>>(&self, f: F) -> Result<T> {
        self.read_settings()?;

        // Settings can no longer be None so it's OK to unwrap
        f(self.cache.borrow().as_ref().unwrap())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub version: String,
    pub default_toolchain: Option<String>,
    pub overrides: BTreeMap<String, ToolchainDesc>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            version: DEFAULT_METADATA_VERSION.to_owned(),
            default_toolchain: None,
            overrides: BTreeMap::new(),
        }
    }
}

impl Settings {
    pub fn parse(data: &str) -> Result<Self> {
        let value = toml::from_str(data).map_err(Error::ParsingSettings)?;
        Self::from_toml(value, "")
    }

    fn path_to_key(path: &Path, notify_handler: &dyn Fn(Notification<'_>)) -> String {
        if path.exists() {
            utils::canonicalize_path(path, &|n| notify_handler(n.into()))
                .display()
                .to_string()
        } else {
            path.display().to_string()
        }
    }

    pub fn dir_override(
        &self,
        dir: &Path,
        notify_handler: &dyn Fn(Notification<'_>),
    ) -> Option<ToolchainDesc> {
        let key = Self::path_to_key(dir, notify_handler);
        self.overrides.get(&key).cloned()
    }

    pub fn from_toml(mut table: toml::value::Table, path: &str) -> Result<Self> {
        let version = get_string(&mut table, "version", path)?;
        if !SUPPORTED_METADATA_VERSIONS.contains(&&*version) {
            return Err(Error::UnknownMetadataVersion(version));
        }
        Ok(Settings {
            version,
            default_toolchain: get_opt_string(&mut table, "default_toolchain", path)?,
            overrides: Self::table_to_overrides(&mut table, path)?,
        })
    }

    fn table_to_overrides(
        table: &mut toml::value::Table,
        path: &str,
    ) -> Result<BTreeMap<String, ToolchainDesc>> {
        let mut result = BTreeMap::new();
        let pkg_table = get_table(table, "overrides", path)?;

        for (k, v) in pkg_table {
            if let toml::Value::String(t) = v {
                result.insert(k, ToolchainDesc::from_resolved_str(&t)?);
            }
        }

        Ok(result)
    }
}
