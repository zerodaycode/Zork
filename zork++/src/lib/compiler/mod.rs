//! The crate responsible for executing the core work of `Zork++`,
//! generate command lines and execute them in a shell of the current
//! operating system against the designed compilers in the configuration
//! file.

pub mod data_factory;

use color_eyre::Result;
use std::path::Path;


use crate::bounds::{ExecutableTarget, ExtraArgs, TranslationUnit};
use crate::cli::output::arguments::{clang_args, msvc_args, Arguments};
use crate::cli::output::commands::{CommandExecutionResult, SourceCommandLine};
use crate::compiler::helpers::flag_source_file_without_changes;
use crate::project_model::compiler::StdLibMode;
use crate::utils::constants;
use crate::{
    cache::ZorkCache,
    cli::output::{arguments::Argument, commands::Commands},
    project_model::{
        compiler::CppCompiler,
        modules::{ModuleImplementationModel, ModuleInterfaceModel},
        ZorkModel,
    },
};

use self::data_factory::{
    ClangCommonArgs, CommonArgs, CompilerCommonArguments, GccCommonArgs, MsvcCommonArgs,
};

/// The entry point of the compilation process
///
/// Whenever this process gets triggered, the files declared within the
/// configuration file will be build
pub fn build_project<'a>(
    model: &'a ZorkModel<'a>,
    cache: &mut ZorkCache,
    tests: bool,
) -> Result<Commands> {
    // Generate the Flyweight struct, with the repetitive data and details of the command lines
    let general_args = CommonArgs::from(model);
    let compiler_specific_common_args: Box<dyn CompilerCommonArguments> =
        compiler_common_arguments_factory(model);

    // A registry of the generated command lines
    let mut commands = Commands::new(model, general_args, compiler_specific_common_args);
    // TODO from cache, and find them here instead from the cache

    // Pre-tasks
    if model.compiler.cpp_compiler == CppCompiler::GCC && !model.modules.sys_modules.is_empty() {
        helpers::build_sys_modules(model, &mut commands, cache)
    }

    // TODO: add them to the commands DS, so they are together until they're generated
    // Build the std library as a module
    build_modular_stdlib(model, cache, &mut commands); // TODO: ward it with an if for only call this fn for the

    // 1st - Build the modules
    process_modules(model, cache, &mut commands)?;
    // 2nd - Build the non module sources
    build_sources(model, cache, &mut commands, tests)?;
    // 3rd - Build the executable or the tests
    build_executable(model, cache, &mut commands, tests)?;

    Ok(commands)
}

/// Factory function for bring the data structure that holds the common arguments of a source
/// command line for every translation unit, regardeless the underlying choosen compiler
fn compiler_common_arguments_factory(model: &ZorkModel<'_>) -> Box<dyn CompilerCommonArguments> {
    // TODO: consider having a union (enum) instead of a fat ptr
    match model.compiler.cpp_compiler {
        CppCompiler::CLANG => Box::new(ClangCommonArgs::new(model)),
        CppCompiler::MSVC => Box::new(MsvcCommonArgs::new()),
        CppCompiler::GCC => Box::new(GccCommonArgs::new()),
    }
}

