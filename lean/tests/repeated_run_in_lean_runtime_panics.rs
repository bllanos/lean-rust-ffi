#![forbid(unsafe_code)]

use std::convert::Infallible;

use lean::{MimallocAllocator, MinimalComponents, NoModules, Runtime};

#[global_allocator]
static ALLOCATOR: MimallocAllocator = MimallocAllocator {};

#[test]
#[should_panic(
    expected = "attempt to reuse the Lean runtime. The runtime is single-use to eliminate overhead from repeatedly checking whether it has already been initialized"
)]
fn repeated_run_in_lean_runtime_panics() {
    lean::run_in_lean_runtime_with_default_error_handler(
        |_runtime: &Runtime<MinimalComponents, NoModules>| -> Result<(), Infallible> { Ok(()) },
    )
    .unwrap();

    lean::run_in_lean_runtime_with_default_error_handler(
        |_runtime: &Runtime<MinimalComponents, NoModules>| -> Result<(), Infallible> {
            unreachable!()
        },
    )
    .unwrap();
}
