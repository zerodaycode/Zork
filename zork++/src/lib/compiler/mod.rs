//! The crate responsible for executing the core work of `Zork++`,
//! generate command lines and execute them in a shell of the current
//! operating system against the designed compilers in the configuration
//! file.

pub mod data_factory;

use color_eyre::Result;
use std::path::Path;

use crate::bounds::{ExecutableTarget, TranslationUnit};
use crate::cli::input::{CliArgs, Command};
use crate::cli::output::arguments::{msvc_args, Arguments};
use crate::cli::output::commands::{CommandExecutionResult, SourceCommandLine};
use crate::project_model::compiler::StdLibMode;
use crate::project_model::sourceset::SourceFile;
use crate::utils::constants;
use crate::{
    cache::ZorkCache,
    cli::output::arguments::Argument,
    project_model::{
        compiler::CppCompiler,
        modules::{ModuleImplementationModel, ModuleInterfaceModel},
        ZorkModel,
    },
};

use self::data_factory::CommonArgs;

/// The core procedure. Generates the commands that will be sent to the compiler
/// for every translation unit declared by the user for its project
pub fn generate_commands<'a>(
    model: &'a ZorkModel<'a>,
    mut cache: ZorkCache<'a>,
    cli_args: &'a CliArgs,
) -> Result<ZorkCache<'a>> {
    // TODO: guard it with a presence check */
    // They should only be generated the first time or on every cache reset
    cache.generated_commands.general_args = CommonArgs::from(model);
    cache.generated_commands.compiler_common_args =
        data_factory::compiler_common_arguments_factory(model, &cache);

    // TODO: add them to the commands DS, so they are together until they're generated
    // Build the std library as a module
    generate_modular_stdlibs_cmds(model, &mut cache); // TODO: ward it with an if for only call this fn for the

    // 1st - Build the modules
    if let Some(modules) = &model.modules {
        // TODO: re-think again this optional
        // Pre-tasks
        if model.compiler.cpp_compiler == CppCompiler::GCC && !modules.sys_modules.is_empty() {
            helpers::build_sys_modules(model, &mut cache)
        }

        process_modules(model, &mut cache, cli_args)?;
    };

    // 2nd - Generate the commands for the non-module sources
    generate_sources_cmds_args(model, &mut cache, cli_args)?;
    // 3rd - Genates the commands for the 'targets' declared by the user
    generate_linkage_targets_commands(model, &mut cache, cli_args)?;

    Ok(cache)
}

/// Generates the cmds for build the C++ standard libraries (std and std.compat) acording to the specification
/// of each compiler vendor
fn generate_modular_stdlibs_cmds(model: &ZorkModel<'_>, cache: &mut ZorkCache) {
    let compiler = model.compiler.cpp_compiler;
    let lpe = cache.last_program_execution;

    // TODO: remaining ones: Clang, GCC
    if compiler.eq(&CppCompiler::MSVC) {
        let vs_stdlib_path = cache
            .compilers_metadata
            .msvc
            .vs_stdlib_path
            .as_ref()
            .unwrap()
            .path();
        if let Some(cpp_stdlib_cmd) = cache.generated_commands.cpp_stdlib.as_mut() {
            let build_std = helpers::translation_unit_must_be_built(
                compiler,
                &lpe,
                cpp_stdlib_cmd,
                vs_stdlib_path,
            );

            cpp_stdlib_cmd.need_to_build = build_std;
            if !build_std
                && !cpp_stdlib_cmd
                    .execution_result
                    .eq(&CommandExecutionResult::Cached)
            {
                cpp_stdlib_cmd.execution_result = CommandExecutionResult::Cached;
            } else {
                cpp_stdlib_cmd.execution_result = CommandExecutionResult::PendingToBuild;
            }
        } else {
            log::info!(
                "Generating the command for build the {:?} C++ standard library implementation",
                compiler
            );
            let cpp_stdlib = msvc_args::generate_std_cmd(model, cache, StdLibMode::Cpp);
            cache.generated_commands.cpp_stdlib = Some(cpp_stdlib);
        }

        let vs_ccompat_stdlib_path = cache
            .compilers_metadata
            .msvc
            .vs_c_stdlib_path
            .as_ref()
            .unwrap()
            .path();
        if let Some(ccompat_stdlib_cmd) = cache.generated_commands.c_compat_stdlib.as_mut() {
            let build_ccompat = helpers::translation_unit_must_be_built(
                compiler,
                &lpe,
                ccompat_stdlib_cmd,
                vs_ccompat_stdlib_path,
            );

            ccompat_stdlib_cmd.need_to_build = build_ccompat;
            if !build_ccompat
                && !ccompat_stdlib_cmd
                    .execution_result
                    .eq(&CommandExecutionResult::Cached)
            {
                ccompat_stdlib_cmd.execution_result = CommandExecutionResult::Cached;
            } else {
                ccompat_stdlib_cmd.execution_result = CommandExecutionResult::PendingToBuild;
            }
        } else {
            log::info!(
                "Generating the command for build the {:?} C++ standard library implementation",
                compiler
            );
            let ccompat_stdlib = msvc_args::generate_std_cmd(model, cache, StdLibMode::CCompat);
            cache.generated_commands.c_compat_stdlib = Some(ccompat_stdlib);
        }
    }
}

