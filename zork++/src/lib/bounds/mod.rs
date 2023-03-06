use core::fmt::Debug;
use std::{fmt::Display, path::Path};
use std::path::PathBuf;

use crate::{cli::output::arguments::Argument, project_model::sourceset::SourceSet};

///! The higher abstractions of the program

pub trait ExtraArgs<'a> {
    fn extra_args(&'a self) -> &'a [Argument<'a>];
}

pub trait ExecutableTarget<'a>: ExtraArgs<'a> {
    fn name(&'a self) -> &'a str;
    fn sourceset(&'a self) -> &'a SourceSet<'a>;
}

/// Represents any kind of translation unit and the generic operations
/// applicable to all the implementors
pub trait TranslationUnit: Display + Debug {
    /// Returns the file, being the addition of the path property plus the extension property
    fn file(&self) -> PathBuf;

    /// Outputs the declared path for `self`, being self the translation unit
    fn path(&self) -> PathBuf;

    /// Outputs the declared extension for `self`
    fn extension(&self) -> String;

    fn filestem(&self) -> String {
        self.path()
            .file_stem()
            .unwrap_or_else(|| {
                panic!(
                    "Unexpected error getting the filename of {:?}",
                    self.path().with_extension(self.extension())
                )
            })
            .to_str()
            .unwrap()
            .to_string()
    }
}

