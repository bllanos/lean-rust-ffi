use std::borrow::Borrow;
use std::error::Error;
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::slice;

use lean_sys::{
    b_lean_obj_arg, lean_alloc_array, lean_array_cptr, lean_array_size, lean_box_float,
    lean_box_float32, lean_box_uint32, lean_box_uint64, lean_box_usize, lean_object,
    lean_unbox_float, lean_unbox_float32, lean_unbox_uint32, lean_unbox_uint64, lean_unbox_usize,
};

use super::{
    Owner, Reference,
    object::{Obj, Object},
};
use crate::{Minimal, Modules, Runtime};

/// A trait that array types use to convert Lean array elements to and from Rust
/// types
///
/// # Safety
///
/// Implementations of this trait must not mutate array elements nor allow array
/// elements to be mutated by external code.
pub unsafe trait LeanArrayTypeTag {
    type Input;
    type Output;

    /// Create an owned Lean object (or a scalar) to be stored in an array
    fn into_element(input: Self::Input) -> *mut lean_object;

    /// View a borrowed Lean array element as the output type
    ///
    /// # Safety
    ///
    /// `element` must point to a valid object of the expected type, and the
    /// object must live as long as the return value.
    unsafe fn from_element(element: b_lean_obj_arg) -> Self::Output;
}

impl<TypeTag: LeanArrayTypeTag> Obj<TypeTag> {
    fn as_slice(&self) -> &[*mut lean_object] {
        unsafe {
            let array = self.as_mut_raw();
            let array_data = lean_array_cptr(array).cast_const();
            let arr_size = lean_array_size(array);
            slice::from_raw_parts(array_data, arr_size)
        }
    }

    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = TypeTag::Output> + DoubleEndedIterator + FusedIterator {
        self.as_slice()
            .iter()
            .map(|item| unsafe { TypeTag::from_element(*item) })
    }
}

impl<TypeTag: LeanArrayTypeTag> Object<TypeTag> {
    pub fn from_exact_size_iterator<
        R: Minimal,
        M: Modules,
        T: Into<TypeTag::Input>,
        I: IntoIterator<Item = T>,
    >(
        _runtime: &Runtime<R, M>,
        data: I,
    ) -> Self
    where
        <I as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        let iterator = data.into_iter();
        let len = iterator.len();
        let object = unsafe { lean_alloc_array(len, len) };
        let arr_data = unsafe { lean_array_cptr(object) };

        for (i, element) in iterator.enumerate() {
            unsafe {
                *(arr_data.add(i)) = TypeTag::into_element(element.into());
            }
        }

        unsafe { Self::new(object) }
    }

    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = TypeTag::Output> + DoubleEndedIterator + FusedIterator {
        <Self as Borrow<Obj<_>>>::borrow(self).iter()
    }
}

pub enum U32ArrayTypeTag {}

unsafe impl LeanArrayTypeTag for U32ArrayTypeTag {
    type Input = u32;
    type Output = Self::Input;

    fn into_element(input: Self::Input) -> *mut lean_object {
        unsafe { lean_box_uint32(input) }
    }

    unsafe fn from_element(element: b_lean_obj_arg) -> Self::Output {
        unsafe { lean_unbox_uint32(element) }
    }
}

pub type U32Arr = Obj<U32ArrayTypeTag>;
pub type U32Array = Object<U32ArrayTypeTag>;

pub struct Integer32ArrayTypeTag<T: Into<i32> + TryFrom<i32>>(PhantomData<T>)
where
    <T as TryFrom<i32>>::Error: Error;

unsafe impl<T: Into<i32> + TryFrom<i32>> LeanArrayTypeTag for Integer32ArrayTypeTag<T>
where
    <T as TryFrom<i32>>::Error: Error,
{
    type Input = T;
    type Output = Self::Input;

    fn into_element(input: Self::Input) -> *mut lean_object {
        let u32_value = input.into() as u32;
        unsafe { lean_box_uint32(u32_value) }
    }

    unsafe fn from_element(element: b_lean_obj_arg) -> Self::Output {
        let u32_value = unsafe { lean_unbox_uint32(element) };
        (u32_value as i32).try_into().unwrap()
    }
}

pub type Integer32Arr<T> = Obj<Integer32ArrayTypeTag<T>>;
pub type Integer32Array<T> = Object<Integer32ArrayTypeTag<T>>;

pub enum U64ArrayTypeTag {}

unsafe impl LeanArrayTypeTag for U64ArrayTypeTag {
    type Input = u64;
    type Output = Self::Input;

    fn into_element(input: Self::Input) -> *mut lean_object {
        unsafe { lean_box_uint64(input) }
    }

    unsafe fn from_element(element: b_lean_obj_arg) -> Self::Output {
        unsafe { lean_unbox_uint64(element) }
    }
}

pub type U64Arr = Obj<U64ArrayTypeTag>;
pub type U64Array = Object<U64ArrayTypeTag>;

pub enum Integer64ArrayTypeTag {}

unsafe impl LeanArrayTypeTag for Integer64ArrayTypeTag {
    type Input = i64;
    type Output = Self::Input;

    fn into_element(input: Self::Input) -> *mut lean_object {
        let u64_value = input as u64;
        unsafe { lean_box_uint64(u64_value) }
    }

    unsafe fn from_element(element: b_lean_obj_arg) -> Self::Output {
        let u64_value = unsafe { lean_unbox_uint64(element) };
        u64_value.try_into().unwrap()
    }
}

pub type Integer64Arr = Obj<Integer64ArrayTypeTag>;
pub type Integer64Array = Object<Integer64ArrayTypeTag>;

pub enum UsizeArrayTypeTag {}

unsafe impl LeanArrayTypeTag for UsizeArrayTypeTag {
    type Input = usize;
    type Output = Self::Input;

    fn into_element(input: Self::Input) -> *mut lean_object {
        unsafe { lean_box_usize(input) }
    }

    unsafe fn from_element(element: b_lean_obj_arg) -> Self::Output {
        unsafe { lean_unbox_usize(element) }
    }
}

pub type UsizeArr = Obj<UsizeArrayTypeTag>;
pub type UsizeArray = Object<UsizeArrayTypeTag>;

pub enum F32ArrayTypeTag {}

unsafe impl LeanArrayTypeTag for F32ArrayTypeTag {
    type Input = f32;
    type Output = Self::Input;

    fn into_element(input: Self::Input) -> *mut lean_object {
        unsafe { lean_box_float32(input) }
    }

    unsafe fn from_element(element: b_lean_obj_arg) -> Self::Output {
        unsafe { lean_unbox_float32(element) }
    }
}

pub type F32Arr = Obj<F32ArrayTypeTag>;
pub type F32Array = Object<F32ArrayTypeTag>;

pub enum F64ArrayTypeTag {}

unsafe impl LeanArrayTypeTag for F64ArrayTypeTag {
    type Input = f64;
    type Output = Self::Input;

    fn into_element(input: Self::Input) -> *mut lean_object {
        unsafe { lean_box_float(input) }
    }

    unsafe fn from_element(element: b_lean_obj_arg) -> Self::Output {
        unsafe { lean_unbox_float(element) }
    }
}

pub type F64Arr = Obj<F64ArrayTypeTag>;
pub type F64Array = Object<F64ArrayTypeTag>;
