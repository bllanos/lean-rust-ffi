use std::error::Error;
use std::marker::PhantomData;

use lean_sys::{
    b_lean_obj_arg, lean_dec, lean_finalize_thread, lean_initialize_thread, lean_io_mk_world,
    lean_io_result_get_error, lean_io_result_is_ok, lean_obj_res, lean_object,
};

use crate::{
    LeanInitializationError, Modules, Runtime, RuntimeComponents, ThreadRuntime,
    sync::NonSendNonSync,
};

pub struct RuntimeImpl<C: RuntimeComponents, M: Modules> {
    runtime_components: PhantomData<C>,
    modules_initializer: PhantomData<M>,
    is_main_thread: bool,
    non_send_non_sync: NonSendNonSync,
}

impl<C: RuntimeComponents, M: Modules> RuntimeImpl<C, M> {
    fn new(is_main_thread: bool) -> Self {
        Self {
            runtime_components: PhantomData,
            modules_initializer: PhantomData,
            is_main_thread,
            non_send_non_sync: PhantomData,
        }
    }

    /// Initialize Lean modules
    ///
    /// # Safety
    ///
    /// Callers must call this function on the main thread and must call it at
    /// most once. The Lean runtime must already be initialized.
    unsafe fn initialize_modules() -> Result<(), lean_obj_res> {
        let res: *mut lean_object;
        // Use same default as for Lean executables
        // See <https://github.com/leanprover/lean4/blob/master/doc/dev/ffi.md#initialization>
        let builtin: u8 = 1;

        unsafe {
            res = M::initialize_modules(builtin, lean_io_mk_world());
            if lean_io_result_is_ok(res) {
                lean_dec(res);
                Ok(())
            } else {
                Err(res)
            }
        }
    }

    /// Create a new runtime for use on the main thread
    ///
    /// Initializes the Lean runtime features and the Lean modules corresponding
    /// to this type's generic parameters.
    ///
    /// # Safety
    ///
    /// Callers must only call this function on the main thread and must call it
    /// at most once.
    pub unsafe fn new_main_thread<
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
        { unsafe { C::initialize_runtime() } }
            .map_err(LeanInitializationError::RuntimeInitialization)?;
        { unsafe { Self::initialize_modules() } }.map_err(|lean_io_result| {
            let lean_io_error = unsafe { lean_io_result_get_error(lean_io_result) };
            let converted_error = modules_initialization_error_handler(lean_io_error);
            unsafe { lean_dec(lean_io_result) };
            LeanInitializationError::ModulesInitialization(converted_error)
        })?;
        unsafe {
            C::post_modules_initialization();
        }
        Ok(Self::new(true))
    }
}

impl<C: RuntimeComponents, M: Modules> Drop for RuntimeImpl<C, M> {
    fn drop(&mut self) {
        if self.is_main_thread {
            unsafe {
                C::finalize_runtime();
            }
        } else {
            unsafe {
                lean_finalize_thread();
            }
        }
    }
}

unsafe impl<C: RuntimeComponents, M: Modules> Runtime<C, M> for RuntimeImpl<C, M> {}

unsafe impl<C: RuntimeComponents, M: Modules> ThreadRuntime<C, M> for RuntimeImpl<C, M> {
    unsafe fn new_thread() -> Self {
        unsafe {
            lean_initialize_thread();
        }
        Self::new(false)
    }
}
