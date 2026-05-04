use lean::lean_types::{self, Owner, any::LeanAnyObject};
use lean_sys::{lean_obj_arg, lean_object};

/// A Lean object that can be held by [`async_effect::async_io::AsyncIo`]
///
/// In order to implement [`Send`] and [`Sync`], which are required by
/// [`async_effect::async_io::Value`], this type preemptively marks all internal
/// Lean objects as multi-threaded. Consequently, it assumes that it will not be
/// used as a persistent value by Lean code. It also assumes the Lean runtime
/// uses the object safely between threads.
///
/// Ideally, internal Lean objects would be marked as multi-threaded or
/// persistent only on-demand, but this would not satisfy the contracts of
/// [`Send`] and [`Sync`]. It would also require refactoring
/// [`async_effect::async_io::AsyncIo`], which presently requires Lean objects
/// to be stored inside type-erased Rust closures in some cases. Closures cannot
/// provide access to any Lean objects they contain, so Lean objects must be
/// marked as multi-threaded before being moved into the closures.
pub struct InternalLeanObject(LeanAnyObject);

impl InternalLeanObject {
    /// Create an instance that wraps an existing object
    ///
    /// # Safety
    ///
    /// 1. `obj` has an associated reference counting token
    /// 2. `obj` points to the same object for the lifetime of the new instance
    /// 3. `obj` is not currently marked as persistent and will not need to be
    ///    marked as persistent in the future
    pub unsafe fn new(obj: lean_obj_arg) -> Self {
        Self(unsafe { lean_types::new_multi_threaded(obj) })
    }

    pub fn into_raw(self) -> *mut lean_object {
        self.0.into_raw()
    }
}

unsafe impl Send for InternalLeanObject {}

unsafe impl Sync for InternalLeanObject {}

impl Clone for InternalLeanObject {
    fn clone(&self) -> Self {
        Self(self.0.share())
    }
}
