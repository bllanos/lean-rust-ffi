use std::marker::PhantomData;

use crate::{Modules, RuntimeComponents, sync::NonSendNonSync};

pub struct Runtime<R: RuntimeComponents, M: Modules> {
    runtime_components: PhantomData<R>,
    modules_initializer: PhantomData<M>,
    non_send_non_sync: NonSendNonSync,
}

impl<R: RuntimeComponents, M: Modules> Runtime<R, M> {
    pub(crate) fn new() -> Self {
        Self {
            runtime_components: PhantomData,
            modules_initializer: PhantomData,
            non_send_non_sync: PhantomData,
        }
    }
}
