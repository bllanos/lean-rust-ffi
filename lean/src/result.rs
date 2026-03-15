use std::ffi::CString;
use std::str::FromStr;

use lean_sys::{
    lean_dec, lean_io_result_is_ok, lean_io_result_mk_error, lean_io_result_mk_ok,
    lean_mk_io_user_error, lean_mk_string, lean_obj_res,
};

use crate::{LeanIoError, lean_types::unit};

pub type LeanIoResult<T> = Result<T, LeanIoError>;

/// Run a Lean `IO Unit` and convert its result to a [`LeanIoResult`]
///
/// # Safety
///
/// `io_unit` must return a valid, owned Lean IO result object.
pub unsafe fn run_lean_io_unit<F: FnOnce() -> lean_obj_res>(io_unit: F) -> LeanIoResult<()> {
    let lean_result = io_unit();
    let result = if unsafe { lean_io_result_is_ok(lean_result) } {
        Ok(())
    } else {
        let error = unsafe { LeanIoError::from_lean_io_result(lean_result) };
        Err(error)
    };
    unsafe { lean_dec(lean_result) };
    result
}

pub fn make_lean_io_result_ok_unit() -> lean_obj_res {
    unsafe { lean_io_result_mk_ok(unit::make_lean_unit()) }
}

/// Create a Lean IO result object from a string
///
/// # Panics
///
/// Panics if the error is not convertible to [`CString`].
pub fn make_lean_io_result_error(error: &str) -> lean_obj_res {
    let cstring = CString::from_str(error).unwrap();
    let cstring_ptr = cstring.as_ptr();

    unsafe {
        let lean_string = lean_mk_string(cstring_ptr);
        let lean_io_error = lean_mk_io_user_error(lean_string);
        lean_io_result_mk_error(lean_io_error)
    }
}
