#![forbid(unsafe_code)]

use std::error::Error;

use lean::{MimallocAllocator, MinimalComponents, Runtime};
use map_array::{MapArrayModuleInitializer, MapOptions};

#[global_allocator]
static ALLOCATOR: MimallocAllocator = MimallocAllocator {};

/// A test that uses Lean's allocator (assumed to be mimalloc) for dynamic
/// memory allocation in Rust and exercises functions from the Lean `MapArray`
/// module.
#[test]
fn map_array_custom_allocator() -> Result<(), Box<dyn Error>> {
    lean::run_in_lean_runtime_with_default_error_handler(
        |runtime: &Runtime<MinimalComponents, MapArrayModuleInitializer>| {
            let addend: i32 = 2;
            let multiplicand: i32 = 3;
            let map_options = MapOptions::new(runtime, addend, multiplicand);

            let map_options_string = map_options.to_string();
            assert_eq!(&map_options_string, "{ addend := 2, multiplicand := 3 }");

            let mut array_data: [u8; 6] = Default::default();
            for (i, element) in array_data.iter_mut().enumerate() {
                *element = (i * 5).try_into()?;
            }

            let array_out = map_array::my_map(runtime, map_options, array_data);

            let expected_data = [6_i32, 21, 36, 51, 66, 81];
            assert!(expected_data.into_iter().eq(array_out.iter()));

            Ok::<_, <u8 as TryFrom<usize>>::Error>(())
        },
    )?;
    Ok(())
}
