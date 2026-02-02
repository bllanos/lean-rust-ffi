use std::env;
use std::ffi::{c_char, c_int};

// Force linking of this crate even though there is no direct reference to it
// from Rust code
extern crate sleeper_lean;

use lean::MimallocAllocator;
use lean_sys::{
    lean_dec, lean_finalize_task_manager, lean_init_task_manager, lean_initialize_runtime_module,
    lean_io_mark_end_initialization, lean_io_result_is_ok, lean_io_result_show_error, lean_object,
    lean_setup_args,
};
use sequential_sys::Sequential_c::{initialize_sequential_Sequential, sequential_main};

#[global_allocator]
static ALLOCATOR: MimallocAllocator = MimallocAllocator {};

fn main() -> anyhow::Result<()> {
    unsafe {
        // Lean initialization
        // -------------------
        let argv_iter = env::args_os();
        let argc: c_int = argv_iter.len().try_into()?;
        let argv_buffer = argv_iter
            .map(|arg| {
                let bytes = arg.as_encoded_bytes();
                let mut buffer = Vec::with_capacity(bytes.len() + 1);
                buffer.extend_from_slice(bytes);
                buffer.push(0);
                buffer.leak().as_ptr() as *const c_char
            })
            .collect::<Vec<*const c_char>>();
        let argv = argv_buffer.leak().as_ptr();
        // libuv may take ownership of the pointer
        // Reference: <https://docs.libuv.org/en/v1.x/misc.html#c.uv_setup_args>
        lean_setup_args(argc, argv);
        lean_initialize_runtime_module();
    }

    // Lean module initialization
    // --------------------------
    let res: *mut lean_object;
    // Use same default as for Lean executables
    // See https://github.com/leanprover/lean4/blob/master/doc/dev/ffi.md#initialization
    let builtin: u8 = 1;

    unsafe {
        res = initialize_sequential_Sequential(builtin);
        if lean_io_result_is_ok(res) {
            lean_dec(res);
        } else {
            lean_io_result_show_error(res);
            lean_dec(res);
            // do not access Lean declarations if initialization failed
            anyhow::bail!("Lean module initialization failed");
        }
        lean_io_mark_end_initialization();
        lean_init_task_manager();
    }

    // Program logic
    // -------------
    unsafe {
        let res = sequential_main();
        if lean_io_result_is_ok(res) {
            lean_dec(res);
        } else {
            lean_io_result_show_error(res);
            lean_dec(res);
            anyhow::bail!("Lean main function failed");
        }
    }

    // Lean cleanup
    // --------------------
    unsafe {
        lean_finalize_task_manager();
    }

    Ok(())
}
