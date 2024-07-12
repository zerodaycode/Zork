//! The crate responsible for executing the core work of `Zork++`,
//! generate command lines and execute them in a shell of the current
//! operating system against the designed compilers in the configuration
//! file.

use std::path::Path;

use color_eyre::Result;

use crate::cli::output::commands::TranslationUnitStatus;
use crate::project_model::modules::SystemModule;
use crate::{
    cache::ZorkCache,
    cli::{
        input::{CliArgs, Command},
        output::{arguments::Argument, commands::SourceCommandLine},
    },
    domain::{
        target::ExecutableTarget,
        translation_unit::{TranslationUnit, TranslationUnitKind},
    },
    project_model::{
        compiler::{CppCompiler, StdLibMode},
        modules::{ModuleImplementationModel, ModuleInterfaceModel},
        sourceset::SourceFile,
        ZorkModel,
    },
    utils::constants,
};

use self::data_factory::CommonArgs;

pub mod data_factory;

/// The core procedure. Generates the commands that will be sent to the compiler
/// for every translation unit declared by the user for its project
pub fn generate_commands<'a>(
    model: &'a ZorkModel<'a>,
    mut cache: ZorkCache<'a>,
    cli_args: &'a CliArgs,
) -> Result<ZorkCache<'a>> {
    // Load the general args and the compiler specific ones if it's necessary
    load_flyweights_for_general_shared_data(model, &mut cache);

    // Build the std library as a module
    generate_modular_stdlibs_cmds(model, &mut cache, cli_args);

    // Pre-tasks
    if model.compiler.cpp_compiler != CppCompiler::MSVC && !model.modules.sys_modules.is_empty() {
        generate_sys_modules_commands(model, &mut cache, cli_args);
    }

    // Translation units and linker

    // 1st - Build the modules
    process_modules(model, &mut cache, cli_args)?;
    // 2nd - Generate the commands for the non-module sources
    generate_sources_cmds_args(model, &mut cache, cli_args)?;
    // 3rd - Generate the linker command for the 'target' declared by the user
    generate_linkage_targets_commands(model, &mut cache, cli_args);

    Ok(cache)
}

/// Adds to the cache the data on the *flyweight* data structures that holds all the
/// command line arguments that are shared among the command lines
fn load_flyweights_for_general_shared_data(model: &ZorkModel, cache: &mut ZorkCache) {
    if cache.generated_commands.general_args.is_none() {
        cache.generated_commands.general_args = Some(CommonArgs::from(model));
    }

    if cache.generated_commands.compiler_common_args.is_none() {
        cache.generated_commands.compiler_common_args = Some(
            data_factory::compiler_common_arguments_factory(model, cache),
        );
    }
}

/// Generates the cmds for build the C++ standard libraries (std and std.compat) according to the specification
/// of each compiler vendor
fn generate_modular_stdlibs_cmds(model: &ZorkModel, cache: &mut ZorkCache, _cli_args: &CliArgs) {
    // TODO: remaining ones: Clang, GCC.
    // NOTE: Provisionally 'If' guarded because only MSVC is supported now to build the
    // C++ standard library implementations
    if model.compiler.cpp_compiler.eq(&CppCompiler::MSVC) {
        modules::generate_modular_cpp_stdlib_args(model, cache, StdLibMode::Cpp);
        modules::generate_modular_cpp_stdlib_args(model, cache, StdLibMode::CCompat);
    }
}

/// Procedure to generate the commands for the system headers of their standard C++ library
/// for a given compiler
///
/// These commands are the ones that allows to translate C++ standard headers to named modules
fn generate_sys_modules_commands<'a>(
    model: &'a ZorkModel<'a>,
    cache: &mut ZorkCache<'a>,
    cli_args: &'a CliArgs,
) {
    process_kind_translation_units(
        model,
        cache,
        cli_args,
        &model.modules.sys_modules,
        TranslationUnitKind::SystemHeader,
    );
}

