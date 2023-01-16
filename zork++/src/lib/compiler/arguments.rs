//! Types and procedures that represents a command line argument, 
//! or collections of command line arguments

/// Type for represent a command line argument
pub struct Argument {
    pub value: String
}

impl Argument {
    pub fn new<T: ToString>(val: T) {
        
    }
} 