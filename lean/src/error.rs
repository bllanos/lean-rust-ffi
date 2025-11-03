use std::error::Error;
use std::ffi::CString;
use std::fmt;

use lean_sys::{b_lean_obj_arg, lean_inc, lean_io_error_to_string, lean_io_result_get_error};

use crate::lean_types::{Owner, string::LeanString};

#[derive(Debug, Eq, PartialEq)]
pub struct LeanIoError(pub CString);

impl fmt::Display for LeanIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_string_lossy())
    }
}

impl Error for LeanIoError {}

impl LeanIoError {
    /// Create an instance from a Lean IO error
    ///
    /// # Safety
    ///
    /// Callers must ensure that `lean_io_error` points to a valid error object
    pub unsafe fn from_lean_io_error(lean_io_error: b_lean_obj_arg) -> Self {
        let lean_cstring;
        unsafe {
            lean_inc(lean_io_error);
            let lean_string = lean_io_error_to_string(lean_io_error);
            lean_cstring = LeanString::new(lean_string);
        }
        let owned_string = lean_cstring.as_cstr().to_owned();
        Self(owned_string)
    }

    /// Create an instance from a Lean IO error contained in a Lean IO result
    ///
    /// # Safety
    ///
    /// Callers must ensure that `lean_io_result` points to a valid result
    /// object and that it is an error variant
    pub unsafe fn from_lean_io_result(lean_io_result: b_lean_obj_arg) -> Self {
        unsafe {
            let lean_io_error = lean_io_result_get_error(lean_io_result);
            Self::from_lean_io_error(lean_io_error)
        }
    }
}

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
pub enum LeanError<ModulesInitializationError: Error, RunError: Error> {
    #[error("Lean modules initialization error")]
    ModulesInitialization(#[source] ModulesInitializationError),
    #[error(transparent)]
    Run(#[from] RunError),
}
