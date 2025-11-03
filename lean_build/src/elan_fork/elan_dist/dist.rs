use std::fmt;

use regex::Regex;

use super::errors::*;

// Fully-resolved toolchain descriptors. These are used for canonical
// identification, such as naming their installation directory.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolchainDesc {
    // A linked toolchain
    Local {
        name: String,
    },
    Remote {
        // The GitHub source repository to use (if "nightly" is specified, we append "-nightly" to this).
        origin: String,
        // The release name, usually a Git tag
        release: String,
        // The channel name the release was resolved from, if any
        from_channel: Option<String>,
    },
}

impl ToolchainDesc {
    pub fn from_resolved_str(name: &str) -> Result<Self> {
        let pattern = r"^(?:([a-zA-Z0-9-]+[/][a-zA-Z0-9-]+)[:])?([a-zA-Z0-9-.]+)$";

        let re = Regex::new(pattern).unwrap();
        if let Some(c) = re.captures(name) {
            match c.get(1) {
                Some(origin) => {
                    let origin = origin.as_str().to_owned();
                    let release = c.get(2).unwrap().as_str().to_owned();
                    Ok(ToolchainDesc::Remote {
                        origin,
                        release,
                        from_channel: None,
                    })
                }
                None => {
                    let name = c.get(2).unwrap().as_str().to_owned();
                    Ok(ToolchainDesc::Local { name })
                }
            }
        } else {
            Err(Error::InvalidToolchainName(name.to_string()))
        }
    }

    pub fn from_toolchain_dir(dir_name: &str) -> Result<Self> {
        // de-sanitize toolchain file names (best effort...)
        let name = dir_name.replace("---", ":").replace("--", "/");
        Self::from_resolved_str(&name)
    }
}

impl fmt::Display for ToolchainDesc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolchainDesc::Local { name } => write!(f, "{name}"),
            ToolchainDesc::Remote {
                origin, release, ..
            } => write!(f, "{origin}:{release}"),
        }
    }
}
