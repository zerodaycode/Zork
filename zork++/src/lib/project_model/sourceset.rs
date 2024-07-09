use core::fmt;
use std::borrow::Cow;
use std::path::PathBuf;

use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};
use transient::Transient;

use crate::cli::output::arguments::Argument;
use crate::domain::translation_unit::TranslationUnit;
use crate::impl_translation_unit_for;

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize, Transient)]
pub struct SourceFile<'a> {
    pub path: PathBuf,
    pub file_stem: Cow<'a, str>,
    pub extension: Cow<'a, str>,
}

impl_translation_unit_for!(SourceFile<'a>);

impl<'a> fmt::Display for SourceFile<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:?}/{:?}.{:?})",
            self.path, self.file_stem, self.extension
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Source {
    File(PathBuf),
    Glob(GlobPattern),
}

impl Source {
    #[inline(always)]
    pub fn paths(&self) -> Result<Vec<PathBuf>> {
        match self {
            Source::File(file) => Ok(vec![file.to_path_buf()]),
            Source::Glob(pattern) => pattern.resolve(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GlobPattern(pub PathBuf);

impl GlobPattern {
    #[inline(always)]
    fn resolve(&self) -> Result<Vec<PathBuf>> {
        glob::glob(self.0.to_str().unwrap_or_default())?
            .map(|path| path.with_context(|| ""))
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SourceSet<'a> {
    pub sources: Vec<SourceFile<'a>>,
}

impl<'a> SourceSet<'a> {
    pub fn as_args_to(&self, dst: &mut Vec<Argument>) -> Result<()> {
        let args = self.sources.iter().map(|sf| sf.path()).map(Argument::from);

        dst.extend(args);

        Ok(())
    }
}
