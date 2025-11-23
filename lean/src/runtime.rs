use std::error::Error;
use std::sync::Once;

use lean_sys::b_lean_obj_arg;

use crate::{LeanError, LeanIoError, Modules, RuntimeComponents};

mod components;
mod runtime_impl;

pub use components::{
    ArgcError, LeanPackage, LeanPackageComponents, Minimal, MinimalComponents,
    RuntimeInitializationError,
};
pub use runtime_impl::RuntimeImpl;

static ONCE_INITIALIZATION_GUARD: Once = Once::new();

/// Initializes sets of Lean runtime components and modules and passes the
/// runtime to a function that depends on Lean functionality
///
/// # Safety
///
/// Callers must either avoid initializing the Lean runtime multiple times, or
/// must use runtime components that are safe to initialize multiple times.
pub unsafe fn run_in_lean_runtime_unchecked<
    C: RuntimeComponents,
    M: Modules,
    T,
    ModulesInitializationError: Error,
    ModulesInitializationErrorHandler: FnOnce(b_lean_obj_arg) -> ModulesInitializationError,
    RunError: Error,
    Run: FnOnce(&RuntimeImpl<C, M>) -> Result<T, RunError>,
>(
    modules_initialization_error_handler: ModulesInitializationErrorHandler,
    run: Run,
) -> Result<
    T,
    LeanError<<C as RuntimeComponents>::InitializationError, ModulesInitializationError, RunError>,
> {
    let runtime = { unsafe { RuntimeImpl::new_main_thread(modules_initialization_error_handler) } }
        .map_err(LeanError::Initialization)?;
    let value = run(&runtime)?;
    Ok(value)
}

/// Initializes sets of Lean runtime components and modules and passes the
/// runtime to a function that depends on Lean functionality
///
/// Uses `LeanIoError::from_lean_io_error()` to convert Lean module
/// initialization errors to `LeanError`.
///
/// # Safety
///
/// Callers must either avoid initializing the Lean runtime multiple times, or
/// must use runtime components that are safe to initialize multiple times.
pub unsafe fn run_in_lean_runtime_with_default_error_handler_unchecked<
    C: RuntimeComponents,
    M: Modules,
    T,
    RunError: Error,
    Run: FnOnce(&RuntimeImpl<C, M>) -> Result<T, RunError>,
>(
    run: Run,
) -> Result<T, LeanError<<C as RuntimeComponents>::InitializationError, LeanIoError, RunError>> {
    unsafe {
        run_in_lean_runtime_unchecked(
            |lean_io_error| LeanIoError::from_lean_io_error(lean_io_error),
            run,
        )
    }
}

/// Initializes sets of Lean runtime components and modules and passes the
/// runtime to a function that depends on Lean functionality
///
/// # Panics
///
/// Panics if this function has already been called. The runtime is single-use
/// to eliminate overhead from repeatedly checking whether it has already been
/// initialized. There is no need to call this function multiple times in the
/// same program.
///
/// See also [`run_in_lean_runtime_unchecked()`] which does not panic but
/// delegates repeated initialization checks to the caller.
pub fn run_in_lean_runtime<
    C: RuntimeComponents,
    M: Modules,
    T,
    ModulesInitializationError: Error,
    ModulesInitializationErrorHandler: FnOnce(b_lean_obj_arg) -> ModulesInitializationError,
    RunError: Error,
    Run: FnOnce(&RuntimeImpl<C, M>) -> Result<T, RunError>,
>(
    modules_initialization_error_handler: ModulesInitializationErrorHandler,
    run: Run,
) -> Result<
    T,
    LeanError<<C as RuntimeComponents>::InitializationError, ModulesInitializationError, RunError>,
> {
    let mut result = None;
    ONCE_INITIALIZATION_GUARD.call_once(|| {
        result = Some(unsafe {
            run_in_lean_runtime_unchecked(modules_initialization_error_handler, run)
        });
    });
    result.expect("attempt to reuse the Lean runtime. The runtime is single-use to eliminate overhead from repeatedly checking whether it has already been initialized")
}

/// Initializes sets of Lean runtime components and modules and passes the
/// runtime to a function that depends on Lean functionality
///
/// Uses `LeanIoError::from_lean_io_error()` to convert Lean module
/// initialization errors to `LeanError`.
///
/// # Panics
///
/// Panics if this function or any other functions that call
/// [`run_in_lean_runtime()`] have already been called. The runtime is
/// single-use to eliminate overhead from repeatedly checking whether it has
/// already been initialized. There is no need to call this function multiple
/// times in the same program.
///
/// See also [`run_in_lean_runtime_with_default_error_handler_unchecked()`] which
/// does not panic but delegates repeated initialization checks to the caller.
pub fn run_in_lean_runtime_with_default_error_handler<
    C: RuntimeComponents,
    M: Modules,
    T,
    RunError: Error,
    Run: FnOnce(&RuntimeImpl<C, M>) -> Result<T, RunError>,
>(
    run: Run,
) -> Result<T, LeanError<<C as RuntimeComponents>::InitializationError, LeanIoError, RunError>> {
    run_in_lean_runtime(
        |lean_io_error| unsafe { LeanIoError::from_lean_io_error(lean_io_error) },
        run,
    )
}
