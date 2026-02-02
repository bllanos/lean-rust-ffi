// Force linking of these crates even though there are no direct reference to
// them from Rust code. They are dependencies of Lean libraries.
// TODO Move these dependencies to a `sleeper-sys` crate that wraps the Lean Sleeper library, and depend on `sleeper-sys` here.
extern crate async_effect_ffi;
extern crate async_effect_sys;
