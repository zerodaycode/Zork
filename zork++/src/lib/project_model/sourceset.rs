use core::fmt;
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

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: PathBuf,
    pub file_stem: String,
    pub extension: String,
}

impl TranslationUnit for SourceFile {
    fn file(&self) -> PathBuf {
        let mut tmp = self.path.join(&self.file_stem).into_os_string();
        tmp.push("."); // TODO: use the correct PATH APIs
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
pub struct SourceSet {
    pub sources: Vec<SourceFile>,
}

impl SourceSet {
    pub fn as_args_to(&self, dst: &mut Vec<Argument>) -> Result<()> {
        let args = self.sources.iter().map(|sf| sf.file()).map(Argument::from);

        dst.extend(args);

        Ok(())
    }
}
