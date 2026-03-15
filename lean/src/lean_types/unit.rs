use lean_sys::{lean_box, lean_obj_res};

pub fn make_lean_unit() -> lean_obj_res {
    unsafe { lean_box(0) }
}
