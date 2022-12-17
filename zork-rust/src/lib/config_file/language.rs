///! Contains the types designated to describe the different possible configurations
///! over the language

use serde::Deserialize;

/// TODO
#[derive(Deserialize, Debug, PartialEq)]
pub struct LanguageAttribute {  // Should this one be part of CompilerAttribute?
    pub cpp_standard: u16,
    pub std_lib: Option<String>, // TODO ENUM
    pub modules: Option<bool>
    // TODO Delete this prop? And only allow within Zork++ C++ modules projects???!
    // or probaly, just check if modules attribute is provided?
}

/// The C++ ISO standard levels of the language
pub enum LanguageLevel {
    L11, L14, L17, L20, L23, L2A, L2B 
}

/// The standard library version that the user
/// desired to link against
pub enum StdLib {
    STDLIBC,
    LIBC
}