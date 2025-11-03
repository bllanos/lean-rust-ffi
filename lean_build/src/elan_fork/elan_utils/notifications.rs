use std::fmt::{self, Display};
use std::path::Path;

use super::notify::NotificationLevel;

#[derive(Debug)]
pub enum Notification<'a> {
    NoCanonicalPath(&'a Path),
}

impl<'a> Notification<'a> {
    pub fn level(&self) -> NotificationLevel {
        use self::Notification::*;
        match *self {
            NoCanonicalPath(_) => NotificationLevel::Warn,
        }
    }
}

impl<'a> Display for Notification<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> ::std::result::Result<(), fmt::Error> {
        use self::Notification::*;
        match *self {
            NoCanonicalPath(path) => write!(f, "could not canonicalize path: '{}'", path.display()),
        }
    }
}
