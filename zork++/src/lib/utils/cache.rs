//TODO

use chrono::{DateTime, Utc};
use color_eyre::{
    eyre::{Context, ContextCompat},
    Report,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

///
/// ### Tests
/// ```rust
/// use chrono::{DateTime,Utc};
/// use std::path::Path;
/// use zork::utils::{
///     cache::CacheFile,
///     cache::FileInfo,
///     fs::{serialize_object,deserilize_file}
/// };
///
/// let cache = CacheFile {
///     files: vec![FileInfo {
///         path: "asdasd".to_owned(),
///         last_modificacion: Utc::now(),
///     }],
///     last_date_execution: Utc::now(),
/// };
/// let path_file = Path::new("file.txt");
/// serialize_object(path_file, &cache).unwrap();
///
/// let cache_deserialize: CacheFile = deserilize_file(path_file).unwrap();
/// assert_eq!(cache,cache_deserialize);
///
/// std::fs::remove_file(path_file);
///  ```
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct CacheFile {
    pub files: Vec<FileInfo>,
    pub last_date_execution: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct FileInfo {
    pub path: String,
    pub last_modificacion: DateTime<Utc>,
}

//TODO require test
pub fn get_files_info_directories(root_path: &Path) -> Result<Vec<FileInfo>, Report> {
    let mut file_info: Vec<FileInfo> = vec![];

    for e in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
        if e.file_type().is_file() {
            file_info.push(get_file_info(e).with_context(|| "Cant recover file info")?);
        }
    }
    Ok(file_info)
}

fn get_file_info(dir_entry: DirEntry) -> Result<FileInfo, Report> {
    let path = dir_entry
        .path()
        .to_str()
        .with_context(|| "Error parse path to str")?
        .to_owned();
    let metadata_file = dir_entry
        .metadata()
        .with_context(|| "Error extract metadata in this plataform")?;
    let date_time: DateTime<Utc> = metadata_file
        .modified()
        .with_context(|| "Error extract time modified in this plataform")?
        .into();

    Ok(FileInfo {
        path: path,
        last_modificacion: date_time,
    })
}
