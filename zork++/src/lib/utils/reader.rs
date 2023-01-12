use crate::utils::constants::CONFIG_FILE_NAME;
use color_eyre::{eyre::Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// Checks for the existence of the `zork.toml` configuration files.
/// For now, when finds the first one, stops. Pending to decide if
/// Zork++ should support multiple configuration files (for nested projects)
/// or just by parsing one config file
///
/// This function panics if there's no configuration file
/// (or isn't present in any directory of the project)
pub fn find_config_file(base_path: &Path) -> Result<String> {
    let mut path: PathBuf = base_path.into();

    for e in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        if e.metadata().unwrap().is_file() && e.path().ends_with(CONFIG_FILE_NAME) {
            path.push(e.path());
            break;
        }
    }

    fs::read_to_string(&path).with_context(|| format!("Could not read {path:?}"))
}
