//! The crate responsable for executing the core work of `Zork++`,
// generate command lines and execute them in a shell of the current
// operating system against the designed compilers in the configuration
// file.
mod commands;
mod arguments;

use color_eyre::{eyre::Context, Result};
use std::path::Path;

use crate::{
    cli::CliArgs,
    config_file::{compiler::CppCompiler, modules::{ModuleInterface, ModuleImplementation}, ZorkConfigFile},
    utils::{self, constants::DEFAULT_OUTPUT_DIR, reader::find_config_file}, compiler::commands::Commands
};

use self::{commands::execute_command, arguments::Argument};

/// The entry point of the compilation process
///
/// Whenever this process gets triggered, the files declared within the
/// configuration file will be build.
///
/// TODO Decision path for building the executable command line,
/// the tests executable command line, a static lib, a dylib...
pub fn build_project(base_path: &Path, _cli_args: &CliArgs) -> Result<()> {
    let config_file: String = find_config_file(base_path)
        .with_context(|| "Failed to read configuration file")?;
    let config: ZorkConfigFile = toml::from_str(config_file.as_str())
        .with_context(|| "Could not parse configuration file")?;

    // A registry of the generated command lines
    let mut commands = Commands::new(&config.compiler.cpp_compiler);

    // Create the directory for dump the generated files
    create_output_directory(base_path, &config)?;

    // 1st - Build the modules
    build_modules(&config, &mut commands)?;
    // 2st - Build the executable or the tests
    let bmi_and_obj_files = helpers::get_bmi_and_obj_files(
        &config.compiler.cpp_compiler, &commands.interfaces, &commands.implementations,
    );

    commands.sources = build_executable(&config, bmi_and_obj_files)?;

    Ok(())
}

/// Triggers the build process for compile the source files declared for the project
/// and the
fn build_executable<'a>(
    config: &'a ZorkConfigFile,
    bmis_and_obj_files: impl Iterator<Item = Argument<'a>>,
) -> Result<Vec<Argument<'a>>> {
    let mut r = Vec::new();
    
    if let Some(executable_attr) = &config.executable {
        if let Some(source_files) = &executable_attr.sources {
            let sources = helpers::glob_resolver(source_files)?;

            r.extend(
                sources::generate_main_command_line_args(config, sources, bmis_and_obj_files, false)
            );
            log::info!("Command for the binary: {:?}", r);
            execute_command(&config.compiler.cpp_compiler, &r)?;
        }
    }

    Ok(r)
}

/// Triggers the build process for compile the declared modules in the project
///
/// This function acts like a operation result processor, by running instances
/// and parsing the obtained result, handling the flux according to the
/// compiler responses>
fn build_modules(
    config: &ZorkConfigFile<'_>,
    commands: &mut Commands<'_>
) -> Result<()> {
    // TODO Dev todo's!
    // Change the string types for strong types (ie, unit structs with strong typing)
    // Also, can we check first is modules and interfaces .is_some() and then lauch this process?
    if let Some(modules) = &config.modules {
        if let Some(interfaces) = &modules.interfaces {
            prebuild_module_interfaces(config, interfaces, commands);

            for miu in &commands.interfaces {
                execute_command(commands.compiler, miu)?
            }
        }

        if let Some(impls) = &modules.implementations {
            compile_module_implementations(config, impls, commands);

            for impls in &commands.implementations {
                execute_command(commands.compiler, impls)?
            }
        }
    }

    Ok(())
}

/// Parses the configuration in order to build the BMIs declared for the project,
/// by precompiling the module interface units
fn prebuild_module_interfaces<'a>(
    config: &ZorkConfigFile,
    interfaces: &Vec<ModuleInterface>,
    commands: &mut Commands
) {
    interfaces.iter().for_each(|module_interface| {
        sources::generate_module_interfaces_args(config, module_interface, commands);
    });
}

