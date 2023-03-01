use std::{path::{Path, PathBuf}, process::ExitStatus};

///! Contains helpers and data structure to process in
/// a nice and neat way the commands generated to be executed
/// by Zork++
use crate::{cache::ZorkCache, project_model::compiler::CppCompiler, utils::constants};
use color_eyre::{eyre::Context, Result, Report};
use serde::{Deserialize, Serialize};

use super::arguments::Argument;


pub fn run_generated_commands(commands: &mut Commands<'_>, cache: &ZorkCache) -> Result<()> {
    if !commands.interfaces.is_empty() {
        log::debug!("Executing the commands for the module interfaces...");
    }
    for miu in &mut commands.interfaces {
        if !miu.processed {
            miu.execution_result = CommandExecutionResult::from(
                execute_command(&commands.compiler, &miu.args, cache)
            );
        } else {
            miu.execution_result = CommandExecutionResult::Cached;
            log::debug!("Translation unit: {:?} was not modified since the last iteration. No need to rebuilt it again", miu.path);
        }
    }

    if !commands.implementations.is_empty() {
        log::debug!("Executing the commands for the module implementations...");
    }
    for implm in &mut commands.implementations {
        if !implm.processed {
            implm.execution_result = CommandExecutionResult::from(
                execute_command(&commands.compiler, &implm.args, cache)
            );
        } else {
            implm.execution_result = CommandExecutionResult::Cached;
            log::debug!("Translation unit: {:?} was not modified since the last iteration. No need to rebuilt it again", implm.path);
        }
    }

    if !commands.sources.args.is_empty() {
        log::debug!("Executing the main command line...");
        commands.sources.execution_result = CommandExecutionResult::from(
            execute_command(&commands.compiler, &commands.sources.args, cache)
        );
    }

    Ok(())
}

/// Executes a new [`std::process::Command`] to run the generated binary
/// after the build process in the specified shell
pub fn autorun_generated_binary(
    compiler: &CppCompiler,
    output_dir: &Path,
    executable_name: &str,
) -> Result<()> {
    let args = &[Argument::from(
        output_dir
            .join(compiler.as_ref())
            .join(executable_name)
            .with_extension(constants::BINARY_EXTENSION),
    )];

    log::info!(
        "[{compiler}] - Executing the generated binary => {:?}",
        args.join(" ")
    );

    std::process::Command::new(Argument::from(
        output_dir.join(compiler.as_ref()).join(executable_name),
    ))
    .spawn()?
    .wait()
    .with_context(|| format!("[{compiler}] - Command {:?} failed!", args.join(" ")))?;

    Ok(())
}

