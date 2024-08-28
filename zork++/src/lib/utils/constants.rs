//! Constant value definitions to use across the whole program

pub const ZORK: &str = "zork";

pub mod compilers {
    pub const CLANG: &str = "clang";
}

/// The names of the `Zork++`specific directories, not their paths
pub mod dir_names {
    pub const DEFAULT_OUTPUT_DIR: &str = "out";
    pub const CACHE: &str = "cache";
    pub const STD: &str = "std";
    pub const MODULES: &str = "modules";
    pub const INTRINSICS: &str = "intrinsics";
    pub const INTERFACES: &str = "interfaces";
    pub const IMPLEMENTATIONS: &str = "implementations";
    pub const OBJECT_FILES: &str = "obj_files";
}

pub mod env_vars {
    pub const VS_VERSION: &str = "VisualStudioVersion";
    pub const VC_TOOLS_INSTALL_DIR: &str = "VCToolsInstallDir";
}

pub mod debug_messages {
    pub const MAPPING_CFG_TO_MODEL: &str = "Proceding to map the configuration file to the ZorkModel entity, since no cached project model was found";
}

pub mod error_messages {
    pub const READ_CFG_FILE: &str = "Could not read the configuration file";
    pub const PARSE_CFG_FILE: &str = "Could not parse the configuration file";
    pub const REMOVE_FILE: &str = "Unable to remove file from fs";
    pub const FAILURE_GENERATING_COMMANDS: &str =
        "Failed to generated the commands for the project";
    pub const FAILED_BUILD_FOR_CFG_FILE: &str = "Failed to build the project for the config file";
    pub const FAILURE_CREATING_CACHE_FILE: &str = "Error creating the cache file";
    pub const FAILURE_GATHERING_PROJECT_ROOT_ABS_PATH: &str =
        "An unexpected error happened while resolving the absolute path to the project root";
    pub const FAILURE_CREATING_COMPILER_CACHE_DIR: &str =
        "Error creating the cache subdirectory for compiler";
    pub const FAILURE_LOADING_CACHE: &str = "Failed to load the Zork++ cache";
    pub const FAILURE_LOADING_COMPILER_METADATA: &str =
        "Failed while gathering the current compiler's metadata";
    pub const FAILURE_CLEANING_CACHE: &str = "Error cleaning the Zork++ cache";
    pub const FAILURE_LOADING_FLYWEIGHT_DATA: &str =
        "Failed while initializating the flyweight data of the shared command lines arguments";
    pub const FAILURE_SAVING_CACHE: &str = "Error saving data to the Zork++ cache";
    pub const CHECK_FOR_DELETIONS: &str = "Error while checking the user files deletions on cfg";
    pub const GENERAL_ARGS_NOT_FOUND: &str = "Something went wrong loading the general arguments";
    pub const PROJECT_MODEL_MAPPING: &str = "Error building the project model";
    pub const PROJECT_MODEL_LOAD: &str = "Error loading from the fs the project model";
    pub const PROJECT_MODEL_SAVE: &str = "Error caching and saving to the fs the project model";
    pub const TARGET_ENTRY_NOT_FOUND: &str =
        "Unlikely error happened while adding linkage data to a target";
    pub const COMPILER_SPECIFIC_COMMON_ARGS_NOT_FOUND: &str =
        "Something went wrong loading the general arguments";
    pub const DEFAULT_OF_COMPILER_COMMON_ARGUMENTS: &str =
        "Reached the default implementation of the CompilerCommonArgument data structure.\
        This is a bug, so please, report it by opening an issue on https://github.com/zerodaycode/Zork/issues";
    pub const CLI_ARGS_CMD_NEW_BRANCH: &str =
        "This branch should never be reached for now, as do not exists commands that may\
        trigger them. The unique remaining, is ::New, that is already processed\
        at the very beginning";

    pub const FAILURE_MODULE_INTERFACES: &str =
        "An error happened while generating the commands for the module interfaces";
    pub const FAILURE_MODULE_IMPLEMENTATIONS: &str =
        "An error happened while generating the commands for the module implementations";
    pub const TARGET_SOURCES_FAILURE: &str =
        "An error happened while generating the commands for the declared sources of the target";
    pub const FAILURE_FINDING_TARGET: &str =
        "An error happened while retrieving the target information";
    pub const FAILURE_SYSTEM_MODULES: &str =
        "An error happened while generating the commands for the declared system headers as modules";
    pub const WRONG_DOWNCAST_FOR: &str = "An error happened while resolving the original type of";
    pub const FILTERING_COMPILE_BUT_DONT_LINK: &str = "Unlikely error happened while removing the compile but don't link flag from the flyweight data. This is a BUG, so please, open an issue on upsteam";

    pub mod msvc {
        pub const STDLIB_MODULES_NOT_FOUND: &str =
            "Can't find the MSVC standard library modules. Did you installed them?";
        pub const MISSING_VCTOOLS_DIR: &str =
            "Unable to find MSVC VCToolsInstallDir. Did you installed the required C++ tools for the compiler?";
        pub const FAILURE_LOADING_VS_ENV_VARS: &str =
            "Zork++ wasn't unable to find the VS env vars";
        pub const ILL_FORMED_KEY_ON_ENV_VARS_PARSING: &str =
            "Ill-formed key while parsing MSVC env vars";
        pub const MISSING_OR_CORRUPTED_MSVC_DEV_COMMAND_PROMPT: &str =
            "Missing or corrupted path for the MSVC developers command prompt";
    }

    pub mod clang {
        pub const FAILURE_READING_CLANG_DRIVER_INFO: &str =
            "Unable to read and parse the metadata of the declared compiler driver";
        pub const MISSING_LIBCPP_INSTALLATION: &str = "Unable to find a LIBC++ installation for the invoked driver. Please, provide the right one explicitly via the configuration file.";
        pub const WRONG_LIBCPP_DIR: &str =
            "Provided LIBC++ path on the cfg file is incorrect, such directory doens't exists";
        pub const METADATA_GATHER_FAILED: &str =
            "Unable to gather information about the configured Clang driver";
        pub const FAILURE_PARSING_CLANG_VERSION: &str = "Unable to parse the clang version";
        pub const FAILURE_GETTING_VER_MAJOR: &str =
            "Unable to parse clang version and get the major component";
        pub const FAILURE_GETTING_VER_MINOR: &str =
            "Unable to parse clang version and get the minor component";
        pub const FAILURE_GETTING_VER_PATCH: &str =
            "Unable to parse clang version and get the patch component";
        pub const INSTALLED_DIR: &str =
            "Unable to parse the installed dir of the invoked clang driver";
    }
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

[targets.executable]
output_name = 'zork'
sources = [ 'main.cpp' ]
extra_args = [ '-Werr' ]

[targets.tests]
output_name = 'zork_tests'
sources = [ 'tests_main.cpp' ]
target_kind = 'executable'

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
