use std::env;
use std::ffi::{c_char, c_int};

use anyhow::Context;

// Force linking of this crate even though there is no direct reference to it
// from Rust code
extern crate sleeper_lean;

use concurrent_sys::Concurrent_c::{concurrent_main, initialize_concurrent_Concurrent};
use lean::{MimallocAllocator, run_lean_io_unit};
use lean_sys::{
    lean_finalize_task_manager, lean_init_task_manager, lean_initialize_runtime_module,
    lean_io_mark_end_initialization, lean_setup_args,
};

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
    // Use same default as for Lean executables
    // See https://github.com/leanprover/lean4/blob/master/doc/dev/ffi.md#initialization
    let builtin: u8 = 1;

    unsafe {
        run_lean_io_unit(|| initialize_concurrent_Concurrent(builtin))
            .context("Lean module initialization failed")?;
        lean_io_mark_end_initialization();
        lean_init_task_manager();
    }

    // Program logic
    // -------------
    let result;
    unsafe {
        result = run_lean_io_unit(|| concurrent_main()).context("Lean main function failed");
    }

    // Lean cleanup
    // --------------------
    unsafe {
        lean_finalize_task_manager();
    }

    result
}
