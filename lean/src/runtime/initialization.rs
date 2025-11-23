use std::marker::PhantomData;

use lean_sys::lean_obj_res;

use crate::{Modules, ModulesInitializer, RuntimeComponents, sync::NonSendNonSync};

pub struct RuntimeInitializer<C: RuntimeComponents, M: Modules> {
    runtime_components: PhantomData<C>,
    modules_initializer: PhantomData<M>,
    non_send_non_sync: NonSendNonSync,
}

impl<C: RuntimeComponents, M: Modules> RuntimeInitializer<C, M> {
    fn initialize_fields() -> Self {
        Self {
            runtime_components: PhantomData,
            modules_initializer: PhantomData,
            non_send_non_sync: PhantomData,
        }
    }

    pub fn new() -> Result<Self, <C as RuntimeComponents>::InitializationError> {
        unsafe { C::initialize_runtime() }?;
        Ok(Self::initialize_fields())
    }

    pub fn initialize_modules(self) -> Result<ModulesInitializer<C, M>, lean_obj_res> {
        ModulesInitializer::new()
    }
}
