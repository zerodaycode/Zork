use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub struct BuildModel<'a> {
    pub output_dir: &'a Path,
}
