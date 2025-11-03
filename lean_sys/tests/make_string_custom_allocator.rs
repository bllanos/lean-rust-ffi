use std::ffi::{CStr, CString};
use std::str::FromStr;

use lean::MimallocAllocator;
use lean_sys::{
    lean_dec_ref, lean_initialize_runtime_module, lean_io_mark_end_initialization, lean_mk_string,
    lean_obj_res, lean_string_cstr, lean_string_push,
};

#[global_allocator]
static ALLOCATOR: MimallocAllocator = MimallocAllocator {};

/// This test is a version of [`make_string_std()`](./make_string_std.rs) that
/// uses Lean's allocator (assumed to be mimalloc) for dynamic memory allocation
/// in Rust.
#[test]
fn make_string_custom_allocator() {
    unsafe {
        lean_initialize_runtime_module();
        lean_io_mark_end_initialization();
    }

    let initial_string = String::from("Hello, world");
    let new_char = b'!';
    let initial_cstring = CString::from_str(&initial_string).unwrap();
    let initial_string_ptr = initial_cstring.as_ptr();

    let longer_lean_string: lean_obj_res;
    let final_cstring: &CStr;

    unsafe {
        let lean_string = lean_mk_string(initial_string_ptr);
        longer_lean_string = lean_string_push(lean_string, new_char as u32);
        let longer_lean_cstring = lean_string_cstr(longer_lean_string);
        final_cstring = CStr::from_ptr(longer_lean_cstring);
    }

    let final_string = final_cstring.to_str().unwrap();
    let mut expected_string = initial_string.clone();
    expected_string.push(new_char as char);
    assert_eq!(final_string, expected_string);

    unsafe {
        lean_dec_ref(longer_lean_string);
    }
}
