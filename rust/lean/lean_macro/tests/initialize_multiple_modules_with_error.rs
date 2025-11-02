use std::ffi::CString;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};

use lean::{LeanIoError, MimallocAllocator, combine_lean_module_initializers, create_module_trait};

#[global_allocator]
static ALLOCATOR: MimallocAllocator = MimallocAllocator {};

static GLOBAL_INITIALIZATION_STATE: AtomicUsize = AtomicUsize::new(0);

#[create_module_trait]
enum OneModuleInitializer {}

unsafe impl lean::Modules for OneModuleInitializer {
    unsafe fn initialize_modules(
        _builtin: u8,
        _lean_io_world: lean_sys::lean_obj_arg,
    ) -> lean_sys::lean_obj_res {
        GLOBAL_INITIALIZATION_STATE.fetch_add(1, Ordering::Relaxed);
        let cstring = CString::from_str(Self::ERROR_MESSAGE).unwrap();
        let cstring_ptr = cstring.as_ptr();

        unsafe {
            let lean_string = lean_sys::lean_mk_string(cstring_ptr);
            let lean_io_error = lean_sys::lean_mk_io_user_error(lean_string);
            lean_sys::lean_io_result_mk_error(lean_io_error)
        }
    }
}

impl OneModuleInitializer {
    const ERROR_MESSAGE: &str = "test user error message";
}

#[create_module_trait]
enum TwoModuleInitializer {}

unsafe impl lean::Modules for TwoModuleInitializer {
    unsafe fn initialize_modules(
        _builtin: u8,
        _lean_io_world: lean_sys::lean_obj_arg,
    ) -> lean_sys::lean_obj_res {
        unreachable!()
    }
}

combine_lean_module_initializers! {
    pub BothModules {
        TwoModuleInitializer : TwoModule,
        OneModuleInitializer : OneModule,
    }
}

fn assert_module_initialization_error<T: OneModule + TwoModule>() {
    let res: *mut lean_sys::lean_object;
    // Use same default as for Lean executables
    // See <https://github.com/leanprover/lean4/blob/master/doc/dev/ffi.md#initialization>
    let builtin: u8 = 1;

    let error;
    unsafe {
        res = T::initialize_modules(builtin, lean_sys::lean_io_mk_world());
        assert!(!lean_sys::lean_io_result_is_ok(res));
        error = LeanIoError::from_lean_io_result(res);
        lean_sys::lean_dec(res);
    }
    assert_eq!(&format!("{error}"), OneModuleInitializer::ERROR_MESSAGE);
}

#[test]
fn initialize_multiple_modules_with_error() {
    assert_module_initialization_error::<BothModules>();
    assert_eq!(GLOBAL_INITIALIZATION_STATE.load(Ordering::Relaxed), 1);
}
