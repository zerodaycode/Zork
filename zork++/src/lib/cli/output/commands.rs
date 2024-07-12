//! Contains helpers and data structures to be processed in a nice and neat way the commands generated to be executed
//! by Zork++

use std::ffi::OsStr;
use std::fmt::Debug;
use std::slice::Iter;
use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
};

use super::arguments::Argument;
use crate::cache::EnvVars;
use crate::cli::output::arguments::Arguments;
use crate::compiler::data_factory::{CommonArgs, CompilerCommonArguments};
use crate::domain::translation_unit::TranslationUnit;
use crate::utils::constants::error_messages;
use crate::{
    cache::ZorkCache,
    project_model::{compiler::CppCompiler, ZorkModel},
    utils::constants,
};
use color_eyre::eyre::ContextCompat;
use color_eyre::{
    eyre::{eyre, Context},
    Report, Result,
};
use serde::{Deserialize, Serialize};

pub fn run_generated_commands<'a>(
    program_data: &ZorkModel<'a>,
    cache: &mut ZorkCache<'a>,
) -> Result<()> {
    log::info!("Proceeding to execute the generated commands...");

    let generated_commands = &mut cache.generated_commands;

    let general_args = generated_commands
        .general_args
        .as_mut()
        .expect(error_messages::GENERAL_ARGS_NOT_FOUND)
        .get_args();

    let compiler_specific_shared_args = generated_commands
        .compiler_common_args
        .as_mut()
        .with_context(|| error_messages::COMPILER_SPECIFIC_COMMON_ARGS_NOT_FOUND)?
        .get_args();

    let env_vars = match program_data.compiler.cpp_compiler {
        CppCompiler::MSVC => &cache.compilers_metadata.msvc.env_vars,
        CppCompiler::CLANG => &cache.compilers_metadata.clang.env_vars,
        CppCompiler::GCC => &cache.compilers_metadata.gcc.env_vars,
    };

    let translation_units = generated_commands
        .cpp_stdlib
        .as_mut_slice()
        .iter_mut()
        .chain(generated_commands.c_compat_stdlib.as_mut_slice().iter_mut())
        .chain(generated_commands.system_modules.as_mut_slice().iter_mut())
        .chain(generated_commands.interfaces.as_mut_slice().iter_mut())
        .chain(generated_commands.implementations.as_mut_slice().iter_mut())
        .chain(generated_commands.sources.as_mut_slice().iter_mut())
        .filter(|scl| scl.status.eq(&TranslationUnitStatus::PendingToBuild))
        .collect::<Vec<&mut SourceCommandLine>>();

    // let translation_units = generated_commands // TODO: how can I borrow twice generated_commands?
    //     .get_all_command_lines()
    //     .filter(|scl| scl.status.eq(&TranslationUnitStatus::PendingToBuild))
    //     .collect::<Vec<&mut SourceCommandLine>>();

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

        let r = execute_command(program_data, &translation_unit_cmd_args, env_vars);
        translation_unit_cmd.status = TranslationUnitStatus::from(&r);

        if let Err(e) = r {
            cache.save(program_data)?;
            return Err(e);
        } else if !r.as_ref().unwrap().success() {
            let err = eyre!(
                "Ending the program, because the build of: {:?} failed",
                translation_unit_cmd.filename
            );
            return Err(err);
        }
    }

    log::info!("Processing the linker command line...");
    let r = execute_command(
        program_data,
        &general_args
            .iter()
            .chain(compiler_specific_shared_args.iter())
            .chain(
                generated_commands
                    .linker
                    .get_target_output_for(program_data.compiler.cpp_compiler)
                    .iter(),
            )
            .chain(generated_commands.linker.byproducts.iter())
            .collect::<Arguments>(),
        env_vars,
    );

    cache.generated_commands.linker.execution_result = TranslationUnitStatus::from(&r);

    if let Err(e) = r {
        return Err(e);
    } else if !r.as_ref().unwrap().success() {
        return Err(eyre!(
            "Ending the program, because the linker command line execution failed",
        ));
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

/// Type for representing the command line that will be sent to the target compiler, and
/// store its different components
///
/// * directory*: the path where the translation unit lives
/// * filename*: the translation unit declared name on the fs with the extension
/// * args*: member that holds all the cmd arguments that will be passed to the compiler driver
/// * status*: A [`TranslationUnitStatus`] that represents all the different phases that a source command
/// line can have among all the different iterations of the program, changing according to the modifications
/// over the translation unit in the fs and the result of the build execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCommandLine<'a> {
    pub directory: PathBuf,
    pub filename: String,
    pub args: Arguments<'a>,
    pub status: TranslationUnitStatus,
}

impl<'a> SourceCommandLine<'a> {
    pub fn new<T: TranslationUnit<'a>>(tu: &T, args: Arguments<'a>) -> Self {
        Self {
            directory: PathBuf::from(tu.parent()),
            filename: tu.filename(),
            args,
            status: TranslationUnitStatus::PendingToBuild,
        }
    }

    pub fn path(&self) -> PathBuf {
        self.directory.join(Path::new(&self.filename))
    }

    pub fn filename(&self) -> &String {
        &self.filename
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LinkerCommandLine<'a> {
    pub target: Argument<'a>,
    pub byproducts: Arguments<'a>,
    pub extra_args: Arguments<'a>,
    pub execution_result: TranslationUnitStatus,
}

