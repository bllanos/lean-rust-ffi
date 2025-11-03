use std::borrow::Borrow;
use std::iter::FusedIterator;
use std::mem;
use std::slice;

use lean_sys::{lean_alloc_sarray, lean_sarray_cptr, lean_sarray_size};

use super::{
    Owner, Reference,
    object::{Obj, Object},
};
use crate::{Minimal, Modules, Runtime};

pub enum ByteArrayTypeTag {}

impl Obj<ByteArrayTypeTag> {
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let array = self.as_mut_raw();
            let array_data = lean_sarray_cptr(array).cast_const();
            let arr_size = lean_sarray_size(array);
            slice::from_raw_parts(array_data, arr_size)
        }
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = u8> + DoubleEndedIterator + FusedIterator {
        self.as_slice().iter().copied()
    }
}

impl Object<ByteArrayTypeTag> {
    pub fn from_exact_size_iterator<
        R: Minimal,
        M: Modules,
        T: Into<u8>,
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
        let object = unsafe { lean_alloc_sarray(mem::size_of::<u8>() as u32, len, len) };
        let arr_data = unsafe { lean_sarray_cptr(object) };

        for (i, element) in iterator.enumerate() {
            unsafe {
                *(arr_data.add(i)) = element.into();
            }
        }

        unsafe { Self::new(object) }
    }

    pub fn as_slice(&self) -> &[u8] {
        <Self as Borrow<Obj<_>>>::borrow(self).as_slice()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = u8> + DoubleEndedIterator + FusedIterator {
        <Self as Borrow<Obj<_>>>::borrow(self).iter()
    }
}

pub type ByteArr = Obj<ByteArrayTypeTag>;
pub type ByteArray = Object<ByteArrayTypeTag>;
