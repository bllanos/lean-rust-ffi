use std::convert::Infallible;
use std::error::Error;
use std::ffi::CString;
use std::str::FromStr;

use lean::{
    LeanError, LeanInitializationError, LeanIoError, MimallocAllocator, MinimalComponents, Modules,
};
use lean_sys::{lean_io_result_mk_error, lean_mk_io_user_error, lean_mk_string, lean_obj_res};

#[global_allocator]
static ALLOCATOR: MimallocAllocator = MimallocAllocator {};

enum TestModule {}

impl TestModule {
    const ERROR_MESSAGE: &str = "test user error message";
}

unsafe impl Modules for TestModule {
    unsafe fn initialize_modules(_builtin: u8) -> lean_obj_res {
        let cstring = CString::from_str(Self::ERROR_MESSAGE).unwrap();
        let cstring_ptr = cstring.as_ptr();

        unsafe {
            let lean_string = lean_mk_string(cstring_ptr);
            let lean_io_error = lean_mk_io_user_error(lean_string);
            lean_io_result_mk_error(lean_io_error)
        }
    }
}

#[test]
fn module_initialization_error() {
    let error = lean::run_in_lean_runtime_with_default_error_handler::<
        MinimalComponents,
        TestModule,
        _,
        _,
        _,
    >(|_runtime| -> Result<(), Infallible> { unreachable!() })
    .unwrap_err();

    let cstring = CString::from_str(TestModule::ERROR_MESSAGE).unwrap();
    assert_eq!(
        error,
        LeanError::Initialization(LeanInitializationError::ModulesInitialization(LeanIoError(
            cstring
        )))
    );
    assert_eq!(&format!("{error}"), "initialization error");
    let mut source = error.source().unwrap();
    assert_eq!(&format!("{source}"), "Lean modules initialization error");
    source = source.source().unwrap();
    assert_eq!(&format!("{source}"), TestModule::ERROR_MESSAGE);
    assert!(source.source().is_none());
}
