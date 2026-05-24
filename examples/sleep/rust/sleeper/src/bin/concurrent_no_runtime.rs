use anyhow::Context;

use concurrent_sys::Concurrent_c::{concurrent_main, initialize_concurrent_Concurrent};
use lean::{MimallocAllocator, run_lean_io_unit};
use lean_sys::lean_io_mark_end_initialization;

#[global_allocator]
static ALLOCATOR: MimallocAllocator = MimallocAllocator {};

fn main() -> anyhow::Result<()> {
    // Lean module initialization
    // --------------------------
    // Use same default as for Lean executables
    // See https://lean-lang.org/doc/reference/latest/Run-Time-Code/Foreign-Function-Interface/#ffi-initialization
    let builtin: u8 = 1;

    unsafe {
        run_lean_io_unit(|| initialize_concurrent_Concurrent(builtin))
            .context("Lean module initialization failed")?;
        lean_io_mark_end_initialization();
    }

    // Program logic
    // -------------
    unsafe { run_lean_io_unit(|| concurrent_main()).context("Lean main function failed") }
}
