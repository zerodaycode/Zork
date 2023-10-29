use std::collections::HashMap;
use std::fmt::Debug;
use std::slice::Iter;
use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
};

use crate::bounds::TranslationUnit;
use crate::cli::output::arguments::Arguments;
/// Contains helpers and data structure to process in
/// a nice and neat way the commands generated to be executed
/// by Zork++
use crate::{
    cache::{self, ZorkCache},
    project_model::{compiler::CppCompiler, ZorkModel},
    utils::constants,
};
use color_eyre::{
    eyre::{eyre, Context},
    Report, Result,
};
use serde::{Deserialize, Serialize};

use super::arguments::Argument;

pub fn run_generated_commands(
    program_data: &ZorkModel<'_>,
    mut commands: Commands<'_>,
    cache: &mut ZorkCache,
    test_mode: bool,
) -> Result<CommandExecutionResult> {
    log::info!("Proceeding to execute the generated commands...");
    let mut total_exec_commands = 0;
    let compiler = commands.compiler;

    for sys_module in &commands.system_modules {
        execute_command(compiler, program_data, sys_module.1, cache)?;
    }

    let sources = commands
        .interfaces
        .iter_mut()
        .chain(commands.implementations.iter_mut())
        .chain(commands.sources.iter_mut());

    for source_file in sources {
        if !source_file.processed {
            let r = execute_command(compiler, program_data, &source_file.args, cache);
            source_file.execution_result = CommandExecutionResult::from(&r);
            total_exec_commands += 1;
            if let Err(e) = r {
                cache::save(program_data, cache, commands, test_mode)?;
                return Err(e);
            } else if !r.as_ref().unwrap().success() {
                let err = eyre!(
                    "Ending the program, because the build of: {:?} wasn't ended successfully",
                    source_file.file
                );
                cache::save(program_data, cache, commands, test_mode)?;
                return Err(err);
            }
        }
    }

    if !commands.main.args.is_empty() {
        log::debug!("Executing the main command line...");

        let r = execute_command(compiler, program_data, &commands.main.args, cache);
        commands.main.execution_result = CommandExecutionResult::from(&r);

        if let Err(e) = r {
            cache::save(program_data, cache, commands, test_mode)?;
            return Err(e);
        } else if !r.as_ref().unwrap().success() {
            cache::save(program_data, cache, commands, test_mode)?;
            return Err(eyre!(
                "Ending the program, because the main command line execution wasn't ended successfully",
            ));
        }
    }

    log::debug!("A total of: {total_exec_commands} command lines has been executed");
    cache::save(program_data, cache, commands, test_mode)?;
    Ok(CommandExecutionResult::Success)
}

/// Executes a new [`std::process::Command`] to run the generated binary
/// after the build process in the specified shell
pub fn autorun_generated_binary(
    compiler: &CppCompiler,
    output_dir: &Path,
    executable_name: &str,
) -> Result<CommandExecutionResult> {
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

    Ok(CommandExecutionResult::from(
        std::process::Command::new(Argument::from(
            output_dir.join(compiler.as_ref()).join(executable_name),
        ))
        .spawn()?
        .wait()
        .with_context(|| format!("[{compiler}] - Command {:?} failed!", args.join(" "))),
    ))
}

/// Executes a new [`std::process::Command`] configured according the chosen
/// compiler and the current operating system
fn execute_command(
    compiler: CppCompiler,
    model: &ZorkModel,
    arguments: &[Argument<'_>],
    cache: &ZorkCache,
) -> Result<ExitStatus, Report> {
    log::trace!(
        "[{compiler}] - Executing command => {:?}",
        format!(
            "{} {}",
            compiler.get_driver(&model.compiler),
            arguments.join(" ")
        )
    );

    if compiler.eq(&CppCompiler::MSVC) {
        std::process::Command::new(
            cache
                .compilers_metadata
                .msvc
                .dev_commands_prompt
                .as_ref()
                .expect("Zork++ wasn't able to found a correct installation of MSVC"),
        )
        .arg("&&")
        .arg(compiler.get_driver(&model.compiler))
        .args(arguments)
        .spawn()?
        .wait()
        .with_context(|| format!("[{compiler}] - Command {:?} failed!", arguments.join(" ")))
    } else {
        let driver = compiler.get_driver(&model.compiler);
        let command = std::process::Command::new(driver)
            .args(arguments)
            .spawn();
        let e = format!(
            "[{driver}] - Command {:?} executed! - Details: {:?}",
            arguments.join(" "),
            command
        );
        log::info!("{:?}", e);
        command?.wait().with_context(|| e)
    }
}

/// The pieces and details for the generated command line for
/// for some translation unit
#[derive(Debug)]
pub struct SourceCommandLine<'a> {
    pub directory: PathBuf,
    pub file: String,
    pub args: Arguments<'a>,
    pub processed: bool,
    pub execution_result: CommandExecutionResult,
}

