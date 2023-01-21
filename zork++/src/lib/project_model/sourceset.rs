use std::path::Path;

use color_eyre::Result;

use super::arguments::Argument;

#[derive(Debug, PartialEq, Eq)]
pub enum Source<'a> {
    File(&'a str),
    Glob(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub struct SourceSet<'a> {
    pub base_path: &'a Path,
    pub sources: Vec<Source<'a>>,
}

impl<'a> SourceSet<'a> {
    pub fn as_args_to(&'a self, dst: &mut Vec<Argument<'a>>) -> Result<()> {
        let paths: Result<Vec<Vec<Argument<'a>>>> = self
            .sources
            .iter()
            .map(|source| resolve_glob(self.base_path, source))
            .collect();

        let paths = paths?.into_iter().flatten();

        dst.extend(paths);

        Ok(())
    }
}

fn resolve_glob<'a>(base_path: &'a Path, source: &'a Source) -> Result<Vec<Argument<'a>>> {
    match source {
        Source::File(file) => {
            let path = base_path.join(file);
            Ok(vec![Argument::from(format!("{path:?}"))])
        }
        Source::Glob(glob_pattern) => glob::glob(glob_pattern)?
            .map(|glob_match| {
                let path = base_path.join(glob_match?);
                Ok(Argument::from(format!("{path:?}")))
            })
            .collect(),
    }
}