/// Builds the C++ standard library as a pre-step acording to the specification
/// of each compiler vendor
fn build_modular_stdlib(model: &ZorkModel<'_>, cache: &mut ZorkCache, commands: &mut Commands) {
    let compiler = model.compiler.cpp_compiler;

    // TODO: remaining ones: Clang, GCC
    if compiler.eq(&CppCompiler::MSVC) {
        let built_stdlib_path = &cache.compilers_metadata.msvc.stdlib_bmi_path;
        let cpp_stdlib = if !built_stdlib_path.exists() {
            log::trace!(
                "Building the {:?} C++ standard library implementation",
                compiler
            );
            msvc_args::generate_std_cmd(model, cache, StdLibMode::Cpp) // TODO move mod msvc_args to commands
        } else {
            // TODO: p.ej: existe y además tiene status cached? modificar por &mut
            // TODO: no será mejor sacarla de la caché?
            let source_command_line = SourceCommandLine {
                directory: built_stdlib_path.file_stem().unwrap().into(),
                filename: built_stdlib_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                args: Arguments::default(),
                need_to_build: false,
                execution_result: CommandExecutionResult::Cached,
            };
            source_command_line
        };
        commands.pre_tasks.push(cpp_stdlib);

        let built_stdlib_compat_path = &cache.compilers_metadata.msvc.c_stdlib_bmi_path;
        let c_cpp_stdlib = if !built_stdlib_path.exists() {
            log::trace!("Building the {:?} C compat CPP std lib", compiler);
            msvc_args::generate_std_cmd(model, cache, StdLibMode::CCompat)
        } else {
            let source_command_line = SourceCommandLine {
                directory: built_stdlib_compat_path.file_stem().unwrap().into(),
                filename: built_stdlib_compat_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                args: Arguments::default(),
                need_to_build: false,
                execution_result: CommandExecutionResult::Cached,
            };
            source_command_line
        };
        commands.pre_tasks.push(c_cpp_stdlib);
    }
}

/// Triggers the build process for compile the source files declared for the project
/// If this flow is enabled by the Cli arg `Tests`, then the executable will be generated
/// for the files and properties declared for the tests section in the configuration file
fn build_executable(
    model: &ZorkModel<'_>,
    cache: &ZorkCache,
    commands: &'_ mut Commands,
    tests: bool,
) -> Result<()> {
    // TODO: Check if the command line is the same as the previous? If there's no new sources?
    // And avoid re-executing?
    // TODO refactor this code, just having the if-else branch inside the fn
    if tests {
        generate_main_command_line_args(model, cache, commands, &model.tests)
    } else {
        generate_main_command_line_args(model, cache, commands, &model.executable)
    }
}

fn build_sources(
    model: &ZorkModel<'_>,
    cache: &ZorkCache,
    commands: &'_ mut Commands,
    tests: bool,
) -> Result<()> {
    log::info!("Generating the commands for the source files...");
    let srcs = if tests {
        &model.tests.sourceset.sources
    } else {
        &model.executable.sourceset.sources
    };

    srcs.iter().for_each(|src| if !flag_source_file_without_changes(&model.compiler.cpp_compiler, cache, &src.file()) {
        sources::generate_sources_arguments(model, commands, cache, &model.tests, src);
    } else {
        let command_line = SourceCommandLine::from_translation_unit(
            src, Arguments::default(), true, CommandExecutionResult::Cached
        );

        log::trace!("Source file: {:?} was not modified since the last iteration. No need to rebuilt it again.", &src.file());
        commands.sources.push(command_line);
        commands.add_linker_file_path_owned(helpers::generate_obj_file_path(
            model.compiler.cpp_compiler, &model.build.output_dir, src
        ))
    });

    Ok(())
}

/// Triggers the build process for compile the declared modules in the project
///
/// This function acts like a operation result processor, by running instances
/// and parsing the obtained result, handling the flux according to the
/// compiler responses
fn process_modules(model: &ZorkModel, cache: &mut ZorkCache, commands: &mut Commands) -> Result<()> {
    log::info!("Generating the commands for the module interfaces and partitions...");
    process_module_interfaces(model, cache, &model.modules.interfaces, commands);

    log::info!("Generating the commands for the module implementations and partitions...");
    process_module_implementations(model, cache, &model.modules.implementations, commands);

    Ok(())
}

