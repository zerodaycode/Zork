//! Contains helpers and data structures to be processed in a nice and neat way the commands generated to be executed
//! by Zork++

use std::ffi::OsStr;
use std::{path::Path, process::ExitStatus};

use crate::cache::EnvVars;
use crate::domain::commands::arguments::Argument;
use crate::domain::commands::command_lines::ModulesCommands;
use crate::domain::flyweight_data::FlyweightData;
use crate::domain::target::{Target, TargetIdentifier};
use crate::domain::translation_unit::TranslationUnitStatus;
use crate::utils::constants::error_messages;
use crate::{
    project_model::{compiler::CppCompiler, ZorkModel},
    utils::constants,
};
use color_eyre::eyre::ContextCompat;
use color_eyre::{eyre::Context, Report, Result};
use indexmap::IndexMap;

pub fn run_modules_generated_commands(
    program_data: &ZorkModel<'_>,
    flyweight_data: &FlyweightData,
    modules_generated_commands: &mut ModulesCommands<'_>,
) -> Result<()> {
    log::info!("Proceeding to execute the generated modules commands...");

    helpers::process_std_modules_commands(
        program_data,
        flyweight_data,
        modules_generated_commands,
    )?;
    helpers::process_user_modules_commands(program_data, flyweight_data, modules_generated_commands)
}

