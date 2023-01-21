use core::fmt;
use std::fmt::Display;

/// Represents any kind of translation unit and the generic operations
/// applicable to all the implementors
pub trait TranslationUnit: Display + fmt::Debug {
    /// Outputs the declared filename for `self` being the translation unit
    fn get_filename(&self) -> String;
}

impl TranslationUnit for &str {
    fn get_filename(&self) -> String {
        self.to_string()
    }
}

impl TranslationUnit for String {
    fn get_filename(&self) -> String {
        self.clone()
    }
}

/// Interfaces the behaviour for some property of retrieve
/// it's extra arguments field if exists on T
pub trait ExtraArgs {
    fn get_extra_args(&self) -> Option<Vec<&str>>; // Provisional
    fn get_extra_args_alloc(&self) -> Vec<String>;
}
