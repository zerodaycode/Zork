use std::fmt::Display;

use crate::config_file::compiler;
use clap::{Parser, Subcommand, ValueEnum};


/// [`CliArgs`] is the command line arguments parser
///
/// #Test
/// ```rust
/// use clap::Parser;
/// use zork::cli::{CliArgs, Command, CppCompiler};
///
/// let parser = CliArgs::parse_from(["", "-vv"]);
/// assert_eq!(2, parser.verbose);
///
/// let parser = CliArgs::parse_from(["", "tests"]);
/// assert_eq!(parser.command, Some(Command::Tests));
///
// Create Template Project
/// let parser = CliArgs::parse_from(["", "-n", "--git", "--compiler", "clang"]);
/// assert_eq!(parser.new_template, true);
/// assert_eq!(parser.git, true);
/// assert_eq!(parser.compiler, Some(CppCompiler::CLANG));
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
    pub command: Option<Command>,

    #[arg(short, long, action = clap::ArgAction::Count, help="Zork++ maximum allowed verbosity level is: '-vv'")]
    pub verbose: u8,
}

/// [`Command`] -  The core enum commands
#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Command {
    /// Executes the tests under the specified directory in the config file
    Test,
    New {
        #[arg(long, help = "Initialize a new local git repo")]
        git: bool,
        #[arg(long, default_value_t = CppCompiler::CLANG, help = "Which compiler to use")]
        compiler: CppCompiler,
    }
}

/// [`CppCompiler`] The C++ compilers available within Zork++ as a command line argument for the `new` argument
/// TODO Possible future interesting on support the Intel's C++ compiler?
#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum CppCompiler {
    CLANG,
    MSVC,
    GCC,
}

impl Display for CppCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            CppCompiler::CLANG => write!(f, "clang"),
            CppCompiler::MSVC => write!(f, "msvc"),
            CppCompiler::GCC => write!(f, "gcc"),
        }
    }
}

impl Into<compiler::CppCompiler> for CppCompiler {
    fn into(self) -> compiler::CppCompiler {
        match self {
            CppCompiler::CLANG => compiler::CppCompiler::CLANG,
            CppCompiler::MSVC => compiler::CppCompiler::MSVC,
            CppCompiler::GCC => compiler::CppCompiler::GCC,
        }
    }
}