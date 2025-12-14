use lean_sys::{lean_box, lean_io_result_mk_ok, lean_obj_res};

use crate::Modules;

pub enum NoModules {}

unsafe impl Modules for NoModules {
    unsafe fn initialize_modules(_builtin: u8) -> lean_obj_res {
        unsafe { lean_io_result_mk_ok(lean_box(0)) }
    }
}