/// Parses the configuration in order to build the BMIs declared for the project,
/// by pre-compiling the module interface units
fn process_module_interfaces<'a>(
    model: &'a ZorkModel<'_>,
    cache: &mut ZorkCache,
    interfaces: &'a [ModuleInterfaceModel],
    commands: &mut Commands,
) {
    interfaces.iter().for_each(|module_interface| {
        let compiler = cache.compiler;
        let lpe = cache.last_program_execution();
        let command_line = if let Some(generated_cmd) = cache.get_module_ifc_cmd(module_interface) {
            let translation_unit_must_be_rebuilt = helpers::translation_unit_must_be_rebuilt(compiler, lpe, generated_cmd, &module_interface.file());
            log::trace!("Source file: {:?} must be rebuilt: {translation_unit_must_be_rebuilt}", &module_interface.file());

            if !translation_unit_must_be_rebuilt { log::trace!("Source file:{:?} was not modified since the last iteration. No need to rebuilt it again.", &module_interface.file());
            }
            let mut cached_cmd_line = generated_cmd.clone(); // TODO: somehow, we should manage to solve this on the future
            cached_cmd_line.need_to_build = translation_unit_must_be_rebuilt;
            commands.linker.add_owned_buildable_at(helpers::generate_prebuilt_miu( // TODO: extremely provisional
                model.compiler.cpp_compiler, &model.build.output_dir, module_interface
            ));
            cached_cmd_line
        } else {
            sources::generate_module_interface_cmd(model, cache, module_interface, commands)
        };

        // TODO: should we get rid of commands and just store everything on the Cache?
        commands.interfaces.push(command_line);
        // commands.linker.add_owned_buildable_at(helpers::generate_prebuilt_miu(
        //     model.compiler.cpp_compiler, &model.build.output_dir, module_interface
        // ))
    });
}

/// Generates the commands for every [`ModuleImplementationModel`]
fn process_module_implementations<'a>(
    model: &'a ZorkModel,
    cache: &ZorkCache,
    impls: &'a [ModuleImplementationModel],
    commands: &mut Commands,
) {
    impls.iter().for_each(|module_impl| {
        let compiler = cache.compiler;
        let lpe = cache.last_program_execution();
        let command_line = if let Some(generated_cmd) = cache.get_module_impl_cmd(module_impl) {
            let translation_unit_must_be_rebuilt = helpers::translation_unit_must_be_rebuilt(compiler, lpe, generated_cmd, &module_impl.file());
            log::trace!("Source file: {:?} must be rebuilt: {translation_unit_must_be_rebuilt}", &module_impl.file());

            if !translation_unit_must_be_rebuilt {
                log::trace!("Source file: {:?} was not modified since the last iteration. No need to rebuilt it again.", &module_impl.file());
            }
            let mut cached_cmd_line = generated_cmd.clone(); // TODO: somehow, we should manage to solve this on the future
            cached_cmd_line.need_to_build = translation_unit_must_be_rebuilt;
            commands.linker.add_owned_buildable_at(helpers::generate_impl_obj_file( // TODO: extremely provisional
                                                                                   model.compiler.cpp_compiler, &model.build.output_dir, module_impl
            ));
            cached_cmd_line
        } else {
            sources::generate_module_implementation_cmd(model, cache, module_impl, commands)
        };

        commands.interfaces.push(command_line);
    });
}

