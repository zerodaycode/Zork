//! Constant value definitions to use across the whole program

pub const ZORK: &str = "zork";

/// The names of the `Zork++`specific directories, not their paths
pub mod dir_names {
    pub const DEFAULT_OUTPUT_DIR: &str = "out";
    pub const CACHE: &str = "cache";
    pub const STD: &str = "std";
    pub const INTRINSICS: &str = "intrinsics";
    pub const INTERFACES: &str = "interfaces";
    pub const IMPLEMENTATIONS: &str = "implementations";
    pub const OBJECT_FILES: &str = "obj_files";
}

pub mod error_messages {
    pub const READ_CFG_FILE: &str = "Could not read the configuration file";
    pub const PARSE_CFG_FILE: &str = "Could not parse the configuration file";
    pub const FAILURE_GENERATING_COMMANDS: &str =
        "Failed to generated the commands for the project";
    pub const FAILED_BUILD_FOR_CFG_FILE: &str = "Failed to build the project for the config file";
    pub const GENERAL_ARGS_NOT_FOUND: &str = "Something went wrong loading the general arguments";
    pub const PROJECT_MODEL_MAPPING: &str = "Error building the project model";
    pub const COMPILER_SPECIFIC_COMMON_ARGS_NOT_FOUND: &str =
        "Something went wrong loading the general arguments";
    pub const CLI_ARGS_CMD_NEW_BRANCH: &str =
        "This branch should never be reached for now, as do not exists commands that may\
        trigger them. The unique remaining, is ::New, that is already processed\
        at the very beggining";
}

pub const CONFIG_FILE_NAME: &str = "zork";
pub const CONFIG_FILE_EXT: &str = "toml";
pub const CACHE_FILE_EXT: &str = "json";

pub const BINARY_EXTENSION: &str = if cfg!(target_os = "windows") {
    "exe"
} else {
    ""
};

pub const ZORK_CACHE_FILENAME: &str = "cache.json";
pub const COMPILATION_DATABASE: &str = "compile_commands.json";

pub const GCC_CACHE_DIR: &str = "gcm.cache";

pub const WIN_CMD: &str = "C:\\Windows\\system32\\cmd";
pub const MSVC_REGULAR_BASE_SCAPED_PATH: &str =
    "C:\\\"Program Files\"\\\"Microsoft Visual Studio\"";
pub const MSVC_REGULAR_BASE_PATH: &str = "C:\\Program Files\\Microsoft Visual Studio";
pub const MS_ENV_VARS_BAT: &str = "vcvars64.bat";
pub const CONFIG_FILE_MOCK: &str = r#"

[project]
name = "Zork++"
authors = ["zerodaycode.gz@gmail.com"]
compilation_db = true

[compiler]
cpp_compiler = "clang"
cpp_standard = "2b"
std_lib = "libc++"
extra_args = [ "-Wall" ]

[build]
output_dir = ""

[executable]
executable_name = "zork"
sources_base_path = "bin"
sources = [
    "*.cpp"
]
extra_args = [ "-Werr" ]

[tests]
test_executable_name = "zork_check"
sources_base_path = "test"
sources = [
    "*.cpp"
]
extra_args = [ "-pedantic" ]

[modules]
base_ifcs_dir = "ifcs"
interfaces = [
    { file = "maths.cppm" },
    { file = 'some_module.cppm', module_name = 'maths' }
]

base_impls_dir = "srcs"
implementations = [
    { file = "maths.cpp" },
    { file = 'some_module_impl.cpp', dependencies = ['iostream'] }
]
sys_modules = [ "iostream" ]
extra_args = [ "-Wall" ]
"#;
