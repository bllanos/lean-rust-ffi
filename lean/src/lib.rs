pub use lean_sys::{ELAN_TOOLCHAIN, LEAN_GITHASH, lean_obj_arg, lean_obj_res};

// Re-export #[derive(Modules)]
#[cfg(feature = "lean_derive")]
#[allow(unused_imports)]
pub use lean_derive::*;

// Re-export other procedural macros
#[cfg(feature = "lean_macro")]
#[allow(unused_imports)]
pub use lean_macro::*;

mod alloc;
mod error;
pub mod lean_types;
mod module;
mod runtime;
mod sync;
mod thread;

use module::ModulesInitializer;

pub use alloc::MimallocAllocator;
pub use error::{LeanError, LeanIoError};
pub use module::NoModules;
pub use runtime::{
    LeanPackage, LeanPackageComponents, Minimal, MinimalComponents, Runtime, run_in_lean_runtime,
    run_in_lean_runtime_unchecked, run_in_lean_runtime_with_default_error_handler,
    run_in_lean_runtime_with_default_error_handler_unchecked,
};
pub use thread::{
    run_in_custom_scoped_thread_with_lean_runtime, run_in_custom_thread_with_lean_runtime,
    run_in_thread_with_lean_runtime,
};

/// A set of features that are available in the Lean runtime
///
/// Crates that wrap Lean modules with safe Rust bindings can expose functions
/// that require references to [`Runtime<R, _>`] where `R` is some type that
/// implements this trait as well as any traits representing the specific Lean
/// runtime components(s) needed. Those functions can therefore only called
/// after the Lean runtime components have been initialized.
///
/// # Safety
///
/// Implementations of this trait must guarantee that the Lean runtime is
/// properly initialized.
pub unsafe trait RuntimeComponents {
    /// Initialize the Lean runtime
    ///
    /// # Safety
    ///
    /// Callers must ensure that the Lean runtime is initialized at most once.
    unsafe fn initialize_runtime();

    /// Mark the end of the initialization phase
    ///
    /// This function will be called after both the Lean runtime and any Lean
    /// modules have been initialized.
    ///
    /// # Safety
    ///
    /// This function must not be called more than once.
    unsafe fn mark_end_initialization();
}

/// A trait to be implemented by types that initialize one or more Lean modules
///
/// Crates that wrap Lean modules with safe Rust bindings can expose functions
/// that require references to [`Runtime<_, M>`] where `M` is some type that
/// implements this trait as well as any traits representing the specific Lean
/// module(s). Those functions can therefore only called after the Lean modules
/// have been initialized.
///
/// # Safety
///
/// Implementations of this trait must guarantee that the Lean modules are
/// properly initialized.
pub unsafe trait Modules {
    /// Initialize all required Lean modules
    ///
    /// It is not necessary for implementors to initialize the `Lean` module, as
    /// this can be done by using [`LeanPackageComponents`], which implements
    /// [`RuntimeComponents`].
    ///
    /// The signature of this function is that of a Lean module initializer. See
    /// <https://github.com/leanprover/lean4/blob/master/doc/dev/ffi.md#initialization>.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the Lean runtime has been initialized before
    /// calling this function.
    unsafe fn initialize_modules(builtin: u8, lean_io_world: lean_obj_arg) -> lean_obj_res;
}
