use std::convert::Infallible;

use lean_sys::{
    lean_finalize_task_manager, lean_finalize_thread, lean_init_task_manager, lean_initialize,
    lean_initialize_runtime_module, lean_initialize_thread, lean_io_mark_end_initialization,
};

use crate::{RuntimeComponents, SyncRuntimeComponents};

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

    unsafe fn post_modules_initialization() {
        post_modules_initialization();
    }

    unsafe fn finalize_runtime() {
        finalize_runtime();
    }
}

unsafe impl SyncRuntimeComponents for MinimalComponents {
    type ThreadInitializationError = Infallible;

    unsafe fn initialize_thread() -> Result<(), Self::ThreadInitializationError> {
        initialize_thread()
    }

    unsafe fn finalize_thread() {
        finalize_thread();
    }
}

/// A trait implemented by types that initialize the standard Lean runtime
///
/// # Safety
///
/// Implementations of this trait must guarantee that the Lean runtime is
/// initialized.
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

    unsafe fn post_modules_initialization() {
        post_modules_initialization();
    }

    unsafe fn finalize_runtime() {
        finalize_runtime();
    }
}

unsafe impl SyncRuntimeComponents for LeanPackageComponents {
    type ThreadInitializationError = Infallible;

    unsafe fn initialize_thread() -> Result<(), Self::ThreadInitializationError> {
        initialize_thread()
    }

    unsafe fn finalize_thread() {
        finalize_thread();
    }
}

unsafe impl Minimal for LeanPackageComponents {}

/// A trait implemented by types that initialize the Lean package
///
/// # Safety
///
/// Implementations of this trait must guarantee that the Lean package is
/// initialized.
pub unsafe trait LeanPackage: Minimal {}

unsafe impl LeanPackage for LeanPackageComponents {}

fn post_modules_initialization() {
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

fn initialize_thread() -> Result<(), Infallible> {
    unsafe {
        lean_initialize_thread();
    }
    Ok(())
}

fn finalize_thread() {
    unsafe {
        lean_finalize_thread();
    }
}
