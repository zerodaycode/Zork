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

use super::arguments::Argument;
use crate::bounds::TranslationUnit;
use crate::cache::EnvVars;
use crate::cli::output::arguments::Arguments;
use crate::compiler::data_factory::{CommonArgs, CompilerCommonArguments};
use crate::{
    cache::ZorkCache,
    project_model::{compiler::CppCompiler, ZorkModel},
    utils::constants,
};
use color_eyre::{
    eyre::{eyre, Context},
    Report, Result,
};
use serde::{Deserialize, Serialize};

pub fn run_generated_commands(
    program_data: &ZorkModel<'_>,
    cache: &mut ZorkCache<'_>,
) -> Result<CommandExecutionResult> {
    log::info!("Proceeding to execute the generated commands...");

    let general_args = &cache.generated_commands.general_args.get_args();
    let compiler_specific_shared_args = &cache.generated_commands.compiler_common_args.get_args();

    let env_args = cache.get_process_env_args().clone(); // TODO: this is yet better than clone the
                                                         // generated commands (maybe) but I'm not
                                                         // happy with it

    for sys_module in &cache.generated_commands.system_modules {
        // TODO: will be deprecated soon, hopefully
        // But while isn't deleted, we could normalize them into SourceCommandLine
        // And then, consider to join them into the all generated commands iter
        execute_command(program_data, sys_module.1, &env_args)?;
    }

    let translation_units = cache
        .generated_commands
        .get_all_command_lines()
        .filter(|scl| scl.need_to_build)
        .collect::<Vec<&mut SourceCommandLine>>();

    let compile_but_dont_link: [Argument; 1] =
        [Argument::from(match program_data.compiler.cpp_compiler {
            CppCompiler::CLANG | CppCompiler::GCC => "-c",
            CppCompiler::MSVC => "/c",
        })];

    for translation_unit_cmd in translation_units {
        // Join the concrete args of any translation unit with the ones held in the flyweights
        let translation_unit_cmd_args: Arguments = general_args
            .iter()
            .chain(compiler_specific_shared_args.iter())
            .chain(&compile_but_dont_link)
            .chain(translation_unit_cmd.args.iter())
            .collect();

        let r = execute_command(program_data, &translation_unit_cmd_args, &env_args);
        translation_unit_cmd.execution_result = CommandExecutionResult::from(&r);

        if let Err(e) = r {
            return Err(e);
        } else if !r.as_ref().unwrap().success() {
            let err = eyre!(
                "Ending the program, because the build of: {:?} failed",
                translation_unit_cmd.filename
            );
            return Err(err);
        }
    }

    if !cache.generated_commands.linker.args.is_empty() {
        log::debug!("Processing the linker command line...");

        let r = execute_command(
            program_data,
            &general_args
                .iter()
                .chain(compiler_specific_shared_args.iter())
                .chain(cache.generated_commands.linker.args.iter())
                .collect::<Arguments>(),
            &env_args,
        );

        cache.generated_commands.linker.execution_result = CommandExecutionResult::from(&r);

        if let Err(e) = r {
            return Err(e);
        } else if !r.as_ref().unwrap().success() {
            return Err(eyre!(
                "Ending the program, because the linker command line execution failed",
            ));
        }
    }

    Ok(CommandExecutionResult::Success) // TODO: consider a new variant, like AllSuccedeed
                                        // or better, change the return for something better
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
fn execute_command<T, S>(
    model: &ZorkModel,
    arguments: T,
    env_vars: &EnvVars,
) -> Result<ExitStatus, Report>
where
    T: IntoIterator<Item = S> + std::fmt::Display + Copy,
    S: AsRef<OsStr>,
{
    let compiler = model.compiler.cpp_compiler;
    log::trace!(
        "[{compiler}] - Executing command => {:?}",
        format!("{} {arguments}", compiler.get_driver(&model.compiler),)
    );

    let driver = compiler.get_driver(&model.compiler);
    let os_driver = OsStr::new(driver.as_ref());
    std::process::Command::new(os_driver)
        .args(arguments)
        .envs(env_vars)
        .spawn()?
        .wait()
        .with_context(|| format!("[{compiler}] - Command {arguments} failed!"))
}

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
    pub fn new<'a, T: TranslationUnit<'a>>(
        // TODO init it as a args holder, but doesn't have the status yet
        tu: &T,
        args: Arguments,
    ) -> Self {
        Self {
            directory: tu.path().to_path_buf(),
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
    pub args: Arguments,
    pub execution_result: CommandExecutionResult,
}

impl LinkerCommandLine {
    /// Saves the path at which a compilation product of any translation unit will be placed,
    /// in order to add it to the files that will be linked to generate the final product
    /// in the two-phase compilation model
    pub fn add_buildable_at(&mut self, path: &Path) {
        self.args.push(Argument::from(path));
    }

    // TODO: just maybe a Cow for everyone?

    /// Owned version of TODO link
    pub fn add_owned_buildable_at(&mut self, path: PathBuf) {
        self.args.push(path.into());
    }
}

/// Holds the generated command line arguments for a concrete compiler
#[derive(Serialize, Deserialize, Default)]
pub struct Commands {
    pub compiler: CppCompiler, // TODO: review if we can afford this field given the new
    // architechture
    pub cpp_stdlib: Option<SourceCommandLine>,
    pub c_compat_stdlib: Option<SourceCommandLine>,
    pub system_modules: HashMap<String, Arguments>,

    pub general_args: CommonArgs,
    pub compiler_common_args: Box<dyn CompilerCommonArguments>,

    pub interfaces: Vec<SourceCommandLine>,
    pub implementations: Vec<SourceCommandLine>,
    pub sources: Vec<SourceCommandLine>,
    pub linker: LinkerCommandLine,
}

impl Commands {
    /// Returns an [std::iter::Chain] (behind the opaque impl clause return type signature)
    /// which points to all the generated commmands for the two variants of the compilers vendors C++ modular
    /// standard libraries implementations (see: [crate::project_model::compiler::StdLibMode])
    /// joined to all the commands generated for every [TranslationUnit] declared by the user for
    /// its project
    pub fn get_all_command_lines(
        &mut self,
    ) -> impl Iterator<Item = &mut SourceCommandLine> + Debug + '_ {
        self.cpp_stdlib
            .as_mut_slice()
            .iter_mut()
            .chain(self.c_compat_stdlib.as_mut_slice().iter_mut())
            .chain(self.interfaces.as_mut_slice().iter_mut())
            .chain(self.implementations.as_mut_slice().iter_mut())
            .chain(self.sources.as_mut_slice().iter_mut())
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
    /// Whenever a translation unit must be rebuilt
    PendingToBuild,
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
