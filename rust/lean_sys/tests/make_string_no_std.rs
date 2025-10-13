#![no_std]

use core::ffi::CStr;

use lean_sys::{
    lean_dec_ref, lean_initialize_runtime_module, lean_io_mark_end_initialization, lean_mk_string,
    lean_obj_res, lean_string_cstr, lean_string_push,
};

/// This test is a version of [`make_string_std()`](./make_string_std.rs) that
/// does not rely on dynamic memory allocation in Rust.
#[test]
fn make_string_no_std() {
    unsafe {
        lean_initialize_runtime_module();
        lean_io_mark_end_initialization();
    }

    let new_char = b'!';
    let initial_cstring = b"Hello, world\0";
    let initial_string_ptr = initial_cstring.as_ptr();

    let longer_lean_string: lean_obj_res;
    let final_cstring: &CStr;

    unsafe {
        let lean_string = lean_mk_string(initial_string_ptr as *const i8);
        longer_lean_string = lean_string_push(lean_string, new_char as u32);
        let longer_lean_cstring = lean_string_cstr(longer_lean_string);
        final_cstring = CStr::from_ptr(longer_lean_cstring);
    }

    let final_string = final_cstring.to_str().unwrap();
    let expected_string = "Hello, world!";
    assert_eq!(final_string, expected_string);

    unsafe {
        lean_dec_ref(longer_lean_string);
    }
}
