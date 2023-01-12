use std::{
    fs::{DirBuilder, File},
    io::Write,
    path::Path,
};

use color_eyre::{eyre::Context, Result};

pub fn create_file<'a>(path: &Path, filename: &'a str, buff_write: &'a [u8]) -> Result<()> {
    let file_path = path.join(filename);

    File::create(&file_path)
        .with_context(|| format!("Could not create file {file_path:?}"))?
        .write_all(buff_write)
        .with_context(|| format!("Could not write to file {file_path:?}"))
}

pub fn create_directory(path_create: &Path) -> Result<()> {
    DirBuilder::new()
        .recursive(true)
        .create(path_create)
        .with_context(|| format!("Could not create directory {path_create:?}"))
}
