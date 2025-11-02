use std::sync::atomic::{AtomicUsize, Ordering};

use lean::{MimallocAllocator, combine_lean_module_initializers, create_module_trait};

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
        unsafe { lean_sys::lean_io_result_mk_ok(lean_sys::lean_box(0)) }
    }
}

#[create_module_trait]
enum TwoModuleInitializer {}

unsafe impl lean::Modules for TwoModuleInitializer {
    unsafe fn initialize_modules(
        _builtin: u8,
        _lean_io_world: lean_sys::lean_obj_arg,
    ) -> lean_sys::lean_obj_res {
        GLOBAL_INITIALIZATION_STATE.fetch_add(2, Ordering::Relaxed);
        unsafe { lean_sys::lean_io_result_mk_ok(lean_sys::lean_box(0)) }
    }
}

combine_lean_module_initializers! {
    pub BothModules {
        TwoModuleInitializer : TwoModule,
        OneModuleInitializer : OneModule,
    }
}

fn assert_initializes_both_modules<T: OneModule + TwoModule>() {
    let res: *mut lean_sys::lean_object;
    // Use same default as for Lean executables
    // See <https://github.com/leanprover/lean4/blob/master/doc/dev/ffi.md#initialization>
    let builtin: u8 = 1;

    unsafe {
        res = T::initialize_modules(builtin, lean_sys::lean_io_mk_world());
        assert!(lean_sys::lean_io_result_is_ok(res));
        lean_sys::lean_dec(res);
    }
}

#[test]
fn initialize_multiple_modules() {
    assert_initializes_both_modules::<BothModules>();
    assert_eq!(GLOBAL_INITIALIZATION_STATE.load(Ordering::Relaxed), 3);
}
