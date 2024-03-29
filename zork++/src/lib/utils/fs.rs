use color_eyre::eyre::ContextCompat;
use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{DirBuilder, File},
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

use super::constants;

/// Creates a new file in the filesystem if the given does not exists yet at the specified location
pub fn create_file<'a>(path: &Path, filename: &'a str, buff_write: &'a [u8]) -> Result<()> {
    let file_path = path.join(filename);

    if !file_path.exists() {
        File::create(&file_path)
            .with_context(|| format!("Could not create file {file_path:?}"))?
            .write_all(buff_write)
            .with_context(|| format!("Could not write to file {file_path:?}"))
    } else {
        Ok(())
    }
}

/// Recursively creates a new directory pointed at the value of target if not exists yet
pub fn create_directory(target: &Path) -> Result<()> {
    if !target.exists() {
        DirBuilder::new()
            .recursive(true)
            .create(target)
            .with_context(|| format!("Could not create directory {target:?}"))
    } else {
        Ok(())
    }
}

#[inline(always)]
pub fn get_project_root_absolute_path(project_root: &Path) -> Result<PathBuf> {
    let mut canonical = project_root.canonicalize().with_context(|| {
        format!("Error getting the canonical path for the project root: {project_root:?}")
    })?;
    if cfg!(target_os = "windows") {
        canonical = canonical
            .to_str()
            .map(|unc| &unc[4..])
            .unwrap_or_default()
            .into()
    }

    Ok(canonical)
}

/// Returns a tuple of elements containing the directory of a file, its file stem and its extension
pub fn get_file_details<P: AsRef<Path>>(p: P) -> Result<(PathBuf, String, String)> {
    let file_stem = p
        .as_ref()
        .file_stem()
        .with_context(|| format!("Unable to get the file stem for {:?}", p.as_ref()))?;

    Ok((
        p.as_ref()
            .parent()
            .unwrap_or_else(|| panic!("Unexpected error getting the parent of {:?}", p.as_ref()))
            .to_path_buf(),
        file_stem.to_str().unwrap_or_default().to_string(),
        p.as_ref().extension().map_or_else(
            || String::with_capacity(0),
            |os_str| os_str.to_str().unwrap_or_default().to_string(),
        ),
    ))
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
