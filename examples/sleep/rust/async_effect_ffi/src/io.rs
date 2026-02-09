use async_effect::io;
use lean::lean_types::{Borrower, string::LeanStr};
use lean_sys::{b_lean_obj_arg, lean_obj_res};

#[unsafe(no_mangle)]
pub extern "C" fn async_effect_ffi_monotonic_now_immediate() -> lean_obj_res {
    let now = io::monotonic_now_immediate();
    // TODO: Return an opaque Instant type and remove sleep
    std::thread::sleep(std::time::Duration::from_millis(1002));
    let count = now.elapsed().as_millis() as u64;
    unsafe { lean_sys::lean_uint64_to_nat(count) }
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
