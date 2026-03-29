use async_effect::io;
use lean::lean_types::{Borrower, external::ExternalClassHolder, string::LeanStr};
use lean_sys::{b_lean_obj_arg, lean_obj_res};

use crate::time::LeanInstant;

#[unsafe(no_mangle)]
pub extern "C" fn async_effect_ffi_monotonic_now_immediate() -> lean_obj_res {
    let now = io::monotonic_now_immediate();
    LeanInstant(now).into_lean_object()
}

/// Prints a string with a trailing newline to standard output
///
/// # Safety
///
/// Callers must ensure that `s` points to a borrowed Lean string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn async_effect_ffi_println_immediate(s: b_lean_obj_arg) -> lean_obj_res {
    let lean_str = unsafe { LeanStr::new(s) };
    match lean_str.as_cstr().to_str() {
        Ok(rust_str) => {
            io::println_immediate(rust_str);
            lean::make_lean_io_result_ok_unit()
        }
        Err(_) => lean::make_lean_io_result_error(&format!(
            "attempt to print invalid UTF-8 string with lossy value \"{}\"",
            lean_str.as_cstr().to_string_lossy()
        )),
    }
}
