use lean_sys::{lean_dec, lean_io_result_is_ok, lean_obj_res};

use crate::LeanIoError;

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
