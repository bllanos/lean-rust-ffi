use std::marker::PhantomData;

use crate::{Modules, RuntimeComponents, sync::NonSendNonSync};

pub struct Runtime<R: RuntimeComponents, M: Modules> {
    runtime_components: PhantomData<R>,
    modules_initializer: PhantomData<M>,
    is_main_thread: bool,
    non_send_non_sync: NonSendNonSync,
}

impl<R: RuntimeComponents, M: Modules> Runtime<R, M> {
    fn new(is_main_thread: bool) -> Self {
        Self {
            runtime_components: PhantomData,
            modules_initializer: PhantomData,
            is_main_thread,
            non_send_non_sync: PhantomData,
        }
    }

    pub(crate) fn new_main_thread() -> Self {
        Self::new(true)
    }

    pub(crate) fn new_secondary_thread() -> Self {
        Self::new(false)
    }
}

impl<R: RuntimeComponents, M: Modules> Drop for Runtime<R, M> {
    fn drop(&mut self) {
        if self.is_main_thread {
            unsafe {
                R::finalize_runtime();
            }
        }
    }
}
