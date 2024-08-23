use core::fmt;
use std::{borrow::Cow, path::Path};

use crate::domain::commands::arguments::Argument;
use serde::{Deserialize, Serialize};

use crate::domain::target::ExtraArgs;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CompilerModel<'a> {
    pub cpp_compiler: CppCompiler,
    pub driver_path: Cow<'a, str>,
    pub cpp_standard: LanguageLevel,
    pub std_lib: Option<StdLib>,
    pub std_lib_installed_dir: Option<Cow<'a, Path>>,
    pub extra_args: Vec<Argument<'a>>,
}

impl<'a> CompilerModel<'a> {
    pub fn language_level(&self) -> Cow<'static, str> {
        match self.cpp_compiler {
            CppCompiler::CLANG | CppCompiler::GCC => format!("-std=c++{}", self.cpp_standard),
            CppCompiler::MSVC => format!("/std:c++{}", self.cpp_standard),
        }
        .into()
    }

    pub fn language_level_arg(&self) -> Argument {
        Argument::from(self.language_level())
    }

    pub fn stdlib_arg(&self) -> Option<Argument> {
        self.std_lib
            .as_ref()
            .map(|lib| Argument::from(format!("-stdlib={lib}")))
    }
}

impl<'a> ExtraArgs<'a> for CompilerModel<'a> {
    fn extra_args(&'a self) -> &'a [Argument] {
        &self.extra_args
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize, Default)]
pub enum CppCompiler {
    #[default]
    CLANG,
    MSVC,
    GCC,
}

impl fmt::Display for CppCompiler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl AsRef<str> for CppCompiler {
    fn as_ref(&self) -> &str {
        match *self {
            CppCompiler::CLANG => "clang",
            CppCompiler::MSVC => "msvc",
            CppCompiler::GCC => "gcc",
        }
    }
}

impl CppCompiler {
    /// Returns an &str representing the compiler driver that will be called
    /// in the command line to generate the build events
    pub fn get_driver<'a>(&self, compiler_model: &'a CompilerModel) -> Cow<'a, str> {
        if !compiler_model.driver_path.is_empty() {
            Cow::Borrowed(&compiler_model.driver_path)
        } else {
            Cow::Borrowed(match *self {
                CppCompiler::CLANG => "clang++",
                CppCompiler::MSVC => "cl",
                CppCompiler::GCC => "g++",
            })
        }
    }

    pub fn default_module_extension<'a>(&self) -> Cow<'a, str> {
        Cow::Borrowed(match *self {
            CppCompiler::CLANG => "cppm",
            CppCompiler::MSVC => "ixx",
            CppCompiler::GCC => "cc",
        })
    }
    pub fn get_default_module_extension<'a>(&self) -> Cow<'a, str> {
        Cow::Borrowed(match *self {
            CppCompiler::CLANG => "cppm",
            CppCompiler::MSVC => "ixx",
            CppCompiler::GCC => "cc",
        })
    }

    pub fn typical_bmi_extension(&self) -> Cow<'_, str> {
        Cow::Borrowed(match *self {
            CppCompiler::CLANG => "pcm",
            CppCompiler::MSVC => "ifc",
            CppCompiler::GCC => "o",
        })
    }

    pub fn get_typical_bmi_extension(&self) -> &str {
        match *self {
            CppCompiler::CLANG => "pcm",
            CppCompiler::MSVC => "ifc",
            CppCompiler::GCC => "o",
        }
    }

    #[inline(always)]
    pub fn obj_file_extension(&self) -> Cow<'_, str> {
        Cow::Borrowed(match *self {
            CppCompiler::CLANG | CppCompiler::GCC => "o",
            CppCompiler::MSVC => "obj",
        })
    }

    #[inline(always)]
    pub fn get_obj_file_extension(&self) -> &str {
        match *self {
            CppCompiler::CLANG | CppCompiler::GCC => "o",
            CppCompiler::MSVC => "obj",
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum LanguageLevel {
    CPP20,
    #[default]
    CPP23,
    CPP2A,
    CPP2B,
    LATEST,
}

impl fmt::Display for LanguageLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl AsRef<str> for LanguageLevel {
    fn as_ref(&self) -> &'static str {
        match *self {
            LanguageLevel::CPP20 => "20",
            LanguageLevel::CPP23 => "23",
            LanguageLevel::CPP2A => "2a",
            LanguageLevel::CPP2B => "2b",
            LanguageLevel::LATEST => "latest",
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum StdLib {
    STDLIBCPP,
    #[default]
    LIBCPP,
}

impl StdLib {
    pub fn as_arg(&self) -> Argument {
        Argument::from(match *self {
            StdLib::STDLIBCPP => "-stdlib=libstdc++",
            StdLib::LIBCPP => "-stdlib=libc++",
        })
    }
}

impl fmt::Display for StdLib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl AsRef<str> for StdLib {
    fn as_ref(&self) -> &str {
        match *self {
            StdLib::STDLIBCPP => "libstdc++",
            StdLib::LIBCPP => "libc++",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StdLibMode {
    Cpp,     //< The C++ STD library implemented for every vendor
    CCompat, //< Same, but extending it with the C ISO standard library
}

impl StdLibMode {
    pub fn printable_info(&self) -> &str {
        match self {
            StdLibMode::Cpp => "C++ standard library implementation",
            StdLibMode::CCompat => "C++ C compat standard library implementation",
        }
    }
}

impl fmt::Display for StdLibMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.printable_info())
    }
}
