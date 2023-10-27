//! The higher abstractions of the program

use core::fmt::Debug;
use std::fmt::Display;
use std::path::PathBuf;

use crate::{cli::output::arguments::Argument, project_model::sourceset::SourceSet};

/// Bound for the user defined arguments that are passed to the compiler
pub trait ExtraArgs<'a> {
    fn extra_args(&'a self) -> &'a [Argument<'a>];
}

/// Contracts for the executable operations
pub trait ExecutableTarget<'a>: ExtraArgs<'a> {
    fn name(&'a self) -> &'a str;
    fn sourceset(&'a self) -> &'a SourceSet;
}

/// Represents any kind of translation unit and the generic operations
/// applicable to all the implementors
pub trait TranslationUnit: Display + Debug {
    /// Returns the file, being the addition of the path property plus the file stem plus
    /// the extension property
    fn file(&self) -> PathBuf;

    /// Outputs the declared path for `self`, being self the translation unit
    fn path(&self) -> PathBuf;

    /// Outputs the declared file stem for this translation unit
    fn file_stem(&self) -> String;

    /// Outputs the declared extension for `self`
    fn extension(&self) -> String;

    /// Outputs the file stem concatenated with the extension for a given tu
    fn file_with_extension(&self) -> String {
        format!("{}.{}", self.file_stem(), self.extension())
    }
}
