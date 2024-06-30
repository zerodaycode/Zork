use core::fmt;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

use crate::bounds::TranslationUnit;
use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};

use crate::cli::output::arguments::Argument;

// Since every file on the system has a path, this acts as a cheap conceptual
// conversion to unify PATH querying operations over anything that can be
// saved on a persistence system with an access route
pub trait File {
    fn get_path(&self) -> PathBuf;
}

impl File for Path {
    fn get_path(&self) -> PathBuf {
        self.to_path_buf()
    }
}

impl File for PathBuf {
    fn get_path(&self) -> PathBuf {
        self.to_path_buf()
    }
}

// TODO: All the trait File impl as well as the trait aren't required anymore

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct SourceFile<'a> {
    pub path: PathBuf,
    pub file_stem: Cow<'a, str>,
    pub extension: Cow<'a, str>,
}

impl<'a> TranslationUnit for SourceFile<'a> {
    fn file(&self) -> PathBuf {
        let file_name = format!("{}.{}", self.file_stem, self.extension);
        self.path().join(file_name)
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn file_stem(&self) -> Cow<'_, str> {
        self.file_stem.clone()
    }

    fn extension(&self) -> Cow<'_, str> {
        self.extension.clone()
    }
}

impl<'a> TranslationUnit for &'a SourceFile<'a> {
    fn file(&self) -> PathBuf {
        let file_name = format!("{}.{}", self.file_stem, self.extension);
        self.path().join(file_name)
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn file_stem(&self) -> Cow<'_, str> {
        self.file_stem.clone()
    }

    fn extension(&self) -> Cow<'_, str> {
        self.extension.clone()
    }
}

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
        let args = self.sources.iter().map(|sf| sf.file()).map(Argument::from);

        dst.extend(args);

        Ok(())
    }
}
