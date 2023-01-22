//! Types and procedures that represents a command line argument,
//! or collections of command line arguments

use std::{borrow::Borrow, ffi::OsStr, path::PathBuf};

/// Type for represent a command line argument
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argument<'a> {
    pub value: &'a str,
}

impl<'a> From<&'a str> for Argument<'a> {
    fn from(value: &'a str) -> Self {
        Self { value }
    }
}

impl<'a> From<String> for Argument<'a> {
    fn from(value: String) -> Argument<'a> {
        Self {
            value: Box::leak(value.into_boxed_str()),
        }
    }
}

impl<'a> From<PathBuf> for Argument<'a> {
    fn from(value: PathBuf) -> Self {
        Self::from(format!("{value:?}"))
    }
}

impl<'a> Borrow<str> for Argument<'a> {
    fn borrow(&self) -> &str {
        self.value
    }
}

impl<'a> AsRef<OsStr> for Argument<'a> {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self.value)
    }
}

impl<'a> core::fmt::Display for Argument<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
