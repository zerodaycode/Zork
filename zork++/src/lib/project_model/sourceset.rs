use std::path::{Path, PathBuf};

use color_eyre::{eyre::Context, Result};

use crate::cli::output::arguments::Argument;

#[derive(Debug, PartialEq, Eq)]
pub enum Source<'a> {
    File(&'a Path),
    Glob(GlobPattern<'a>),
}

impl<'a> Source<'a> {
    fn paths(&self) -> Result<Vec<PathBuf>> {
        match self {
            Source::File(file) => Ok(vec![file.to_path_buf()]),
            Source::Glob(pattern) => pattern.resolve(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GlobPattern<'a>(pub &'a str);

impl<'a> GlobPattern<'a> {
    fn resolve(&self) -> Result<Vec<PathBuf>> {
        glob::glob(self.0)?
            .map(|path| path.with_context(|| ""))
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SourceSet<'a> {
    pub base_path: &'a Path,
    pub sources: Vec<Source<'a>>,
}

impl<'a> SourceSet<'a> {
    pub fn as_args_to(&'a self, dst: &mut Vec<Argument<'a>>) -> Result<()> {
        let paths: Result<Vec<Vec<PathBuf>>> = self.sources
            .iter()
            .map(Source::paths)
            .collect();

        let paths = paths?
            .into_iter()
            .flatten()
            .map(|path| self.base_path.join(path))
            .map(Argument::from);

        dst.extend(paths);

        Ok(())
    }
}
