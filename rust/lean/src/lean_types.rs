use std::borrow::Borrow;

use lean_sys::{b_lean_obj_arg, lean_obj_arg, lean_object};

pub mod array;
pub mod byte_array;
pub mod float_array;
pub mod object;
pub mod string;

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
    /// Callers must ensure that `obj`` has an associated reference counting
    /// token, points to the same object for the lifetime of the new instance, and
    /// that the object is of the correct type.
    unsafe fn new(obj: lean_obj_arg) -> Self;

    /// Transfers this object's reference counting token to the caller
    fn into_raw(self) -> *mut lean_object;

    /// Create a new owning reference to the same Lean object
    fn share(&self) -> Self;
}
