use async_effect::io;
use lean_sys::lean_obj_res;

#[unsafe(no_mangle)]
pub extern "C" fn monotonic_now_immediate() -> lean_obj_res {
    let now = io::monotonic_now_immediate();
    // TODO: Return an opaque Instant type
    let count = now.elapsed().as_millis() as u64;
    unsafe { lean_sys::lean_uint64_to_nat(count) }
}
