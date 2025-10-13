use std::ffi::{CStr, CString};
use std::str::FromStr;

use lean_sys::{
    lean_dec_ref, lean_initialize_runtime_module, lean_io_mark_end_initialization, lean_mk_string,
    lean_obj_res, lean_string_cstr, lean_string_push,
};

/// This test demonstrates that both Rust and Lean can use dynamic memory
/// allocation with their respective default allocators in the same program. In
/// practice, it is probably better to use `#![no_std]` and only have one
/// allocator in the program, by exclusively using Lean objects, allocated using
/// Lean's allocation functions for specific types of Lean objects. This
/// approach is used in [`make_string_no_std()`](./make_string_no_std.rs).
///
/// Rather than use Lean's allocation functions for specific types of Lean
/// objects, one could program using Rust standard library objects but use
/// Lean's allocator to allocate them. Using Lean's allocator with Rust's
/// standard library objects is complicated because Rust code needs to be aware
/// of Lean's build-time memory allocation configuration, specifically the
/// `LEAN_MIMALLOC` and `LEAN_SMALL_ALLOCATOR` options. An example of using
/// Lean's allocator as Rust's global allocator in this way is shown at
/// <https://github.com/digama0/lean-sys/blob/dd7ff0cfa4a70ad8d1aecc7f8cb6ced776664c11/src/alloc.rs>.
///
/// We can probably assume that Lean is going to use mimalloc in preference to
/// its special-purpose small object allocator for the foreseeable future, based
/// on the discussion in the [mimalloc technical
/// report](https://www.microsoft.com/en-us/research/publication/mimalloc-free-list-sharding-in-action),
/// which says that mimalloc, "outperform[s] our own custom allocators for small
/// objects in Lean". We demonstrate using a simple Rust global allocator that
/// uses mimalloc in
/// [`make_string_no_custom_allocator.rs`](./make_string_no_custom_allocator.rs).
#[test]
fn make_string_std() {
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
