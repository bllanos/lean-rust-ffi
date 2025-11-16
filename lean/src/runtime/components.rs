use lean_sys::{lean_initialize, lean_initialize_runtime_module, lean_io_mark_end_initialization};

use crate::RuntimeComponents;

pub enum MinimalComponents {}

unsafe impl RuntimeComponents for MinimalComponents {
    unsafe fn initialize_runtime() {
        unsafe {
            lean_initialize_runtime_module();
        }
    }

    unsafe fn mark_end_initialization() {
        unsafe {
            lean_io_mark_end_initialization();
        }
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
    unsafe fn initialize_runtime() {
        unsafe {
            lean_initialize();
        }
    }

    unsafe fn mark_end_initialization() {
        unsafe {
            lean_io_mark_end_initialization();
        }
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
