use super::object::{Obj, Object};

/// A type tag representing a dynamically-typed Lean object
///
/// This type tag is intended for use by Rust code that needs to operate on
/// arbitrary Lean objects (objects whose types are unknown at compile time).
pub enum AnyTypeTag {}

pub type LeanAnyObj = Obj<AnyTypeTag>;

pub type LeanAnyObject = Object<AnyTypeTag>;
