use std::{
    fs::{DirBuilder, File},
    io::{BufReader, Write},
    path::Path,
};

use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};

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

pub fn serialize_object<T>(path: &Path, cache_file: &T) -> Result<()>
where
    T: Serialize,
{
    serde_json::to_writer(
        File::create(path).with_context(|| "Error create file")?,
        cache_file,
    )
    .with_context(|| "Error serialize cache")
}

pub fn deserilize_to_object<T>(path: &Path) -> Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    let buffer = BufReader::new(File::open(path).with_context(|| "Error open file cache")?);
    serde_json::from_reader(buffer).with_context(|| "Error deserilize cache file")
}
