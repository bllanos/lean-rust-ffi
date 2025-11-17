use std::env;
use std::ffi::{OsStr, c_char, c_int};

use lean_sys::lean_setup_args;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("failed to convert program arguments count {argc} to the type required by Lean")]
pub struct ArgcError {
    argc: usize,
    source: <usize as TryInto<c_int>>::Error,
}

struct LeanArgs {
    argc: c_int,
    argv: *const *const c_char,
}

impl LeanArgs {
    pub fn from_args_os<T: AsRef<OsStr>, I: IntoIterator<Item = T> + ExactSizeIterator>(
        args: I,
    ) -> Result<Self, ArgcError>
    where
        <I as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        let iterator = args.into_iter();
        let argc = iterator.len();
        let argc: c_int = argc.try_into().map_err(|error| ArgcError {
            argc,
            source: error,
        })?;
        let argv_buffer = iterator
            .map(|arg| {
                let bytes = arg.as_ref().as_encoded_bytes();
                let mut buffer = Vec::with_capacity(bytes.len() + 1);
                buffer.extend_from_slice(bytes);
                buffer.push(0);
                buffer.leak().as_ptr() as *const c_char
            })
            .collect::<Vec<*const c_char>>();
        let argv = argv_buffer.leak().as_ptr();
        Ok(Self { argc, argv })
    }
}

pub fn call_lean_setup_args() -> Result<(), ArgcError> {
    let LeanArgs { argc, argv } = LeanArgs::from_args_os(env::args_os())?;
    // libuv may take ownership of the pointer
    // Reference: <https://docs.libuv.org/en/v1.x/misc.html#c.uv_setup_args>
    unsafe { lean_setup_args(argc, argv) };
    Ok(())
}
