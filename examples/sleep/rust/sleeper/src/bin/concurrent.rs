use anyhow::Context;

use concurrent_sys::Concurrent_c::{concurrent_main, initialize_concurrent_Concurrent};
use lean::{MimallocAllocator, args, run_lean_io_unit};
use lean_sys::{
    lean_finalize_task_manager, lean_init_task_manager, lean_initialize_runtime_module,
    lean_io_mark_end_initialization,
};

#[global_allocator]
static ALLOCATOR: MimallocAllocator = MimallocAllocator {};

fn main() -> anyhow::Result<()> {
    unsafe {
        // Lean initialization
        // -------------------
        args::call_lean_setup_args()?;
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
    let result =
        unsafe { run_lean_io_unit(|| concurrent_main()).context("Lean main function failed") };

    // Lean cleanup
    // --------------------
    unsafe {
        lean_finalize_task_manager();
    }

    result
}
