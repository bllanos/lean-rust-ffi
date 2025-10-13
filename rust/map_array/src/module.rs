use lean::Modules;
use map_array_sys::MapArray_c::initialize_MapArray;

#[derive(Modules)]
pub enum MapArrayModuleInitializer {}

/// A trait implemented by types that initialize the Lean MapArray module
///
/// # Safety
///
/// Implementations of this trait must guarantee that the module is properly
/// initialized.
pub unsafe trait MapArrayModule: Modules {}

unsafe impl MapArrayModule for MapArrayModuleInitializer {}