/// Parses the configuration in order to compile the module implementation
/// translation units declared for the project
fn compile_module_implementations<'a>(
    config: &ZorkConfigFile,
    impls: &Vec<ModuleImplementation>,
    commands: &mut Commands
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
pub fn create_output_directory(base_path: &Path, config: &ZorkConfigFile) -> Result<()> {
    let out_dir = config
        .build
        .as_ref()
        .and_then(|build| build.output_dir)
        .unwrap_or(DEFAULT_OUTPUT_DIR);

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
    use crate::config_file::{ZorkConfigFile, modules::{ModuleInterface, ModuleImplementation}, compiler::CppCompiler, TranslationUnit};

    use super::{helpers, arguments::Argument, commands::Commands};

    /// Generates the command line arguments for non-module source files, including the one that
    /// holds the main function
    pub fn generate_main_command_line_args<'a>(
        config: &'a ZorkConfigFile<'_>,
        sources: Vec<impl TranslationUnit>,
        bmis_and_obj_files: impl Iterator<Item = Argument<'a>>,
        is_tests_process: bool
    ) -> Vec<Argument<'a>> {
        let compiler = &config.compiler.cpp_compiler;
        let (base_path, out_dir, executable_name) = 
            helpers::generate_common_args_for_binary(config, is_tests_process);

        let mut arguments = Vec::new();
        arguments.push(Argument::from(config.compiler.cpp_standard.as_cmd_arg(compiler)));

        match compiler {
            CppCompiler::CLANG => {
                arguments.push(Argument::from("-fimplicit-modules"));
                arguments.push(Argument::from(
                    format!("-fprebuilt-module-path={out_dir}/{compiler}/modules/interfaces")
                ));

                if cfg!(target_os = "windows") {
                    arguments.push(Argument::from(
                        format!("-fmodule-map-file={out_dir}/zork/intrinsics/zork.modulemap")
                    ))
                } else {
                    arguments.push(Argument::from("-fimplicit-module-maps"))
                }

                arguments.push(Argument::from("-o"));
                arguments.push(Argument::from(
                    format!(
                        "{out_dir}/{compiler}/{executable_name}{}",
                        if cfg!(target_os = "windows") {".exe"} else {""}
                    )
                ));
                
                arguments.extend(bmis_and_obj_files);
            },
            CppCompiler::MSVC => {
                arguments.push(Argument::from("/EHsc"));
                arguments.push(Argument::from("/nologo"));
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

            },
            CppCompiler::GCC => todo!(),
        };

        log::info!("Sources: {sources:?}");

        // Adding the source files
        sources.iter().for_each(|source_file| {
            log::info!("Adding source file: {source_file:?}");
            arguments.push(Argument::from(
                format!(".{base_path}/{}", &source_file)
            ))
        });

        arguments
    }

    /// Generates the expected arguments for precompile the BMIs depending on self
    pub fn generate_module_interfaces_args<'a>(
        config: &ZorkConfigFile,
        interface: &ModuleInterface,
        commands: &mut Commands
    ) {
        let compiler = &config.compiler.cpp_compiler;
        let base_path = config.modules.as_ref().map(|modules_attr|
            modules_attr.base_ifcs_dir.unwrap_or_default()
        );
        let out_dir = config.build.as_ref().map_or("", |build_attribute| {
            build_attribute.output_dir.unwrap_or_default()
        });

        let mut arguments = Vec::with_capacity(8);
        arguments.push(Argument::from(config.compiler.cpp_standard.as_cmd_arg(compiler)));

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
                        Argument::from(
                            format!("-fmodule-map-file={out_dir}/zork/intrinsics/zork.modulemap")
                        ),
                    )
                } else {
                    arguments.push(Argument::from("-fimplicit-module-maps"))
                }

                // The resultant BMI as a .pcm file
                arguments.push(Argument::from("-o"));
                // The output file
                let miu = helpers::generate_prebuild_miu(compiler, out_dir, interface);
                commands.generated_files_paths.push(miu.clone());
                arguments.push(Argument::from(miu));
                // The input file
                arguments.push(Argument::from(
                    helpers::add_input_file(interface, base_path)
                ));
            },
            CppCompiler::MSVC => {
                arguments.push(Argument::from("-EHsc"));
                arguments.push(Argument::from("-c"));
                // The output .ifc file
                arguments.push(Argument::from("-ifcOutput"));
                let miu = helpers::generate_prebuild_miu(compiler, out_dir, interface);
                commands.generated_files_paths.push(miu.clone());
                arguments.push(Argument::from(miu));
                // The output .obj file
                arguments.push(Argument::from(
                    format!("/Fo{out_dir}/{compiler}/modules/interfaces\\")
                ));
                // The input file
                arguments.push(Argument::from("-interface"));
                arguments.push(Argument::from("-TP"));
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
                arguments.push(Argument::from(
                    helpers::add_input_file(interface, base_path)
                ));
                // The output file
                arguments.push(Argument::from("-o"));
                let miu = helpers::generate_prebuild_miu(compiler, out_dir, interface);
                commands.generated_files_paths.push(miu.clone());
                arguments.push(Argument::from(miu));
            },
        }

        
        commands.interfaces.push(arguments);
    }

    /// Generates the expected arguments for compile the implementation module files
    pub fn generate_module_implementation_args<'a>(
        config: &ZorkConfigFile,
        implementation: &ModuleImplementation,
        commands: &mut Commands<'_>
    ) {
        let compiler = &config.compiler.cpp_compiler;
        let base_path = config.modules.as_ref().map(|modules_attr|
            modules_attr.base_impls_dir.unwrap_or_default()
        );
        let out_dir = config.build.as_ref().map_or("", |build_attribute| {
            build_attribute.output_dir.unwrap_or_default()
        });

        let mut arguments = Vec::with_capacity(8);
        arguments.push(Argument::from(config.compiler.cpp_standard.as_cmd_arg(compiler)));

        match *compiler {
            CppCompiler::CLANG => {
                if let Some(std_lib) = &config.compiler.std_lib {
                    arguments.push(Argument::from(format!("-stdlib={}", std_lib.as_str())))
                }

                arguments.push(Argument::from("-fimplicit-modules"));
                arguments.push(Argument::from("-c"));

                if std::env::consts::OS.eq("windows") {
                    arguments.push(Argument::from(
                        format!("-fmodule-map-file={out_dir}/zork/intrinsics/zork.modulemap")
                    ))
                } else {
                    arguments.push(Argument::from("-fimplicit-module-maps"))
                }

                // The resultant object file
                arguments.push(Argument::from("-o"));
                let obj_file = helpers::generate_impl_obj_file(compiler, out_dir, implementation);
                commands.generated_files_paths.push(helpers::generate_impl_obj_file(compiler, out_dir, implementation));
                arguments.push(Argument::from(obj_file));
                // Explicit direct module dependencies
                if let Some(ifc_dependencies) = &implementation.dependencies {
                    ifc_dependencies.iter().for_each(|ifc_dep| {
                        arguments.push(Argument::from(
                            format!("-fmodule-file={out_dir}/{compiler}/modules/interfaces/{ifc_dep}.pcm")
                        ))
                    })
                } else {
                    // If the implementation file does not declared any explicit dependency, we 
                    // assume that the unique direct dependency is it's related interface file,
                    // and that both files matches the same filename (without counting the extension)
                    arguments.push(Argument::from(
                        format!(
                            "-fmodule-file={out_dir}/{compiler}/modules/interfaces/{}.pcm",
                            implementation.filename.split(".").collect::<Vec<_>>()[0]
                        )
                    ))
                }
                // The input file
                arguments.push(Argument::from(
                    helpers::add_input_file(implementation, base_path)
                ))
            },
            CppCompiler::MSVC => {
                arguments.push(Argument::from("-EHsc"));
                arguments.push(Argument::from("-c"));
                arguments.push(Argument::from("-ifcSearchDir"));
                arguments.push(Argument::from(
                    format!("{out_dir}/{compiler}/modules/interfaces/")
                ));
                // The input file
                arguments.push(Argument::from(
                    helpers::add_input_file(implementation, base_path)
                ));
                // The output .obj file
                
                let obj_file = format!(
                    "/Fo{out_dir}/{compiler}/modules/implementations/{}",
                    implementation.filename.split(".").collect::<Vec<_>>()[0]
                );
                commands.generated_files_paths.push(obj_file.clone());
                arguments.push(Argument::from(obj_file));
            },
            CppCompiler::GCC => {
                arguments.push(Argument::from("-fmodules-ts"));
                arguments.push(Argument::from("-c"));
                // The input file
                arguments.push(Argument::from(
                    helpers::add_input_file(implementation, base_path)
                ));
                // The output file
                arguments.push(Argument::from("-o"));
                arguments.push(Argument::from(
                    helpers::generate_impl_obj_file(compiler, out_dir, implementation)
                ));
            },
        }
        
        commands.implementations.push(arguments);
    }
}