/// Generates the command line arguments for the desired target
pub fn generate_main_command_line_args<'a>(
    model: &'a ZorkModel,
    cache: &ZorkCache,
    commands: &mut Commands,
    target: &'a impl ExecutableTarget<'a>,
) -> Result<()> {
    log::info!("Generating the main command line...");

    let compiler = &model.compiler.cpp_compiler;
    let out_dir: &Path = model.build.output_dir.as_ref();
    let executable_name = target.name();

    let mut arguments = Arguments::default();
    arguments.push(model.compiler.language_level_arg());
    arguments.extend_from_slice(model.compiler.extra_args());
    arguments.extend_from_slice(target.extra_args());

    match compiler {
        CppCompiler::CLANG => {
            arguments.push_opt(model.compiler.stdlib_arg());
            arguments.create_and_push("-fimplicit-modules");
            arguments.push(clang_args::implicit_module_map(out_dir));

            arguments.create_and_push(format!(
                "-fprebuilt-module-path={}",
                out_dir
                    .join(compiler.as_ref())
                    .join("modules")
                    .join("interfaces")
                    .display()
            ));

            arguments.create_and_push("-o");
            arguments.create_and_push(format!(
                "{}",
                out_dir
                    .join(compiler.as_ref())
                    .join(executable_name)
                    .with_extension(constants::BINARY_EXTENSION)
                    .display()
            ));
        }
        CppCompiler::MSVC => {
            arguments.create_and_push("/EHsc");
            arguments.create_and_push("/nologo");
            arguments.create_and_push("/ifcSearchDir");
            arguments.create_and_push(
                out_dir
                    .join(compiler.as_ref())
                    .join("modules")
                    .join("interfaces"),
            );
            arguments.create_and_push(format!(
                "/Fo{}\\",
                out_dir.join(compiler.as_ref()).display()
            ));
            arguments.create_and_push(format!(
                "/Fe{}",
                out_dir
                    .join(compiler.as_ref())
                    .join(executable_name)
                    .with_extension(constants::BINARY_EXTENSION)
                    .display()
            ));
            // Add the .obj file of the modular stdlib to the linker command
            arguments.create_and_push(&cache.compilers_metadata.msvc.stdlib_obj_path);
            arguments.create_and_push(&cache.compilers_metadata.msvc.c_stdlib_obj_path);
        }
        CppCompiler::GCC => {
            arguments.create_and_push("-fmodules-ts");
            arguments.create_and_push("-o");
            arguments.create_and_push(format!(
                "{}",
                out_dir
                    .join(compiler.as_ref())
                    .join(executable_name)
                    .with_extension(constants::BINARY_EXTENSION)
                    .display()
            ));
        }
    };

    arguments.extend(commands.linker.built_files.iter().map(Argument::from)); // TODO can't we avoid this, and just add the pathbufs?

    commands.linker.args.extend(arguments);
    commands.linker.built_files = target // TODO: built_files means raw cpp sources
        // TODO: add a custom collector on the mod sources
        // TODO: just name the field 'sources'
        .sourceset()
        .sources
        .iter()
        .map(|s| s.file())
        .collect::<Vec<_>>();

    Ok(())
}

/// Specific operations over source files
mod sources {
    use std::path::Path;

    use super::helpers;
    use crate::bounds::ExtraArgs;
    use crate::cache::ZorkCache;
    use crate::cli::output::arguments::Arguments;
    use crate::project_model::sourceset::SourceFile;
    use crate::{
        bounds::{ExecutableTarget, TranslationUnit},
        cli::output::{
            arguments::clang_args,
            commands::{CommandExecutionResult, Commands, SourceCommandLine},
        },
        project_model::{
            compiler::CppCompiler,
            modules::{ModuleImplementationModel, ModuleInterfaceModel},
            ZorkModel,
        },
    };

