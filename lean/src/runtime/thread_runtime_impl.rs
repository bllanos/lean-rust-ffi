use std::error::Error;

use lean_sys::b_lean_obj_arg;

use super::RuntimeImpl;
use crate::{
    LeanInitializationError, Modules, Runtime, RuntimeComponents, SyncRuntimeComponents,
    ThreadRuntime,
};

pub struct ThreadRuntimeImpl<C: SyncRuntimeComponents, M: Modules> {
    primary_thread_runtime: Option<RuntimeImpl<C, M>>,
}

impl<C: SyncRuntimeComponents, M: Modules> ThreadRuntimeImpl<C, M> {
    /// Create a new runtime on the primary thread
    ///
    /// Initializes the Lean runtime features and the Lean modules corresponding
    /// to this type's generic parameters.
    ///
    /// # Safety
    ///
    /// Callers must only call this function on the primary thread and must call
    /// it at most once.
    pub unsafe fn new_primary_thread<
        ModulesInitializationError: Error,
        ModulesInitializationErrorHandler: FnOnce(b_lean_obj_arg) -> ModulesInitializationError,
    >(
        modules_initialization_error_handler: ModulesInitializationErrorHandler,
    ) -> Result<
        Self,
        LeanInitializationError<
            <C as RuntimeComponents>::InitializationError,
            ModulesInitializationError,
        >,
    > {
        Ok(Self {
            primary_thread_runtime: Some(
                (unsafe { RuntimeImpl::new_primary_thread(modules_initialization_error_handler) })?,
            ),
        })
    }
}

impl<C: SyncRuntimeComponents, M: Modules> Drop for ThreadRuntimeImpl<C, M> {
    fn drop(&mut self) {
        match self.primary_thread_runtime {
            Some(_) => {}
            None => unsafe {
                C::finalize_thread();
            },
        }
    }
}

unsafe impl<C: SyncRuntimeComponents, M: Modules> Runtime<C, M> for ThreadRuntimeImpl<C, M> {}

unsafe impl<C: SyncRuntimeComponents, M: Modules> ThreadRuntime<C, M> for ThreadRuntimeImpl<C, M> {
    type ThreadInitializationError = <C as SyncRuntimeComponents>::ThreadInitializationError;

    unsafe fn new_secondary_thread() -> Result<Self, Self::ThreadInitializationError> {
        (unsafe { C::initialize_thread() })?;
        Ok(Self {
            primary_thread_runtime: None,
        })
    }
}
