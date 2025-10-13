use lean::Modules;
use lean_sys::{lean_obj_arg, lean_obj_res};
use map_array_sys::MapArray_c::initialize_MapArray;

pub enum MapArrayModuleInitializer {}

unsafe impl Modules for MapArrayModuleInitializer {
    unsafe fn initialize_modules(builtin: u8, lean_io_world: lean_obj_arg) -> lean_obj_res {
        unsafe { initialize_MapArray(builtin, lean_io_world) }
    }
}

/// A trait implemented by types that initialize the Lean MapArray module
///
/// # Safety
///
/// Implementations of this trait must guarantee that the module is properly
/// initialized.
pub unsafe trait MapArrayModule: Modules {}

unsafe impl MapArrayModule for MapArrayModuleInitializer {}