    /// Generates the command line arguments for non-module source files
    pub fn generate_sources_arguments<'a>(
        model: &'a ZorkModel,
        commands: &mut Commands,
        cache: &ZorkCache,
        target: &'a impl ExecutableTarget<'a>,
        source: &'a SourceFile,
    ) {
        let compiler = model.compiler.cpp_compiler;
        let out_dir = model.build.output_dir.as_ref();

        let mut arguments = Arguments::default();
        arguments.push(model.compiler.language_level_arg());
        arguments.create_and_push(if compiler.eq(&CppCompiler::MSVC) {
            "/c"
        } else {
            "-c"
        });
        arguments.extend_from_slice(model.compiler.extra_args());
        arguments.extend_from_slice(target.extra_args());

        match compiler {
            CppCompiler::CLANG => {
                arguments.push_opt(model.compiler.stdlib_arg());
                arguments.create_and_push("-fimplicit-modules");
                arguments.push(clang_args::implicit_module_map(out_dir));
                arguments.push(clang_args::add_prebuilt_module_path(compiler, out_dir));
                arguments.create_and_push("-o");
            }
            CppCompiler::MSVC => {
                arguments.create_and_push("/EHsc");
                arguments.create_and_push("/nologo");
                arguments.create_and_push("/reference");
                arguments.create_and_push(format! {
                    "std={}", cache.compilers_metadata.msvc.stdlib_bmi_path.display()
                });
                arguments.create_and_push("/reference");
                arguments.create_and_push(format! {
                    "std.compat={}", cache.compilers_metadata.msvc.c_stdlib_bmi_path.display()
                });
                arguments.create_and_push("/ifcSearchDir");
                arguments.create_and_push(
                    out_dir
                        .join(compiler.as_ref())
                        .join("modules")
                        .join("interfaces"),
                );
            }
            CppCompiler::GCC => {
                arguments.create_and_push("-fmodules-ts");
                arguments.create_and_push("-o");
            }
        };

        let obj_file = helpers::generate_obj_file_path(compiler, out_dir, source);
        let fo = if compiler.eq(&CppCompiler::MSVC) {
            "/Fo"
        } else {
            ""
        };
        arguments.create_and_push(format!("{fo}{}", obj_file.display()));
        arguments.create_and_push(source.file());

        let command_line = SourceCommandLine::from_translation_unit(
            source,
            arguments,
            false,
            CommandExecutionResult::default(),
        );
        commands.sources.push(command_line);
        commands.add_linker_file_path_owned(obj_file)
    }

    /// Generates the expected arguments for precompile the BMIs depending on self
    pub fn generate_module_interface_cmd<'a>(
        model: &'a ZorkModel,
        cache: &ZorkCache,
        interface: &'a ModuleInterfaceModel,
        commands: &mut Commands,
    ) -> SourceCommandLine {
        let compiler = model.compiler.cpp_compiler;
        let out_dir: &Path = model.build.output_dir.as_ref();

        let mut arguments = Arguments::default();

        match compiler {
            CppCompiler::CLANG => {
                // arguments.push_opt(model.compiler.stdlib_arg());
                // arguments.create_and_push("-fimplicit-modules");
                arguments.create_and_push("-x");
                arguments.create_and_push("c++-module");
                arguments.create_and_push("--precompile");
                // arguments.push(clang_args::implicit_module_map(out_dir));
                /* arguments.create_and_push(format!(
                    "-fprebuilt-module-path={}/clang/modules/interfaces",
                    out_dir.display()
                )); */
                clang_args::add_direct_module_interfaces_dependencies(
                    &interface.dependencies,
                    compiler,
                    out_dir,
                    &mut arguments,
                );

                // The resultant BMI as a .pcm file
                arguments.create_and_push("-o");
                // The output file
                let obj_file = helpers::generate_prebuilt_miu(compiler, out_dir, interface);
                arguments.create_and_push(&obj_file);
                commands.add_linker_file_path_owned(obj_file);
                // The input file
                arguments.create_and_push(interface.file());
            }
            CppCompiler::MSVC => {
                arguments.create_and_push("/EHsc");
                arguments.create_and_push("/nologo");
                arguments.create_and_push("/c");

                arguments.create_and_push("/reference");
                arguments.create_and_push(format! {
                    "std={}", cache.compilers_metadata.msvc.stdlib_bmi_path.display()
                });
                arguments.create_and_push("/reference");
                arguments.create_and_push(format! {
                    "std.compat={}", cache.compilers_metadata.msvc.c_stdlib_bmi_path.display()
                });
                let implicit_lookup_mius_path = out_dir
                    .join(compiler.as_ref())
                    .join("modules")
                    .join("interfaces")
                    .display()
                    .to_string(); // TODO Can we avoid this conversions?
                arguments.create_and_push("/ifcSearchDir");
                arguments.create_and_push(implicit_lookup_mius_path.clone());
                arguments.create_and_push("/ifcOutput");
                arguments.create_and_push(implicit_lookup_mius_path);

                // The output .obj file
                let obj_file = helpers::generate_prebuilt_miu(compiler, out_dir, interface);
                arguments.create_and_push(format!("/Fo{}", obj_file.display()));
                commands.add_linker_file_path_owned(obj_file);

                if let Some(partition) = &interface.partition {
                    if partition.is_internal_partition {
                        arguments.create_and_push("/internalPartition");
                    } else {
                        arguments.create_and_push("/interface");
                    }
                } else {
                    arguments.create_and_push("/interface");
                }
                arguments.create_and_push("/TP");
                // The input file
                arguments.create_and_push(interface.file())
            }
            CppCompiler::GCC => {
                arguments.create_and_push("-fmodules-ts");
                arguments.create_and_push("-x");
                arguments.create_and_push("c++");
                arguments.create_and_push("-c");
                // The input file
                arguments.create_and_push(interface.file());
                // The output file
                arguments.create_and_push("-o");
                let obj_file = helpers::generate_prebuilt_miu(compiler, out_dir, interface);
                arguments.create_and_push(&obj_file);
                commands.add_linker_file_path_owned(obj_file);
            }
        }

        SourceCommandLine::for_translation_unit(interface, arguments)
    }

    /// Generates the expected arguments for compile the implementation module files
    pub fn generate_module_implementation_cmd<'a>(
        model: &'a ZorkModel,
        cache: &ZorkCache,
        implementation: &'a ModuleImplementationModel,
        commands: &mut Commands,
    ) -> SourceCommandLine {
        let compiler = model.compiler.cpp_compiler;
        let out_dir = model.build.output_dir.as_ref();

        let mut arguments = Arguments::default();
        arguments.push(model.compiler.language_level_arg());
        arguments.extend_from_slice(model.compiler.extra_args());
        arguments.extend_from_slice(model.modules.extra_args());

        match compiler {
            CppCompiler::CLANG => {
                arguments.push_opt(model.compiler.stdlib_arg());
                arguments.create_and_push("-fimplicit-modules");
                arguments.create_and_push("-c");
                arguments.push(clang_args::implicit_module_map(out_dir));

                // The resultant object file
                arguments.create_and_push("-o");
                let obj_file_path = helpers::generate_impl_obj_file(
                    // TODO review this ones, since they module impl are just raw cpp sources
                    compiler,
                    out_dir,
                    implementation,
                );
                commands.add_linker_file_path(&obj_file_path);
                arguments.create_and_push(obj_file_path);

                clang_args::add_direct_module_interfaces_dependencies(
                    &implementation.dependencies,
                    compiler,
                    out_dir,
                    &mut arguments,
                );

                // The input file
                arguments.create_and_push(implementation.file())
            }
            CppCompiler::MSVC => {
                arguments.create_and_push("/EHsc");
                arguments.create_and_push("/nologo");
                arguments.create_and_push("/c");
                arguments.create_and_push("/reference");
                arguments.create_and_push(format! {
                    "std={}", cache.compilers_metadata.msvc.stdlib_bmi_path.display()
                });
                arguments.create_and_push("/reference");
                arguments.create_and_push(format! {
                    "std.compat={}", cache.compilers_metadata.msvc.c_stdlib_bmi_path.display()
                });
                arguments.create_and_push("/ifcSearchDir");
                arguments.create_and_push(
                    out_dir
                        .join(compiler.as_ref())
                        .join("modules")
                        .join("interfaces"),
                );
                // The input file
                arguments.create_and_push(implementation.file());
                // The output .obj file
                let obj_file_path = out_dir // TODO use the helper
                    .join(compiler.as_ref())
                    .join("modules")
                    .join("implementations")
                    .join(implementation.file_stem())
                    .with_extension(compiler.get_obj_file_extension());

                commands.add_linker_file_path(&obj_file_path);
                arguments.create_and_push(format!("/Fo{}", obj_file_path.display()));
            }
            CppCompiler::GCC => {
                arguments.create_and_push("-fmodules-ts");
                arguments.create_and_push("-c");
                // The input file
                arguments.create_and_push(implementation.file());
                // The output file
                arguments.create_and_push("-o");
                let obj_file_path =
                    helpers::generate_impl_obj_file(compiler, out_dir, implementation);
                commands.add_linker_file_path(&obj_file_path);
                arguments.create_and_push(obj_file_path);
            }
        }

        let command_line = SourceCommandLine::for_translation_unit(implementation, arguments);
        // commands.implementations.push(command_line);

        command_line
    }
}

