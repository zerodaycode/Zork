
use std::{path::PathBuf, fs};
use walkdir::WalkDir;
use super::constants::CONFIG_FILE_NAME;

/// Checks for the existence of the `zork.toml` configuration files
/// 
/// This function panics if there's no configuration file
/// (or isn't present in any directory of the project)
pub fn find_config_file() -> String {
    let mut path: PathBuf = PathBuf::new();
    
    for e in WalkDir::new(".").into_iter()
        .filter_map(|e| e.ok())
        .into_iter() 
    {
        if e.metadata().unwrap().is_file() && e.path().ends_with(CONFIG_FILE_NAME) {
            path.push(e.path());
            break
        }
    }

    fs::read_to_string(path)
        .expect("Error opening or reading the configuration file")
}
