use crate::config_file::compiler;
use clap::{Parser, Subcommand, ValueEnum};

/// [`CliArgs`] is the command line arguments parser
///
/// #Test
/// ```rust
/// use clap::Parser;
/// use zork::cli::{CliArgs, Command, CppCompiler};
///
/// let parser = CliArgs::parse_from(["", "-vv", "test"]);
/// assert_eq!(parser.command, Command::Test);
/// assert_eq!(parser.verbose, 2);
///
// Create Template Project
/// let parser = CliArgs::parse_from(["", "new", "example", "--git", "--compiler", "clang"]);
/// assert_eq!(parser.command, Command::New{name: "example".to_owned(), git: true, compiler: CppCompiler::CLANG});
/// ```
#[derive(Parser, Debug)]
#[command(name = "Zork++")]
#[command(author = "Zero Day Code")]
#[command(version = "0.5.0")]
#[command(
    about = "Zork++ is a build system for modern C++ projects",
    long_about = "Zork++ is a project of Zero Day Code. Find us: https://github.com/zerodaycode/Zork"
)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Command,

    #[arg(short, long, action = clap::ArgAction::Count, help="Zork++ maximum allowed verbosity level is: '-vv'")]
    pub verbose: u8,
}

/// [`Command`] -  The core enum commands
#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Command {
    /// Triggers the process that builds the project based on the config file directives
    Build,
    /// Executes the tests under the specified directory in the config file
    Test,
    /// Creates a new template project
    New {
        #[arg(help = "Name of the new project")]
        name: String,
        #[arg(long, help = "Initialize a new local git repo")]
        git: bool,
        #[arg(long, default_value = "clang", help = "Which compiler to use")]
        compiler: CppCompiler,
    },
    /// Operations with zork cache
    Cache {
        #[arg(long, help = "Show cache data")]
        show: bool,
    },
}

/// [`CppCompiler`] The C++ compilers available within Zork++ as a command line argument for the `new` argument
/// TODO Possible future interesting on support the Intel's C++ compiler?
#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum CppCompiler {
    CLANG,
    MSVC,
    GCC,
}

// Clippy warns to prefer implementing the From trait instead of Into.
// That would require that the project model know about cli details, which is ugly.
#[allow(clippy::from_over_into)]
impl Into<compiler::CppCompiler> for CppCompiler {
    fn into(self) -> compiler::CppCompiler {
        match self {
            CppCompiler::CLANG => compiler::CppCompiler::CLANG,
            CppCompiler::MSVC => compiler::CppCompiler::MSVC,
            CppCompiler::GCC => compiler::CppCompiler::GCC,
        }
    }
}