/// Helpers for reduce the cyclomatic complexity introduced by the
/// kind of workflow that should be done with this parse, format and
/// generate.
mod helpers {
    use chrono::{DateTime, Utc};

    use super::*;
    use crate::project_model::sourceset::SourceFile;
    use crate::{
        bounds::TranslationUnit, cache::ZorkCache, cli::output::commands::CommandExecutionResult,
    };
    use std::path::PathBuf;

    /// Creates the path for a prebuilt module interface, based on the default expected
    /// extension for BMI's given a compiler
    ///
    /// We use join for the extension instead `with_extension` because modules are allowed to contain
    /// dots in their module identifier declaration. So, for example, a module with a declaration of:
    /// `export module dotted.module`, in Clang, due to the expected `.pcm` extension, the final path
    /// will be generated as `dotted.pcm`, instead `dotted.module.pcm`.
    ///
    /// For MSVC, we are relying in the autogeneration feature of the BMI automatically by the compiler,
    /// so the output file that we need is an obj file (.obj), and not the
    /// binary module interface (.ifc)
    pub(crate) fn generate_prebuilt_miu(
        compiler: CppCompiler,
        out_dir: &Path,
        interface: &ModuleInterfaceModel,
    ) -> PathBuf {
        let mod_unit = if compiler.eq(&CppCompiler::CLANG) {
            let mut temp = String::new();
            if let Some(partition) = &interface.partition {
                temp.push_str(partition.module);
                temp.push('-');
                if !partition.partition_name.is_empty() {
                    temp.push_str(partition.partition_name)
                } else {
                    temp.push_str(&interface.file_stem())
                }
            } else {
                temp.push_str(interface.module_name)
            }
            temp
        } else {
            interface.module_name.to_string()
        };

        out_dir
            .join(compiler.as_ref())
            .join("modules")
            .join("interfaces")
            .join(format!(
                "{mod_unit}.{}",
                if compiler.eq(&CppCompiler::MSVC) {
                    compiler.get_obj_file_extension()
                } else {
                    compiler.get_typical_bmi_extension()
                }
            ))
    }

