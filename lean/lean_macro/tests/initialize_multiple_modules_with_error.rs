use std::sync::atomic::{AtomicUsize, Ordering};

use lean::{MimallocAllocator, combine_lean_module_initializers, create_module_trait};

#[global_allocator]
static ALLOCATOR: MimallocAllocator = MimallocAllocator {};

static GLOBAL_INITIALIZATION_STATE: AtomicUsize = AtomicUsize::new(0);

#[create_module_trait]
enum OneModuleInitializer {}

unsafe impl lean::Modules for OneModuleInitializer {
    unsafe fn initialize_modules(_builtin: u8) -> lean_sys::lean_obj_res {
        GLOBAL_INITIALIZATION_STATE.fetch_add(1, Ordering::Relaxed);
        lean::make_lean_io_result_error(Self::ERROR_MESSAGE)
    }
}

impl OneModuleInitializer {
    const ERROR_MESSAGE: &str = "test user error message";
}

#[create_module_trait]
enum TwoModuleInitializer {}

unsafe impl lean::Modules for TwoModuleInitializer {
    unsafe fn initialize_modules(_builtin: u8) -> lean_sys::lean_obj_res {
        unreachable!()
    }
}

combine_lean_module_initializers! {
    pub BothModules {
        TwoModuleInitializer : TwoModule, // Testing the full syntax
        OneModuleInitializer, // Testing the shorter syntax
    }
}

fn assert_module_initialization_error<T: OneModule + TwoModule>() {
    // Use same default as for Lean executables
    // See <https://github.com/leanprover/lean4/blob/master/doc/dev/ffi.md#initialization>
    let builtin: u8 = 1;

    let error;
    unsafe {
        error = lean::run_lean_io_unit(|| T::initialize_modules(builtin)).unwrap_err();
    }
    assert_eq!(&format!("{error}"), OneModuleInitializer::ERROR_MESSAGE);
}

#[test]
fn initialize_multiple_modules_with_error() {
    assert_module_initialization_error::<BothModules>();
    assert_eq!(GLOBAL_INITIALIZATION_STATE.load(Ordering::Relaxed), 1);
}
