use std::alloc::{GlobalAlloc, Layout};
use std::ffi::c_void;

use lean_sys::{mi_aligned_alloc, mi_free_size_aligned, mi_realloc_aligned, mi_zalloc_aligned};

pub struct MimallocAllocator {}

// Reference:
// <https://github.com/purpleprotocol/mimalloc_rust/blob/000709797d05324e449739ab428180cbe1199712/src/lib.rs>
unsafe impl GlobalAlloc for MimallocAllocator {
    // Required trait methods

    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { mi_aligned_alloc(layout.align(), layout.size()) as *mut u8 }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            mi_free_size_aligned(ptr as *mut c_void, layout.size(), layout.align());
        }
    }

    // Provided trait methods that mimalloc also supports

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { mi_zalloc_aligned(layout.size(), layout.align()) as *mut u8 }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { mi_realloc_aligned(ptr as *mut c_void, new_size, layout.align()) as *mut u8 }
    }
}
