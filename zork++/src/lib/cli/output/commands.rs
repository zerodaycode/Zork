use std::path::Path;

///! Contains helpers and data structure to process in
/// a nice and neat way the commands generated to be executed
/// by Zork++

use crate::project_model::compiler::CppCompiler;
use color_eyre::{eyre::Context, Result};

use super::arguments::Argument;

/// Executes a new [`std::process::Command`] configured according the choosen
/// compiler and the current operating system
pub fn execute_command(compiler: &CppCompiler, arguments: &[Argument<'_>]) -> Result<()> {
    log::info!(
        "[{compiler}] - Executing command => {:?}",
        format!("{} {}", compiler.get_driver(), arguments.join(" "))
    );

    let process = if compiler.eq(&CppCompiler::MSVC) {
        std::process::Command::new( // TODO The initialization process + cache process MUST dynamically get this path and store it in cache
            "C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Auxiliary\\Build\\vcvars64.bat"
        ).arg("&&")
            .arg(compiler.get_driver())
            .args(arguments)
            .spawn()?
            .wait()
            .with_context(|| format!("[{compiler}] - Command {:?} failed!", arguments.join(" ")))?
    } else {
        std::process::Command::new(compiler.get_driver())
            .args(arguments)
            .spawn()?
            .wait()
            .with_context(|| format!("[{compiler}] - Command {:?} failed!", arguments.join(" ")))?
    };

    log::info!("[{compiler}] - Result: {:?}", process);
    Ok(())
}

/// Executes a new [`std::process::Command`] configured according the choosen
/// compiler and the current operating system composed of multiple prebuilt command
/// lines joining them in one statement
pub fn execute_commands(compiler: &CppCompiler, arguments: &[Argument<'_>]) -> Result<()> {
    log::info!(
        "[{compiler}] - Executing command => {:?}",
        format!("{} {}", compiler.get_driver(), arguments.join(" "))
    );

    let process = if compiler.eq(&CppCompiler::MSVC) {
        std::process::Command::new( // TODO The initialization process + cache process MUST dynamically get this path and store it in cache
            "C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Auxiliary\\Build\\vcvars64.bat"
        ).arg("&&")
            .arg(compiler.get_driver())
            .args(arguments)
            .spawn()?
            .wait()
            .with_context(|| format!("[{compiler}] - Command {:?} failed!", arguments.join(" ")))?
    } else {
        std::process::Command::new(compiler.get_driver())
            .args(arguments)
            .spawn()?
            .wait()
            .with_context(|| format!("[{compiler}] - Command {:?} failed!", arguments.join(" ")))?
    };

    log::info!("[{compiler}] - Result: {:?}", process);
    Ok(())
}


/// Executes a new [`std::process::Command`] to run the generated binary
/// after the build process in the specified shell
pub fn autorun_generated_binary(
    compiler: &CppCompiler,
    output_dir: &Path,
    executable_name: &str
    // arguments: &[Argument<'_>]
) -> Result<()> {
    let args = &[
        Argument::from(
            output_dir.join(compiler.as_ref()).join(executable_name)
        ),
    ];

    log::info!(
        "\n\n[{compiler}] - Executing the generated binary => {:?}", args.join(" ")
    );

    // TODO still need to match against OS to launch cmd on Windows 

    let process = std::process::Command::new(
        Argument::from(
        output_dir.join(compiler.as_ref()).join(executable_name)
    )).spawn()?
    .wait()
    .with_context(|| format!("[{compiler}] - Command {:?} failed!", args.join(" ")))?;

    log::info!("[{compiler}] - Result: {:?}", process);
    Ok(())
}

/// A kind of caché of the generated command lines
#[derive(Debug)]
pub struct Commands<'a> {
    pub compiler: &'a CppCompiler,
    pub interfaces: Vec<Vec<Argument<'a>>>,
    pub implementations: Vec<Vec<Argument<'a>>>,
    pub sources: Vec<Argument<'a>>,
    pub generated_files_paths: Vec<Argument<'a>>,
}

impl<'a> Commands<'a> {
    pub fn new(compiler: &'a CppCompiler) -> Self {
        Self {
            compiler,
            interfaces: Vec::with_capacity(0),
            implementations: Vec::with_capacity(0),
            sources: Vec::with_capacity(0),
            generated_files_paths: Vec::with_capacity(0),
        }
    }
}

impl<'a> core::fmt::Display for Commands<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Commands for [{}]:\n- Interfaces: {:?},\n- Implementations: {:?},\n- Main command line: {:?}",
            self.compiler,
            self.interfaces.iter().map(|vec| { vec.iter().map(|e| e.value).collect::<Vec<_>>().join(" "); }),
            self.implementations.iter().map(|vec| { vec.iter().map(|e| e.value).collect::<Vec<_>>().join(" "); }),
            self.sources.iter().map(|e| e.value).collect::<Vec<_>>().join(" ")
        )
    }
}
