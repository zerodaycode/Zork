//TODO

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

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
pub fn get_files_info_directories(root_path: &Path, exclusions: Vec<&str>) -> Vec<FileInfo> {
    let mut file_info: Vec<FileInfo> = vec![];

    for e in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
        let metadata_file = e.metadata().unwrap();
        if exclusions
            .iter()
            .find(|exclusion| {
                e.path()
                    .to_str()
                    .unwrap() //TODO revise
                    .contains(*exclusion)
            })
            .is_some()
        {
            continue;
        }
        if metadata_file.is_file() {
            let date_time: DateTime<Utc> = metadata_file.modified().unwrap().into();
            file_info.push(FileInfo {
                path: e.path().to_str().unwrap().to_owned(),
                last_modificacion: date_time,
            }); //TODO posible error this field not avalible on al plataforms
        }
    }
    file_info
}