/// The procedure that takes care of generating the [`SourceCommandLine`] to build the user's declared
/// C++ standard names modules
fn process_modules<'a>(
    model: &'a ZorkModel<'a>,
    cache: &mut ZorkCache<'a>,
    cli_args: &'a CliArgs,
) -> Result<()> {
    let modules = &model.modules;

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

/// Processor for generate the commands of the non-modular translation units
///
/// *NOTE*: This will be changed on the future, when we decide how we should architecture the implementation
/// of named targets
///
/// *IMPL_NOTE*: Consider in the future if it's worth to maintain two paths for build module implementations
/// and source, since they are basically (almost) the same thing
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
) {
    // TODO: Shouldn't we start to think about named targets? So introduce the static and dynamic
    // libraries wouldn't be such a pain?
    let is_tests_run = cli_args.command.eq(&Command::Test);
    if is_tests_run {
        generate_linker_general_command_line_args(model, cache, &model.tests);
    } else {
        generate_linker_general_command_line_args(model, cache, &model.executable);
    }
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
) {
    log::info!("Generating the linker command line...");

    let linker = &mut cache.generated_commands.linker;

    let compiler = &model.compiler.cpp_compiler;
    let out_dir: &Path = model.build.output_dir.as_ref();
    let executable_name = target.name();

    let target_output = Argument::from(
        out_dir
            .join(compiler.as_ref())
            .join(executable_name)
            .with_extension(constants::BINARY_EXTENSION),
    );

    if linker.target.ne(&target_output) {
        match compiler {
            CppCompiler::CLANG | CppCompiler::GCC => linker.target = target_output,
            CppCompiler::MSVC => linker.target = Argument::from(format!("/Fe{}", target_output)),
        };
    }

    if Iterator::ne(linker.extra_args.iter(), target.extra_args().iter()) {
        linker.extra_args.clear();
        linker.extra_args.extend_from_slice(target.extra_args());
    }
}

/// The core procedure of the commands generation process.
///
/// It takes care of generate the [`SourceCommandLine`] for a set of given implementors of [`TranslationUnit`],
/// processing them according to the passed [`TranslationUnitKind`] discriminator if the command doesn't exist,
/// otherwise, it will handle the need of tracking individually every translation unit in every program iteration
/// (while the cache isn't purged by the user) to set their [`TranslationUnitStatus`] flag, which ultimately
/// decides on every run if the file must be sent to build to the target [`CppCompiler`]
fn process_kind_translation_units<'a, T: TranslationUnit<'a>>(
    model: &ZorkModel<'_>,
    cache: &mut ZorkCache<'a>,
    cli_args: &CliArgs,
    translation_units: &'a [T],
    for_kind: TranslationUnitKind,
) {
    translation_units.iter().for_each(|translation_unit| {
        process_kind_translation_unit(model, cache, cli_args, translation_unit, &for_kind)
    });
}

fn process_kind_translation_unit<'a, T: TranslationUnit<'a>>(
    model: &ZorkModel<'_>,
    cache: &mut ZorkCache<'a>,
    cli_args: &CliArgs,
    translation_unit: &'a T,
    for_kind: &TranslationUnitKind,
) {
    let compiler = cache.compiler;
    let lpe = cache.metadata.last_program_execution;

    if let Some(generated_cmd) = cache.get_cmd_for_translation_unit_kind(translation_unit, for_kind)
    {
        let build_translation_unit =
            helpers::determine_translation_unit_status(compiler, &lpe, generated_cmd);

        if build_translation_unit.ne(&TranslationUnitStatus::PendingToBuild) {
            log::trace!("Source file: {:?} was not modified since the last iteration. No need to rebuilt it again.", &translation_unit.path());
        }

        generated_cmd.status = build_translation_unit;
    } else {
        let tu_with_erased_type = translation_unit.as_any();
        // TODO: remove the .unwrap() (s) below for some other robust solution
        match &for_kind {
            TranslationUnitKind::ModuleInterface => {
                let resolved_tu =
                    transient::Downcast::downcast_ref::<ModuleInterfaceModel>(tu_with_erased_type);
                modules::generate_module_interface_cmd(model, cache, resolved_tu.unwrap());
            }
            TranslationUnitKind::ModuleImplementation => {
                modules::generate_module_implementation_cmd(
                    model,
                    cache,
                    transient::Downcast::downcast_ref::<ModuleImplementationModel>(
                        tu_with_erased_type,
                    )
                    .unwrap(),
                )
            }
            TranslationUnitKind::SourceFile => {
                let target = if cli_args.command.eq(&Command::Test) {
                    &model.tests as &dyn ExecutableTarget
                } else {
                    &model.executable as &dyn ExecutableTarget
                };
                sources::generate_sources_arguments(
                    model,
                    cache,
                    transient::Downcast::downcast_ref::<SourceFile>(tu_with_erased_type).unwrap(),
                    target,
                )
            }
            TranslationUnitKind::SystemHeader => modules::generate_sys_module_cmd(
                model,
                cache,
                transient::Downcast::downcast_ref::<SystemModule>(tu_with_erased_type).unwrap(),
            ),
            _ => todo!(),
        }
    };
}

