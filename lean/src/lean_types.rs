use std::borrow::Borrow;

use lean_sys::{b_lean_obj_arg, lean_mark_mt, lean_mark_persistent, lean_obj_arg, lean_object};

pub mod any;
pub mod array;
pub mod byte_array;
pub mod external;
pub mod float_array;
pub mod object;
pub mod string;
pub mod unit;

/// A trait implemented by types that point to immutable Lean objects
///
/// # Safety
///
/// Implementations of this trait must guarantee that the Lean objects are never
/// mutated when used exclusively through their methods.
pub unsafe trait Reference {
    /// # Safety
    ///
    /// Callers must ensure that the object is not mutated
    unsafe fn as_mut_raw(&self) -> *mut lean_object;

    fn as_raw(&self) -> *const lean_object {
        unsafe { self.as_mut_raw() }.cast_const()
    }
}

/// A trait implemented by types that borrow immutable Lean objects
///
///
/// See [Lean's FFI
/// documentation](https://github.com/leanprover/lean4/blob/master/doc/dev/ffi.md#borrowing)
/// for more information.
///
/// # Safety
///
/// Implementations of this trait must guarantee that the Lean objects are never
/// mutated when used exclusively through their methods.
pub unsafe trait Borrower: Reference + ToOwned {
    /// Create an instance that wraps an existing object
    ///
    /// # Safety
    ///
    /// Callers must ensure that `obj` points to the same borrowed object for the
    /// lifetime of the new instance and that the object is of the correct type.
    unsafe fn new(obj: b_lean_obj_arg) -> Self;
}

/// A trait implemented by types that own immutable Lean objects
///
/// Objects are not owned in the Rust sense of the word, but in the Lean sense
/// of the word. Ownership in Lean means that when the pointer to the object is
/// no longer needed, the reference count associated with the object it points
/// to must be decremented.
///
/// See [Lean's FFI
/// documentation](https://github.com/leanprover/lean4/blob/master/doc/dev/ffi.md#borrowing)
/// for more information.
///
/// # Safety
///
/// Implementations of this trait must guarantee that the Lean objects are never
/// mutated when used exclusively through their methods.
pub unsafe trait Owner<T: Borrower>: Reference + Borrow<T> {
    /// Create an instance that wraps an existing object
    ///
    /// # Safety
    ///
    /// Callers must ensure that `obj` has an associated reference counting
    /// token, points to the same object for the lifetime of the new instance,
    /// and that the object is of the correct type.
    unsafe fn new(obj: lean_obj_arg) -> Self;

    /// Transfers this object's reference counting token to the caller
    fn into_raw(self) -> *mut lean_object;

    /// Create a new owning reference to the same Lean object
    fn share(&self) -> Self;
}

/// Create an instance of an [`Owner`] that wraps an existing object and mark
/// the object as multi-threaded
///
/// # Safety
///
/// Callers must ensure that:
///
/// 1. `obj` has an associated reference counting token
/// 2. `obj` points to the same object for the lifetime of the new instance
/// 3. `obj` is of the correct type
/// 4. `obj` is not currently marked as persistent
pub unsafe fn new_multi_threaded<B: Borrower, T: Owner<B>>(obj: lean_obj_arg) -> T {
    unsafe {
        // This function is assumed to check if the object is already multi-threaded
        // See [`object.cpp`](https://github.com/leanprover/lean4/blob/ec565f3bf7a3985b6b8592f5cb5fa063b86a0ecf/src/runtime/object.cpp#L596)
        lean_mark_mt(obj);
        T::new(obj)
    }
}

/// Create an instance of an [`Owner`] that wraps an existing object and mark
/// the object as persistent
///
/// # Safety
///
/// Callers must ensure that:
///
/// 1. `obj` has an associated reference counting token
/// 2. `obj` points to the same object for the lifetime of the new instance
/// 3. `obj` is of the correct type
/// 4. It is appropriate for `obj` to be marked as persistent
pub unsafe fn new_persistent<B: Borrower, T: Owner<B>>(obj: lean_obj_arg) -> T {
    unsafe {
        // This function is assumed to check if the object is already persistent
        // See [`object.cpp`]https://github.com/leanprover/lean4/blob/ec565f3bf7a3985b6b8592f5cb5fa063b86a0ecf/src/runtime/object.cpp#L518)
        lean_mark_persistent(obj);
        T::new(obj)
    }
}
