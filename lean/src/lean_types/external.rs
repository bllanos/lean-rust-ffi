use core::ffi::c_void;

use std::borrow::Borrow;
use std::marker::PhantomData;

use lean_sys::{
    b_lean_obj_arg, lean_alloc_external, lean_apply_1, lean_dec, lean_external_class,
    lean_get_external_data, lean_inc, lean_is_exclusive, lean_obj_res,
};

use super::{
    Owner, Reference,
    object::{Obj, Object},
};

#[cfg(feature = "lean-derive")]
pub use lean_derive::LeanExternalTypeTag;

/// Traits required to use objects safely from arbitrary Lean code
///
/// Lean programs may use objects from multiple threads.
pub trait ExternalValue: 'static + Sized + Send + Sync {}

impl<T: 'static + Sized + Send + Sync> ExternalValue for T {}

/// A trait that allows Lean external objects to be allocated for a type
///
/// # Safety
///
/// Implementations of this trait must safely manage memory when iterating over
/// any contained Lean objects.
pub unsafe trait LeanExternalTypeTag: ExternalValue {
    /// An iterator over any Lean objects stored by the object
    ///
    /// The items returned by the iterator are assumed to be borrowed objects.
    type InternalLeanObjectIterator: Iterator<Item = b_lean_obj_arg>;

    /// Return an iterator over any Lean objects stored by the object
    ///
    /// The Lean runtime will use this function to mark the internal Lean
    /// objects as multi-threaded or persistent for the purpose of reference
    /// counting. See
    /// [`lean.h`](https://github.com/leanprover/lean4/blob/ec565f3bf7a3985b6b8592f5cb5fa063b86a0ecf/src/include/lean/lean.h#L115)
    /// for documentation on reference counting and
    /// [`object.cpp`](https://github.com/leanprover/lean4/blob/ec565f3bf7a3985b6b8592f5cb5fa063b86a0ecf/src/runtime/object.cpp)
    /// for uses of iterators over internal Lean objects (`m_foreach`).
    fn iter(&self) -> Self::InternalLeanObjectIterator;
}

pub struct ExternalClass<T: LeanExternalTypeTag>(lean_external_class, PhantomData<T>);

impl<T: LeanExternalTypeTag> ExternalClass<T> {
    pub const fn new(
        finalize: unsafe extern "C" fn(instance: *mut c_void),
        foreach: unsafe extern "C" fn(instance: *mut c_void, f: b_lean_obj_arg),
    ) -> Self {
        Self(
            lean_external_class {
                m_finalize: Some(finalize),
                m_foreach: Some(foreach),
            },
            PhantomData,
        )
    }
}

/// A trait ensuring that a type has a Lean external class
///
/// Lean's source code currently only has examples where external classes are
/// stored as global variables. It seems like registering them with the Lean
/// runtime using [`lean_sys::lean_register_external_class()`] is unnecessary as
/// the list of registered classes seems to be present only for memory
/// management purposes. Furthermore, external classes registered by this
/// function are deallocated when the runtime is finalized, which leaves the
/// global variables with dangling pointers. Refer to how the list of external
/// classes is used in
/// [object.cpp](https://github.com/leanprover/lean4/blob/4a9a3eaf6bed9840cb0c04fcff84a616489224e4/src/runtime/object.cpp#L2732)
/// for details.
///
/// Therefore, this trait assumes that Lean external classes are statically
/// allocated. Static allocation eliminates lifetime and memory management
/// concerns that may otherwise be addressed by incurring runtime overhead.
pub trait ExternalClassHolder: LeanExternalTypeTag {
    fn get_external_class() -> &'static ExternalClass<Self>;

    fn into_lean_object(self) -> lean_obj_res {
        Object::from(self).into_raw()
    }
}

impl<T: LeanExternalTypeTag> AsRef<T> for Obj<T> {
    fn as_ref(&self) -> &T {
        unsafe {
            &*(lean_get_external_data(self.as_mut_raw())
                .cast::<T>()
                .cast_const())
        }
    }
}

impl<T: LeanExternalTypeTag> AsRef<T> for Object<T> {
    fn as_ref(&self) -> &T {
        <Object<T> as Borrow<Obj<T>>>::borrow(&self).as_ref()
    }
}

impl<T: ExternalClassHolder> From<T> for Object<T> {
    fn from(instance: T) -> Self {
        let boxed_instance = Box::new(instance);
        unsafe {
            let object = lean_alloc_external(
                (&T::get_external_class().0 as *const lean_external_class).cast_mut(),
                Box::into_raw(boxed_instance).cast(),
            );
            Self::new(object)
        }
    }
}

impl<T: Clone + ExternalClassHolder> Obj<T> {
    pub fn clone_inner(&self) -> T {
        self.as_ref().clone()
    }
}

impl<T: Clone + ExternalClassHolder> Object<T> {
    pub fn clone_inner(&self) -> T {
        self.as_ref().clone()
    }

    /// Destructive update
    ///
    /// This function mutates the inner Rust object with `f` and returns
    /// [`None`] if the object is exclusively owned. Otherwise, it mutates a
    /// clone of the object and returns the clone wrapped with [`Some`].
    pub fn clone_on_write<F: for<'a> FnOnce(&'a mut T)>(&mut self, f: F) -> Option<Self> {
        let inner_lean_object = unsafe { self.as_mut_raw() };
        let is_exclusive = unsafe { lean_is_exclusive(inner_lean_object) };
        if is_exclusive {
            let inner_rust_object =
                unsafe { &mut *(lean_get_external_data(inner_lean_object).cast::<T>()) };
            f(inner_rust_object);
            None
        } else {
            let mut cloned_rust_object = self.clone_inner();
            f(&mut cloned_rust_object);
            Some(cloned_rust_object.into())
        }
    }
}

/// Destroy an object inside a Lean external object
///
/// This function is intended to be used to construct [`ExternalClass<T>`]
/// instances.
///
/// # Safety
///
/// The argument must be an owned pointer to type `T` created using
/// [`Box::into_raw`]. Use [`ExternalClassHolder::into_lean_object()`] to
/// satisfy this condition.
pub unsafe fn finalize<T: LeanExternalTypeTag>(instance: *mut c_void) {
    drop(unsafe { Box::from_raw(instance.cast::<T>()) });
}

/// Iterate over the Lean objects inside a Lean external object
///
/// This function is intended to be used to construct [`ExternalClass<T>`]
/// instances.
///
/// # Safety
///
/// `instance` must be an owned pointer to type `T`.
pub unsafe fn foreach<T: LeanExternalTypeTag>(instance: *mut c_void, f: b_lean_obj_arg) {
    let instance: &T = unsafe { &*(instance.cast::<T>().cast_const()) };
    let iterator = instance.iter();
    for lean_object in iterator {
        unsafe {
            lean_inc(f);
            // It is not clear whether the closure accepts a borrowed or owned
            // object as its argument. Lean's source code seems to treat the
            // argument as borrowed, for example
            // [`tcp.cpp`](https://github.com/leanprover/lean4/blob/2f8c85af89b1afdb03cdb15005524886e18be414/src/runtime/uv/tcp.cpp#L60),
            // but there are cases where it looks like it is owned, specifically
            // [`lean_mark_mt()`](https://github.com/leanprover/lean4/blob/2f8c85af89b1afdb03cdb15005524886e18be414/src/runtime/object.cpp#L580)
            let res = lean_apply_1(f, lean_object);
            lean_dec(res);
        }
    }
}
