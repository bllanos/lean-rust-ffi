use std::fmt::{self, Display};

use super::super::elan_utils::{self, notify::NotificationLevel};

#[derive(Debug)]
pub enum Notification<'a> {
    Utils(elan_utils::Notification<'a>),
}

impl<'a> From<elan_utils::Notification<'a>> for Notification<'a> {
    fn from(n: elan_utils::Notification<'a>) -> Notification<'a> {
        Notification::Utils(n)
    }
}

impl Notification<'_> {
    pub fn level(&self) -> NotificationLevel {
        use self::Notification::*;
        match *self {
            Utils(ref n) => n.level(),
        }
    }
}

impl Display for Notification<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> ::std::result::Result<(), fmt::Error> {
        use self::Notification::*;
        match *self {
            Utils(ref n) => n.fmt(f),
        }
    }
}
