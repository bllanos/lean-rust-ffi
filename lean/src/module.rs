use std::marker::PhantomData;

use lean_sys::{
    lean_box, lean_dec, lean_io_mk_world, lean_io_result_is_ok, lean_io_result_mk_ok, lean_obj_arg,
    lean_obj_res, lean_object,
};

use crate::{Modules, Runtime, RuntimeComponents, sync::NonSendNonSync};

pub enum NoModules {}

unsafe impl Modules for NoModules {
    unsafe fn initialize_modules(_builtin: u8, _lean_io_world: lean_obj_arg) -> lean_obj_res {
        unsafe { lean_io_result_mk_ok(lean_box(0)) }
    }
}

pub struct ModulesInitializer<R: RuntimeComponents, M: Modules> {
    runtime_components: PhantomData<R>,
    modules_initializer: PhantomData<M>,
    non_send_non_sync: NonSendNonSync,
}

impl<R: RuntimeComponents, M: Modules> ModulesInitializer<R, M> {
    fn initialize_fields() -> Self {
        Self {
            runtime_components: PhantomData,
            modules_initializer: PhantomData,
            non_send_non_sync: PhantomData,
        }
    }

    pub fn new() -> Result<Self, lean_obj_res> {
        let res: *mut lean_object;
        // Use same default as for Lean executables
        // See <https://github.com/leanprover/lean4/blob/master/doc/dev/ffi.md#initialization>
        let builtin: u8 = 1;

        unsafe {
            res = M::initialize_modules(builtin, lean_io_mk_world());
            if lean_io_result_is_ok(res) {
                lean_dec(res);
                Ok(Self::initialize_fields())
            } else {
                Err(res)
            }
        }
    }

    pub fn mark_end_initialization(self) -> Runtime<R, M> {
        unsafe {
            R::mark_end_initialization();
        }
        Runtime::new_main_thread()
    }
}
