use lean::{
    Minimal, Runtime,
    lean_types::{
        Owner,
        array::{Integer32Array, U32Array},
    },
};
use map_array_sys::MapArray::Basic_c::my_map as my_map_sys;

use crate::{MapArrayModule, MapOptions};

pub fn my_map<C: Minimal, M: MapArrayModule, R: Runtime<C, M>, I: IntoIterator<Item = u8>>(
    runtime: &R,
    options: MapOptions,
    data: I,
) -> Integer32Array<i32>
where
    <I as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let lean_array = U32Array::from_exact_size_iterator(runtime, data);
    unsafe {
        Integer32Array::new(my_map_sys(
            options.into_inner().into_raw(),
            lean_array.into_raw(),
        ))
    }
}
