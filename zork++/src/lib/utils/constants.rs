//! Constant value definitions to use across the whole program

pub const CONFIG_FILE_NAME: &str = "zork";
pub const CONFIG_FILE_EXT: &str = ".toml";
pub const DEFAULT_OUTPUT_DIR: &str = "out";

pub const BINARY_EXTENSION: &str = if cfg!(target_os = "windows") {
    "exe"
} else {
    ""
};

pub const ZORK_CACHE_FILENAME: &str = "cache.json";
pub const COMPILATION_DATABASE: &str = "compile_commands.json";

pub const GCC_CACHE_DIR: &str = "gcm.cache";

pub const MSVC_BASE_PATH: &str = "C:\\Program Files\\Microsoft Visual Studio";
pub const MS_DEVS_PROMPT_BAT: &str = "vcvars64.bat";

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
