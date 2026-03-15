use core::ffi::c_void;

use lean::lean_types::{
    Owner,
    external::{self, ExternalClass, ExternalClassHolder, LeanExternalTypeTag},
    object::Object,
};
use lean_sys::{b_lean_obj_arg, lean_apply_1, lean_obj_arg, lean_obj_res};

use async_effect::async_io::AsyncIo;

mod internal_lean_object;

use internal_lean_object::InternalLeanObject;

/// An [`AsyncIo`] that holds a Lean object
///
/// This type assumes that it will not be used as a persistent value by Lean
/// code, because it marks Lean objects as multi-threaded.
// The inner field must be [`Some`] at all times (except during destruction). It
// is an [`Option`] only because [`AsyncIo`] methods take `self` by value
// whereas the Lean runtime always passes external objects to functions by
// reference.
#[derive(Clone, LeanExternalTypeTag)]
pub struct LeanAsyncIo(Option<AsyncIo<InternalLeanObject>>);

impl LeanAsyncIo {
    /// Create an instance that wraps an existing object
    ///
    /// # Safety
    ///
    /// 1. `obj` has an associated reference counting token
    /// 2. `obj` points to the same object for the lifetime of the new instance
    /// 3. `obj` is not currently marked as persistent and will not need to be
    ///    marked as persistent in the future
    pub unsafe fn pure(obj: lean_obj_arg) -> Self {
        let internal_object = unsafe { InternalLeanObject::new(obj) };
        Self(Some(AsyncIo::pure(internal_object)))
    }

    fn take_inner(&mut self) -> Option<AsyncIo<InternalLeanObject>> {
        self.0.take()
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn async_effect_ffi_lean_async_io_finalize(instance: *mut c_void) {
    unsafe { external::finalize::<LeanAsyncIo>(instance) };
}

#[unsafe(no_mangle)]
unsafe extern "C" fn async_effect_ffi_lean_async_io_foreach(
    instance: *mut c_void,
    f: b_lean_obj_arg,
) {
    unsafe { external::foreach::<LeanAsyncIo>(instance, f) };
}

static ASYNC_IO_EXTERNAL_CLASS: ExternalClass<LeanAsyncIo> = ExternalClass::new(
    async_effect_ffi_lean_async_io_finalize,
    async_effect_ffi_lean_async_io_foreach,
);

impl ExternalClassHolder for LeanAsyncIo {
    fn get_external_class() -> &'static ExternalClass<Self> {
        &ASYNC_IO_EXTERNAL_CLASS
    }
}

/// Destructively update a [`LeanAsyncIo`]
///
/// # Safety
///
/// Callers must ensure that:
/// 1. `instance` has an associated reference counting token
/// 2. `instance` is a Lean external object containing a [`LeanAsyncIo`] object
unsafe fn clone_on_write_async_io<
    F: FnOnce(AsyncIo<InternalLeanObject>) -> AsyncIo<InternalLeanObject>,
>(
    f: F,
    instance: lean_obj_arg,
) -> lean_obj_res {
    let mut instance_object: Object<LeanAsyncIo> = unsafe { Object::new(instance) };
    let mapped_object = instance_object
        .clone_on_write(move |lean_async_io| lean_async_io.0 = lean_async_io.take_inner().map(f))
        .unwrap_or(instance_object);
    mapped_object.into_raw()
}

/// Destructively extract the inner [`AsyncIo`] of a [`LeanAsyncIo`]
///
/// # Safety
///
/// Callers must ensure that:
/// 1. `instance` has an associated reference counting token
/// 2. `instance` is a Lean external object containing a [`LeanAsyncIo`] object
unsafe fn clone_on_take_async_io(instance: lean_obj_arg) -> AsyncIo<InternalLeanObject> {
    let mut instance_object: Object<LeanAsyncIo> = unsafe { Object::new(instance) };
    let mut inner_object = None;
    // Allow the Lean object's reference count to decrease
    _ = instance_object.clone_on_write(|lean_async_io| inner_object = lean_async_io.take_inner());
    inner_object.unwrap()
}

/// Create an instance storing a value
///
/// # Safety
///
/// See [`LeanAsyncIo::pure`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn async_effect_ffi_async_io_pure(obj: lean_obj_arg) -> lean_obj_res {
    unsafe { LeanAsyncIo::pure(obj) }.into_lean_object()
}

/// Map over the internal value
///
/// # Safety
///
/// Callers must ensure that:
/// 1. The arguments have associated reference counting tokens
/// 2. `f` is a Lean closure that accepts a single argument of the type of value
///    currently stored by `instance`
/// 3. `instance` is a Lean external object containing a [`LeanAsyncIo`]
///    object
#[unsafe(no_mangle)]
pub unsafe extern "C" fn async_effect_ffi_async_io_map(
    f: lean_obj_arg,
    instance: lean_obj_arg,
) -> lean_obj_res {
    unsafe {
        let safe_f = InternalLeanObject::new(f);
        clone_on_write_async_io(
            move |async_io| {
                async_io.map(move |value| {
                    let res = lean_apply_1(safe_f.clone().into_raw(), value.into_raw());
                    InternalLeanObject::new(res)
                })
            },
            instance,
        )
    }
}

/// Monadic bind
///
/// # Safety
///
/// Callers must ensure that:
/// 1. The arguments have associated reference counting tokens
/// 2. `instance` is a Lean external object containing a [`LeanAsyncIo`] object
/// 3. `f` is a Lean closure that accepts a single argument of the type of value
///    currently stored by `instance` and returns a Lean external object
///    containing a [`LeanAsyncIo`] object
#[unsafe(no_mangle)]
pub unsafe extern "C" fn async_effect_ffi_async_io_bind(
    instance: lean_obj_arg,
    f: lean_obj_arg,
) -> lean_obj_res {
    unsafe {
        let safe_f = InternalLeanObject::new(f);
        clone_on_write_async_io(
            move |async_io| {
                async_io.bind(move |value| {
                    let res = lean_apply_1(safe_f.clone().into_raw(), value.into_raw());
                    clone_on_take_async_io(res)
                })
            },
            instance,
        )
    }
}