/// Command line arguments generators procedures for C++ standard modules
mod modules {
    use std::path::Path;

    use crate::cache::ZorkCache;
    use crate::cli::output::arguments::{clang_args, msvc_args, Arguments};
    use crate::cli::output::commands::{SourceCommandLine, TranslationUnitStatus};
    use crate::compiler::helpers;
    use crate::compiler::helpers::generate_bmi_file_path;
    use crate::domain::translation_unit::TranslationUnit;
    use crate::project_model::compiler::{CppCompiler, StdLibMode};
    use crate::project_model::modules::{
        ModuleImplementationModel, ModuleInterfaceModel, SystemModule,
    };
    use crate::project_model::ZorkModel;

    /// Generates the expected arguments for precompile the BMIs depending on self
    pub fn generate_module_interface_cmd<'a>(
        model: &'a ZorkModel,
        cache: &'a mut ZorkCache,
        interface: &'a ModuleInterfaceModel,
    ) {
        let mut arguments = Arguments::default();
        let compiler = model.compiler.cpp_compiler;
        let out_dir: &Path = model.build.output_dir.as_ref();

        // The Path of the generated binary module interface
        let binary_module_ifc = helpers::generate_prebuilt_miu(compiler, out_dir, interface);

        match compiler {
            CppCompiler::CLANG => {
                arguments.push("-x");
                arguments.push("c++-module");
                arguments.push("--precompile");
                clang_args::add_direct_module_interfaces_dependencies(
                    &interface.dependencies,
                    compiler,
                    out_dir,
                    &mut arguments,
                );

                // The generated BMI
                arguments.push("-o");
                arguments.push(&binary_module_ifc);
            }
            CppCompiler::MSVC => {
                arguments.push("/ifcOutput");
                let implicit_lookup_mius_path = out_dir
                    .join(compiler.as_ref())
                    .join("modules")
                    .join("interfaces");
                arguments.push(implicit_lookup_mius_path);

                // The output .obj file
                arguments.push(format!("/Fo{}", binary_module_ifc.display()));

                if let Some(partition) = &interface.partition {
                    if partition.is_internal_partition {
                        arguments.push("/internalPartition");
                    } else {
                        arguments.push("/interface");
                    }
                } else {
                    arguments.push("/interface");
                }
                arguments.push("/TP");
            }
            CppCompiler::GCC => {
                arguments.push("-x");
                arguments.push("c++");
                // The output file
                arguments.push("-o");
                arguments.push(&binary_module_ifc);
            }
        }

        // The input file
        arguments.push(interface.path());

        cache
            .generated_commands
            .add_linker_file_path(binary_module_ifc);

        let cmd_line = SourceCommandLine::new(interface, arguments);
        cache.generated_commands.interfaces.push(cmd_line);
    }

    /// Generates the required arguments for compile the implementation module files
    pub fn generate_module_implementation_cmd<'a>(
        model: &'a ZorkModel,
        cache: &mut ZorkCache,
        implementation: &'a ModuleImplementationModel,
    ) {
        let compiler = model.compiler.cpp_compiler;
        let out_dir = model.build.output_dir.as_ref();

        let mut arguments = Arguments::default();

        // The input file
        arguments.push(implementation.path());
        let obj_file_path = helpers::generate_obj_file(compiler, out_dir, implementation);

        match compiler {
            CppCompiler::CLANG => {
                // The resultant object file
                arguments.push("-o");
                arguments.push(&obj_file_path);

                clang_args::add_direct_module_interfaces_dependencies(
                    &implementation.dependencies,
                    compiler,
                    out_dir,
                    &mut arguments,
                );
            }
            CppCompiler::MSVC => {
                // The output .obj file
                arguments.push(format!("/Fo{}", obj_file_path.display()));
            }
            CppCompiler::GCC => {
                // The output file
                arguments.push("-o");
                arguments.push(&obj_file_path);
            }
        }

        cache.generated_commands.add_linker_file_path(obj_file_path);

        let cmd = SourceCommandLine::new(implementation.to_owned(), arguments);
        cache.generated_commands.implementations.push(cmd);
    }

    /// System headers can be imported as modules, but they must be built before being imported.
    ///
    /// This feature is supported by `GCC` and `Clang`
    /// NOTE: With the inclusion of std named modules, want we to support this anymore?
    pub(crate) fn generate_sys_module_cmd(
        model: &ZorkModel,
        cache: &mut ZorkCache,
        sys_module: &SystemModule,
    ) {
        let sys_module_name = &sys_module.file_stem;
        let generated_bmi_path = generate_bmi_file_path(
            &model.build.output_dir,
            model.compiler.cpp_compiler,
            sys_module_name,
        );

        let mut args = Arguments::default();
        args.push("-x");
        args.push("c++-system-header");
        args.push(sys_module_name);

        match model.compiler.cpp_compiler {
            CppCompiler::CLANG => {
                args.push("-o");
                args.push(&generated_bmi_path);
            }
            CppCompiler::GCC => {
                // `GCC` system headers built as modules goes directly to their `gcm.cache`
                args.push("-fmodules-ts");
            }
            _ => {}
        };

        let cmd = SourceCommandLine {
            directory: generated_bmi_path, // TODO: we are using the generated byproduct as the path for checking for its
            // existence instead of the system header one while we don't implement the feature that inspects where the standard
            // libraries lives. Isn't ideal, but serves the basic purpose of regenerating the commands when they it's needed, and
            // system headers aren't unlikely to change (unless an stdlib update)
            filename: sys_module.to_string(),
            args,
            status: TranslationUnitStatus::PendingToBuild,
        };
        cache.generated_commands.system_modules.push(cmd);
    }

    pub(crate) fn generate_modular_cpp_stdlib_args(
        model: &ZorkModel,
        cache: &mut ZorkCache,
        stdlib_mode: StdLibMode,
    ) {
        let compiler = model.compiler.cpp_compiler;
        let lpe = cache.metadata.last_program_execution;

        if let Some(cpp_stdlib_cmd) = cache.get_cpp_stdlib_cmd_by_kind(stdlib_mode) {
            cpp_stdlib_cmd.status =
                helpers::determine_translation_unit_status(compiler, &lpe, cpp_stdlib_cmd);
        } else {
            let compiler = model.compiler.cpp_compiler;
            log::info!(
                "Generating the command for build the {:?} {}",
                compiler,
                stdlib_mode.printable_info()
            );

            let scl = msvc_args::generate_std_cmd(cache, stdlib_mode);
            cache.set_cpp_stdlib_cmd_by_kind(stdlib_mode, Some(scl));
            // TODO: see the Some(scl) above? well, implement the generators for the other compilers
            // and just return optional none?, so we can get rid out of all the todo
        }
    }
}

