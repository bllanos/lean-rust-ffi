use std::marker::PhantomData;

use lean_sys::{lean_finalize_thread, lean_initialize_thread};

use crate::{Modules, Runtime, RuntimeComponents, ThreadRuntime, sync::NonSendNonSync};

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

    /// Create a new runtime for use on the main thread
    ///
    /// # Safety
    ///
    /// Callers must only call this function on the main thread and must ensure
    /// that the Lean runtime features and Lean modules have already been
    /// initialized.
    pub unsafe fn new_main_thread() -> Self {
        Self::new(true)
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
