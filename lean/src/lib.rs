use std::error::Error;

pub use lean_sys::{ELAN_TOOLCHAIN, LEAN_GITHASH, lean_obj_arg, lean_obj_res};

#[cfg(feature = "lean-derive")]
pub use lean_derive::Modules;

#[cfg(feature = "lean-macro")]
pub use lean_macro::*;

mod alloc;
mod error;
pub mod lean_types;
mod module;
pub mod number;
mod result;
mod runtime;
mod sync;
mod thread;

pub use alloc::MimallocAllocator;
pub use error::{LeanError, LeanInitializationError, LeanIoError};
pub use module::NoModules;
pub use result::{
    LeanIoResult, make_lean_io_result_error, make_lean_io_result_ok_unit, run_lean_io_unit,
};
pub use runtime::{
    LeanPackage, LeanPackageComponents, Minimal, MinimalComponents, RuntimeImpl,
    RuntimeInitializationError, ThreadRuntimeImpl, args, run_in_lean_runtime,
    run_in_lean_runtime_unchecked, run_in_lean_runtime_with_default_error_handler,
    run_in_lean_sync_runtime, run_in_lean_sync_runtime_unchecked,
    run_in_lean_sync_runtime_with_default_error_handler,
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
    /// Callers must ensure that:
    ///
    /// 1. [`Self::initialize_runtime()`] has previously succeeded
    /// 2. This function is called at most once and is called on the same thread
    ///    as [`Self::initialize_runtime()`]
    unsafe fn post_modules_initialization();

    /// Finalize the Lean runtime
    ///
    /// # Safety
    ///
    /// Callers must ensure that:
    ///
    /// 1. [`Self::initialize_runtime()`] and
    ///    [`Self::post_modules_initialization()`] have been previously called
    ///    and succeeded
    /// 2. This function is called at most once and is called on the same thread
    ///    as the previous functions
    /// 3. Lean runtime features are not used after this function is called
    unsafe fn finalize_runtime();
}

/// A set of features that are available in the Lean runtime for use on multiple
/// threads
///
/// # Safety
///
/// Implementations of this trait must guarantee that the Lean runtime is
/// initialized and that all runtime features can be used on multiple threads.
pub unsafe trait SyncRuntimeComponents: RuntimeComponents {
    type ThreadInitializationError: Error;

    /// Initialize per-thread resources
    ///
    /// # Safety
    ///
    /// Callers must ensure that:
    ///
    /// 1. [`RuntimeComponents::initialize_runtime()`] and
    ///    [`RuntimeComponents::post_modules_initialization()`]
    ///    have been previously called and succeeded
    /// 2. This function is called at most once per new thread, and is called on
    ///    the new threads.
    unsafe fn initialize_thread() -> Result<(), Self::ThreadInitializationError>;

    /// Finalize per-thread resources
    ///
    /// # Safety
    ///
    /// Callers must ensure that:
    ///
    /// 1. [`Self::initialize_thread()`] has been previously called and succeeded
    /// 2. This function is called at most once and is called on the same thread
    ///    as [`Self::initialize_thread()`]
    /// 3. Lean runtime features are not used on the current thread after this
    ///    function is called
    unsafe fn finalize_thread();
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
    unsafe fn initialize_modules(builtin: u8) -> lean_obj_res;
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
/// on multiple threads
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
pub unsafe trait ThreadRuntime<C: SyncRuntimeComponents, M: Modules>:
    Sized + Runtime<C, M>
{
    type ThreadInitializationError: Error;

    /// Create a runtime for using Lean functions on a new thread
    ///
    /// # Safety
    ///
    /// Callers must invoke this function on the new thread. Callers must have
    /// successfully initialized the Lean runtime features on the primary
    /// thread using the functions from [`Runtime`].
    unsafe fn new_secondary_thread() -> Result<Self, Self::ThreadInitializationError>;
}
