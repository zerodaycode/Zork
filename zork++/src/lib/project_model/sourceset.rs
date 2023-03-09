use core::fmt;
use std::path::{Path, PathBuf};

use color_eyre::{eyre::Context, Result};
use crate::bounds::TranslationUnit;

use crate::cli::output::arguments::Argument;

#[derive(Debug, PartialEq, Eq)]
pub enum Source<'a> {
    File(&'a Path),
    Glob(GlobPattern<'a>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct SourceFile {
    pub path: PathBuf,
    pub file_stem: String,
    pub extension: String
}

impl TranslationUnit for SourceFile {
    fn file(&self) -> PathBuf {
        let mut tmp = self.path.join(&self.file_stem).into_os_string();
        tmp.push(".");
        tmp.push(&self.extension);
        PathBuf::from(tmp)
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn file_stem(&self) -> String {
        self.file_stem.clone()
    }

    fn extension(&self) -> String {
        self.extension.clone()
    }
}

impl TranslationUnit for &SourceFile {
    fn file(&self) -> PathBuf {
        let mut tmp = self.path.join(&self.file_stem).into_os_string();
        tmp.push(".");
        tmp.push(&self.extension);
        PathBuf::from(tmp)
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn file_stem(&self) -> String {
        self.file_stem.clone()
    }

    fn extension(&self) -> String {
        self.extension.clone()
    }
}


impl fmt::Display for SourceFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:?}/{:?}.{:?})",
            self.path, self.file_stem, self.extension
        )
    }
}

impl<'a> Source<'a> {
    #[inline(always)]
    pub fn paths(&self) -> Result<Vec<PathBuf>> {
        match self {
            Source::File(file) => Ok(vec![file.to_path_buf()]),
            Source::Glob(pattern) => pattern.resolve(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GlobPattern<'a>(pub &'a str);

impl<'a> GlobPattern<'a> {
    #[inline(always)]
    fn resolve(&self) -> Result<Vec<PathBuf>> {
        glob::glob(self.0)?
            .map(|path| path.with_context(|| ""))
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SourceSet {
    pub sources: Vec<SourceFile>,
}

impl SourceSet {
    pub fn as_args_to(&self, dst: &mut Vec<Argument<'_>>) -> Result<()> {
        let args = self.sources
            .iter()
            .map(|sf| sf.file())
            .map(Argument::from);

        dst.extend(args);

        Ok(())
    }
}
