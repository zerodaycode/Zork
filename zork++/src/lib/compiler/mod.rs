//! The crate responsible for executing the core work of `Zork++`,
// generate command lines and execute them in a shell of the current
// operating system against the designed compilers in the configuration
// file.

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

/// The entry point of the compilation process
///
/// Whenever this process gets triggered, the files declared within the
/// configuration file will be build
pub fn build_project<'a>(
    model: &'a ZorkModel<'a>,
    cache: &ZorkCache,
    tests: bool,
) -> Result<Commands<'a>> {
    // A registry of the generated command lines
    let mut commands = Commands::new(&model.compiler.cpp_compiler);

    // Pre-tasks
    if model.compiler.cpp_compiler == CppCompiler::GCC && !model.modules.sys_modules.is_empty() {
        helpers::build_sys_modules(model, &mut commands, cache)
    }
    // Build the std library as a module
    build_modular_stdlib(model, cache, &mut commands); // TODO: ward it with an if for only call this fn for the

    // 1st - Build the modules
    build_modules(model, cache, &mut commands)?;
    // 2nd - Build the non module sources
    build_sources(model, cache, &mut commands, tests)?;
    // 3rd - Build the executable or the tests
    build_executable(model, cache, &mut commands, tests)?;

    Ok(commands)
}

/// Builds the C++ standard library as a pre-step acording to the specification
/// of each compiler vendor
fn build_modular_stdlib<'a>(
    model: &'a ZorkModel<'_>,
    cache: &ZorkCache,
    commands: &mut Commands<'a>,
) {
    let compiler = model.compiler.cpp_compiler;

    // TODO: remaining ones: Clang, GCC
    if compiler.eq(&CppCompiler::MSVC) {
        if !cache.compilers_metadata.msvc.is_loaded() {
            return;
        }

        let cpp_stdlib = msvc_args::generate_std_cmd_args(model, cache, StdLibMode::Cpp);
        commands.pre_tasks.push(cpp_stdlib);

        let c_cpp_stdlib = msvc_args::generate_std_cmd_args(model, cache, StdLibMode::CCompat);
        commands.pre_tasks.push(c_cpp_stdlib);
    }
}

/// Triggers the build process for compile the source files declared for the project
/// If this flow is enabled by the Cli arg `Tests`, then the executable will be generated
/// for the files and properties declared for the tests section in the configuration file
fn build_executable<'a>(
    model: &'a ZorkModel<'_>,
    cache: &ZorkCache,
    commands: &'_ mut Commands<'a>,
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

fn build_sources<'a>(
    model: &'a ZorkModel<'_>,
    cache: &ZorkCache,
    commands: &'_ mut Commands<'a>,
    tests: bool,
) -> Result<()> {
    log::info!("Building the source files...");
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
        commands.generated_files_paths.push(Argument::from(helpers::generate_obj_file(
            model.compiler.cpp_compiler, &model.build.output_dir, src
        ).to_string()))
    });

    Ok(())
}

/// Triggers the build process for compile the declared modules in the project
///
/// This function acts like a operation result processor, by running instances
/// and parsing the obtained result, handling the flux according to the
/// compiler responses>
fn build_modules<'a>(
    model: &'a ZorkModel,
    cache: &ZorkCache,
    commands: &mut Commands<'a>,
) -> Result<()> {
    log::info!("Building the module interfaces and partitions...");
    build_module_interfaces(model, cache, &model.modules.interfaces, commands);

    log::info!("Building the module implementations...");
    build_module_implementations(model, cache, &model.modules.implementations, commands);

    Ok(())
}

/// Parses the configuration in order to build the BMIs declared for the project,
/// by precompiling the module interface units
fn build_module_interfaces<'a>(
    model: &'a ZorkModel<'_>,
    cache: &ZorkCache,
    interfaces: &'a [ModuleInterfaceModel],
    commands: &mut Commands<'a>,
) {
    interfaces.iter().for_each(|module_interface| {
        if !flag_source_file_without_changes(&model.compiler.cpp_compiler, cache, &module_interface.file()) {
            sources::generate_module_interfaces_args(model, cache, module_interface, commands);
        } else {
            let command_line = SourceCommandLine::from_translation_unit(
                module_interface, Arguments::default(), true, CommandExecutionResult::Cached
            );

            log::trace!("Source file:{:?} was not modified since the last iteration. No need to rebuilt it again.", &module_interface.file());
            commands.interfaces.push(command_line);
            commands.generated_files_paths.push(Argument::from(helpers::generate_prebuilt_miu(
                model.compiler.cpp_compiler, &model.build.output_dir, module_interface
            )))
        }
    });
}

