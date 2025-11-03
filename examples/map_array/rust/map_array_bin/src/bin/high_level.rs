#![forbid(unsafe_code)]

use std::error::Error;

use lean::{MimallocAllocator, MinimalComponents, Runtime};
use lean_sys::ELAN_TOOLCHAIN;
use map_array::{MapArrayModuleInitializer, MapOptions};

#[global_allocator]
static ALLOCATOR: MimallocAllocator = MimallocAllocator {};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Program start");
    println!(
        "Lean toolchain version used to build the lean-sys crate: {}",
        ELAN_TOOLCHAIN
    );

    lean::run_in_lean_runtime_with_default_error_handler(
        |runtime: &Runtime<MinimalComponents, MapArrayModuleInitializer>| {
            let addend: i32 = 2;
            let multiplicand: i32 = 3;
            let map_options = MapOptions::new(runtime, addend, multiplicand);

            println!("MapOptions instance: {}", map_options);

            let mut array: [u8; 6] = Default::default();
            for (i, element) in array.iter_mut().enumerate() {
                *element = (i * 5).try_into()?;
            }
            println!("Input array: {:?}", array);

            let array_out = map_array::my_map(runtime, map_options, array);

            print!("Output array: [ ");
            for value in array_out.iter() {
                print!("{}, ", value);
            }
            println!("]");

            Ok::<_, <u8 as TryFrom<usize>>::Error>(())
        },
    )?;

    println!("Program end");

    Ok(())
}
