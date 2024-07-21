//! Contains helpers and data structures to be processed in a nice and neat way the commands generated to be executed
//! by Zork++

use std::ffi::OsStr;
use std::{path::Path, process::ExitStatus};

use crate::cache::EnvVars;
use crate::domain::commands::arguments::{Argument, Arguments};
use crate::domain::commands::command_lines::SourceCommandLine;
use crate::domain::translation_unit::TranslationUnitStatus;
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

    let mut mock = vec![];

    let translation_units_commands: Vec<&mut SourceCommandLine> =
        helpers::get_translation_units_commands(
            // Independent borrows to avoid have borrow checker yielding at me
            &mut generated_commands.cpp_stdlib,
            &mut generated_commands.c_compat_stdlib,
            &mut generated_commands.system_modules,
            &mut generated_commands.interfaces,
            &mut generated_commands.implementations,
            // &mut generated_commands.sources,
            &mut mock,
        );

    let compile_but_dont_link: [Argument; 1] =
        [Argument::from(match program_data.compiler.cpp_compiler {
            CppCompiler::CLANG | CppCompiler::GCC => "-c",
            CppCompiler::MSVC => "/c",
        })];

    for translation_unit_cmd in translation_units_commands {
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
            return Err(e);
        } else if !r.as_ref().unwrap().success() {
            let err = eyre!(
                "Ending the program, because the build of: {:?} failed",
                translation_unit_cmd.filename
            );
            return Err(err);
        }
    }

    /* log::info!("Processing the linker command line...");
    let r = helpers::execute_linker_command_line(
        program_data,
        general_args,
        compiler_specific_shared_args,
        &generated_commands.linker,
        env_vars,
    );
    cache.generated_commands.linker.execution_result = TranslationUnitStatus::from(&r);

    if let Err(e) = r {
        return Err(e);
    } else if !r.as_ref().unwrap().success() {
        return Err(eyre!(
            "Ending the program, because the linker command line execution failed",
        ));
    } */

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

mod helpers {
    use crate::cache::EnvVars;
    use crate::cli::output::executors::execute_command;
    use crate::domain::commands::arguments::Arguments;
    use crate::domain::commands::command_lines::{LinkerCommandLine, SourceCommandLine};
    use crate::domain::translation_unit::TranslationUnitStatus;
    use crate::project_model::ZorkModel;
    use color_eyre::eyre::Result;
    use std::process::ExitStatus;

    pub(crate) fn execute_linker_command_line(
        program_data: &ZorkModel,
        general_args: Arguments,
        compiler_specific_shared_args: Arguments,
        linker_command_line: &LinkerCommandLine,
        env_vars: &EnvVars,
    ) -> Result<ExitStatus> {
        let linker_args =
            linker_command_line.get_target_output_for(program_data.compiler.cpp_compiler);
        let args = general_args
            .iter()
            .chain(linker_args.iter())
            .chain(compiler_specific_shared_args.iter())
            .chain(linker_command_line.byproducts.iter())
            .collect::<Arguments>();
        execute_command(program_data, &args, env_vars)
    }

    pub(crate) fn get_translation_units_commands<'a, 'b>(
        cpp_stdlib: &'b mut Option<SourceCommandLine<'a>>,
        c_compat_stdlib: &'b mut Option<SourceCommandLine<'a>>,
        system_modules: &'b mut Vec<SourceCommandLine<'a>>,
        interfaces: &'b mut Vec<SourceCommandLine<'a>>,
        implementations: &'b mut Vec<SourceCommandLine<'a>>,
        sources: &'b mut Vec<SourceCommandLine<'a>>,
    ) -> Vec<&'b mut SourceCommandLine<'a>> {
        cpp_stdlib
            .as_mut_slice()
            .iter_mut()
            .chain(c_compat_stdlib.as_mut_slice().iter_mut())
            .chain(system_modules.as_mut_slice().iter_mut())
            .chain(interfaces.as_mut_slice().iter_mut())
            .chain(implementations.as_mut_slice().iter_mut())
            .chain(sources.as_mut_slice().iter_mut())
            .filter(|scl| scl.status.eq(&TranslationUnitStatus::PendingToBuild))
            .collect::<Vec<&mut SourceCommandLine>>()
    }
}