    pub(crate) fn generate_impl_obj_file(
        compiler: CppCompiler,
        out_dir: &Path,
        implementation: &ModuleImplementationModel,
    ) -> PathBuf {
        out_dir
            .join(compiler.as_ref())
            .join("modules")
            .join("implementations")
            .join(implementation.file_stem())
            .with_extension(compiler.get_obj_file_extension())
    }

    /// System headers as can be imported as modules must be built before being imported.
    /// First it will compare with the elements stored in the cache, and only we will
    /// generate commands for the non processed elements yet.
    ///
    /// This is for `GCC` and `Clang`
    /// TODO: With the inclusion of std named modules, want we to support this anymore?
    pub(crate) fn build_sys_modules(model: &ZorkModel, commands: &mut Commands, cache: &ZorkCache) {
        if !cache.compilers_metadata.system_modules.is_empty() {
            // TODO BUG - this is not correct.
            // If user later adds a new module, it won't be processed
            log::info!(
                "System modules already build: {:?}. They will be skipped!",
                cache.compilers_metadata.system_modules
            );
        }

        let language_level = model.compiler.language_level_arg();
        let sys_modules = model
            .modules
            .sys_modules
            .iter()
            .filter(|sys_module| {
                !cache
                    .compilers_metadata
                    .system_modules
                    .iter()
                    .any(|s| s.eq(**sys_module))
            })
            .map(|sys_module| {
                let mut v = vec![
                    language_level.clone(),
                    Argument::from("-x"),
                    Argument::from("c++-system-header"),
                    Argument::from(*sys_module),
                ];

                match model.compiler.cpp_compiler {
                    CppCompiler::CLANG => {
                        v.push(Argument::from("-o"));
                        v.push(Argument::from(
                            Path::new(&model.build.output_dir)
                                .join(model.compiler.cpp_compiler.as_ref())
                                .join("modules")
                                .join("interfaces")
                                .join(sys_module)
                                .with_extension(
                                    model.compiler.cpp_compiler.get_typical_bmi_extension(),
                                ),
                        ));
                    }
                    CppCompiler::GCC => {
                        v.push(Argument::from("-fmodules-ts"));
                    }
                    _ => {}
                }

                v
            })
            .collect::<Vec<_>>();

        // Maps the generated command line flags generated for every system module,
        // being the key the name of the system header
        // TODO: is completely unnecessary here a map. We can directly store the flags only one
        // time in a list, because they will always be the same flags for every system module,
        // and the system modules in another list
        // Newest TODO: Can we just store them as Argument(s) in an Arguments? For example, with
        // the new pre-tasks (and therefore, being cached in an unified way?)
        for collection_args in sys_modules {
            commands.system_modules.insert(
                // [3] is for the 4th flag pushed to v
                collection_args[3].value().to_string(),
                Arguments::from_vec(collection_args),
            );
        }
    }

