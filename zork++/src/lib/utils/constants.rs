///! Constant value definitions to use across the whole program

pub const CONFIG_FILE_NAME: &str = "zork.toml";
pub const DEFAULT_OUTPUT_DIR: &str = "./out";

pub const BINARY_EXTENSION: &str = 
    if cfg!(target_os = "windows") { "exe" } else { "" };
pub const CACHE_FILE: &str = "cache.toml";
