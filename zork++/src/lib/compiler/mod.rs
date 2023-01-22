//! The crate responsable for executing the core work of `Zork++`,
// generate command lines and execute them in a shell of the current
// operating system against the designed compilers in the configuration
// file.
mod commands;

use color_eyre::{eyre::Context, Result};
use std::path::Path;

use crate::{
    cli::CliArgs,
    compiler::commands::Commands,
    config_file::ZorkConfigFile,
    project_model::{
        arguments::Argument,
        compiler::CppCompiler,
        modules::{ModuleImplementationModel, ModuleInterfaceModel},
        ZorkModel,
    },
    utils::{
        self,
        reader::{build_model, find_config_file},
    },
};

use self::commands::execute_command;

/// The entry point of the compilation process
///
/// Whenever this process gets triggered, the files declared within the
/// configuration file will be build
pub fn build_project(base_path: &Path, _cli_args: &CliArgs) -> Result<()> {
    let config_file: String =
        find_config_file(base_path).with_context(|| "Failed to read configuration file")?;
    let config: ZorkConfigFile = toml::from_str(config_file.as_str())
        .with_context(|| "Could not parse configuration file")?;

    let model = build_model(&config);

    // A registry of the generated command lines
    let mut commands = Commands::new(&model.compiler.cpp_compiler);

    // Create the directory for dump the generated files
    create_output_directory(base_path, &model)?;

    if model.compiler.cpp_compiler == CppCompiler::GCC {
        // Special GCC case
        helpers::process_gcc_system_modules(&model, &mut commands)
    }

    // 1st - Build the modules
    build_modules(&model, &mut commands)?;
    // 2st - Build the executable or the tests
    build_executable(&model, &mut commands)?;

    Ok(())
}

/// Triggers the build process for compile the source files declared for the project
/// and the
fn build_executable<'a>(
    model: &'a ZorkModel,
    commands: &'a mut Commands<'a>,
) -> Result<Vec<Argument<'a>>> {
    let mut args = Vec::new();

    args.extend(sources::generate_main_command_line_args(
        model,
        commands,
        &model.executable,
    )?);

    log::info!("Command for the binary: {:?}", args);
    execute_command(&model.compiler.cpp_compiler, &args)?;

    Ok(args)
}

/// Triggers the build process for compile the declared modules in the project
///
/// This function acts like a operation result processor, by running instances
/// and parsing the obtained result, handling the flux according to the
/// compiler responses>
fn build_modules<'a>(model: &'a ZorkModel, commands: &mut Commands<'a>) -> Result<()> {
    // TODO Dev todo's!
    // Change the string types for strong types (ie, unit structs with strong typing)
    // Also, can we check first is modules and interfaces .is_some() and then lauch this process?
    log::info!("\n\nBuilding the module interfaces");
    prebuild_module_interfaces(model, &model.modules.interfaces, commands);

    for miu in &commands.interfaces {
        execute_command(commands.compiler, miu)?
    }

    log::info!("\n\nBuilding the module implementations");
    compile_module_implementations(model, &model.modules.implementations, commands);

    for impls in &commands.implementations {
        execute_command(commands.compiler, impls)?
    }

    Ok(())
}

/// Parses the configuration in order to build the BMIs declared for the project,
/// by precompiling the module interface units
fn prebuild_module_interfaces<'a>(
    model: &'a ZorkModel,
    interfaces: &'a [ModuleInterfaceModel],
    commands: &mut Commands<'a>,
) {
    interfaces.iter().for_each(|module_interface| {
        sources::generate_module_interfaces_args(model, module_interface, commands);
    });
}

/// Parses the configuration in order to compile the module implementation
/// translation units declared for the project
fn compile_module_implementations<'a>(
    model: &'a ZorkModel,
    impls: &'a [ModuleImplementationModel],
    commands: &mut Commands<'a>,
) {
    impls.iter().for_each(|module_impl| {
        sources::generate_module_implementation_args(model, module_impl, commands);
    });
}

