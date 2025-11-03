use std::borrow::Borrow;
use std::marker::PhantomData;

use lean_sys::{b_lean_obj_arg, lean_dec, lean_inc, lean_obj_arg, lean_object};

use super::{Borrower, Owner, Reference};

/// A borrowed Lean object
///
/// This type behaves like a Rust reference to a Lean object.
pub struct Obj<TypeTag>(*mut lean_object, PhantomData<TypeTag>);

impl<TypeTag> ToOwned for Obj<TypeTag> {
    type Owned = Object<TypeTag>;

    fn to_owned(&self) -> Self::Owned {
        unsafe {
            lean_inc(self.0);
            Object(Self(self.0, PhantomData))
        }
    }
}

unsafe impl<TypeTag> Reference for Obj<TypeTag> {
    unsafe fn as_mut_raw(&self) -> *mut lean_object {
        self.0
    }
}

unsafe impl<TypeTag> Borrower for Obj<TypeTag> {
    unsafe fn new(obj: b_lean_obj_arg) -> Self {
        Self(obj, PhantomData)
    }
}

/// An owned Lean object
///
/// This type behaves like a shared pointer to a Lean object. The object may
/// have owners besides instances of this type unless the caller can guarantee
/// otherwise.
pub struct Object<TypeTag>(Obj<TypeTag>);

impl<TypeTag> Borrow<Obj<TypeTag>> for Object<TypeTag> {
    fn borrow(&self) -> &Obj<TypeTag> {
        &self.0
    }
}

unsafe impl<TypeTag> Reference for Object<TypeTag> {
    unsafe fn as_mut_raw(&self) -> *mut lean_object {
        unsafe { self.0.as_mut_raw() }
    }
}

unsafe impl<TypeTag> Owner<Obj<TypeTag>> for Object<TypeTag> {
    unsafe fn new(obj: lean_obj_arg) -> Self {
        Self(unsafe { Obj::new(obj) })
    }

    fn into_raw(self) -> *mut lean_object {
        let ptr = unsafe { self.as_mut_raw() };
        std::mem::forget(self);
        ptr
    }

    fn share(&self) -> Self {
        self.0.to_owned()
    }
}

impl<TypeTag> Drop for Object<TypeTag> {
    fn drop(&mut self) {
        unsafe { lean_dec(self.as_mut_raw()) };
    }
}
