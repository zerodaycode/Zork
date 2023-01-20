//! The crate responsable for executing the core work of `Zork++`,
// generate command lines and execute them in a shell of the current
// operating system against the designed compilers in the configuration
// file.
mod arguments;
mod commands;


use color_eyre::{eyre::Context, Result};
use std::path::Path;

use crate::{
    cli::CliArgs,
    compiler::commands::Commands,
    project_model::{
        compiler::CppCompiler,
        modules::{ModuleImplementationModel, ModuleInterfaceModel},
        ZorkModel,
    },
    utils::{self, reader::load_model},
};

use self::{arguments::Argument, commands::execute_command};

/// The entry point of the compilation process
///
/// Whenever this process gets triggered, the files declared within the
/// configuration file will be build
pub fn build_project(base_path: &Path, _cli_args: &CliArgs) -> Result<()> {
    let config = load_model(base_path).with_context(|| "Failed to load project model")?;

    // A registry of the generated command lines
    let mut commands = Commands::new(&config.compiler.cpp_compiler);

    // Create the directory for dump the generated files
    create_output_directory(base_path, &config)?;

    if config.compiler.cpp_compiler == CppCompiler::GCC { // Special GCC case
        helpers::process_gcc_system_modules(&config, &mut commands)
    }

    // 1st - Build the modules
    build_modules(&config, &mut commands)?;
    // 2st - Build the executable or the tests
    build_executable(&config, &mut commands)?;

    Ok(())
}

/// Triggers the build process for compile the source files declared for the project
/// and the
fn build_executable<'a>(
    config: &ZorkModel,
    commands: &'a mut Commands<'a>,
) -> Result<Vec<Argument<'a>>> {
    let mut args = Vec::new();

    let sources = helpers::glob_resolver(&config.executable.sources)?;

    args.extend(sources::generate_main_command_line_args(
        config, sources, commands, false,
    ));

    log::info!("Command for the binary: {:?}", args);
    execute_command(&config.compiler.cpp_compiler, &args)?;

    Ok(args)
}

/// Triggers the build process for compile the declared modules in the project
///
/// This function acts like a operation result processor, by running instances
/// and parsing the obtained result, handling the flux according to the
/// compiler responses>
fn build_modules(config: &ZorkModel, commands: &mut Commands<'_>) -> Result<()> {
    // TODO Dev todo's!
    // Change the string types for strong types (ie, unit structs with strong typing)
    // Also, can we check first is modules and interfaces .is_some() and then lauch this process?
    log::info!("\n\nBuilding the module interfaces");
    prebuild_module_interfaces(config, &config.modules.interfaces, commands);

    for miu in &commands.interfaces {
        execute_command(commands.compiler, miu)?
    }

    log::info!("\n\nBuilding the module implementations");
    compile_module_implementations(config, &config.modules.implementations, commands);

    for impls in &commands.implementations {
        execute_command(commands.compiler, impls)?
    }

    Ok(())
}

/// Parses the configuration in order to build the BMIs declared for the project,
/// by precompiling the module interface units
fn prebuild_module_interfaces(
    config: &ZorkModel,
    interfaces: &[ModuleInterfaceModel],
    commands: &mut Commands,
) {
    interfaces.iter().for_each(|module_interface| {
        sources::generate_module_interfaces_args(config, module_interface, commands);
    });
}

