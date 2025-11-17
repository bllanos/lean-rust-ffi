use lean_sys::{
    lean_finalize_task_manager, lean_init_task_manager, lean_initialize,
    lean_initialize_runtime_module, lean_io_mark_end_initialization,
};

use crate::RuntimeComponents;

mod args;

pub use args::ArgcError;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum RuntimeInitializationError {
    #[error(transparent)]
    Argc(#[from] ArgcError),
}

pub enum MinimalComponents {}

unsafe impl RuntimeComponents for MinimalComponents {
    type InitializationError = RuntimeInitializationError;

    unsafe fn initialize_runtime() -> Result<(), Self::InitializationError> {
        args::call_lean_setup_args()?;
        unsafe {
            lean_initialize_runtime_module();
        }
        Ok(())
    }

    unsafe fn mark_end_initialization() {
        mark_end_initialization();
    }

    unsafe fn finalize_runtime() {
        finalize_runtime();
    }
}

/// A trait implemented by types that initialize the standard Lean runtime
///
/// # Safety
///
/// Implementations of this trait must guarantee that the Lean runtime is
/// properly initialized.
pub unsafe trait Minimal: RuntimeComponents {}

unsafe impl Minimal for MinimalComponents {}

pub enum LeanPackageComponents {}

unsafe impl RuntimeComponents for LeanPackageComponents {
    type InitializationError = RuntimeInitializationError;

    unsafe fn initialize_runtime() -> Result<(), Self::InitializationError> {
        args::call_lean_setup_args()?;
        unsafe {
            lean_initialize();
        }
        Ok(())
    }

    unsafe fn mark_end_initialization() {
        mark_end_initialization();
    }

    unsafe fn finalize_runtime() {
        finalize_runtime();
    }
}

unsafe impl Minimal for LeanPackageComponents {}

/// A trait implemented by types that initialize the Lean package
///
/// # Safety
///
/// Implementations of this trait must guarantee that the Lean package is
/// properly initialized.
pub unsafe trait LeanPackage: Minimal {}

unsafe impl LeanPackage for LeanPackageComponents {}

fn mark_end_initialization() {
    unsafe {
        lean_io_mark_end_initialization();
        lean_init_task_manager();
    }
}

fn finalize_runtime() {
    unsafe {
        lean_finalize_task_manager();
    }
}
