use std::convert::Infallible;
use std::ffi::{CStr, CString};
use std::str::FromStr;

use lean::{
    MimallocAllocator, MinimalComponents, NoModules,
    lean_types::{Owner, string::LeanString},
};
use lean_sys::lean_string_push;

#[global_allocator]
static ALLOCATOR: MimallocAllocator = MimallocAllocator {};

#[test]
fn make_string() {
    lean::run_in_lean_runtime_with_default_error_handler::<MinimalComponents, NoModules, _, _, _>(
        |runtime| {
            let initial_string = String::from("Hello, world");
            let new_char = b'!';
            let initial_cstring = CString::from_str(&initial_string).unwrap();

            let lean_string = LeanString::from_cstr(runtime, initial_cstring);
            let longer_lean_string = unsafe {
                LeanString::new(lean_string_push(lean_string.into_raw(), new_char as u32))
            };

            let final_cstring: &CStr = longer_lean_string.as_cstr();

            let final_string = final_cstring.to_str().unwrap();
            let mut expected_string = initial_string.clone();
            expected_string.push(new_char as char);
            assert_eq!(final_string, expected_string);
            Ok::<_, Infallible>(())
        },
    )
    .unwrap();
}