fn process_modules<'a>(
    model: &'a ZorkModel<'a>,
    cache: &mut ZorkCache<'a>,
    cli_args: &'a CliArgs,
) -> Result<()> {
    let modules = model.modules.as_ref().unwrap(); // TODO: review this opt again

    log::info!("Generating the commands for the module interfaces and partitions...");
    process_kind_translation_units(
        model,
        cache,
        cli_args,
        &modules.interfaces,
        TranslationUnitKind::ModuleInterface,
    );

    log::info!("Generating the commands for the module implementations and partitions...");
    process_kind_translation_units(
        model,
        cache,
        cli_args,
        &modules.implementations,
        TranslationUnitKind::ModuleImplementation,
    );

    Ok(())
}

/// Generates the command line that will be passed to the linker to generate an [`ExecutableTarget`]
/// Generates the commands for the C++ modules declared in the project
///
/// Legacy:
/// If this flow is enabled by the Cli arg `Tests`, then the executable will be generated
/// for the files and properties declared for the tests section in the configuration file
fn generate_linkage_targets_commands<'a>(
    model: &'a ZorkModel<'_>,
    cache: &'a mut ZorkCache<'_>,
    cli_args: &'a CliArgs,
) -> Result<()> {
    // TODO: Shouldn't we start to think about named targets? So introduce the static and dynamic
    // libraries wouldn't be such a pain?
    let is_tests_run = cli_args.command.eq(&Command::Test);
    if is_tests_run {
        generate_linker_general_command_line_args(model, cache, &model.tests)
    } else {
        generate_linker_general_command_line_args(model, cache, &model.executable)
    }
}

fn generate_sources_cmds_args<'a>(
    model: &'a ZorkModel<'a>,
    cache: &mut ZorkCache<'a>,
    cli_args: &'a CliArgs,
) -> Result<()> {
    log::info!("Generating the commands for the source files...");
    // TODO: tests manual run must be start to be deprecated in favour of the future
    // named targets, so we won't mess now with them
    let is_tests_run = cli_args.command.eq(&Command::Test);
    let srcs = if is_tests_run {
        &model.tests.sourceset.sources
    } else {
        &model.executable.sourceset.sources
    };

    process_kind_translation_units(
        model,
        cache,
        cli_args,
        srcs,
        TranslationUnitKind::SourceFile,
    );

    Ok(())
}

pub enum TranslationUnitKind {
    ModuleInterface,
    ModuleImplementation,
    SourceFile,
    HeaderFile,
    ModularStdLib,
}

fn process_kind_translation_units<'a, T: TranslationUnit<'a>>(
    model: &ZorkModel<'_>,
    cache: &mut ZorkCache<'a>,
    cli_args: &CliArgs,
    translation_units: &'a [T],
    for_kind: TranslationUnitKind,
) {
    let compiler = cache.compiler;
    let lpe = cache.last_program_execution;

    translation_units.iter().for_each(|translation_unit| {
        if let Some(generated_cmd) = cache.get_cmd_for_translation_unit_kind(translation_unit, &for_kind) {
            let translation_unit_must_be_rebuilt = helpers::translation_unit_must_be_built(compiler, &lpe, generated_cmd, &translation_unit.file());

            if !translation_unit_must_be_rebuilt {
                log::trace!("Source file:{:?} was not modified since the last iteration. No need to rebuilt it again.", &translation_unit.file());
                // TODO: here is where we should use the normalize_execution_result_status
                // function, to mark executions as cached (when tu musn't be rebuilt)
            }
            generated_cmd.need_to_build = translation_unit_must_be_rebuilt;
        } else {
            let tu_with_erased_type = translation_unit.as_any();
            match for_kind {
                TranslationUnitKind::ModuleInterface => {
                    let resolved_tu = transient::Downcast::downcast_ref::<ModuleInterfaceModel>(tu_with_erased_type);
                    sources::generate_module_interface_cmd(model, cache, resolved_tu.unwrap());
                },
                TranslationUnitKind::ModuleImplementation => sources::generate_module_implementation_cmd(model, cache, transient::Downcast::downcast_ref::<ModuleImplementationModel>(tu_with_erased_type).unwrap()),
                TranslationUnitKind::SourceFile => {
                    let target = if cli_args.command.eq(&Command::Test) { &model.tests as &dyn ExecutableTarget } else { &model.executable as &dyn ExecutableTarget };
                    sources::generate_sources_arguments(model, cache, transient::Downcast::downcast_ref::<SourceFile>(tu_with_erased_type).unwrap(), target)
                }
                _ => todo!()
            }
        };
    });
}