/// Creates the directory for output the elements generated
/// during the build process. Also, it will generate the
/// ['output_build_dir'/zork], which is a subfolder
/// where Zork dumps the things that needs to work correctly
/// under different conditions.
///
/// Under /zork, some new folders are created:
/// - a /intrinsics folder in created as well,
/// where different specific details of Zork++ are stored
/// related with the C++ compilers
///
/// - a /cache folder, where lives the metadata cached by Zork++
/// in order to track different aspects of the program (last time
/// modified files, last process build time...)
///  
/// TODO Generate the caché process, like last time project build,
/// and only rebuild files that is metadata contains a newer last
/// time modified date that the last Zork++ process
pub fn create_output_directory(base_path: &Path, model: &ZorkModel) -> Result<()> {
    let out_dir = &model.build.output_dir;
    let compiler = &model.compiler.cpp_compiler;

    // Recursively create a directory and all of its parent components if they are missing
    let modules_path = Path::new(base_path)
        .join(out_dir)
        .join(compiler.to_string())
        .join("modules");
    let zork_path = base_path.join(out_dir).join("zork");
    let zork_cache_path = zork_path.join("cache");
    let zork_intrinsics_path = zork_path.join("intrinsics");

    utils::fs::create_directory(&modules_path.join("interfaces"))?;
    utils::fs::create_directory(&modules_path.join("implementations"))?;
    utils::fs::create_directory(&zork_cache_path)?;
    utils::fs::create_directory(&zork_intrinsics_path)?;

    // TODO This possibly would be temporary
    if compiler.eq(&CppCompiler::CLANG) && cfg!(target_os = "windows") {
        utils::fs::create_file(
            &zork_intrinsics_path,
            "std.h",
            utils::template::resources::STD_HEADER.as_bytes(),
        )?;
        utils::fs::create_file(
            &zork_intrinsics_path,
            "zork.modulemap",
            utils::template::resources::ZORK_MODULEMAP.as_bytes(),
        )?;
    }

    Ok(())
}

/// Specific operations over source files
mod sources {
    use color_eyre::Result;

    use crate::project_model::{
        arguments::Argument,
        compiler::CppCompiler,
        modules::{ModuleImplementationModel, ModuleInterfaceModel},
        ExecutableTarget, TranslationUnit, ZorkModel,
    };

    use super::{commands::Commands, helpers};