/// Specific operations over source files
mod sources {
    use crate::cache::ZorkCache;
    use crate::cli::output::arguments::Arguments;
    use crate::domain::target::ExecutableTarget;
    use crate::domain::translation_unit::TranslationUnit;
    use crate::project_model::sourceset::SourceFile;
    use crate::{
        cli::output::commands::SourceCommandLine,
        project_model::{compiler::CppCompiler, ZorkModel},
    };

    use super::helpers;

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
        arguments.extend_from_slice(target.extra_args());

        let obj_file = helpers::generate_obj_file(compiler, out_dir, source);
        let fo = if compiler.eq(&CppCompiler::MSVC) {
            "/Fo"
        } else {
            "-o"
        };
        arguments.push(format!("{fo}{}", obj_file.display()));
        arguments.push(source.path());

        let command_line = SourceCommandLine::new(source, arguments);
        cache.generated_commands.sources.push(command_line);
        cache.generated_commands.add_linker_file_path(obj_file)
    }
}

/// Helpers for reduce the cyclomatic complexity introduced by the
/// kind of workflow that should be done with this parse, format and
/// generate.
mod helpers {
    use std::path::PathBuf;

    use chrono::{DateTime, Utc};

    use crate::cli::output::commands::TranslationUnitStatus;

    use crate::utils::constants::dir_names;

    use super::*;

    /// Creates the path for a prebuilt module interface, based on the default expected
    /// extension for BMI's given a compiler
    ///
    /// We use join for the extension instead `with_extension` because modules are allowed to contain
    /// dots in their module identifier declaration. So, for example, a module with a declaration of:
    /// `export module dotted.module`, in Clang, due to the expected `.pcm` extension, the final path
    /// will be generated as `dotted.pcm`, instead `dotted.module.pcm`.
    ///
    /// For MSVC, we are relying on the auto generation feature of the BMI automatically by the compiler,
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

