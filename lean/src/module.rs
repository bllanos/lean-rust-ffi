use lean_sys::lean_obj_res;

use crate::Modules;

pub enum NoModules {}

unsafe impl Modules for NoModules {
    unsafe fn initialize_modules(_builtin: u8) -> lean_obj_res {
        crate::make_lean_io_result_ok_unit()
    }
}
