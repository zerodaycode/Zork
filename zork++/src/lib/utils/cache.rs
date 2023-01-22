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

//TODO pending comment and change
pub mod builder {
    use super::FileInfo;
    use crate::config_file::modules::ModulesAttribute;
    use chrono::{DateTime, Utc};
    use color_eyre::{
        eyre::{Context, ContextCompat},
        Report,
    };
    use std::path::{Path, PathBuf};

    pub fn get_files_info_in_zork_config(
        module_attribute: &ModulesAttribute,
    ) -> Result<Vec<FileInfo>, Report> {
        let mut files_info: Vec<FileInfo> = vec![];
        if module_attribute.base_ifcs_dir.is_some() && module_attribute.interfaces.is_some() {
            let interface_in_str: Vec<&str> = module_attribute
                .interfaces
                .as_ref()
                .unwrap()
                .iter()
                .map(|interface| interface.filename)
                .collect();
            files_info.append(
                &mut get_file_info(module_attribute.base_ifcs_dir.unwrap(), interface_in_str)
                    .with_context(|| "Error extract interface to cache")?,
            );
        }

        if module_attribute.base_impls_dir.is_some() && module_attribute.implementations.is_some() {
            let implementation_in_str: Vec<&str> = module_attribute
                .implementations
                .as_ref()
                .unwrap()
                .iter()
                .map(|implementation| implementation.filename)
                .collect();
            files_info.append(
                &mut get_file_info(
                    module_attribute.base_impls_dir.unwrap(),
                    implementation_in_str,
                )
                .with_context(|| "Error extract Implementations to cache")?,
            );
        }

        Ok(files_info)
    }

    fn get_file_info(base_path: &str, files: Vec<&str>) -> Result<Vec<FileInfo>, Report> {
        let mut files_info: Vec<FileInfo> = vec![];

        for file in files {
            let path = make_path(base_path, file);
            if path.exists() && path.is_file() {
                files_info
                    .push(make_file_info(&path).with_context(|| {
                        format!("Error recover files info to path: {:?}", path)
                    })?);
            }
        }
        Ok(files_info)
    }

    fn make_path(base: &str, file: &str) -> PathBuf {
        Path::new(base).join(file)
    }

    fn make_file_info(path: &PathBuf) -> Result<FileInfo, Report> {
        let path_string = path
            .to_str()
            .with_context(|| "Error parse path to str")?
            .to_owned();
        let metadata_file = path
            .metadata()
            .with_context(|| "Error extract metadata in this plataform")?;
        let date_time: DateTime<Utc> = metadata_file
            .modified()
            .with_context(|| "Error extract time modified in this plataform")?
            .into();

        Ok(FileInfo {
            path: path_string,
            last_modificacion: date_time,
        })
    }
}
