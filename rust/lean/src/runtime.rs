use std::error::Error;
use std::sync::Once;

use lean_sys::{b_lean_obj_arg, lean_dec, lean_io_result_get_error};

mod types;

use crate::{LeanError, LeanIoError, Modules, RuntimeComponents};
pub use types::{
    LeanPackage, LeanPackageComponents, Minimal, MinimalComponents, Runtime, RuntimeInitializer,
};

static ONCE_INITIALIZATION_GUARD: Once = Once::new();

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
        let runtime_initializer = RuntimeInitializer::new();
        result = match runtime_initializer.initialize_modules() {
            Ok(modules_initializer) => {
                let runtime = modules_initializer.mark_end_initialization();
                match run(&runtime) {
                    Ok(value) => Some(Ok(value)),
                    Err(e) => Some(Err(e.into())),
                }
            }
            Err(lean_io_result) => {
                let lean_io_error = unsafe { lean_io_result_get_error(lean_io_result) };
                let converted_error = modules_initialization_error_handler(lean_io_error);
                unsafe { lean_dec(lean_io_result) };
                Some(Err(LeanError::ModulesInitialization(converted_error)))
            }
        };
    });
    result.expect("attempt to reuse the Lean runtime. The runtime is single-use to eliminate overhead from repeatedly checking whether it has already been initialized")
}

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