    /// Generates the command line arguments for non-module source files, including the one that
    /// holds the main function
    pub fn generate_main_command_line_args<'a>(
        model: &'a ZorkModel,
        commands: &mut Commands<'a>,
        target: &'a impl ExecutableTarget<'a>,
    ) -> Result<Vec<Argument<'a>>> {
        log::info!("\n\nGenerating the main command line");

        let compiler = &model.compiler.cpp_compiler;
        let out_dir = model.build.output_dir;
        let executable_name = target.name();

        let mut arguments = Vec::new();
        arguments.push(model.compiler.language_level_arg());

        match compiler {
            CppCompiler::CLANG => {
                if let Some(arg) = model.compiler.stdlib_arg() {
                    arguments.push(arg);
                }
                arguments.extend_from_slice(target.extra_args());

                arguments.push(Argument::from("-fimplicit-modules"));

                if cfg!(target_os = "windows") {
                    arguments.push(Argument::from(format!(
                        "-fmodule-map-file={:?}",
                        out_dir
                            .join("zork")
                            .join("intrinsics")
                            .join("zork.modulemap")
                    )))
                } else {
                    arguments.push(Argument::from("-fimplicit-module-maps"))
                }

                arguments.push(Argument::from(format!(
                    "-fprebuilt-module-path={:?}",
                    out_dir
                        .join(compiler.as_ref())
                        .join("modules")
                        .join("interfaces")
                )));

                arguments.push(Argument::from("-o"));
                arguments.push(Argument::from(format!(
                    "{}{}",
                    out_dir
                        .join(compiler.as_ref())
                        .join(executable_name)
                        .display(),
                    if cfg!(target_os = "windows") {
                        ".exe"
                    } else {
                        ""
                    }
                )));

                arguments.extend(commands.generated_files_paths.clone().into_iter());
            }
            CppCompiler::MSVC => {
                arguments.push(Argument::from("/EHsc"));
                arguments.push(Argument::from("/nologo"));
                // If /std:c++20 this, else should be the direct options
                // available on C++23 to use directly import std by precompiling the standard library
                arguments.push(Argument::from("/experimental:module"));
                arguments.push(Argument::from("/stdIfcDir \"$(VC_IFCPath)\""));

                // helpers::add_extra_args_if_present(&config.executable, &mut arguments);
                arguments.extend_from_slice(target.extra_args());
                arguments.push(Argument::from("/ifcSearchDir"));
                arguments.push(Argument::from(
                    out_dir
                        .join(compiler.as_ref())
                        .join("modules")
                        .join("interfaces"),
                ));
                arguments.push(Argument::from(format!(
                    "/Fo{}\\",
                    out_dir.join(compiler.as_ref()).display()
                )));
                arguments.push(Argument::from(format!(
                    "/Fe{}",
                    out_dir
                        .join(compiler.as_ref())
                        .join(executable_name)
                        .with_extension("exe")
                        .display()
                )));
                arguments.extend(commands.generated_files_paths.clone().into_iter());
            }
            CppCompiler::GCC => {
                arguments.push(Argument::from("-fmodules-ts"));
                arguments.push(Argument::from("-o"));
                arguments.push(Argument::from(format!(
                    "{}{}",
                    out_dir
                        .join(compiler.as_ref())
                        .join(executable_name)
                        .display(),
                    if cfg!(target_os = "windows") {
                        ".exe"
                    } else {
                        ""
                    }
                )));
                arguments.extend(commands.generated_files_paths.clone().into_iter());
            }
        };

        target.sourceset().as_args_to(&mut arguments)?;

        Ok(arguments)
    }

    /// Generates the expected arguments for precompile the BMIs depending on self
    pub fn generate_module_interfaces_args<'a>(
        model: &'a ZorkModel,
        interface: &'a ModuleInterfaceModel,
        commands: &mut Commands<'a>,
    ) {
        let compiler = &model.compiler.cpp_compiler;
        let base_path = model.modules.base_ifcs_dir;
        let out_dir = model.build.output_dir;

        let mut arguments = Vec::with_capacity(8);
        arguments.push(model.compiler.language_level_arg());

        match *compiler {
            CppCompiler::CLANG => {
                if let Some(arg) = model.compiler.stdlib_arg() {
                    arguments.push(arg);
                }

                arguments.push(Argument::from("-fimplicit-modules"));
                arguments.push(Argument::from("-x"));
                arguments.push(Argument::from("c++-module"));
                arguments.push(Argument::from("--precompile"));

                if cfg!(target_os = "windows") {
                    arguments.push(
                        // This is a Zork++ feature to allow the users to write `import std;`
                        // under -std=c++20 with clang linking against GCC under Windows with
                        // some MinGW installation or similar.
                        // Should this be handled in another way?
                        Argument::from(format!(
                            "-fmodule-map-file={:?}",
                            out_dir
                                .join("zork")
                                .join("intrinsics")
                                .join("zork.modulemap")
                        )),
                    )
                } else {
                    arguments.push(Argument::from("-fimplicit-module-maps"))
                }

                // The resultant BMI as a .pcm file
                arguments.push(Argument::from("-o"));
                // The output file
                let miu_file_path =
                    Argument::from(helpers::generate_prebuild_miu(compiler, out_dir, interface));
                commands.generated_files_paths.push(miu_file_path.clone());
                arguments.push(miu_file_path);
                // The input file
                arguments.push(Argument::from(helpers::add_input_file(
                    interface, base_path,
                )));
            }
            CppCompiler::MSVC => {
                arguments.push(Argument::from("/EHsc"));
                arguments.push(Argument::from("/nologo"));
                arguments.push(Argument::from("/c"));
                // The output .ifc file
                arguments.push(Argument::from("/ifcOutput"));
                let miu_file_path =
                    Argument::from(helpers::generate_prebuild_miu(compiler, out_dir, interface));
                // commands.generated_files_paths.push(miu_file_path.clone()); // TODO Review this line MSVC Doesn't need to include .ifc files on the linkage
                arguments.push(miu_file_path);
                // The output .obj file
                arguments.push(Argument::from(format!(
                    "/Fo{}",
                    out_dir
                        .join(compiler.as_ref())
                        .join("modules")
                        .join("interfaces")
                        .display()
                )));
                // The input file
                arguments.push(Argument::from("/interface"));
                arguments.push(Argument::from("/TP"));
                arguments.push(Argument::from(helpers::add_input_file(
                    interface, base_path,
                )))
            }
            CppCompiler::GCC => {
                arguments.push(Argument::from("-fmodules-ts"));
                arguments.push(Argument::from("-x"));
                arguments.push(Argument::from("c++"));
                arguments.push(Argument::from("-c"));
                // The input file
                arguments.push(Argument::from(helpers::add_input_file(
                    interface, base_path,
                )));
                // The output file
                arguments.push(Argument::from("-o"));
                let miu_file_path =
                    Argument::from(helpers::generate_prebuild_miu(compiler, out_dir, interface));
                commands.generated_files_paths.push(miu_file_path.clone());
                arguments.push(miu_file_path);
            }
        }

        commands.interfaces.push(arguments);
    }

    /// Generates the expected arguments for compile the implementation module files
    pub fn generate_module_implementation_args<'a>(
        model: &'a ZorkModel,
        implementation: &'a ModuleImplementationModel,
        commands: &mut Commands<'a>,
    ) {
        let compiler = &model.compiler.cpp_compiler;
        let base_path = model.modules.base_impls_dir;
        let out_dir = model.build.output_dir;

        let mut arguments = Vec::with_capacity(8);
        arguments.push(model.compiler.language_level_arg());

        match *compiler {
            CppCompiler::CLANG => {
                if let Some(arg) = model.compiler.stdlib_arg() {
                    arguments.push(arg);
                }

                arguments.push(Argument::from("-fimplicit-modules"));
                arguments.push(Argument::from("-c"));

                if std::env::consts::OS.eq("windows") {
                    arguments.push(Argument::from(format!(
                        "-fmodule-map-file={:?}",
                        out_dir
                            .join("zork")
                            .join("intrinsics")
                            .join("zork.modulemap")
                    )))
                } else {
                    arguments.push(Argument::from("-fimplicit-module-maps"))
                }

                // The resultant object file
                arguments.push(Argument::from("-o"));
                let obj_file_path = Argument::from(helpers::generate_impl_obj_file(
                    compiler,
                    out_dir,
                    implementation,
                ));
                commands.generated_files_paths.push(obj_file_path.clone());
                arguments.push(obj_file_path);

                implementation.dependencies.iter().for_each(|ifc_dep| {
                    arguments.push(Argument::from(format!(
                        "-fmodule-file={:?}",
                        out_dir
                            .join(compiler.as_ref())
                            .join("modules")
                            .join("interfaces")
                            .join(ifc_dep)
                            .with_extension(".pcm")
                    )))
                });

                // The input file
                arguments.push(Argument::from(helpers::add_input_file(
                    implementation,
                    base_path,
                )))
            }
            CppCompiler::MSVC => {
                arguments.push(Argument::from("/EHsc"));
                arguments.push(Argument::from("/nologo"));
                arguments.push(Argument::from("-c"));
                arguments.push(Argument::from("-ifcSearchDir"));
                arguments.push(Argument::from(
                    out_dir
                        .join(compiler.as_ref())
                        .join("modules")
                        .join("interfaces"),
                ));
                // The input file
                arguments.push(Argument::from(helpers::add_input_file(
                    implementation,
                    base_path,
                )));
                // The output .obj file
                let obj_file_path = out_dir
                    .join(compiler.as_ref())
                    .join("modules")
                    .join("implementations")
                    .join(implementation.filestem())
                    .with_extension(".obj");

                commands
                    .generated_files_paths
                    .push(Argument::from(obj_file_path.clone()));
                arguments.push(Argument::from(format!("/Fo{}", obj_file_path.display())));
            }
            CppCompiler::GCC => {
                arguments.push(Argument::from("-fmodules-ts"));
                arguments.push(Argument::from("-c"));
                // The input file
                arguments.push(Argument::from(helpers::add_input_file(
                    implementation,
                    base_path,
                )));
                // The output file
                arguments.push(Argument::from("-o"));
                let obj_file_path = Argument::from(helpers::generate_impl_obj_file(
                    compiler,
                    out_dir,
                    implementation,
                ));
                commands.generated_files_paths.push(obj_file_path.clone());
                arguments.push(obj_file_path);
            }
        }

        commands.implementations.push(arguments);
    }
}

