use lean::{Modules, create_module_trait};

#[create_module_trait]
#[derive(Modules)]
#[initializer(map_array_sys::MapArray_c::initialize_map_x2darray_MapArray)]
pub enum MapArrayModuleInitializer {}
