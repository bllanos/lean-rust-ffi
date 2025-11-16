use std::io::Write;

mod elan;
mod elan_fork;
mod file;
mod lake;
pub mod library_build;
pub mod runtime_build;
mod rust;
mod unicode;

use file::get_out_dir;
use unicode::display_slice;

pub use file::{FileOutputError, OutDirError};
pub use lake::{LakeEnvironmentDescriber, LakeEnvironmentDescription};
pub use unicode::{NotUnicode, NotUnicodeBytes, NotUnicodeString};

/// The environment variable used to specify the Lean toolchain
const ELAN_TOOLCHAIN: &str = "ELAN_TOOLCHAIN";

fn write_warning_allow_directives<W: Write>(mut writer: W) -> std::io::Result<()> {
    writeln!(
        writer,
        "#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]
// These warnings will hopefully be resolved in a future ion of the
// `bindgen` crate
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::useless_transmute)]"
    )
}