/// Helpers for reduce the cyclomatic complexity introduced by the
/// kind of workflow that should be done with this parse, format and
/// generate
mod helpers {
    use std::path::PathBuf;

    use crate::project_model::TranslationUnit;

    use super::*;

    /// Formats the string that represents an input file that will be the target of
    /// the build process and that will be passed to the compiler
    pub(crate) fn add_input_file<T: TranslationUnit>(
        translation_unit: &T,
        base_path: &Path,
    ) -> PathBuf {
        base_path.join(translation_unit.filename())
    }

    pub(crate) fn generate_prebuild_miu(
        compiler: &CppCompiler,
        out_dir: &Path,
        interface: &ModuleInterfaceModel,
    ) -> PathBuf {
        out_dir
            .join(compiler.as_ref())
            .join("modules")
            .join("interfaces")
            .join(interface.module_name)
            .with_extension(compiler.get_typical_bmi_extension())
    }

    pub(crate) fn generate_impl_obj_file(
        compiler: &CppCompiler,
        out_dir: &Path,
        implementation: &ModuleImplementationModel,
    ) -> PathBuf {
        out_dir
            .join(compiler.as_ref())
            .join("modules")
            .join("implementations")
            .join(implementation.filestem())
            .with_extension(".o")
    }

    /// GCC specific requirement. System headers as modules must be built before being imported
    pub(crate) fn process_gcc_system_modules<'a>(
        model: &'a ZorkModel,
        commands: &mut Commands<'a>,
    ) {
        let language_level = model.compiler.language_level_arg();
        let sys_modules = model.modules.gcc_sys_headers.iter().map(|sys_module| {
            vec![
                language_level.clone(),
                Argument::from("-fmodules-ts"),
                Argument::from("-x"),
                Argument::from("c++-system-header"),
                Argument::from(sys_module.to_path_buf()),
            ]
        });
        commands.interfaces.extend(sys_modules);
    }
}

#[cfg(test)]
mod tests {
    use color_eyre::Result;
    use tempfile::tempdir;

    use crate::{
        config_file::ZorkConfigFile,
        utils::{reader::build_model, template::resources::CONFIG_FILE},
    };

    use super::*;

    #[test]
    fn test_creation_directories() -> Result<()> {
        let temp = tempdir()?;

        let zcf: ZorkConfigFile = toml::from_str(CONFIG_FILE)?;
        let model = build_model(&zcf);

        // This should create and out/ directory in the ./zork++ folder at the root of this project
        create_output_directory(temp.path(), &model)?;

        assert!(temp.path().join("out").exists());
        assert!(temp.path().join("out/zork").exists());
        assert!(temp.path().join("out/zork/cache").exists());
        assert!(temp.path().join("out/zork/intrinsics").exists());

        Ok(())
    }
}