pub fn run_targets_generated_commands(
    program_data: &ZorkModel<'_>,
    flyweight_data: &FlyweightData,
    targets: &mut IndexMap<TargetIdentifier, Target>,
    modules: &ModulesCommands<'_>,
) -> Result<()> {
    log::info!("Proceeding to execute the generated commands...");

    let shared_args = flyweight_data
        .general_args
        .iter()
        .chain(flyweight_data.shared_args.iter())
        .chain(flyweight_data.std_references.iter());

    // Process the user declared targets
    for (target_identifier, target_data) in targets
        .iter_mut()
        .filter(|(_, target_data)| target_data.enabled_for_current_program_iteration)
    {
        let env_vars = &flyweight_data.env_vars;

        log::info!(
            "Executing the generated commands of the sources declared for target: {:?}",
            target_identifier.name()
        );

        let extra_args = &program_data
            .targets
            .get(target_identifier)
            .with_context(|| error_messages::TARGET_ENTRY_NOT_FOUND)?
            .extra_args;

        let target_shared_args = shared_args
            .clone()
            .chain(flyweight_data.compile_but_dont_link.iter())
            .chain(extra_args.as_slice())
            .collect();

        // Send to build to the compiler the sources declared for the current iteration target
        for source in target_data
            .sources
            .iter_mut()
            .filter(|scl| scl.status.eq(&TranslationUnitStatus::PendingToBuild))
        {
            helpers::execute_source_command_line(
                program_data,
                &target_shared_args,
                env_vars,
                source,
            )?;
        }

        log::info!(
            "Executing the linker command line for target: {:?}",
            target_identifier.name()
        );

        // Invoke the linker to generate the final product for the current iteration target
        helpers::execute_linker_command_line(
            program_data,
            flyweight_data,
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
fn execute_command<T, S>(
    model: &ZorkModel,
    arguments: T,
    env_vars: &EnvVars,
) -> Result<ExitStatus, Report>
where
    T: IntoIterator<Item = S> + std::fmt::Display + std::marker::Copy,
    S: AsRef<OsStr>,
{
    let compiler = model.compiler.cpp_compiler;
    log::trace!(
        "[{compiler}] - Executing command => {:?}",
        format!("{} {}", compiler.get_driver(&model.compiler), arguments)
    );

    let driver = compiler.get_driver(&model.compiler);
    let os_driver = OsStr::new(driver.as_ref());
    std::process::Command::new(os_driver)
        .args(arguments)
        .envs(env_vars)
        .spawn()?
        .wait()
        .with_context(|| format!("[{compiler}] - Command {} failed!", arguments))
}

mod helpers {
    use crate::cache::EnvVars;
    use crate::cli::output::executors::execute_command;
    use crate::domain::commands::arguments::Arguments;
    use crate::domain::commands::command_lines::{ModulesCommands, SourceCommandLine};
    use crate::domain::flyweight_data::FlyweightData;
    use crate::domain::target::Target;
    use crate::domain::translation_unit::TranslationUnitStatus;
    use crate::project_model::compiler::CppCompiler;
    use crate::project_model::ZorkModel;

    use color_eyre::eyre::{eyre, Result};
    use std::process::ExitStatus;

    pub(crate) fn execute_source_command_line(
        program_data: &ZorkModel<'_>,
        shared_args: &Arguments<'_>,
        env_vars: &EnvVars,
        source: &mut SourceCommandLine<'_>,
    ) -> Result<()> {
        let args = shared_args
            .as_slice()
            .iter()
            .chain(source.args.as_slice().iter())
            .collect::<Arguments>();

        let r = execute_command(program_data, &args, env_vars);
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
        flyweight_data: &FlyweightData,
        modules: &ModulesCommands<'_>,
        env_vars: &EnvVars,
        target_data: &mut Target,
    ) -> Result<ExitStatus> {
        let compiler = program_data.compiler.cpp_compiler;
        let target_output = target_data.linker.get_target_output_for(compiler);

        let linker_sources_byproducts = target_data.sources.iter().map(|scl| &scl.byproduct);
        let modules_byproducts = modules
            .cpp_stdlib
            .as_slice()
            .iter()
            .chain(modules.c_compat_stdlib.iter())
            .chain(modules.interfaces.iter())
            .chain(modules.implementations.iter())
            .chain(if compiler.eq(&CppCompiler::CLANG) {
                // NOTE: gcc handles them itself with the
                // gcm.cache. MSVC doesn't need them and
                // this should be removed since when
                // import std is impl for the big 3
                modules.system_modules.iter()
            } else {
                [].iter()
            })
            .map(|scl| &scl.byproduct);

        let args = flyweight_data
            .general_args
            .iter()
            .chain(flyweight_data.shared_args.iter())
            .chain(flyweight_data.std_references.iter())
            .chain(target_data.linker.args.iter())
            .chain(target_data.linker.extra_args.iter())
            .chain(target_output.iter())
            .chain(modules_byproducts)
            .chain(linker_sources_byproducts)
            .collect::<Arguments>();

        let r = execute_command(program_data, &args, env_vars);
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

    pub(crate) fn process_std_modules_commands(
        program_data: &ZorkModel<'_>,
        flyweight_data: &FlyweightData,
        generated_commands: &mut ModulesCommands<'_>,
    ) -> Result<()> {
        let std_libs_commands: Vec<&mut SourceCommandLine> =
            get_std_modules_commands(generated_commands);

        if std_libs_commands.is_empty() {
            return Ok(());
        }

        for std_lib in std_libs_commands {
            // Join the concrete args of any translation unit with the ones held in the flyweights
            let translation_unit_cmd_args = flyweight_data
                .general_args
                .iter()
                .chain(flyweight_data.shared_args.iter())
                .chain(flyweight_data.compile_but_dont_link.iter()) // NOTE: non-required in Clang
                .chain(std_lib.args.iter())
                .collect::<Arguments>();

            let r = execute_command(
                program_data,
                &translation_unit_cmd_args,
                &flyweight_data.env_vars,
            );
            std_lib.status = TranslationUnitStatus::from(&r);

            if let Err(e) = r {
                return Err(e);
            } else if !r.as_ref().unwrap().success() {
                let err = eyre!(
                    "Ending the program, because the build of: {:?} failed",
                    std_lib.filename
                );
                return Err(err);
            }
        }

        Ok(())
    }

    pub(crate) fn process_user_modules_commands(
        program_data: &ZorkModel<'_>,
        flyweight_data: &FlyweightData,
        generated_commands: &mut ModulesCommands<'_>,
    ) -> Result<()> {
        let translation_units_commands: Vec<&mut SourceCommandLine> =
            get_user_modules_translation_units_commands(generated_commands);

        if translation_units_commands.is_empty() {
            log::debug!(
                "No user or system modules to process, build or rebuild in this iteration."
            );
            return Ok(());
        }

        for translation_unit_cmd in translation_units_commands {
            // Join the concrete args of any translation unit with the ones held in the flyweights
            let translation_unit_cmd_args = flyweight_data
                .general_args
                .iter()
                .chain(flyweight_data.shared_args.iter())
                .chain(flyweight_data.std_references.iter())
                .chain(flyweight_data.compile_but_dont_link.iter())
                .chain(translation_unit_cmd.args.iter())
                .collect::<Arguments>();

            let r = execute_command(
                program_data,
                &translation_unit_cmd_args,
                &flyweight_data.env_vars,
            );
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

    pub(crate) fn get_std_modules_commands<'a, 'b>(
        generated_commands: &'b mut ModulesCommands<'a>,
    ) -> Vec<&'b mut SourceCommandLine<'a>> {
        let cpp_stdlib = generated_commands.cpp_stdlib.as_mut_slice().iter_mut();
        let c_compat_stdlib = generated_commands.c_compat_stdlib.as_mut_slice().iter_mut();

        cpp_stdlib
            .chain(c_compat_stdlib)
            .filter(|scl| scl.status.eq(&TranslationUnitStatus::PendingToBuild))
            .collect::<Vec<&mut SourceCommandLine>>()
    }

    pub(crate) fn get_user_modules_translation_units_commands<'a, 'b>(
        generated_commands: &'b mut ModulesCommands<'a>,
    ) -> Vec<&'b mut SourceCommandLine<'a>> {
        let system_modules = generated_commands.system_modules.as_mut_slice().iter_mut();
        let interfaces = generated_commands.interfaces.as_mut_slice().iter_mut();
        let implementations = generated_commands.implementations.as_mut_slice().iter_mut();

        system_modules
            .chain(interfaces)
            .chain(implementations)
            .filter(|scl| scl.status.eq(&TranslationUnitStatus::PendingToBuild))
            .collect::<Vec<&mut SourceCommandLine>>()
    }
}