/// Helpers for reduce the cyclomatic complexity introduced by the
/// kind of workflow that should be done with this parse, format and
/// generate
mod helpers {
    use crate::config_file::TranslationUnit;

    use super::*;

    /// Generates common arguments, like the base path
    pub(crate) fn generate_common_args_for_binary(config: &ZorkConfigFile, is_tests_process: bool) -> (String, String, String) {
        if !is_tests_process {
            (
                config.executable.as_ref().map_or("", |exec_attr|
                    exec_attr.sources_base_path.unwrap_or_default()
                ).to_string(),
                config.build.as_ref().map_or("", |build_attribute|
                    build_attribute.output_dir.unwrap_or_default()
                ).to_string(),
                config.executable.as_ref().map_or("", |exec_attr| 
                    exec_attr.executable_name.unwrap_or_default()
                ).to_string()
            )
        } else {
            (
                config.tests.as_ref().map_or("", |tests_attr|
                    tests_attr.source_base_path.unwrap_or_default()
                ).to_string(),
                config.build.as_ref().map_or("", |build_attribute|
                    build_attribute.output_dir.unwrap_or_default()
                ).to_string(),
                config.tests.as_ref().map_or("", |tests_attr| 
                    tests_attr.test_executable_name.unwrap_or_default()
                ).to_string()
            )
        }
    }

