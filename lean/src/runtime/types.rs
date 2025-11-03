use std::marker::PhantomData;

use lean_sys::{
    lean_initialize, lean_initialize_runtime_module, lean_io_mark_end_initialization, lean_obj_res,
};

use crate::{Modules, ModulesInitializer, RuntimeComponents, util::NonSendNonSync};

pub enum MinimalComponents {}

unsafe impl RuntimeComponents for MinimalComponents {
    unsafe fn initialize_runtime() {
        unsafe {
            lean_initialize_runtime_module();
        }
    }

    unsafe fn mark_end_initialization() {
        unsafe {
            lean_io_mark_end_initialization();
        }
    }
}

/// A trait implemented by types that initialize the standard Lean runtime
///
/// # Safety
///
/// Implementations of this trait must guarantee that the Lean runtime is
/// properly initialized.
pub unsafe trait Minimal: RuntimeComponents {}

unsafe impl Minimal for MinimalComponents {}

pub enum LeanPackageComponents {}

unsafe impl RuntimeComponents for LeanPackageComponents {
    unsafe fn initialize_runtime() {
        unsafe {
            lean_initialize();
        }
    }

    unsafe fn mark_end_initialization() {
        unsafe {
            lean_io_mark_end_initialization();
        }
    }
}

unsafe impl Minimal for LeanPackageComponents {}

/// A trait implemented by types that initialize the Lean package
///
/// # Safety
///
/// Implementations of this trait must guarantee that the Lean package is
/// properly initialized.
pub unsafe trait LeanPackage: Minimal {}

unsafe impl LeanPackage for LeanPackageComponents {}

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

    pub fn new() -> Self {
        unsafe {
            R::initialize_runtime();
        }
        Self::initialize_fields()
    }

    pub fn initialize_modules(self) -> Result<ModulesInitializer<R, M>, lean_obj_res> {
        ModulesInitializer::new()
    }
}

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
