use std::{
    fs::{DirBuilder, File},
    io::{BufReader, Write},
    path::Path,
};

use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};

use super::constants;

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

pub fn serialize_object_to_file<T>(path: &Path, data: &T) -> Result<()>
where
    T: Serialize,
{
    serde_json::to_writer_pretty(
        File::create(path).with_context(|| "Error creating the cache file")?,
        data,
    )
    .with_context(|| "Error serializing data to the cache")
}

pub fn load_and_deserialize<T, P>(path: &P) -> Result<T>
where
    T: for<'a> Deserialize<'a> + Default,
    P: AsRef<Path>,
{
    let buffer = BufReader::new(
        File::open(path.as_ref().join(constants::ZORK_CACHE_FILENAME))
            .with_context(|| "Error opening the cache file")?,
    );
    Ok(serde_json::from_reader(buffer).unwrap_or_default())
}
