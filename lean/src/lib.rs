use std::error::Error;

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
    ArgcError, LeanPackage, LeanPackageComponents, Minimal, MinimalComponents, RuntimeImpl,
    RuntimeInitializationError, run_in_lean_runtime, run_in_lean_runtime_unchecked,
    run_in_lean_runtime_with_default_error_handler,
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
/// initialized.
pub unsafe trait RuntimeComponents {
    type InitializationError: Error;

    /// Initialize the Lean runtime
    ///
    /// # Safety
    ///
    /// Callers must ensure that the Lean runtime is initialized at most once.
    unsafe fn initialize_runtime() -> Result<(), Self::InitializationError>;

    /// End the initialization phase
    ///
    /// This function will be called after both the Lean runtime and any Lean
    /// modules have been initialized.
    ///
    /// # Safety
    ///
    /// This function must not be called more than once.
    unsafe fn post_modules_initialization();

    /// Finalize the Lean runtime
    ///
    /// # Safety
    ///
    /// Callers must ensure that the Lean runtime has been previously
    /// initialized and is finalized at most once
    unsafe fn finalize_runtime();
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
/// initialized.
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

/// A set of initialized Lean runtime features and Lean modules
///
/// Rust functions that require Lean runtime features and Lean modules to be
/// initialized first can require a parameter of a type that implements this
/// trait for the given runtime features and modules.
///
/// # Safety
///
/// Implementations of this trait must guarantee that the Lean runtime and
/// modules are initialized.
pub unsafe trait Runtime<C: RuntimeComponents, M: Modules> {}

/// A set of initialized Lean runtime features and Lean modules that can be used
/// on a secondary thread
///
/// Rust functions that need to spawn threads that use Lean runtime features and
/// Lean modules can require a parameter of a type that implements this trait
/// for the given runtime features and modules.
///
/// # Safety
///
/// Implementations of this trait must guarantee that the Lean runtime and
/// modules are initialized and are safe to use across multiple threads. They
/// must also initialize and clean up per-thread resources.
pub unsafe trait ThreadRuntime<C: RuntimeComponents, M: Modules>: Runtime<C, M> {
    /// Create a runtime for using Lean functions on a new thread
    ///
    /// # Safety
    ///
    /// Callers must invoke this function on the new thread.
    unsafe fn new_thread() -> Self;
}
