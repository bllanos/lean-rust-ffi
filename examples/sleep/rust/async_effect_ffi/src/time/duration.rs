use core::ffi::c_void;

use std::time::Duration;

use lean::{
    lean_types::{
        Borrower,
        external::{self, ExternalClass, ExternalClassHolder, LeanExternalTypeTag},
        object::Obj,
    },
    number,
};
use lean_sys::{b_lean_obj_arg, lean_obj_res};

#[derive(LeanExternalTypeTag)]
pub struct LeanDuration(pub Duration);

#[unsafe(no_mangle)]
unsafe extern "C" fn async_effect_ffi_lean_duration_finalize(instance: *mut c_void) {
    unsafe { external::finalize::<LeanDuration>(instance) };
}

#[unsafe(no_mangle)]
unsafe extern "C" fn async_effect_ffi_lean_duration_foreach(
    instance: *mut c_void,
    f: b_lean_obj_arg,
) {
    unsafe { external::foreach::<LeanDuration>(instance, f) };
}

static INSTANT_EXTERNAL_CLASS: ExternalClass<LeanDuration> = ExternalClass::new(
    async_effect_ffi_lean_duration_finalize,
    async_effect_ffi_lean_duration_foreach,
);

impl ExternalClassHolder for LeanDuration {
    fn get_external_class() -> &'static ExternalClass<Self> {
        &INSTANT_EXTERNAL_CLASS
    }
}

/// Return the number of seconds corresponding to a duration
///
/// # Safety
///
/// Callers must ensure that the argument points to a borrowed Lean external
/// object containing a [`LeanDuration`] instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn async_effect_ffi_duration_as_secs(d: b_lean_obj_arg) -> u64 {
    let duration: Obj<LeanDuration> = unsafe { Obj::new(d) };
    duration.as_ref().0.as_secs()
}

/// Create a duration from a number of seconds
#[unsafe(no_mangle)]
pub extern "C" fn async_effect_ffi_duration_from_secs(s: u64) -> lean_obj_res {
    LeanDuration(Duration::from_secs(s)).into_lean_object()
}

/// Return the number of milliseconds corresponding to a duration
///
/// # Safety
///
/// Callers must ensure that the argument points to a borrowed Lean external
/// object containing a [`LeanDuration`] instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn async_effect_ffi_duration_as_millis(d: b_lean_obj_arg) -> lean_obj_res {
    let duration: Obj<LeanDuration> = unsafe { Obj::new(d) };
    let milliseconds = duration.as_ref().0.as_millis();
    number::u128_to_lean_nat(milliseconds)
}

/// Create a duration from a number of milliseconds
#[unsafe(no_mangle)]
pub extern "C" fn async_effect_ffi_duration_from_millis(millis: u64) -> lean_obj_res {
    LeanDuration(Duration::from_millis(millis)).into_lean_object()
}

/// Subtract two durations
///
/// # Safety
///
/// Callers must ensure that the arguments point to borrowed Lean external
/// objects containing [`LeanDuration`] instances.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn async_effect_ffi_duration_subtract(
    x: b_lean_obj_arg,
    y: b_lean_obj_arg,
) -> lean_obj_res {
    let x_duration: Obj<LeanDuration> = unsafe { Obj::new(x) };
    let y_duration: Obj<LeanDuration> = unsafe { Obj::new(y) };
    let duration = x_duration.as_ref().0 - y_duration.as_ref().0;
    LeanDuration(duration).into_lean_object()
}
