use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{bounds::ExtraArgs, cli::output::arguments::Argument};

#[derive(Debug, PartialEq, Eq)]
pub struct CompilerModel<'a> {
    pub cpp_compiler: CppCompiler,
    pub cpp_standard: LanguageLevel,
    pub std_lib: Option<StdLib>,
    pub extra_args: Vec<Argument<'a>>,
}

impl<'a> CompilerModel<'a> {
    pub fn language_level_arg(&self) -> Argument {
        match self.cpp_compiler {
            CppCompiler::CLANG | CppCompiler::GCC => {
                Argument::from(format!("-std=c++{}", self.cpp_standard))
            }
            CppCompiler::MSVC => Argument::from(format!("-std:c++{}", self.cpp_standard)),
        }
    }

    pub fn stdlib_arg(&self) -> Option<Argument> {
        self.std_lib
            .as_ref()
            .map(|lib| Argument::from(format!("-stdlib={lib}")))
    }
}

impl<'a> ExtraArgs<'a> for CompilerModel<'a> {
    fn extra_args(&'a self) -> &'a [Argument<'a>] {
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
    pub fn get_driver(&self) -> &str {
        match *self {
            CppCompiler::CLANG => "clang++",
            CppCompiler::MSVC => "cl",
            CppCompiler::GCC => "g++",
        }
    }

    pub fn get_default_module_extension(&self) -> &str {
        match *self {
            CppCompiler::CLANG => "cppm",
            CppCompiler::MSVC => "ixx",
            CppCompiler::GCC => "cc",
        }
    }

    pub fn get_typical_bmi_extension(&self) -> &str {
        match *self {
            CppCompiler::CLANG => "pcm",
            CppCompiler::MSVC => "ifc",
            CppCompiler::GCC => "o",
        }
    }

    #[inline(always)]
    pub fn get_obj_file_extension(&self) -> &str {
        "o"
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LanguageLevel {
    CPP20,
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

#[derive(Debug, PartialEq, Eq)]
pub enum StdLib {
    STDLIBCPP,
    LIBCPP,
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
