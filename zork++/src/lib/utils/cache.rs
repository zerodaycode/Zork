//TODO

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