/// Parses the configuration in order to compile the module implementation
/// translation units declared for the project
fn build_module_implementations<'a>(
    model: &'a ZorkModel,
    cache: &ZorkCache,
    impls: &'a [ModuleImplementationModel],
    commands: &mut Commands<'a>,
) {
    impls.iter().for_each(|module_impl| {
        if !flag_source_file_without_changes(&model.compiler.cpp_compiler, cache, &module_impl.file()) {
            sources::generate_module_implementation_args(model, cache, module_impl, commands);
        } else {
            let command_line = SourceCommandLine::from_translation_unit(
                module_impl, Arguments::default(), true, CommandExecutionResult::Cached
            );

            log::trace!("Source file:{:?} was not modified since the last iteration. No need to rebuilt it again.", &module_impl.file());
            commands.implementations.push(command_line); // TODO:: There's other todo where we
                                                         // explain why we should change this code
                                                         // (and the other similar ones)
            commands.generated_files_paths.push(Argument::from(helpers::generate_impl_obj_file(
                model.compiler.cpp_compiler, &model.build.output_dir, module_impl
            )))
        }
    });
}

/// Generates the command line arguments for the desired target
pub fn generate_main_command_line_args<'a>(
    model: &'a ZorkModel,
    cache: &ZorkCache,
    commands: &mut Commands<'a>,
    target: &'a impl ExecutableTarget<'a>,
) -> Result<()> {
    log::info!("Generating the main command line...");

    let compiler = &model.compiler.cpp_compiler;
    let out_dir = model.build.output_dir.as_ref();
    let executable_name = target.name();

    let mut arguments = Arguments::default();
    arguments.push(model.compiler.language_level_arg());
    arguments.extend_from_slice(model.compiler.extra_args());
    arguments.extend_from_slice(target.extra_args());

    match compiler {
        CppCompiler::CLANG => {
            arguments.push_opt(model.compiler.stdlib_arg());
            arguments.create_and_push("-fimplicit-modules");
            arguments.push(clang_args::implicit_module_maps(out_dir));

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
    arguments.extend(commands.generated_files_paths.clone());

    commands.main.args.extend(arguments);
    commands.main.sources_paths = target
        .sourceset()
        .sources
        .iter()
        .map(|s| s.file())
        .collect::<Vec<_>>();

    Ok(())
}

/// Specific operations over source files
mod sources {
    use super::helpers;
    use crate::bounds::ExtraArgs;
    use crate::cache::ZorkCache;
    use crate::cli::output::arguments::Arguments;
    use crate::project_model::sourceset::SourceFile;
    use crate::{
        bounds::{ExecutableTarget, TranslationUnit},
        cli::output::{
            arguments::{clang_args, Argument},
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
        commands: &mut Commands<'a>,
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
                arguments.push(clang_args::implicit_module_maps(out_dir));
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

        let obj_file = helpers::generate_obj_file(compiler, out_dir, source);
        let fo = if compiler.eq(&CppCompiler::MSVC) {
            "/Fo"
        } else {
            ""
        };
        arguments.create_and_push(format!("{fo}{obj_file}"));
        arguments.create_and_push(source.file());

        let command_line = SourceCommandLine::from_translation_unit(
            source,
            arguments,
            false,
            CommandExecutionResult::default(),
        );
        commands.sources.push(command_line);
        commands
            .generated_files_paths
            .push(Argument::from(obj_file.to_string()))
    }

    /// Generates the expected arguments for precompile the BMIs depending on self
    pub fn generate_module_interfaces_args<'a>(
        model: &'a ZorkModel,
        cache: &ZorkCache,
        interface: &'a ModuleInterfaceModel,
        commands: &mut Commands<'a>,
    ) {
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
                arguments.create_and_push("-x");
                arguments.create_and_push("c++-module");
                arguments.create_and_push("--precompile");
                arguments.push(clang_args::implicit_module_maps(out_dir));
                arguments.create_and_push(format!(
                    "-fprebuilt-module-path={}/clang/modules/interfaces",
                    out_dir.display()
                ));
                clang_args::add_direct_module_interfaces_dependencies(
                    &interface.dependencies,
                    compiler,
                    out_dir,
                    &mut arguments,
                );

                // The resultant BMI as a .pcm file
                arguments.create_and_push("-o");
                // The output file
                let miu_file_path =
                    Argument::from(helpers::generate_prebuilt_miu(compiler, out_dir, interface));
                commands.generated_files_paths.push(miu_file_path.clone());
                arguments.push(miu_file_path);
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
                let obj_file =
                    Argument::from(helpers::generate_prebuilt_miu(compiler, out_dir, interface));
                commands.generated_files_paths.push(obj_file.clone());
                arguments.create_and_push(format!("/Fo{obj_file}"));

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
                let miu_file_path =
                    Argument::from(helpers::generate_prebuilt_miu(compiler, out_dir, interface));
                commands.generated_files_paths.push(miu_file_path.clone());
                arguments.push(miu_file_path);
            }
        }

        let command_line = SourceCommandLine::from_translation_unit(
            interface,
            arguments,
            false,
            CommandExecutionResult::default(),
        );
        commands.interfaces.push(command_line);
    }

    /// Generates the expected arguments for compile the implementation module files
    pub fn generate_module_implementation_args<'a>(
        model: &'a ZorkModel,
        cache: &ZorkCache,
        implementation: &'a ModuleImplementationModel,
        commands: &mut Commands<'a>,
    ) {
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
                arguments.push(clang_args::implicit_module_maps(out_dir));

                // The resultant object file
                arguments.create_and_push("-o");
                let obj_file_path = Argument::from(helpers::generate_impl_obj_file(
                    compiler,
                    out_dir,
                    implementation,
                ));
                commands.generated_files_paths.push(obj_file_path.clone());
                arguments.push(obj_file_path);

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
                let obj_file_path = out_dir
                    .join(compiler.as_ref())
                    .join("modules")
                    .join("implementations")
                    .join(implementation.file_stem())
                    .with_extension(compiler.get_obj_file_extension());

                commands
                    .generated_files_paths
                    .push(Argument::from(obj_file_path.clone()));
                arguments.create_and_push(format!("/Fo{}", obj_file_path.display()));
            }
            CppCompiler::GCC => {
                arguments.create_and_push("-fmodules-ts");
                arguments.create_and_push("-c");
                // The input file
                arguments.create_and_push(implementation.file());
                // The output file
                arguments.create_and_push("-o");
                let obj_file_path = Argument::from(helpers::generate_impl_obj_file(
                    compiler,
                    out_dir,
                    implementation,
                ));
                commands.generated_files_paths.push(obj_file_path.clone());
                arguments.push(obj_file_path);
            }
        }

        let command_line = SourceCommandLine::from_translation_unit(
            implementation,
            arguments,
            false,
            CommandExecutionResult::default(),
        );
        commands.implementations.push(command_line);
    }
}

