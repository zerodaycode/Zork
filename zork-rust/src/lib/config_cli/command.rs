///! TODO module level doc

use clap::{Subcommand, Args, ValueEnum};


/// TODO
#[derive(Subcommand)]
pub enum Command {
    CreateProject(CommandCreateProject)
}

/// TODO
///
#[derive(Args)]
pub struct CommandCreateProject {
    name: Option<String>,
    legacy: bool,
    git: bool,
    compiler: CppCompiler
}


/// The C++ compilers available within Zork++ as a command line argument for the `new` argument
#[derive(ValueEnum,Clone)]
pub enum CppCompiler {
    CLANG,
    MSVC,
    GCC
    // Possible future interesting on support the Intel's C++ compiler?
}