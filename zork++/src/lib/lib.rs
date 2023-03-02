pub mod bounds;
pub mod cache;
pub mod cli;
pub mod compiler;
pub mod config_file;
pub mod project_model;
pub mod utils;

/// The entry point for the execution of the program.
///
/// This module existence is motivated to let us run
/// integration tests for the whole operations of the program
/// without having to do fancy work about checking the
/// data sent to stdout/stderr
pub mod worker {
    use std::{fs, path::Path};

    use crate::{
        cache::{self, ZorkCache},
        cli::{
            input::{CliArgs, Command},
            output::commands::{self, autorun_generated_binary, CommandExecutionResult, Commands},
        },
        compiler::build_project,
        config_file::ZorkConfigFile,
        project_model::{compiler::CppCompiler, ZorkModel},
        utils::{
            self,
            reader::{build_model, find_config_files, ConfigFile},
            template::create_templated_project,
        },
    };
    use color_eyre::{eyre::Context, Result};

    /// The main work of the project. Runs the tasks
    /// inputted in the CLI
    pub fn run_zork(cli_args: &CliArgs, path: &Path) -> Result<()> {
        if let Command::New {
            ref name,
            git,
            compiler,
            ref template,
        } = cli_args.command
        {
            return create_templated_project(path, name, git, compiler.into(), template);
        };

        let config_files: Vec<ConfigFile> = find_config_files(path)
            .with_context(|| "We didn't found a valid Zork++ configuration file")?;
        log::trace!("Config files found: {config_files:?}");

        for config_file in config_files {
            log::debug!(
                "Launching a Zork++ work event for the configuration file: {:?}, located at: {:?}\n",
                config_file.dir_entry.file_name(),
                config_file.path
            );
            let raw_file = fs::read_to_string(config_file.path)
                .with_context(|| {
                    format!(
                        "An error happened parsing the configuration file: {:?}",
                        config_file.dir_entry.file_name()
                    )
                })
                .expect("Unexpected big error happened reading a config file");

            let config: ZorkConfigFile = toml::from_str(raw_file.as_str())
                .with_context(|| "Could not parse configuration file")?;
            let program_data = build_model(&config);
            create_output_directory(&program_data)?;

            let cache =
                cache::load(&program_data, &cli_args).with_context(|| "Unable to load the Zork++ caché")?;
            let read_only_cache = cache.clone();

            // let generated_commands =
            do_main_work_based_on_cli_input(cli_args, &program_data, cache, &read_only_cache)
                .with_context(|| {
                    format!(
                        "Failed to build the project for the config file: {:?}",
                        config_file.dir_entry.file_name()
                    )
                })?;
        }

        Ok(())
    }

    /// Helper for reduce the cyclomatic complextity of the main fn.
    ///
    /// Contains the main calls to the generation of the compilers commands lines,
    /// the calls to the process that runs those ones, the autorun the generated
    /// binaries, the tests declared for the projects...
    fn do_main_work_based_on_cli_input<'a>(
        cli_args: &'a CliArgs,
        program_data: &'a ZorkModel<'_>,
        cache: ZorkCache,
        read_only_cache: &'a ZorkCache,
    ) -> Result<CommandExecutionResult> {
        let commands: Commands;

        match cli_args.command {
            Command::Build => {
                commands = build_project(program_data, read_only_cache, false)
                    .with_context(|| "Failed to build project")?;

                commands::run_generated_commands(program_data, commands, cache)
            }
            Command::Run => {
                commands = build_project(program_data, read_only_cache, false)
                    .with_context(|| "Failed to build project")?;

                match commands::run_generated_commands(program_data, commands, cache) {
                    Ok(_) => autorun_generated_binary(
                        &program_data.compiler.cpp_compiler,
                        program_data.build.output_dir,
                        program_data.executable.executable_name
                    ),
                    Err(e) => Err(e),
                }
            }
            Command::Test => {
                commands = build_project(program_data, read_only_cache, true)
                    .with_context(|| "Failed to build project")?;

                match commands::run_generated_commands(program_data, commands, cache) {
                    Ok(_) => autorun_generated_binary(
                        &program_data.compiler.cpp_compiler,
                        program_data.build.output_dir,
                        program_data.executable.executable_name
                    ),
                    Err(e) => Err(e),
                }
            }
            _ => todo!("This branch should never be reached for now, as do not exists commands that may trigger them ")
        }
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
    fn create_output_directory(model: &ZorkModel) -> Result<()> {
        let out_dir = &model.build.output_dir;
        let compiler = &model.compiler.cpp_compiler;

        // Recursively create a directory and all of its parent components if they are missing
        let modules_path = Path::new(out_dir)
            .join(compiler.to_string())
            .join("modules");
        let zork_path = out_dir.join("zork");
        let zork_cache_path = zork_path.join("cache");
        let zork_intrinsics_path = zork_path.join("intrinsics");

        utils::fs::create_directory(&modules_path.join("interfaces"))?;
        utils::fs::create_directory(&modules_path.join("implementations"))?;
        utils::fs::create_directory(&zork_cache_path.join(model.compiler.cpp_compiler.as_ref()))?;
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

    #[cfg(test)]
    mod tests {
        use color_eyre::Result;
        use tempfile::tempdir;

        use crate::config_file::ZorkConfigFile;
        use crate::utils::{reader::build_model, template::resources::CONFIG_FILE};

        #[test]
        fn test_creation_directories() -> Result<()> {
            let temp = tempdir()?;

            let normalized_cfg_file = CONFIG_FILE
                .replace("<base_path>", temp.path().to_str().unwrap())
                .replace("<compiler>", "clang")
                .replace('\\', "/");
            let zcf: ZorkConfigFile = toml::from_str(&normalized_cfg_file)?;
            let model = build_model(&zcf);

            // This should create and out/ directory in the ./zork++ folder at the root of this project
            super::create_output_directory(&model)?;

            assert!(temp.path().join("out").exists());
            assert!(temp.path().join("out/zork").exists());
            assert!(temp.path().join("out/zork/cache").exists());
            assert!(temp.path().join("out/zork/intrinsics").exists());

            Ok(())
        }
    }
}
