use std::marker::PhantomData;

use lean_sys::lean_obj_res;

use crate::{Modules, ModulesInitializer, RuntimeComponents, sync::NonSendNonSync};

pub struct RuntimeInitializer<R: RuntimeComponents, M: Modules> {
    runtime_components: PhantomData<R>,
    modules_initializer: PhantomData<M>,
    non_send_non_sync: NonSendNonSync,
}

impl<R: RuntimeComponents, M: Modules> RuntimeInitializer<R, M> {
    fn initialize_fields() -> Self {
        Self {
            runtime_components: PhantomData,
            modules_initializer: PhantomData,
            non_send_non_sync: PhantomData,
        }
    }

    pub fn new() -> Result<Self, <R as RuntimeComponents>::InitializationError> {
        unsafe { R::initialize_runtime() }?;
        Ok(Self::initialize_fields())
    }

    pub fn initialize_modules(self) -> Result<ModulesInitializer<R, M>, lean_obj_res> {
        ModulesInitializer::new()
    }
}
