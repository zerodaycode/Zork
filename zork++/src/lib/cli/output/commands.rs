//! Contains helpers and data structures to be processed in a nice and neat way the commands generated to be executed
//! by Zork++

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::slice::Iter;
use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
};

use crate::bounds::TranslationUnit;
use crate::cli::output::arguments::Arguments;
use crate::compiler::data_factory::{CommonArgs, CompilerCommonArguments};
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
    mut commands: Commands, // TODO: &mut, and then directly store them?
    cache: &mut ZorkCache,
    test_mode: bool,
) -> Result<CommandExecutionResult> {
    log::info!("Proceeding to execute the generated commands...");
    let compiler = commands.compiler;

    for sys_module in &commands.system_modules {
        // TODO: will be deprecated soon, hopefully
        execute_command(compiler, program_data, sys_module.1, cache)?;
    }

    let translation_units = commands
        .pre_tasks
        .iter_mut()
        .chain(commands.interfaces.iter_mut())
        .chain(commands.implementations.iter_mut())
        .chain(commands.sources.iter_mut());

    for translation_unit_cmd in translation_units {
        if translation_unit_cmd.need_to_build {
            let r = execute_command(compiler, program_data, &translation_unit_cmd.args, cache);
            translation_unit_cmd.execution_result = CommandExecutionResult::from(&r);
            if let Err(e) = r {
                cache::save(program_data, cache, commands, test_mode)?;
                return Err(e);
            } else if !r.as_ref().unwrap().success() {
                let err = eyre!(
                    "Ending the program, because the build of: {:?} wasn't ended successfully",
                    translation_unit_cmd.filename
                );
                cache::save(program_data, cache, commands, test_mode)?;
                return Err(err);
            }
        }
    }

    if !commands.linker.args.is_empty() {
        log::debug!("Processing the linker command line...");

        let r = execute_command(compiler, program_data, &commands.linker.args, cache);
        commands.linker.execution_result = CommandExecutionResult::from(&r);

        if let Err(e) = r {
            cache::save(program_data, cache, commands, test_mode)?;
            return Err(e);
        } else if !r.as_ref().unwrap().success() {
            cache::save(program_data, cache, commands, test_mode)?;
            return Err(eyre!(
                "Ending the program, because the linker command line execution wasn't ended successfully",
            ));
        }
    }

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
    arguments: &[Argument],
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

    let driver = compiler.get_driver(&model.compiler);
    let os_driver = OsStr::new(driver.as_ref());
    std::process::Command::new(os_driver)
        .args(arguments)
        .envs(cache.get_process_env_args())
        .spawn()?
        .wait()
        .with_context(|| format!("[{compiler}] - Command {:?} failed!", arguments.join(" ")))
}

/// The pieces and details for the generated command line
/// for some translation unit
///
/// * args* : member that holds all the cmd arguments that will be passed to the compiler driver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCommandLine {
    pub directory: PathBuf,
    pub filename: String,
    pub args: Arguments,
    pub need_to_build: bool,
    pub execution_result: CommandExecutionResult,
    // TODO an enum with the Kind OF TU that is generating this scl?
}

impl SourceCommandLine {
    pub fn from_translation_unit(
        // TODO init it as a args holder, but doesn't have the status yet
        tu: impl TranslationUnit,
        args: Arguments, // TODO: maybe this should be an option? Cached arguments are passed
        // here as default. So probably, even better than having an optional,
        // we must replicate this to have a separate entity like
        // CachedSourceCommandLine, and them just call them over any kind of
        // <T> constrained over some bound that wraps the operation of
        // distinguish between them or not
        processed: bool,
        execution_result: CommandExecutionResult,
    ) -> Self {
        Self {
            directory: tu.path(),
            filename: tu.file_with_extension(),
            args,
            need_to_build: !processed,
            execution_result,
        }
    }

    pub fn for_translation_unit(
        // TODO init it as a args holder, but doesn't have the status yet
        tu: impl TranslationUnit,
        args: Arguments,
    ) -> Self {
        Self {
            directory: tu.path(),
            filename: tu.file_with_extension(),
            args,
            need_to_build: true,
            execution_result: CommandExecutionResult::Unprocessed,
        }
    }

    pub fn path(&self) -> PathBuf {
        self.directory.join(Path::new(&self.filename))
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LinkerCommandLine {
    // pub main: &'a Path, // TODO: can't this disappear? At the end of the day, is just another obj file
    pub built_files: Vec<PathBuf>,
    pub args: Vec<Argument>,
    pub execution_result: CommandExecutionResult,
}

impl LinkerCommandLine {
    /// Saves the path at which a compilation product of any translation unit will be placed,
    /// in order to add it to the files that will be linked to generate the final product
    /// in the two-phase compilation model
    pub fn add_buildable_at(&mut self, path: &Path) {
        self.built_files.push(path.to_path_buf());
    }

    /// Owned version of TODO link
    pub fn add_owned_buildable_at(&mut self, path: PathBuf) {
        self.built_files.push(path);
    }
}

/// Holds the generated command line arguments for a concrete compiler
#[derive(Serialize, Deserialize, Default)]
pub struct Commands {
    pub compiler: CppCompiler,
    pub pre_tasks: Vec<SourceCommandLine>, // TODO: since there's no really pre-tasks (only build the std_lib), create named entries for std and std.compat
    pub system_modules: HashMap<String, Arguments>,

    pub general_args: CommonArgs,
    pub compiler_common_args: Box<dyn CompilerCommonArguments>,

    pub interfaces: Vec<SourceCommandLine>,
    pub implementations: Vec<SourceCommandLine>,
    pub sources: Vec<SourceCommandLine>,
    pub linker: LinkerCommandLine,
}

impl Commands {
    pub fn new(
        model: &ZorkModel<'_>,
        general_args: CommonArgs,
        compiler_specific_common_args: Box<dyn CompilerCommonArguments>,
    ) -> Self {
        Self {
            // TODO: try to see if its possible to move around the code and have a From<T>, avoiding default initialization,
            // since this will always cause reallocations, and 'from' may be able to allocate at the exact required capacity
            // of every collection
            compiler: model.compiler.cpp_compiler,

            general_args,
            compiler_common_args: compiler_specific_common_args,

            pre_tasks: Vec::with_capacity(0),
            system_modules: HashMap::with_capacity(0),
            interfaces: Vec::with_capacity(0),
            implementations: Vec::with_capacity(0),
            sources: Vec::with_capacity(0),
            linker: LinkerCommandLine::default(),
        }
    }

    pub fn add_linker_file_path(&mut self, path: &Path) {
        self.linker.add_buildable_at(path);
    }

    pub fn add_linker_file_path_owned(&mut self, path: PathBuf) {
        self.linker.add_owned_buildable_at(path);
    }
}

impl core::fmt::Display for Commands {
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
fn collect_source_command_line(
    iter: Iter<'_, SourceCommandLine>, // TODO: review this, for see if it's possible to consume the value and not cloning it
) -> impl Iterator + Debug + '_ {
    iter.map(|vec| {
        vec.args
            .iter()
            .map(|arg| arg.value().clone())
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
    Unprocessed,
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