    /// Helper for resolve the wildcarded source code files. First, retrieves the wildcarded ones
    /// and second, takes the non-wildcard and joins them all in a single collection
    pub(crate) fn glob_resolver<T: TranslationUnit>(source_files: &Vec<T>) -> Result<Vec<impl TranslationUnit>> {
        let mut all_sources = Vec::new();
        
        for source_file in source_files.into_iter() {
            let source_file = source_file.to_string();
            
            if source_file.contains('*') {
                let paths = glob::glob(&source_file)
                    .with_context(|| "Failed to read configuration file")?;
                let globs = paths.into_iter()
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
    fn retrive_non_globs<T: TranslationUnit>(source_files: &Vec<T>) -> impl Iterator<Item = String> + '_ {
        source_files.iter()
            .filter_map(
                |src_file| match !(src_file).to_string().contains("*") {
                    true => Some(src_file.to_string()),
                    false => None,
                }
            )
    }

    /// Formats the string that represents an input file that will be the target of
    /// the build process and that will be passed to the compiler
    pub(crate) fn add_input_file<T: TranslationUnit>(
        translation_unit: &T,
        base_path: Option<&str>
    ) -> String {
        base_path.map_or_else(
            || translation_unit.get_filename(),
            |bp| format!("{bp}/{}", translation_unit.get_filename()),
        )
    }

    pub(crate) fn generate_prebuild_miu(
        compiler: &CppCompiler,
        out_dir: &str,
        interface: &ModuleInterface,
    ) -> String {
        let miu_ext = match compiler {
            CppCompiler::CLANG => "pcm",
            CppCompiler::MSVC => "ifc",
            CppCompiler::GCC => "o",
        };

        if let Some(module_name) = interface.module_name {
            format!(
                "{out_dir}/{compiler}/modules/interfaces/{module_name}.{miu_ext}"
            )
        } else {
            format!(
                "{out_dir}/{compiler}/modules/interfaces/{}.{miu_ext}",
                interface.filename.split('.').collect::<Vec<_>>()[0]
            )
        }
    }

    pub(crate) fn generate_impl_obj_file(
        compiler: &CppCompiler,
        out_dir: &str,
        implementation: &ModuleImplementation
    ) -> String {
        format!(
            "{out_dir}/{compiler}/modules/implementations/{}.o",
            implementation.filename.split('.').collect::<Vec<_>>()[0]
        )
    }

    /// Returns a tuple with the generated prebuild module interfaces and object files 
    /// paths.
    /// Tuple.0 are BMI's paths and Tuple.1 are the generated implementation object files
    pub(crate) fn get_bmi_and_obj_files<'a>(
        compiler: &'a CppCompiler,
        ifcs_args: &'a Vec<Vec<Argument<'a>>>,
        impls_args: &'a Vec<Vec<Argument<'a>>>,
    ) -> impl Iterator<Item = Argument<'a>> {
        let target_ext = compiler.get_typical_bmi_extension();

        ifcs_args.iter().chain(impls_args)
            .flatten()
            .filter(move |cmd_arg| 
                cmd_arg.value.contains(target_ext) || cmd_arg.value.ends_with(".o") 
            )
            .map(|i| i.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use color_eyre::Result;
    use tempfile::tempdir;

    use crate::utils::template::resources::CONFIG_FILE;

    use super::*;

    #[test]
    fn test_creation_directories() -> Result<()> {
        let temp = tempdir()?;

        let zcf: ZorkConfigFile = toml::from_str(CONFIG_FILE)?;

        // This should create and out/ directory in the ./zork++ folder at the root of this project
        create_output_directory(temp.path(), &zcf)?;

        assert!(temp.path().join("out").exists());
        assert!(temp.path().join("out/zork").exists());
        assert!(temp.path().join("out/zork/cache").exists());
        assert!(temp.path().join("out/zork/intrinsics").exists());

        Ok(())
    }
}
