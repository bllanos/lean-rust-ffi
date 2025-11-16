use std::ffi::OsStr;
use std::path::PathBuf;

use super::{LakeCommandError, LakeEnvironmentDescriber};
use crate::{NotUnicodeBytes, NotUnicodeString, display_slice};

#[derive(thiserror::Error, Debug)]
#[error("not present")]
pub struct NotPresent;

#[derive(thiserror::Error, Debug)]
pub enum EnvironmentVariableValueError {
    #[error(transparent)]
    NotPresent(#[from] NotPresent),
    #[error(transparent)]
    NotUnicode(#[from] NotUnicodeString),
}

#[derive(thiserror::Error, Debug)]
pub enum LakeEnvEnvironmentVariableValueError {
    #[error(transparent)]
    NotPresent(#[from] NotPresent),
    #[error(transparent)]
    NotUnicodeBytes(NotUnicodeBytes),
    #[error("environment line \"{}\" has the same name as a previous line", display_slice(.line))]
    DuplicateName { line: Vec<u8> },
}

#[derive(thiserror::Error, Debug)]
#[error("error obtaining environment variable \"{name}\" from `lake env` command output")]
pub struct LakeEnvEnvironmentVariableError {
    pub name: String,
    pub source: LakeEnvEnvironmentVariableValueError,
}

#[derive(thiserror::Error, Debug)]
#[error("error obtaining environment variable \"{name}\" from process environment variables")]
pub struct EnvironmentVariableError {
    pub name: String,
    pub source: EnvironmentVariableValueError,
}

#[derive(thiserror::Error, Debug)]
pub enum LakeEnvEnvironmentError {
    #[error("no variable name in environment line \"{}\"", display_slice(.line))]
    MissingName { line: Vec<u8> },
    #[error("variable name is not valid UTF-8")]
    NotUnicodeName { name: Vec<u8> },
    #[error(transparent)]
    VariableError(#[from] LakeEnvEnvironmentVariableError),
}

#[derive(thiserror::Error, Debug)]
pub enum LakeEnvError {
    #[error(transparent)]
    Command(#[from] LakeCommandError),
    #[error(transparent)]
    Environment(#[from] LakeEnvEnvironmentError),
}

#[derive(thiserror::Error, Debug)]
pub enum EnvironmentError {
    #[error(transparent)]
    LakeEnv(#[from] LakeEnvError),
    #[error(transparent)]
    ProcessEnvironment(#[from] EnvironmentVariableError),
}

pub struct LakeEnv {
    elan_toolchain: String,
    lean_githash: String,
    lean_sysroot: PathBuf,
}

impl LakeEnv {
    const ELAN_TOOLCHAIN: &str = crate::ELAN_TOOLCHAIN;
    const LEAN_GITHASH: &str = "LEAN_GITHASH";
    const LEAN_SYSROOT: &str = "LEAN_SYSROOT";

    fn from_posix_env(env: &[u8]) -> Result<Self, LakeEnvEnvironmentError> {
        let tuple = env.split(|c| c.is_ascii_control()).try_fold(
            (String::new(), String::new(), PathBuf::new()),
            |accumulator, slice| -> Result<(String, String, PathBuf), LakeEnvEnvironmentError> {
                let mut parts = slice.splitn(2, |c| *c == b'=');
                let var = parts
                    .next()
                    .ok_or_else(|| LakeEnvEnvironmentError::MissingName { line: slice.into() })?;
                match parts.next() {
                    None => Ok(accumulator),
                    Some(value) => {
                        let var_str = str::from_utf8(var).map_err(|_| {
                            LakeEnvEnvironmentError::NotUnicodeName { name: var.into() }
                        })?;
                        (match var_str {
                            Self::ELAN_TOOLCHAIN => str::from_utf8(value)
                                .map_err(|_| {
                                    LakeEnvEnvironmentVariableValueError::NotUnicodeBytes(
                                        NotUnicodeBytes(value.into()),
                                    )
                                })
                                .and_then(|value_str| {
                                    if accumulator.0.is_empty() {
                                        Ok((String::from(value_str), accumulator.1, accumulator.2))
                                    } else {
                                        Err(LakeEnvEnvironmentVariableValueError::DuplicateName {
                                            line: slice.into(),
                                        })
                                    }
                                }),
                            Self::LEAN_GITHASH => str::from_utf8(value)
                                .map_err(|_| {
                                    LakeEnvEnvironmentVariableValueError::NotUnicodeBytes(
                                        NotUnicodeBytes(value.into()),
                                    )
                                })
                                .and_then(|value_str| {
                                    if accumulator.1.is_empty() {
                                        Ok((accumulator.0, String::from(value_str), accumulator.2))
                                    } else {
                                        Err(LakeEnvEnvironmentVariableValueError::DuplicateName {
                                            line: slice.into(),
                                        })
                                    }
                                }),
                            Self::LEAN_SYSROOT => str::from_utf8(value)
                                .map_err(|_| {
                                    LakeEnvEnvironmentVariableValueError::NotUnicodeBytes(
                                        NotUnicodeBytes(value.into()),
                                    )
                                })
                                .and_then(|value_str| {
                                    if accumulator.2.as_os_str().is_empty() {
                                        Ok((accumulator.0, accumulator.1, PathBuf::from(value_str)))
                                    } else {
                                        Err(LakeEnvEnvironmentVariableValueError::DuplicateName {
                                            line: slice.into(),
                                        })
                                    }
                                }),
                            _ => Ok(accumulator),
                        })
                        .map_err(|error| {
                            LakeEnvEnvironmentVariableError {
                                name: var_str.into(),
                                source: error,
                            }
                            .into()
                        })
                    }
                }
            },
        )?;
        if tuple.0.is_empty() {
            Err(LakeEnvEnvironmentVariableError {
                name: Self::ELAN_TOOLCHAIN.into(),
                source: NotPresent.into(),
            }
            .into())
        } else if tuple.1.is_empty() {
            Err(LakeEnvEnvironmentVariableError {
                name: Self::LEAN_GITHASH.into(),
                source: NotPresent.into(),
            }
            .into())
        } else if tuple.2.as_os_str().is_empty() {
            Err(LakeEnvEnvironmentVariableError {
                name: Self::LEAN_SYSROOT.into(),
                source: NotPresent.into(),
            }
            .into())
        } else {
            Ok(Self {
                elan_toolchain: tuple.0,
                lean_githash: tuple.1,
                lean_sysroot: tuple.2,
            })
        }
    }

    pub fn lean_sysroot_library_directory(&self) -> PathBuf {
        self.lean_sysroot.join("lib")
    }

    pub fn lean_library_directory(&self) -> PathBuf {
        self.lean_sysroot_library_directory().join("lean")
    }

    pub fn lean_include_directory(&self) -> PathBuf {
        self.lean_sysroot.join("include")
    }

    pub fn lean_clang_include_directory(&self) -> PathBuf {
        self.lean_include_directory().join("clang")
    }

    pub fn lean_lean_include_directory(&self) -> PathBuf {
        self.lean_include_directory().join("lean")
    }

    pub fn lean_header_path(&self) -> PathBuf {
        self.lean_lean_include_directory().join("lean.h")
    }

    fn lean_bin_path(&self) -> PathBuf {
        self.lean_sysroot.join("bin")
    }

    pub fn lean_clang_path(&self) -> PathBuf {
        self.lean_bin_path().join("clang")
    }

    pub fn lean_ar_path(&self) -> PathBuf {
        self.lean_bin_path().join("llvm-ar")
    }

    pub fn export_rustc_env(&self) {
        println!(
            "cargo::rustc-env={}={}",
            Self::ELAN_TOOLCHAIN,
            self.elan_toolchain
        );
        println!(
            "cargo::rustc-env={}={}",
            Self::LEAN_GITHASH,
            self.lean_githash
        );
    }
}

/// Invokes Lake to discover Lake environment variables
///
/// If the Lake executable is actually Elan's
/// [proxy](https://rust-lang.github.io/rustup/concepts/index.html#how-rustup-works)
/// ([Elan is a fork of
/// `rustup`](https://github.com/leanprover/elan/blob/2a16e9666f50e5d7f6d71e8dcfa1a5aa345dfd61/README.md?plain=1#L66)),
/// then Elan will install the Lean toolchain if it is missing.
pub fn get_lake_environment<T: LakeEnvironmentDescriber>(
    lake_environment_describer: T,
) -> Result<LakeEnv, EnvironmentError> {
    let lake_executable_path = lake_environment_describer.get_lake_executable_path();
    let args = [OsStr::new("env")];
    let stdout = super::run_lake_command_and_retrieve_stdout(lake_executable_path, &args)
        .map_err(|error| EnvironmentError::LakeEnv(error.into()))?;
    let lake_env = LakeEnv::from_posix_env(&stdout)
        .map_err(|error| EnvironmentError::LakeEnv(error.into()))?;
    Ok(lake_env)
}
