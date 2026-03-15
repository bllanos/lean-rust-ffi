use lean::lean_types::{Owner, object::Object};
use lean_sys::lean_obj_arg;

use async_effect::sleep::Sleep;

use crate::time::LeanDuration;

/// Construct a [`Sleep`] instance from a Lean `Sleep` instance
///
/// # Safety
///
/// Callers must ensure that:
/// 1. `instance` has an associated reference counting token
/// 2. `instance` is a Lean external object containing a Rust
///    [`crate::time::LeanDuration`] object. (Lean `Sleep` objects compile to
///    this representation.)
#[unsafe(no_mangle)]
pub unsafe fn sleep_from_lean_sleep(instance: lean_obj_arg) -> Sleep {
    let duration: Object<LeanDuration> = unsafe { Object::new(instance) };
    Sleep::new(duration.as_ref().0)
}
