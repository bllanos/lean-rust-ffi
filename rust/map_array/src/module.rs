use lean::{Modules, create_module_trait};
use map_array_sys::MapArray_c::initialize_MapArray;

#[create_module_trait]
#[derive(Modules)]
pub enum MapArrayModuleInitializer {}
