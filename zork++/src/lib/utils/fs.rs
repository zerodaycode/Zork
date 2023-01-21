use std::{
    fs::{remove_file, DirBuilder, File},
    io::{BufReader, Write},
    path::Path,
};

use color_eyre::{eyre::Context, Report, Result};
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

//TODO require test
pub fn serialize_object<T>(path: &Path, cache_file: &T) -> Result<(), Report>
where
    T: Serialize,
{
    if path.exists() {
        remove_file(path).with_context(|| "Error remove cache file")?;
    }

    let file: File = File::create(path).with_context(|| "Error create file")?;
    serde_json::to_writer(file, cache_file).with_context(|| "Error serialize cache")?;
    Ok(())
}

//TODO required test
pub fn deserilize_file<T>(path: &Path) -> Result<T, Report>
where
    T: for<'a> Deserialize<'a>,
{
    let file = File::open(path).with_context(|| "Error open file cache")?;

    let buffer = BufReader::new(file);

    let cache_deserilized: T =
        serde_json::from_reader(buffer).with_context(|| "Error deserilize cache file")?;
    Ok(cache_deserilized)
}