        generate_bmi_file_path(out_dir, compiler, &mod_unit)
    }

    /// Generates the [`PathBuf`] of the resultant binary module interface file of a C++ module interface
    pub(crate) fn generate_bmi_file_path(
        out_dir: &Path,
        compiler: CppCompiler,
        module_name: &str,
    ) -> PathBuf {
        out_dir
            .join(compiler.as_ref())
            .join("modules")
            .join("interfaces")
            .join(format!(
                "{module_name}.{}",
                if compiler.eq(&CppCompiler::MSVC) {
                    compiler.get_obj_file_extension()
                } else {
                    compiler.get_typical_bmi_extension()
                }
            ))
    }

    /// Generates the [`PathBuf`] of the resultant `.obj` file of a [`TranslationUnit`] where the
    /// `.obj` file is one of the byproducts of the build process (and the one that will be sent
    /// to the linker)
    pub(crate) fn generate_obj_file<'a, T: TranslationUnit<'a>>(
        compiler: CppCompiler,
        out_dir: &Path,
        implementation: &T,
    ) -> PathBuf {
        out_dir
            .join(compiler.as_ref())
            .join(dir_names::OBJECT_FILES)
            .join::<&str>(implementation.file_stem())
            .with_extension(compiler.get_obj_file_extension())
    }

    /// Template factory function to call the inspectors of the status of a file on the fs that
    /// is represented within `Zork++` as some kind of [`TranslationUnit`] and the status flags
    /// tracked on the entities like [`SourceCommandLine::status`] and others from the [`ZorkCache`]
    /// as well to determine when a concrete user declared file must be sent to the compiler in order
    /// to be built, or we can skip it
    ///
    /// *returns: <[`TranslationUnitStatus`]>* - The state that should be set to the current
    /// [`SourceCommandLine`] in order to be handled
    pub(crate) fn determine_translation_unit_status(
        compiler: CppCompiler,
        last_process_execution: &DateTime<Utc>,
        cached_source_cmd: &SourceCommandLine,
    ) -> TranslationUnitStatus {
        // In case the user deleted the translation unit from the fs but not from the Zork++ cfg file
        let translation_unit_has_been_deleted = !cached_source_cmd.path().exists();
        if translation_unit_has_been_deleted {
            return TranslationUnitStatus::ToDelete;
        }

        // In case the file suffered changes
        let need_to_build = particular_checks_for_sent_to_build(compiler, cached_source_cmd)
            || translation_unit_has_changes_on_fs(last_process_execution, cached_source_cmd);

        if need_to_build {
            TranslationUnitStatus::PendingToBuild
        } else {
            compute_translation_unit_status(cached_source_cmd)
        }
    }

    /// Inspects the status field of a given [`SourceCommandLine`] of a [`TranslationUnit`] among
    /// some other criteria to determine if the translation unit must be built (ex: the first iteration)
    /// or rebuilt again (ex: the file is yet unprocessed because another translation unit failed before it)
    pub(crate) fn particular_checks_for_sent_to_build(
        compiler: CppCompiler,
        cached_source_cmd: &SourceCommandLine,
    ) -> bool {
        if compiler.eq(&CppCompiler::CLANG) && cfg!(target_os = "windows") {
            // TODO: check on a Linux distro
            // that our cache doesn't collide with the clang modules cache, or just skip clang's cache
            // with a cmd arg if possible
            log::trace!("Module unit {:?} will be rebuilt since we've detected that you are using Clang in Windows", cached_source_cmd.path());
            return true;
        }
        false
    }

    // template factory function to set the real status of a translation unit (ScheduledToDelete) on the final tasks
    // on the cache, and set states maybe? And what more?

    /// Checks whenever a [`TranslationUnit`] has been modified on the filesystem and its changes
    /// was made *after* the last time that `Zork++` made a run.
    ///
    /// *returns: <bool>* - true if the target [`TranslationUnit`] has been modified after the last
    /// iteration, false otherwise
    pub fn translation_unit_has_changes_on_fs(
        last_process_execution: &DateTime<Utc>,
        cached_source_cmd: &SourceCommandLine,
    ) -> bool {
        let file = cached_source_cmd.path();
        let file_metadata = file.metadata();

        // If exists and was successful, let's see if has been modified after the program last iteration
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

    /// Determines which kind of [`TranslationUnitStatus`] variant must a [`SourceCommandLine`]
    /// have on every process regarding specific checks and conditions before and after sent to
    /// build
    pub(crate) fn compute_translation_unit_status(
        scl: &SourceCommandLine,
    ) -> TranslationUnitStatus {
        match scl.status {
            TranslationUnitStatus::Success | TranslationUnitStatus::Cached => {
                TranslationUnitStatus::Cached
            }
            _ => TranslationUnitStatus::PendingToBuild,
        }
    }
}