/// Parses the configuration in order to compile the module implementation
/// translation units declared for the project
fn compile_module_implementations(
    config: &ZorkModel,
    impls: &[ModuleImplementationModel],
    commands: &mut Commands,
) {
    impls.iter().for_each(|module_impl| {
        sources::generate_module_implementation_args(config, module_impl, commands);
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
pub fn create_output_directory(base_path: &Path, config: &ZorkModel) -> Result<()> {
    let out_dir = &config.build.output_dir;
    let compiler = &config.compiler.cpp_compiler;

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
    use crate::{project_model::{
        compiler::CppCompiler,
        modules::{ModuleImplementationModel, ModuleInterfaceModel},
        ZorkModel
    }, bounds::TranslationUnit};

    use super::{arguments::Argument, commands::Commands, helpers};

    /// Generates the command line arguments for non-module source files, including the one that
    /// holds the main function
    pub fn generate_main_command_line_args<'a>(
        config: &ZorkModel,
        sources: Vec<impl TranslationUnit>,
        commands: &'a mut Commands<'a>,
        is_tests_process: bool,
    ) -> Vec<Argument<'a>> {
        log::info!("\n\nGenerating the main command line");

        let compiler = &config.compiler.cpp_compiler;
        let (base_path, out_dir, executable_name) =
            helpers::generate_common_args_for_binary(config, is_tests_process);

        let mut arguments = Vec::new();
        arguments.push(Argument::from(config.compiler.language_level_arg()));

        match compiler {
            CppCompiler::CLANG => {
                if let Some(std_lib) = &config.compiler.std_lib {
                    arguments.push(Argument::from(format!("-stdlib={}", std_lib.as_str())))
                }
                helpers::add_extra_args(&config.executable, &mut arguments);

                arguments.push(Argument::from("-fimplicit-modules"));

                if cfg!(target_os = "windows") {
                    arguments.push(Argument::from(format!(
                        "-fmodule-map-file={out_dir}/zork/intrinsics/zork.modulemap"
                    )))
                } else {
                    arguments.push(Argument::from("-fimplicit-module-maps"))
                }

                arguments.push(Argument::from(format!(
                    "-fprebuilt-module-path={out_dir}/{compiler}/modules/interfaces"
                )));

                arguments.push(Argument::from("-o"));
                arguments.push(Argument::from(format!(
                    "{out_dir}/{compiler}/{executable_name}{}",
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
                helpers::add_extra_args(&config.executable, &mut arguments);
                arguments.push(Argument::from("/ifcSearchDir"));
                arguments.push(Argument::from(
                    format!("{out_dir}/{compiler}/modules/interfaces")
                ));
                arguments.push(Argument::from(
                    format!("/Fo{out_dir}/{compiler}\\")
                ));
                arguments.push(Argument::from(
                    format!("/Fe{out_dir}/{compiler}/{executable_name}.exe")
                ));
                arguments.extend(commands.generated_files_paths.clone().into_iter());
            },
            CppCompiler::GCC => {
                arguments.push(Argument::from("-fmodules-ts"));
                arguments.push(Argument::from("-o"));
                arguments.push(Argument::from(
                    format!(
                        "{out_dir}/{compiler}/{executable_name}{}",
                        if cfg!(target_os = "windows") {".exe"} else {""}
                    )
                ));
                arguments.extend(commands.generated_files_paths.clone().into_iter());
            },
        };

        // Adding the source files
        sources.iter().for_each(|source_file| {
            arguments.push(Argument::from(format!("{base_path}/{}", &source_file)))
        });

        arguments
    }

    /// Generates the expected arguments for precompile the BMIs depending on self
    pub fn generate_module_interfaces_args(
        config: &ZorkModel,
        interface: &ModuleInterfaceModel,
        commands: &mut Commands,
    ) {
        let compiler = &config.compiler.cpp_compiler;
        let base_path = &config.modules.base_ifcs_dir;
        let out_dir = &config.build.output_dir;

        let mut arguments = Vec::with_capacity(8);
        arguments.push(Argument::from(config.compiler.language_level_arg()));

        match *compiler {
            CppCompiler::CLANG => {
                if let Some(std_lib) = &config.compiler.std_lib {
                    arguments.push(Argument::from(format!("-stdlib={}", std_lib.as_str())))
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
                            "-fmodule-map-file={out_dir}/zork/intrinsics/zork.modulemap"
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
                let miu_file_path= Argument::from(
                    helpers::generate_prebuild_miu(compiler, out_dir, interface)
                );
                // commands.generated_files_paths.push(miu_file_path.clone()); // TODO Review this line MSVC Doesn't need to include .ifc files on the linkage
                arguments.push(miu_file_path);
                // The output .obj file
                arguments.push(Argument::from(format!(
                    "/Fo{out_dir}/{compiler}/modules/interfaces\\"
                )));
                // The input file
                arguments.push(Argument::from("/interface"));
                arguments.push(Argument::from("/TP"));
                arguments.push(Argument::from(
                    helpers::add_input_file(interface, base_path)
                ))
            },
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
    pub fn generate_module_implementation_args(
        config: &ZorkModel,
        implementation: &ModuleImplementationModel,
        commands: &mut Commands<'_>,
    ) {
        let compiler = &config.compiler.cpp_compiler;
        let base_path = &config.modules.base_impls_dir;
        let out_dir = &config.build.output_dir;

        let mut arguments = Vec::with_capacity(8);
        arguments.push(Argument::from(config.compiler.language_level_arg()));

        match *compiler {
            CppCompiler::CLANG => {
                if let Some(std_lib) = &config.compiler.std_lib {
                    arguments.push(Argument::from(format!("-stdlib={}", std_lib.as_str())))
                }

                arguments.push(Argument::from("-fimplicit-modules"));
                arguments.push(Argument::from("-c"));

                if std::env::consts::OS.eq("windows") {
                    arguments.push(Argument::from(format!(
                        "-fmodule-map-file={out_dir}/zork/intrinsics/zork.modulemap"
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
                        "-fmodule-file={out_dir}/{compiler}/modules/interfaces/{ifc_dep}.pcm"
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
                arguments.push(Argument::from(format!(
                    "{out_dir}/{compiler}/modules/interfaces/"
                )));
                // The input file
                arguments.push(Argument::from(helpers::add_input_file(
                    implementation,
                    base_path,
                )));
                // The output .obj file
                let obj_file_path = format!(
                    "{out_dir}/{compiler}/modules/implementations/{}.obj",
                    implementation.filename.split(".").collect::<Vec<_>>()[0]
                );
                commands.generated_files_paths.push(Argument::from(obj_file_path.clone()));
                arguments.push(Argument::from(format!("/Fo{obj_file_path}")));
            },
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
                let obj_file_path = Argument::from(
                    helpers::generate_impl_obj_file(compiler, out_dir, implementation)
                );
                commands.generated_files_paths.push(obj_file_path.clone());
                arguments.push(obj_file_path);
            },
        }

        commands.implementations.push(arguments);
    }
}

/// Helpers for reduce the cyclomatic complexity introduced by the
/// kind of workflow that should be done with this parse, format and
/// generate
mod helpers {
    use crate::bounds::{TranslationUnit, ExtraArgs};

    use super::*;

    /// Generates common arguments, like the base path
    pub(crate) fn generate_common_args_for_binary(
        model: &ZorkModel,
        is_tests_process: bool,
    ) -> (&String, &String, &String) {
        if !is_tests_process {
            (
                &model.executable.sources_base_path,
                &model.build.output_dir,
                &model.executable.executable_name,
            )
        } else {
            (
                &model.tests.source_base_path,
                &model.build.output_dir,
                &model.tests.test_executable_name,
            )
        }
    }

    /// Helper for resolve the wildcarded source code files. First, retrieves the wildcarded ones
    /// and second, takes the non-wildcard and joins them all in a single collection
    pub(crate) fn glob_resolver<T: TranslationUnit>(
        source_files: &[T],
    ) -> Result<Vec<impl TranslationUnit>> {
        let mut all_sources = Vec::new();

        for source_file in source_files.iter() {
            let source_file = source_file.to_string();

            if source_file.contains('*') {
                let paths = glob::glob(&source_file)
                    .with_context(|| "Failed to read configuration file")?;
                let globs = paths
                    .into_iter()
                    .map(|glob| {
                        glob.with_context(|| "Failed to retrieve the PathBuf on the process")
                            .unwrap()
                            .as_path()
                            .to_str()
                            .map_or(String::from(""), |file_name| file_name.to_string())
                    })
                    .filter(|src_file| !(*src_file).eq(""));

                all_sources.extend(globs)
            }
        }

        all_sources.extend(retrive_non_globs(source_files));

        Ok(all_sources)
    }

    /// Returns an [Iterator] holding the source files which are no wildcard values
    fn retrive_non_globs<T: TranslationUnit>(source_files: &[T]) 
        -> impl Iterator<Item = String> + '_
    {
        source_files
            .iter()
            .filter_map(|src_file| match !(src_file).to_string().contains('*') {
                true => Some(src_file.to_string()),
                false => None,
            })
    }

    /// Formats the string that represents an input file that will be the target of
    /// the build process and that will be passed to the compiler
    pub(crate) fn add_input_file<T: TranslationUnit>(
        translation_unit: &T,
        base_path: &String,
    ) -> String {
        format!("{base_path}/{}", translation_unit.get_filename())
    }

    pub(crate) fn generate_prebuild_miu(
        compiler: &CppCompiler,
        out_dir: &str,
        interface: &ModuleInterfaceModel,
    ) -> String {
        let miu_ext = compiler.get_typical_bmi_extension();
        let module_name = &interface.module_name;

        format!("{out_dir}/{compiler}/modules/interfaces/{module_name}{miu_ext}")
    }

    pub(crate) fn generate_impl_obj_file(
        compiler: &CppCompiler,
        out_dir: &str,
        implementation: &ModuleImplementationModel,
    ) -> String {
        format!(
            "{out_dir}/{compiler}/modules/implementations/{}.o",
            implementation.filename.split('.').collect::<Vec<_>>()[0]
        )
    }

    /// Extends a [`alloc::vec::Vec`] of [`Argument`] with the extra arguments
    /// declared for some property in the configuration file if they are present
    pub(crate) fn add_extra_args<'a, T>(property: &T, dst: &mut Vec<Argument<'a>>) 
        where T: ExtraArgs + 'a
    {
        let args = property
            .get_extra_args_alloc()
            .into_iter()
            .map(|v| Argument::from(v.to_owned()))
            .collect::<Vec<_>>();

        dst.extend(args.into_iter())
    }

    /// GCC specific requirement. System headers as modules must be built before being imported
    pub(crate) fn process_gcc_system_modules<'a>(
        config: &ZorkModel,
        commands: &mut Commands<'a>
    ) {
        let language_level = format!("-std=c++{}", &config.compiler.cpp_standard.as_str());
        let mut sys_modules = Vec::new();

        config.modules.gcc_sys_headers.clone().into_iter().for_each(|sys_module| {
            sys_modules.push(vec![
                Argument::from(language_level.clone()), 
                Argument::from("-fmodules-ts"), 
                Argument::from("-x"), 
                Argument::from("c++-system-header"),
                Argument::from(sys_module)   
            ]);
        });
        commands.interfaces.extend(sys_modules.into_iter());
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