/// Generates the general command line arguments for the desired target
///
/// **implementation note:** All the final byproducts of the compiled translation units, the object
/// files paths, are added in place to the linker args member when they're created, so we can avoid
/// to clone them everytime we create a new [`SourceCommandLine`] for a given translation unit
pub fn generate_linker_general_command_line_args<'a>(
    model: &ZorkModel<'_>,
    cache: &mut ZorkCache<'_>,
    target: &'a impl ExecutableTarget<'a>,
) -> Result<()> {
    log::info!("Generating the linker command line...");

    // NOTE: this early guard is provisional while we don't implement the feature of really having
    // named targets for each user declared one (instead of the actual run/tests kind of thing)
    if cache.generated_commands.linker.args.is_empty() {
        return Ok(());
    }

    let compiler = &model.compiler.cpp_compiler;
    let out_dir: &Path = model.build.output_dir.as_ref();
    let executable_name = target.name();

    let mut arguments = Arguments::default();
    arguments.extend_from_slice(target.extra_args());

    match compiler {
        CppCompiler::CLANG => {
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
        }
        CppCompiler::GCC => {
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

    arguments.extend(&cache.generated_commands.linker.args);
    cache.generated_commands.linker.args = arguments;

    Ok(())
}

/// Specific operations over source files
mod sources {
    use std::path::Path;

    use super::helpers;
    use crate::bounds::ExtraArgs;
    use crate::cache::ZorkCache;
    use crate::cli::output::arguments::Arguments;
    use crate::project_model::modules::ModuleImplementationModel;
    use crate::project_model::sourceset::SourceFile;
    use crate::{
        bounds::{ExecutableTarget, TranslationUnit},
        cli::output::{arguments::clang_args, commands::SourceCommandLine},
        project_model::{compiler::CppCompiler, modules::ModuleInterfaceModel, ZorkModel},
    };

    /// Generates the command line arguments for non-module source files
    pub fn generate_sources_arguments<'a>(
        model: &'a ZorkModel,
        cache: &mut ZorkCache,
        source: &'a SourceFile,
        target: &'a (impl ExecutableTarget<'a> + ?Sized),
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

        let command_line = SourceCommandLine::new(source, arguments);
        cache.generated_commands.sources.push(command_line);
        cache
            .generated_commands
            .add_linker_file_path_owned(obj_file)
    }

    /// Generates the expected arguments for precompile the BMIs depending on self
    pub fn generate_module_interface_cmd<'a>(
        model: &'a ZorkModel,
        cache: &'a mut ZorkCache,
        interface: &'a ModuleInterfaceModel,
    ) {
        let compiler = model.compiler.cpp_compiler;
        let out_dir: &Path = model.build.output_dir.as_ref();

        let mut arguments = Arguments::default();

        match compiler {
            CppCompiler::CLANG => {
                arguments.create_and_push("-x");
                arguments.create_and_push("c++-module");
                arguments.create_and_push("--precompile");
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
                cache
                    .generated_commands
                    .add_linker_file_path_owned(obj_file);
                // The input file
                arguments.create_and_push(interface.file());
            }
            CppCompiler::MSVC => {
                let implicit_lookup_mius_path = out_dir
                    .join(compiler.as_ref())
                    .join("modules")
                    .join("interfaces");
                arguments.create_and_push("/ifcOutput");
                arguments.create_and_push(implicit_lookup_mius_path);

                // The output .obj file
                let obj_file = helpers::generate_prebuilt_miu(compiler, out_dir, interface);
                arguments.create_and_push(format!("/Fo{}", obj_file.display()));
                cache
                    .generated_commands
                    .add_linker_file_path_owned(obj_file);

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
                arguments.create_and_push("-c");
                arguments.create_and_push("-x");
                arguments.create_and_push("c++");
                // The input file
                arguments.create_and_push(interface.file());
                // The output file
                arguments.create_and_push("-o");
                let obj_file = helpers::generate_prebuilt_miu(compiler, out_dir, interface);
                arguments.create_and_push(&obj_file);
                cache
                    .generated_commands
                    .add_linker_file_path_owned(obj_file);
            }
        }

        let cmd_line = SourceCommandLine::new(interface, arguments);
        cache.generated_commands.interfaces.push(cmd_line);
    }

    /// Generates the expected arguments for compile the implementation module files
    pub fn generate_module_implementation_cmd<'a>(
        model: &'a ZorkModel,
        cache: &mut ZorkCache,
        implementation: &'a ModuleImplementationModel,
    ) {
        let compiler = model.compiler.cpp_compiler;
        let out_dir = model.build.output_dir.as_ref();

        let mut arguments = Arguments::default();
        arguments.extend_from_slice(model.compiler.extra_args());

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
                cache
                    .generated_commands
                    .add_linker_file_path(&obj_file_path);
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
                // The input file
                arguments.create_and_push(implementation.file());
                // The output .obj file
                let obj_file_path = out_dir // TODO use the helper
                    .join(compiler.as_ref())
                    .join("modules")
                    .join("implementations")
                    .join::<&str>(implementation.file_stem())
                    .with_extension(compiler.get_obj_file_extension());

                cache
                    .generated_commands
                    .add_linker_file_path(&obj_file_path);
                arguments.create_and_push(format!("/Fo{}", obj_file_path.display()));
            }
            CppCompiler::GCC => {
                // The input file
                arguments.create_and_push(implementation.file());
                // The output file
                arguments.create_and_push("-o");
                let obj_file_path =
                    helpers::generate_impl_obj_file(compiler, out_dir, implementation);
                cache
                    .generated_commands
                    .add_linker_file_path(&obj_file_path);
                arguments.create_and_push(obj_file_path);
            }
        }

        let cmd = SourceCommandLine::new(implementation.to_owned(), arguments);
        cache.generated_commands.implementations.push(cmd);
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
                temp.push_str(&partition.module);
                temp.push('-');
                if !partition.partition_name.is_empty() {
                    temp.push_str(&partition.partition_name)
                } else {
                    temp.push_str(interface.file_stem())
                }
            } else {
                temp.push_str(&interface.module_name)
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
            .join::<&str>(implementation.file_stem())
            .with_extension(compiler.get_obj_file_extension())
    }

    /// System headers as can be imported as modules must be built before being imported.
    /// First it will compare with the elements stored in the cache, and only we will
    /// generate commands for the non processed elements yet.
    ///
    /// This is for `GCC` and `Clang`
    /// TODO: With the inclusion of std named modules, want we to support this anymore?
    pub(crate) fn build_sys_modules(model: &ZorkModel, cache: &mut ZorkCache) {
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
            .as_ref()
            .map(|modules| modules.sys_modules.clone())
            .unwrap_or_default()
            .iter()
            .filter(|sys_module| {
                !cache
                    .compilers_metadata
                    .system_modules
                    .iter()
                    .any(|s| s.eq(*sys_module))
            })
            .map(|sys_module| {
                let mut v = vec![
                    language_level.clone(),
                    Argument::from("-x"),
                    Argument::from("c++-system-header"),
                    Argument::from(sys_module),
                ];

                match model.compiler.cpp_compiler {
                    CppCompiler::CLANG => {
                        v.push(Argument::from("-o"));
                        v.push(Argument::from(
                            Path::new(&model.build.output_dir)
                                .join(model.compiler.cpp_compiler.as_ref())
                                .join("modules")
                                .join("interfaces")
                                .join(sys_module.to_string())
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
            cache.generated_commands.system_modules.insert(
                // [3] is for the 4th flag pushed to v
                collection_args[3].value().to_string(),
                Arguments::from_vec(collection_args),
            );
        }
    }

    /// Checks if some user declared [TranslationUnit] must be built (for example, on the first
    /// iteration it will always be the case), or if the file didn't had changes since the last
    /// Zork++ run and therefore, we can avoid rebuilt it
    pub(crate) fn translation_unit_must_be_built(
        // TODO: separation of concerns? Please
        // Just make two fns, the one that checks for the status and the one that checks for modifications
        // then just use a template-factory design pattern by just abstracting away the two checks in one call
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
        // TODO: Big one. What instead of having the boolean flag of need_to_built we modify the enumerated of execution result, change its name
        // to TranslationUnitStatus or some other better name pls, and we introduce a variant for mark the files that must be built?
        // TODO: also, introduce a variant like PendingToBuilt, that just will be check before executing the commands?
        // TODO: and even better, what if we use the new enum type as a wrapper over execution result? So we can store inside the variants
        // that contains build info the execution result, and in the others maybe nothing, or other interesting data?
        {
            log::trace!(
                    "File {file:?} build process failed previously with status: {:?}. It will be rebuilt again", // TODO: technically, it can be Unprocessed, which isn't a failure
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
            .join::<&str>(source.file_stem())
            .with_extension(compiler.get_obj_file_extension())
    }
}
