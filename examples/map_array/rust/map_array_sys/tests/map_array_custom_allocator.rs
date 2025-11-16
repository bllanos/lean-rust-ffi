use std::ffi::CStr;
use std::slice;

use lean::MimallocAllocator;
use lean_sys::{
    lean_alloc_array, lean_array_cptr, lean_array_size, lean_box_uint32, lean_dec, lean_dec_ref,
    lean_inc, lean_initialize_runtime_module, lean_io_mark_end_initialization, lean_io_mk_world,
    lean_io_result_is_ok, lean_io_result_show_error, lean_object, lean_string_cstr,
    lean_unbox_uint32,
};
use map_array_sys::{
    MapArray::Basic_c::{map_options_to_string, mk_map_options, my_map},
    MapArray_c::initialize_MapArray,
};

#[global_allocator]
static ALLOCATOR: MimallocAllocator = MimallocAllocator {};

/// A test that uses Lean's allocator (assumed to be mimalloc) for dynamic
/// memory allocation in Rust and exercises functions from the Lean `MapArray`
/// module.
#[test]
fn map_array_custom_allocator() -> anyhow::Result<()> {
    unsafe {
        // Lean initialization
        // -------------------
        lean_initialize_runtime_module();
    }

    // Lean module initialization
    // --------------------------
    let res: *mut lean_object;
    // Use same default as for Lean executables
    // See https://github.com/leanprover/lean4/blob/master/doc/dev/ffi.md#initialization
    let builtin: u8 = 1;

    unsafe {
        res = initialize_MapArray(builtin, lean_io_mk_world());
        if lean_io_result_is_ok(res) {
            lean_dec(res);
        } else {
            lean_io_result_show_error(res);
            lean_dec(res);
            // do not access Lean declarations if initialization failed
            anyhow::bail!("Lean module initialization failed");
        }
        lean_io_mark_end_initialization();
    }

    // Program logic
    // -------------
    unsafe {
        let addend: i32 = 2;
        let multiplicand: i32 = 3;

        let map_options: *mut lean_object = mk_map_options(addend as u32, multiplicand as u32);
        // Avoid having `map_options_to_string()` destroy `map_options`
        lean_inc(map_options);
        let map_options_lean_str: *mut lean_object = map_options_to_string(map_options);
        let map_options_cstr = lean_string_cstr(map_options_lean_str);
        let map_options_str = CStr::from_ptr(map_options_cstr);

        assert_eq!(
            map_options_str.to_str()?,
            "{ addend := 2, multiplicand := 3 }"
        );

        // This seems to be an alternative to `lean_dec()` that can be used when
        // the value is known not to be a scalar.
        lean_dec_ref(map_options_lean_str);

        let arr_size: usize = 6;

        let arr: *mut lean_object = lean_alloc_array(arr_size, arr_size);
        let arr_data: *mut *mut lean_object = lean_array_cptr(arr);

        for i in 0..arr_size {
            let value: u8 = (i * 5).try_into()?;
            // There are no functions for boxing `uint8_t` values specifically, so use
            // `lean_box_uint32()`
            *(arr_data.add(i)) = lean_box_uint32(value.into());
        }

        // Note: `my_map()` will call `lean_dec()` on all arguments.
        let arr_out: *mut lean_object = my_map(map_options, arr);
        let arr_data = lean_array_cptr(arr_out);
        let arr_size_from_lean = lean_array_size(arr_out);

        assert_eq!(arr_size_from_lean, arr_size);

        let lean_array_slice = slice::from_raw_parts(arr_data, arr_size_from_lean);

        for (object, i) in lean_array_slice
            .iter()
            .zip([6_i32, 21, 36, 51, 66, 81].iter())
        {
            let value: i32 = lean_unbox_uint32(*object).try_into()?;
            assert_eq!(value, *i);
        }

        lean_dec_ref(arr_out);
    }

    Ok(())
}
