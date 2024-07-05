//! The higher abstractions of the program

use core::fmt::Debug;
use std::borrow::Cow;
use std::fmt::Display;
use std::path::PathBuf;

use transient::{Any, Inv};

use crate::{cli::output::arguments::Argument, project_model::sourceset::SourceSet};

/// Bound for the user defined arguments that are passed to the compiler
pub trait ExtraArgs<'a> {
    fn extra_args(&'a self) -> &'a [Argument];
}

/// Contracts for the executable operations
pub trait ExecutableTarget<'a>: ExtraArgs<'a> {
    fn name(&'a self) -> &'a str;
    fn sourceset(&'a self) -> &'a SourceSet;
}

// Base trait for downcasting
pub trait AsTranslationUnit<'a> {
    fn as_any(&self) -> &dyn Any<Inv<'a>>;
}

// Implementation of AsTranslationUnit for all types implementing TranslationUnit
impl<'a, T: TranslationUnit<'a> + 'a> AsTranslationUnit<'a> for T {
    fn as_any(&self) -> &dyn Any<Inv<'a>> {
        self
    }
}

/// Represents any kind of translation unit and the generic operations
/// applicable to all the implementors
pub trait TranslationUnit<'a>: AsTranslationUnit<'a> + Any<Inv<'a>> + Display + Debug {
    /// Returns the file, being the addition of the path property plus the file stem plus
    /// the extension property
    ///
    /// # Examples
    ///
    /// ```
    /// use std::borrow::Cow;
    /// use std::path::PathBuf;
    /// use zork::bounds::TranslationUnit;
    /// use zork::project_model::sourceset::SourceFile;
    ///
    /// let source_file = SourceFile {
    ///     path: PathBuf::from("/usr/include"),
    ///     file_stem: Cow::from("std"),
    ///     extension: Cow::from("h"),
    /// };
    ///
    /// assert_eq!(source_file.file(), PathBuf::from("/usr/include/std.h"));
    ///
    /// let source_file_compat = SourceFile {
    ///     path: PathBuf::from("/usr/include"),
    ///     file_stem: Cow::from("std.compat"),
    ///     extension: Cow::from("h"),
    /// };
    ///
    /// assert_eq!(source_file_compat.file(), PathBuf::from("/usr/include/std.compat.h"));
    /// ```
    fn file(&self) -> PathBuf;

    /// Outputs the declared path for `self`, being self the translation unit
    fn path(&self) -> &PathBuf;

    /// Outputs the declared file stem for this translation unit
    fn file_stem(&self) -> &Cow<'_, str>;

    /// Outputs the declared extension for `self`
    fn extension(&self) -> &Cow<'_, str>;

    /// Outputs the file stem concatenated with the extension for a given tu
    fn file_with_extension(&self) -> String {
        format!("{}.{}", self.file_stem(), self.extension())
    }
}
