//! file for represent the available configuration properties within Zork++
//! for setting up the target compiler

use serde::{Deserialize, Serialize};

use crate::project_model;

/// [`CompilerAttribute`] - Configuration properties for
/// targeting one of the available compilers within Zork++
///
/// * `cpp_compiler` - One of the available compilers within Zork++
/// They are represented by an enumerated type named [`CppCompiler`],
/// that holds the different options where the user can choose
///
/// * `driver_path` - The specific command line terminal identifier that will
/// call the compiler's binary. ie: clang++-15 will call a specific installation
/// of Clang in the host machine corresponding to the version 15 of the compiler.
/// This entry is particularly useful in Unix based OS or MinGW environments,
/// where multiple versions of the compiler lives at the same time, and their drivers
/// are identified by some sort of name like the one in the example of above
///
/// * `cpp_standard` - An string defining the version of the ISO
/// C++ standard that should be used on the compilation process
///
/// * `std_lib` - The concrete C++ standard library (vendor specific)
/// to link the built code against
///
/// * `extra_args` - A comma separated list of strings that will be passed
/// to the generated command lines. This ones here will be placed in every
/// command line generated by Zork++.
/// For example, if *['-O3', '-Wall']*
/// are included here, this will be wired in the main command line (the executable),
/// the ones generated for compile modules (both interfaces and implementations)
/// and for the command line generated for build the specified test suite and
/// the test executable
///
/// ### Tests
///
/// ```rust
/// use zork::config_file::compiler::{
///     CompilerAttribute, CppCompiler, LanguageLevel, StdLib
/// };
///
/// const CONFIG_FILE_MOCK: &str = r#"
///     #[compiler]
///     cpp_compiler = 'CLANG'
///     cpp_standard = '20'
///     std_lib = 'libcpp'
///     extra_args = ['-O3', '-Wall']
///"#;
///
/// let config: CompilerAttribute = toml::from_str(CONFIG_FILE_MOCK)
///    .expect("A failure happened parsing the Zork toml file");
///
/// assert_eq!(config.cpp_compiler, CppCompiler::CLANG);
/// assert_eq!(config.cpp_standard, LanguageLevel::CPP20);
/// assert_eq!(config.std_lib, Some(StdLib::LIBCPP));
/// assert_eq!(config.extra_args, Some(vec!["-O3", "-Wall"]));
/// assert_eq!(config.system_headers_path, None);
/// ```
///
/// > Note: TOML table are toml commented (#) to allow us to parse
/// the inner attributes as the direct type that they belongs to.
/// That commented tables aren't the real TOML, they are just there
/// for testing and exemplification purposes of the inner attributes
/// of the configuration file.
///
/// For a test over a real example, please look at the
/// [`zork::config_file::ZorkConfigFile`] doc-test
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct CompilerAttribute<'a> {
    pub cpp_compiler: CppCompiler,
    #[serde(borrow)]
    pub driver_path: Option<&'a str>,
    pub cpp_standard: LanguageLevel,
    pub std_lib: Option<StdLib>,
    #[serde(borrow)]
    pub extra_args: Option<Vec<&'a str>>,
    #[serde(borrow)]
    pub system_headers_path: Option<&'a str>,
}

/// The C++ compilers available within Zork++
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum CppCompiler {
    #[serde(alias = "CLANG", alias = "Clang", alias = "clang")]
    #[default]
    CLANG,
    #[serde(alias = "MSVC", alias = "Msvc", alias = "msvc")]
    MSVC,
    #[serde(alias = "GCC", alias = "Gcc", alias = "gcc")]
    GCC,
    // Possible future interesting on support the Intel's C++ compiler?
}

// Clippy warns to prefer implementing the From trait instead of Into.
// That would require that the project model know about config_file details, which is ugly.
#[allow(clippy::from_over_into)]
impl Into<project_model::compiler::CppCompiler> for CppCompiler {
    fn into(self) -> project_model::compiler::CppCompiler {
        match self {
            CppCompiler::CLANG => project_model::compiler::CppCompiler::CLANG,
            CppCompiler::MSVC => project_model::compiler::CppCompiler::MSVC,
            CppCompiler::GCC => project_model::compiler::CppCompiler::GCC,
        }
    }
}

/// The C++ ISO standard levels of the language, represented as an
/// enumerated type in Rust
///
/// Variants *2A* and *2B* represents Clang's way of
/// use the latest features available
///
/// Variant *LATEST* is the `MSVC` specific way of set the language
/// standard level to the latest features available in Microsoft's compiler
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum LanguageLevel {
    #[serde(alias = "20")]
    #[default]
    CPP20,
    #[serde(alias = "23")]
    CPP23,
    #[serde(alias = "2a")]
    CPP2A,
    #[serde(alias = "2b")]
    CPP2B,
    #[serde(alias = "latest")]
    LATEST,
}

// Clippy warns to prefer implementing the From trait instead of Into.
// That would require that the project model know about config_file details, which is ugly.
#[allow(clippy::from_over_into)]
impl Into<project_model::compiler::LanguageLevel> for LanguageLevel {
    fn into(self) -> project_model::compiler::LanguageLevel {
        match self {
            LanguageLevel::CPP20 => project_model::compiler::LanguageLevel::CPP20,
            LanguageLevel::CPP23 => project_model::compiler::LanguageLevel::CPP23,
            LanguageLevel::CPP2A => project_model::compiler::LanguageLevel::CPP2A,
            LanguageLevel::CPP2B => project_model::compiler::LanguageLevel::CPP2B,
            LanguageLevel::LATEST => project_model::compiler::LanguageLevel::LATEST,
        }
    }
}

/// The standard library (compiler specific) that the user
/// desires to link against
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StdLib {
    #[serde(alias = "libstdc++", alias = "gccstdlib", alias = "libstdcpp")]
    STDLIBCPP,
    #[serde(alias = "libc++", alias = "libcpp")]
    LIBCPP,
}

// Clippy warns to prefer implementing the From trait instead of Into.
// That would require that the project model know about config_file details, which is ugly.
#[allow(clippy::from_over_into)]
impl Into<project_model::compiler::StdLib> for StdLib {
    fn into(self) -> project_model::compiler::StdLib {
        match self {
            StdLib::STDLIBCPP => project_model::compiler::StdLib::STDLIBCPP,
            StdLib::LIBCPP => project_model::compiler::StdLib::LIBCPP,
        }
    }
}
