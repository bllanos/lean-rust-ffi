use std::fmt;

use lean::{
    Minimal, Runtime,
    lean_types::{Owner, object::Object, string::LeanString},
};
use map_array_sys::MapArray::Basic_c::{map_options_to_string, mk_map_options};

use crate::MapArrayModule;

pub struct MapOptions(Object<Self>);

impl MapOptions {
    pub fn new<R: Minimal, M: MapArrayModule>(
        _runtime: &Runtime<R, M>,
        addend: i32,
        multiplicand: i32,
    ) -> Self {
        let lean_map_options = unsafe { mk_map_options(addend as u32, multiplicand as u32) };
        Self(unsafe { Object::new(lean_map_options) })
    }

    pub fn into_inner(self) -> Object<Self> {
        self.0
    }
}

impl fmt::Display for MapOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let clone = self.0.share();
        let map_options_lean_string =
            unsafe { LeanString::new(map_options_to_string(clone.into_raw())) };
        f.write_str(map_options_lean_string.as_cstr().to_str().unwrap())
    }
}
