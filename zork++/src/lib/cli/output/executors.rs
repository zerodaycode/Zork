//! Contains helpers and data structures to be processed in a nice and neat way the commands generated to be executed
//! by Zork++

use std::ffi::OsStr;
use std::{path::Path, process::ExitStatus};

use crate::cache::EnvVars;
use crate::domain::commands::arguments::{Argument, Arguments};
use crate::domain::commands::command_lines::ModulesCommands;
use crate::domain::target::{Target, TargetIdentifier};
use crate::{
    project_model::{compiler::CppCompiler, ZorkModel},
    utils::constants,
};
use color_eyre::{eyre::Context, Report, Result};
use indexmap::IndexMap;

pub fn run_modules_generated_commands(
    program_data: &ZorkModel<'_>,
    general_args: &Arguments<'_>,
    compiler_specific_shared_args: &Arguments,
    modules_generated_commands: &mut ModulesCommands<'_>,
    env_vars: &EnvVars,
) -> Result<()> {
    log::info!("Proceeding to execute the generated modules commands...");

    // Process the modules
    helpers::process_modules_commands(
        program_data,
        general_args,
        compiler_specific_shared_args,
        modules_generated_commands,
        env_vars,
    )
}

pub fn run_targets_generated_commands(
    program_data: &ZorkModel<'_>,
    general_args: &Arguments<'_>,
    compiler_specific_shared_args: &Arguments,
    targets: &mut IndexMap<TargetIdentifier, Target>,
    modules: &ModulesCommands<'_>,
    env_vars: &EnvVars,
) -> Result<()> {
    log::info!("Proceeding to execute the generated commands...");

    // Process the user declared targets TODO: Filtered by cli?
    // TODO: avoid the callee of the autorun binary by decoupling modules from linkers execs
    for (target_name, target_data) in targets {
        log::info!(
            "Executing the linker command line for target: {:?}",
            target_name
        );

        // Send to build to the compiler the sources declared for the current iteration target
        for source in target_data.sources.iter_mut() {
            helpers::execute_source_command_line(
                program_data,
                general_args,
                compiler_specific_shared_args,
                env_vars,
                source,
            )?;
        }

        // Invoke the linker to generate the final product for the current iteration target
        helpers::execute_linker_command_line(
            program_data,
            general_args,
            compiler_specific_shared_args,
            modules,
            env_vars,
            target_data,
        )?;
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
fn execute_command<'a, T, S>(
    model: &ZorkModel,
    arguments: &mut T,
    env_vars: &EnvVars,
) -> Result<ExitStatus, Report>
where
    // T: IntoIterator<Item = S> + std::fmt::Display + Copy,
    T: Iterator<Item = S> + std::fmt::Debug,
    S: AsRef<OsStr>,
    Arguments<'a>: FromIterator<S>,
{
    let compiler = model.compiler.cpp_compiler;
    let driver = compiler.get_driver(&model.compiler);
    log::trace!(
        "[{compiler}] - Executing command => {:?}",
        format!("{} {}", driver, arguments.collect::<Arguments>())
    );

    let os_driver = OsStr::new(driver.as_ref());
    std::process::Command::new(os_driver)
        .args(arguments)
        .envs(env_vars)
        .spawn()?
        .wait()
        .with_context(|| format!("[{compiler}] - Command failed!"))
}

mod helpers {
    use crate::cache::EnvVars;
    use crate::cli::output::executors::execute_command;
    use crate::domain::commands::arguments::{Argument, Arguments};
    use crate::domain::commands::command_lines::{ModulesCommands, SourceCommandLine};
    use crate::domain::target::Target;
    use crate::domain::translation_unit::TranslationUnitStatus;
    use crate::project_model::compiler::CppCompiler;
    use crate::project_model::ZorkModel;

    use color_eyre::eyre::{eyre, Result};
    use std::collections::HashMap;
    use std::process::ExitStatus;

    pub(crate) fn execute_source_command_line(
        program_data: &ZorkModel<'_>,
        general_args: &Arguments<'_>,
        compiler_specific_shared_args: &Arguments<'_>,
        env_vars: &HashMap<String, String>,
        source: &mut SourceCommandLine<'_>,
    ) -> Result<()> {
        let compile_but_dont_link = [Argument::from("/c")];
        let mut args = general_args
            .iter()
            .chain(compiler_specific_shared_args.iter())
            .chain(source.args.as_slice().iter())
            .chain(compile_but_dont_link.iter());

        let r = execute_command(program_data, &mut args, env_vars);
        source.status = TranslationUnitStatus::from(&r);

        if let Err(e) = r {
            return Err(e);
        } else if !r.as_ref().unwrap().success() {
            let err = eyre!(
                "Ending the program, because the build of: {:?} failed",
                source.filename
            );
            return Err(err);
        }

        Ok(())
    }

    pub(crate) fn execute_linker_command_line(
        program_data: &ZorkModel,
        general_args: &Arguments,
        compiler_specific_shared_args: &Arguments,
        modules: &ModulesCommands<'_>,
        env_vars: &EnvVars,
        target_data: &mut Target,
    ) -> Result<ExitStatus> {
        let linker_args = target_data
            .linker
            .get_target_output_for(program_data.compiler.cpp_compiler);

        let linker_sources_byproducts = target_data.sources.iter().map(|scl| &scl.byproduct);
        let modules_byproducts = modules
            .cpp_stdlib
            .as_slice()
            .iter()
            .chain(modules.c_compat_stdlib.iter())
            .chain(modules.interfaces.iter())
            .chain(modules.implementations.iter())
            .chain(modules.system_modules.iter())
            .map(|scl| &scl.byproduct);

        let mut args = general_args
            .iter()
            .chain(linker_args.iter())
            .chain(compiler_specific_shared_args.iter())
            // TODO:    // .chain(linker_command_line.byproducts.iter())
            // TODO:    // .chain(modules.byproducts.iter()) review if it's worth to have them on
            // separated caches entries or build the chained iterators
            .chain(linker_sources_byproducts)
            .chain(modules_byproducts);

        let r = execute_command(program_data, &mut args, env_vars);
        target_data.linker.execution_result = TranslationUnitStatus::from(&r);

        if let Err(e) = r {
            return Err(e);
        } else if !r.as_ref().unwrap().success() {
            return Err(eyre!(
                "Ending the program, because the linker command line execution failed",
            ));
        }

        r
    }

    pub(crate) fn process_modules_commands(
        program_data: &ZorkModel<'_>,
        general_args: &Arguments,
        compiler_specific_shared_args: &Arguments,
        generated_commands: &mut ModulesCommands<'_>,
        env_vars: &std::collections::HashMap<String, String>,
    ) -> Result<()> {
        let translation_units_commands: Vec<&mut SourceCommandLine> =
            get_modules_translation_units_commands(
                // Independent borrows to avoid have borrow checker yielding at me
                &mut generated_commands.cpp_stdlib,
                &mut generated_commands.c_compat_stdlib,
                &mut generated_commands.system_modules,
                &mut generated_commands.interfaces,
                &mut generated_commands.implementations,
            );

        if translation_units_commands.is_empty() {
            log::debug!("No modules to process, build or rebuild in this iteration.");
            return Ok(());
        }

        let compile_but_dont_link: [Argument; 1] =
            [Argument::from(match program_data.compiler.cpp_compiler {
                CppCompiler::CLANG | CppCompiler::GCC => "-c",
                CppCompiler::MSVC => "/c",
            })];

        for translation_unit_cmd in translation_units_commands {
            // Join the concrete args of any translation unit with the ones held in the flyweights
            let mut translation_unit_cmd_args = general_args
                .iter()
                .chain(compiler_specific_shared_args.iter())
                .chain(&compile_but_dont_link)
                .chain(translation_unit_cmd.args.iter());

            let r = execute_command(program_data, &mut translation_unit_cmd_args, env_vars);
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
        Ok(())
    }

    /// TODO: create strong types or aliases at least
    pub(crate) fn get_modules_translation_units_commands<'a, 'b>(
        cpp_stdlib: &'b mut Option<SourceCommandLine<'a>>,
        c_compat_stdlib: &'b mut Option<SourceCommandLine<'a>>,
        system_modules: &'b mut Vec<SourceCommandLine<'a>>,
        interfaces: &'b mut Vec<SourceCommandLine<'a>>,
        implementations: &'b mut Vec<SourceCommandLine<'a>>,
    ) -> Vec<&'b mut SourceCommandLine<'a>> {
        cpp_stdlib
            .as_mut_slice()
            .iter_mut()
            .chain(c_compat_stdlib.as_mut_slice().iter_mut())
            .chain(system_modules.as_mut_slice().iter_mut())
            .chain(interfaces.as_mut_slice().iter_mut())
            .chain(implementations.as_mut_slice().iter_mut())
            .filter(|scl| scl.status.eq(&TranslationUnitStatus::PendingToBuild))
            .collect::<Vec<&mut SourceCommandLine>>()
    }
}