/// Helpers for reduce the cyclomatic complexity introduced by the
/// kind of workflow that should be done with this parse, format and
/// generate.
mod helpers {
    use chrono::{DateTime, Utc};
    use std::fmt::Display;

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
    pub(crate) fn build_sys_modules<'a>(
        model: &'a ZorkModel,
        commands: &mut Commands<'a>,
        cache: &ZorkCache,
    ) {
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
                collection_args[3].value.to_string(),
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
        compiler: &CppCompiler,
        cache: &ZorkCache,
        file: &Path,
    ) -> bool {
        if compiler.eq(&CppCompiler::CLANG) && cfg!(target_os = "windows") {
            // TODO: Review this
            // with the new Clang
            // versions
            log::trace!("Module unit {file:?} will be rebuilt since we've detected that you are using Clang in Windows");
            return false;
        }
        // Check first if the file is already on the cache, and if it's last iteration was successful
        if let Some(cached_file) = cache.is_file_cached(file) {
            if cached_file.execution_result != CommandExecutionResult::Success
                && cached_file.execution_result != CommandExecutionResult::Cached
            {
                log::trace!(
                    "File {file:?} with status: {:?}. Marked to reprocess",
                    cached_file.execution_result
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

    pub(crate) fn generate_obj_file(
        compiler: CppCompiler,
        out_dir: &Path,
        source: &SourceFile,
    ) -> impl Display {
        format!(
            "{}",
            out_dir
                .join(compiler.as_ref())
                .join("sources")
                .join(source.file_stem())
                .with_extension(compiler.get_obj_file_extension())
                .display()
        )
    }
}