impl<'a> SourceCommandLine<'a> {
    pub fn from_translation_unit(
        tu: impl TranslationUnit,
        args: Arguments<'a>,
        processed: bool,
        execution_result: CommandExecutionResult,
    ) -> Self {
        Self {
            directory: tu.path(),
            file: tu.file_with_extension(),
            args,
            processed,
            execution_result,
        }
    }

    pub fn path(&self) -> PathBuf {
        self.directory.join(Path::new(&self.file))
    }
}

#[derive(Debug)]
pub struct ExecutableCommandLine<'a> {
    pub main: &'a Path,
    pub sources_paths: Vec<PathBuf>,
    pub args: Vec<Argument<'a>>,
    pub execution_result: CommandExecutionResult,
}

impl<'a> Default for ExecutableCommandLine<'a> {
    fn default() -> Self {
        Self {
            main: Path::new("."),
            sources_paths: Vec::with_capacity(0),
            args: Vec::with_capacity(0),
            execution_result: Default::default(),
        }
    }
}

/// Holds the generated command line arguments for a concrete compiler
#[derive(Debug)]
pub struct Commands<'a> {
    pub compiler: CppCompiler,
    pub system_modules: HashMap<String, Arguments<'a>>,
    pub interfaces: Vec<SourceCommandLine<'a>>,
    pub implementations: Vec<SourceCommandLine<'a>>,
    pub sources: Vec<SourceCommandLine<'a>>,
    pub main: ExecutableCommandLine<'a>,
    pub generated_files_paths: Arguments<'a>,
}

impl<'a> Commands<'a> {
    pub fn new(compiler: &'a CppCompiler) -> Self {
        Self {
            compiler: *compiler,
            system_modules: HashMap::with_capacity(0),
            interfaces: Vec::with_capacity(0),
            implementations: Vec::with_capacity(0),
            sources: Vec::with_capacity(0),
            main: ExecutableCommandLine::default(),
            generated_files_paths: Arguments::default(),
        }
    }
}

impl<'a> core::fmt::Display for Commands<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Commands for [{}]:\n- Interfaces: {:?},\n- Implementations: {:?},\n- Sources: {:?}",
            self.compiler,
            collect_source_command_line(self.interfaces.iter()),
            collect_source_command_line(self.implementations.iter()),
            collect_source_command_line(self.sources.iter())
        )
    }
}

/// Convenient function to avoid code replication
fn collect_source_command_line<'a>(
    iter: Iter<'a, SourceCommandLine<'a>>,
) -> impl Iterator + Debug + 'a {
    iter.map(|vec| {
        vec.args
            .iter()
            .map(|e| e.value)
            .collect::<Vec<_>>()
            .join(" ");
    })
}

/// Holds a custom representation of the execution of
/// a command line in a shell.
#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum CommandExecutionResult {
    /// A command that is executed correctly
    Success,
    /// A skipped command due to previous successful iterations
    Cached,
    /// A command which is return code indicates an unsuccessful execution
    Failed,
    /// The execution failed, returning a [`Result`] with the Err variant
    Error,
    /// A previous state before executing a command line
    #[default]
    Unreached,
}

impl From<Result<ExitStatus, Report>> for CommandExecutionResult {
    fn from(value: Result<ExitStatus, Report>) -> Self {
        handle_command_execution_result(&value)
    }
}

impl From<&Result<ExitStatus, Report>> for CommandExecutionResult {
    fn from(value: &Result<ExitStatus, Report>) -> Self {
        handle_command_execution_result(value)
    }
}

/// Convenient way of handle a command execution result avoiding duplicate code
fn handle_command_execution_result(value: &Result<ExitStatus>) -> CommandExecutionResult {
    match value {
        Ok(r) => {
            if r.success() {
                CommandExecutionResult::Success
            } else {
                CommandExecutionResult::Failed
            }
        }
        Err(_) => CommandExecutionResult::Error,
    }
}
