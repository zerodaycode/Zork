use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub struct BuildModel {
    pub output_dir: PathBuf,
}
