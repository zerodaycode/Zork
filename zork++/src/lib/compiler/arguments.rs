//! Types and procedures that represents a command line argument, 
//! or collections of command line arguments

/// Type for represent a command line argument
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

