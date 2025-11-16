use std::error::Error;
use std::sync::Once;

use lean_sys::{b_lean_obj_arg, lean_dec, lean_io_result_get_error};

use crate::{LeanError, LeanIoError, Modules, RuntimeComponents};

mod components;
mod handle;
mod initialization;

pub use components::{LeanPackage, LeanPackageComponents, Minimal, MinimalComponents};
pub use handle::Runtime;
pub use initialization::RuntimeInitializer;

static ONCE_INITIALIZATION_GUARD: Once = Once::new();

/// Initializes sets of Lean runtime components and modules and passes the
/// runtime to a function that depends on Lean functionality
///
/// # Safety
///
/// Callers must either avoid initializing the Lean runtime multiple times, or
/// must use runtime components that are safe to initialize multiple times.
pub unsafe fn run_in_lean_runtime_unchecked<
    R: RuntimeComponents,
    M: Modules,
    T,
    ModulesInitializationError: Error,
    ModulesInitializationErrorHandler: FnOnce(b_lean_obj_arg) -> ModulesInitializationError,
    RunError: Error,
    Run: FnOnce(&Runtime<R, M>) -> Result<T, RunError>,
>(
    modules_initialization_error_handler: ModulesInitializationErrorHandler,
    run: Run,
) -> Result<T, LeanError<ModulesInitializationError, RunError>> {
    let runtime_initializer = RuntimeInitializer::new();
    match runtime_initializer.initialize_modules() {
        Ok(modules_initializer) => {
            let runtime = modules_initializer.mark_end_initialization();
            match run(&runtime) {
                Ok(value) => Ok(value),
                Err(e) => Err(e.into()),
            }
        }
        Err(lean_io_result) => {
            let lean_io_error = unsafe { lean_io_result_get_error(lean_io_result) };
            let converted_error = modules_initialization_error_handler(lean_io_error);
            unsafe { lean_dec(lean_io_result) };
            Err(LeanError::ModulesInitialization(converted_error))
        }
    }
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
pub fn run_in_lean_runtime_with_default_error_handler_unchecked<
    R: RuntimeComponents,
    M: Modules,
    T,
    RunError: Error,
    Run: FnOnce(&Runtime<R, M>) -> Result<T, RunError>,
>(
    run: Run,
) -> Result<T, LeanError<LeanIoError, RunError>> {
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
    R: RuntimeComponents,
    M: Modules,
    T,
    ModulesInitializationError: Error,
    ModulesInitializationErrorHandler: FnOnce(b_lean_obj_arg) -> ModulesInitializationError,
    RunError: Error,
    Run: FnOnce(&Runtime<R, M>) -> Result<T, RunError>,
>(
    modules_initialization_error_handler: ModulesInitializationErrorHandler,
    run: Run,
) -> Result<T, LeanError<ModulesInitializationError, RunError>> {
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
    R: RuntimeComponents,
    M: Modules,
    T,
    RunError: Error,
    Run: FnOnce(&Runtime<R, M>) -> Result<T, RunError>,
>(
    run: Run,
) -> Result<T, LeanError<LeanIoError, RunError>> {
    run_in_lean_runtime(
        |lean_io_error| unsafe { LeanIoError::from_lean_io_error(lean_io_error) },
        run,
    )
}
