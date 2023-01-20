use core::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct CompilerModel {
    pub cpp_compiler: CppCompiler,
    pub cpp_standard: LanguageLevel,
    pub std_lib: Option<StdLib>,
    pub extra_args: Vec<String>,
    pub system_headers_path: Option<String>,
}

impl CompilerModel {
    pub fn language_level_arg(&self) -> String {
        match self.cpp_compiler {
            CppCompiler::CLANG | CppCompiler::GCC => {
                format!("-std=c++{}", self.cpp_standard.as_str())
            }
            CppCompiler::MSVC => format!("-std:c++{}", self.cpp_standard.as_str()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CppCompiler {
    CLANG,
    MSVC,
    GCC,
}

impl fmt::Display for CppCompiler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            CppCompiler::CLANG => write!(f, "clang"),
            CppCompiler::MSVC => write!(f, "msvc"),
            CppCompiler::GCC => write!(f, "gcc"),
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
            CppCompiler::CLANG => ".pcm",
            CppCompiler::MSVC => ".ifc",
            CppCompiler::GCC => ".o",
        }
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

impl LanguageLevel {
    pub fn as_str(&self) -> &str {
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

impl StdLib {
    pub fn as_str(&self) -> &str {
        match *self {
            StdLib::STDLIBCPP => "libstdc++",
            StdLib::LIBCPP => "libc++",
        }
    }
}
