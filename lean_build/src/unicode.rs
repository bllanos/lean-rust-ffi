use std::ffi::OsString;

#[derive(thiserror::Error, Debug)]
#[error("invalid UTF-8")]
pub struct NotUnicode;

#[derive(thiserror::Error, Debug)]
#[error("invalid UTF-8 string with lossy value \"{}\"", .0.display())]
pub struct NotUnicodeString(pub OsString);

#[derive(thiserror::Error, Debug)]
#[error("invalid UTF-8 bytes")]
pub struct NotUnicodeBytes(pub Vec<u8>);

pub fn display_slice(slice: &[u8]) -> &str {
    str::from_utf8(slice).unwrap_or("[Non-UTF8]")
}