    /// Marks the given source file as already processed,
    /// or if it should be reprocessed again due to a previous failure status,
    /// to avoid losing time rebuilding it if the translation unit
    /// hasn't been modified since the last build process iteration.
    ///
    /// True means 'already processed with previous iteration: Success' and stored on the cache
    pub(crate) fn flag_source_file_without_changes(
        // TODO: kind of `look_for_tu_on_cache_or_generate_command
        compiler: &CppCompiler,
        cache: &ZorkCache,
        file: &Path,
    ) -> bool {
        // TODO: it should return an Optional with the SourceCommandLine from the cache if its already there
        if compiler.eq(&CppCompiler::CLANG) && cfg!(target_os = "windows") {
            // TODO: Review this
            // with the new Clang
            // versions
            log::trace!("Module unit {file:?} will be rebuilt since we've detected that you are using Clang in Windows");
            return false;
        }
        // Check first if the file is already on the cache, and if it's last iteration was successful
        if let Some(cached_compiled_file) = cache.is_file_cached(file) {
            let execution_result = cached_compiled_file.execution_result();
            if execution_result != CommandExecutionResult::Success
                && execution_result != CommandExecutionResult::Cached
            {
                log::trace!(
                    "File {file:?} with status: {:?}. Marked to reprocess",
                    execution_result
                );
                return false;
            };

            // If exists and was successful, let's see if has been modified after the program last iteration
            let last_process_timestamp = cache.last_program_execution;
            let file_metadata = file.metadata();
            match file_metadata {
                Ok(m) => match m.modified() {
                    Ok(modified) => DateTime::<Utc>::from(modified) < last_process_timestamp,
                    Err(e) => {
                        log::error!("An error happened trying to get the last time that the {file:?} was modified. Processing it anyway because {e:?}");
                        false
                    }
                },
                Err(e) => {
                    log::error!("An error happened trying to retrieve the metadata of {file:?}. Processing it anyway because {e:?}");
                    false
                }
            }
        } else {
            false
        }
    }

    /// TODO
    pub(crate) fn translation_unit_must_be_rebuilt(
        compiler: CppCompiler,
        last_process_execution: &DateTime<Utc>,
        cached_source_cmd: &SourceCommandLine,
        file: &Path,
    ) -> bool {
        if compiler.eq(&CppCompiler::CLANG) && cfg!(target_os = "windows") {
            log::trace!("Module unit {:?} will be rebuilt since we've detected that you are using Clang in Windows", cached_source_cmd.path());
            return true;
        }

        let execution_result = cached_source_cmd.execution_result;
        if execution_result != CommandExecutionResult::Success
            && execution_result != CommandExecutionResult::Cached
        {
            log::trace!(
                    "File {file:?} build process failed previously with status: {:?}. It will be rebuilt again",
                    execution_result
                );
            return true;
        };

        // If exists and was successful, let's see if has been modified after the program last iteration
        let file_metadata = file.metadata();
        match file_metadata {
            Ok(m) => match m.modified() {
                Ok(modified) => DateTime::<Utc>::from(modified) > *last_process_execution,
                Err(e) => {
                    log::error!("An error happened trying to get the last time that the {file:?} was modified. Processing it anyway because {e:?}");
                    true
                }
            },
            Err(e) => {
                log::error!("An error happened trying to retrieve the metadata of {file:?}. Processing it anyway because {e:?}");
                true
            }
        }
    }

    pub(crate) fn generate_obj_file_path(
        compiler: CppCompiler,
        out_dir: &Path,
        source: &SourceFile,
    ) -> PathBuf {
        out_dir
            .join(compiler.as_ref())
            .join("sources")
            .join(source.file_stem())
            .with_extension(compiler.get_obj_file_extension())
    }
}
