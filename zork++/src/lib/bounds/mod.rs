use core::fmt::Debug;
use std::{fmt::Display, path::Path};

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
    /// Outputs the declared path for `self` being the translation unit
    fn file(&self) -> &Path;

    fn filestem(&self) -> &str {
        self.file()
            .file_stem()
            .unwrap_or_else(|| 
                panic!(
                    "Unexpected error getting the filename of {:?}", 
                    self.file().as_os_str()
                ))
            .to_str()
            .unwrap()
    }
}

impl TranslationUnit for &str {
    fn file(&self) -> &Path {
        Path::new(self)
    }
}

impl TranslationUnit for String {
    fn file(&self) -> &Path {
        Path::new(self)
    }
}
