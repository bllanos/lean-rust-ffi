use std::borrow::Borrow;
use std::ffi::CStr;

use lean_sys::{lean_mk_string, lean_string_cstr};

use super::{
    Owner, Reference,
    object::{Obj, Object},
};
use crate::{Minimal, Modules, Runtime};

pub enum StringTypeTag {}

pub type LeanStr = Obj<StringTypeTag>;

impl Obj<StringTypeTag> {
    pub fn as_cstr(&self) -> &CStr {
        unsafe {
            let string = self.as_mut_raw();
            let string_cstring = lean_string_cstr(string);
            CStr::from_ptr(string_cstring)
        }
    }
}

pub type LeanString = Object<StringTypeTag>;

impl Object<StringTypeTag> {
    pub fn from_cstr<R: Minimal, M: Modules, T: AsRef<CStr>>(
        _runtime: &Runtime<R, M>,
        value: T,
    ) -> Self {
        let cstr = value.as_ref();
        let ptr = cstr.as_ptr();
        let object = unsafe { lean_mk_string(ptr) };
        unsafe { Self::new(object) }
    }

    pub fn as_cstr(&self) -> &CStr {
        <Self as Borrow<Obj<_>>>::borrow(self).as_cstr()
    }
}