impl<'a> LinkerCommandLine<'a> {
    pub fn get_target_output_for(&self, compiler: CppCompiler) -> Vec<Argument> {
        match compiler {
            CppCompiler::CLANG | CppCompiler::GCC => {
                vec![Argument::from("-o"), self.target.clone()]
            }
            CppCompiler::MSVC => vec![self.target.clone()],
        }
    }

    /// Saves the path at which a compilation product of any translation unit will be placed,
    /// in order to add it to the files that will be linked to generate the final product
    /// in the two-phase compilation model
    pub fn add_byproduct_path(&mut self, path: PathBuf) {
        self.byproducts.push(path);
    }
}

/// Holds the generated command line arguments for a concrete compiler
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Commands<'a> {
    pub cpp_stdlib: Option<SourceCommandLine<'a>>,
    pub c_compat_stdlib: Option<SourceCommandLine<'a>>,
    // pub system_modules: HashMap<String, Arguments>,
    pub system_modules: Vec<SourceCommandLine<'a>>, // TODO: SourceCommandLine while we found a better approach
    // or while we don't implement the parser that gets the path to the compilers std library headers
    pub general_args: Option<CommonArgs<'a>>,
    pub compiler_common_args: Option<Box<dyn CompilerCommonArguments>>,

    pub interfaces: Vec<SourceCommandLine<'a>>,
    pub implementations: Vec<SourceCommandLine<'a>>,
    pub sources: Vec<SourceCommandLine<'a>>,
    pub linker: LinkerCommandLine<'a>,
}

impl<'a> Commands<'a> {
    /// Returns a [std::iter::Chain] (behind the opaque impl clause return type signature)
    /// which points to all the generated commands for the two variants of the compilers vendors C++ modular
    /// standard libraries implementations (see: [crate::project_model::compiler::StdLibMode])
    /// joined to all the commands generated for every [TranslationUnit] declared by the user for
    /// its project
    pub fn get_all_command_lines(
        &mut self,
    ) -> impl Iterator<Item = &mut SourceCommandLine<'a>> + Debug {
        self.cpp_stdlib
            .as_mut_slice()
            .iter_mut()
            .chain(self.c_compat_stdlib.as_mut_slice().iter_mut())
            .chain(self.system_modules.as_mut_slice().iter_mut())
            .chain(self.interfaces.as_mut_slice().iter_mut())
            .chain(self.implementations.as_mut_slice().iter_mut())
            .chain(self.sources.as_mut_slice().iter_mut())
    }

    pub fn add_linker_file_path(&mut self, path: PathBuf) {
        self.linker.add_byproduct_path(path);
    }
}

impl<'a> core::fmt::Display for Commands<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Commands:\n- Interfaces: {:?},\n- Implementations: {:?},\n- Sources: {:?}",
            collect_source_command_line(self.interfaces.iter()),
            collect_source_command_line(self.implementations.iter()),
            collect_source_command_line(self.sources.iter())
        )
    }
}

/// Convenient function to avoid code replication
fn collect_source_command_line<'a>(
    iter: Iter<'a, SourceCommandLine>, // TODO: review this, for see if it's possible to consume the value and not cloning it
) -> impl Iterator + Debug + 'a {
    iter.map(|vec| {
        vec.args
            .iter()
            .map(|arg| arg.value().clone())
            .collect::<Vec<_>>()
            .join(" ");
    })
}

/// The different states of a translation unit in the whole lifecycle of
/// the build process and across different iterations of the same
#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum TranslationUnitStatus {
    /// A command that is executed correctly
    Success,
    /// A skipped command due to previous successful iterations
    Cached,
    /// A command which is return code indicates an unsuccessful execution
    Failed,
    /// Whenever a translation unit must be rebuilt
    #[default]
    PendingToBuild,
    /// The associated [`TranslationUnit`] has been deleted from the user's configuration and therefore,
    /// it should be removed from the cache as well as its generated byproducts
    ToDelete,
    /// The execution failed, returning a [`Result`] with the Err variant
    Error,
}

impl From<Result<ExitStatus, Report>> for TranslationUnitStatus {
    fn from(value: Result<ExitStatus, Report>) -> Self {
        helpers::handle_command_execution_result(&value)
    }
}

impl From<&Result<ExitStatus, Report>> for TranslationUnitStatus {
    fn from(value: &Result<ExitStatus, Report>) -> Self {
        helpers::handle_command_execution_result(value)
    }
}

mod helpers {

    use crate::cli::output::commands::TranslationUnitStatus;

    use color_eyre::eyre::Result;
    use std::process::ExitStatus;

    /// Convenient way of handle a command execution result avoiding duplicate code
    pub(crate) fn handle_command_execution_result(
        value: &Result<ExitStatus>,
    ) -> TranslationUnitStatus {
        match value {
            Ok(r) => {
                if r.success() {
                    TranslationUnitStatus::Success
                } else {
                    TranslationUnitStatus::Failed
                }
            }
            Err(_) => TranslationUnitStatus::Error,
        }
    }
}