/// Executes a new [`std::process::Command`] configured according the choosen
/// compiler and the current operating system
fn execute_command(
    compiler: &CppCompiler,
    arguments: &[Argument<'_>],
    cache: &ZorkCache,
) -> Result<ExitStatus, Report> {
    log::debug!(
        "[{compiler}] - Executing command => {:?}",
        format!("{} {}", compiler.get_driver(), arguments.join(" "))
    );

    let process = if compiler.eq(&CppCompiler::MSVC) {
        std::process::Command::new(
            cache
                .compilers_metadata
                .msvc
                .dev_commands_prompt
                .as_ref()
                .expect("Zork++ wasn't able to found a correct installation of MSVC"),
        )
        .arg("&&")
        .arg(compiler.get_driver())
        .args(arguments)
        .spawn()?
        .wait()
        .with_context(|| format!("[{compiler}] - Command {:?} failed!", arguments.join(" ")))
    } else {
        std::process::Command::new(compiler.get_driver())
            .args(arguments)
            .spawn()?
            .wait()
            .with_context(|| format!("[{compiler}] - Command {:?} failed!", arguments.join(" ")))
    };

    log::debug!("[{compiler}] - Result: {:?}\n", process);
    
    process
}

/// Executes a new [`std::process::Command`] configured according the choosen
/// compiler and the current operating system composed of multiple prebuilt command
/// lines joining them in one statement
///
/// TODO! Probably it would be better only make a big command formed by all the commands
/// for the MSVC compiler in order to avoid to launch the developers command prompt
/// for every commmand, but, as observed, generally speaking opening a shell under
/// Unix using Clang or GCC it's extremily fast, so we can mantain the curren architecture
/// of opening a shell for command, so the user is able to track better failed commands
fn _execute_commands(
    compiler: &CppCompiler,
    arguments_for_commands: &[Vec<Argument<'_>>],
    cache: &ZorkCache,
) -> Result<()> {
    let mut commands = if compiler.eq(&CppCompiler::MSVC) {
        std::process::Command::new(
            cache
                .compilers_metadata
                .msvc
                .dev_commands_prompt
                .as_ref()
                .expect("Zork++ wasn't able to found a correct installation of MSVC"),
        )
    } else {
        std::process::Command::new("sh")
    };

    arguments_for_commands.iter().for_each(|args_collection| {
        log::debug!(
            "[{compiler}] - Generating command => {:?}",
            format!("{} {}", compiler.get_driver(), args_collection.join(" "))
        );

        commands
            .arg("&&")
            .arg(compiler.get_driver())
            .args(args_collection);
    });

    commands
        .spawn()?
        .wait()
        .with_context(|| format!("[{compiler}] - Command {commands:?} failed!"))?;

    log::info!("[{compiler}] - Result: {:?}", commands);
    Ok(())
}

/// Holds a collection of heap allocated arguments. This is introduced in the
/// v0.7.0, for just wrapping the vector that holds the arguments, and for hold
/// a flag that will indicate us that this command line will be used in a module,
/// and that module was already built, and the module source file didn't change
/// since the last iteration of Zork++
#[derive(Debug)]
pub struct ModuleCommandLine<'a> {
    pub path: PathBuf,
    pub args: Vec<Argument<'a>>,
    pub processed: bool,
    pub execution_result: CommandExecutionResult
}

impl<'a> From<Vec<Argument<'a>>> for ModuleCommandLine<'a> {
    fn from(value: Vec<Argument<'a>>) -> Self {
        Self {
            path: PathBuf::new(),
            args: value,
            processed: false,
            execution_result: Default::default()
        }
    }
}

impl<'a> IntoIterator for ModuleCommandLine<'a> {
    type Item = Argument<'a>;
    type IntoIter = std::vec::IntoIter<Argument<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.into_iter()
    }
}

#[derive(Debug, Default)]
pub struct SourcesCommandLine<'a> {
    pub sources_paths: Vec<PathBuf>,
    pub args: Vec<Argument<'a>>,
    // pub processed: bool, // TODO sure?
    pub execution_result: CommandExecutionResult
}

/// Holds the generated command line arguments for a concrete compiler
#[derive(Debug)]
pub struct Commands<'a> {
    pub compiler: CppCompiler,
    pub system_modules: Vec<Vec<Argument<'a>>>,
    pub interfaces: Vec<ModuleCommandLine<'a>>,
    pub implementations: Vec<ModuleCommandLine<'a>>,
    pub sources: SourcesCommandLine<'a>,
    pub generated_files_paths: Vec<Argument<'a>>,
}

impl<'a> Commands<'a> {
    pub fn new(compiler: &'a CppCompiler) -> Self {
        Self {
            compiler: *compiler,
            system_modules: Vec::with_capacity(0),
            interfaces: Vec::with_capacity(0),
            implementations: Vec::with_capacity(0),
            sources: SourcesCommandLine::default(),
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
            self.interfaces.iter().map(|vec| { vec.args.iter().map(|e| e.value).collect::<Vec<_>>().join(" "); }),
            self.implementations.iter().map(|vec| { vec.args.iter().map(|e| e.value).collect::<Vec<_>>().join(" "); }),
            self.sources.args.iter().map(|e| e.value).collect::<Vec<_>>().join(" ")
        )
    }
}

/// Holds a custom representation of the execution of
/// a command line in a shell.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub enum CommandExecutionResult {
    /// A command that is executed correctly
    Success,
    /// A skipped command due to previous successful iterations
    Cached,
    /// A command which is return code indicates an unsuccessful execution 
    Failed,
    /// The execution failed, returning a [`Result`] with the Err variant
    Error(String),
    /// An status before storing a command execution result
    #[default] Unitialized
}

impl From<Result<ExitStatus, Report>> for CommandExecutionResult {
    fn from(value: Result<ExitStatus, Report>) -> Self {
        match value {
            Ok(r) => {
                if r.success() { CommandExecutionResult::Success } else {
                    CommandExecutionResult::Failed
                }
            },
            Err(e) => CommandExecutionResult::Error(e.to_string()),
        }
    }
}
