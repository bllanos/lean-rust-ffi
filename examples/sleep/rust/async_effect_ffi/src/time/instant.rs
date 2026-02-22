use core::ffi::c_void;

use std::time::Instant;

use lean::lean_types::{
    Borrower,
    external::{self, ExternalClass, ExternalClassHolder, LeanExternalTypeTag},
    object::Obj,
};
use lean_sys::{b_lean_obj_arg, lean_obj_res};

use super::LeanDuration;

#[derive(LeanExternalTypeTag)]
pub struct LeanInstant(pub Instant);

#[unsafe(no_mangle)]
unsafe extern "C" fn async_effect_ffi_lean_instant_finalize(instance: *mut c_void) {
    unsafe { external::finalize::<LeanInstant>(instance) };
}

#[unsafe(no_mangle)]
unsafe extern "C" fn async_effect_ffi_lean_instant_foreach(
    instance: *mut c_void,
    f: b_lean_obj_arg,
) {
    unsafe { external::foreach::<LeanInstant>(instance, f) };
}

static INSTANT_EXTERNAL_CLASS: ExternalClass<LeanInstant> = ExternalClass::new(
    async_effect_ffi_lean_instant_finalize,
    async_effect_ffi_lean_instant_foreach,
);

impl ExternalClassHolder for LeanInstant {
    fn get_external_class() -> &'static ExternalClass<Self> {
        &INSTANT_EXTERNAL_CLASS
    }
}

/// Subtract two instants
///
/// # Safety
///
/// Callers must ensure that the arguments point to borrowed Lean external
/// objects containing [`LeanInstant`] instances.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn async_effect_ffi_instant_subtract(
    x: b_lean_obj_arg,
    y: b_lean_obj_arg,
) -> lean_obj_res {
    let x_instant: Obj<LeanInstant> = unsafe { Obj::new(x) };
    let y_instant: Obj<LeanInstant> = unsafe { Obj::new(y) };
    let duration = x_instant.as_ref().0 - y_instant.as_ref().0;
    LeanDuration(duration).into_lean_object()
}
