use core::fmt::Debug;
use std::{fmt::Display, path::Path};

use crate::project_model::{arguments::Argument, sourceset::SourceSet};

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
    /// Outputs the declared filename for `self` being the translation unit
    fn filename(&self) -> &Path;

    fn filestem(&self) -> &str {
        self.filename().file_stem().unwrap().to_str().unwrap()
    }
}

impl TranslationUnit for &str {
    fn filename(&self) -> &Path {
        Path::new(self)
    }
}

impl TranslationUnit for String {
    fn filename(&self) -> &Path {
        Path::new(self)
    }
}
