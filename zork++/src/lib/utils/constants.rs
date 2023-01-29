///! Constant value definitions to use across the whole program

pub const CONFIG_FILE_NAME: &str = "zork.toml";
pub const DEFAULT_OUTPUT_DIR: &str = "./out";

pub const BINARY_EXTENSION: &str = if cfg!(target_os = "windows") {
    "exe"
} else {
    ""
};

pub const ZORK_CACHE_FILENAME: &str = "cache.json";

pub const GCC_CACHE_DIR: &str = "gcm.cache";

pub const MSVC_BASE_PATH: &str = "C:\\Program Files\\Microsoft Visual Studio";
pub const MS_DEVS_PROMPT_BAT: &str = "vcvars64.bat";
