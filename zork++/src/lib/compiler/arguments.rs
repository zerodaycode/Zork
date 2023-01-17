//! Types and procedures that represents a command line argument, 
//! or collections of command line arguments

use std::{borrow::Borrow, ops::Deref, ffi::OsStr};

/// Type for represent a command line argument
#[derive(Debug, Clone)]
pub struct Argument {
    pub value: String
}

impl From<&str> for Argument {
    fn from(value: &str) -> Self {
        Self { value: value.to_string() }
    }
}

impl From<String> for Argument {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl Borrow<str> for Argument {
    fn borrow(&self) -> &str {
        self.value.as_str()
    }
}

impl AsRef<OsStr> for Argument {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self.value.as_str())
    }
}
