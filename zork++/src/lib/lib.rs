extern crate core;

pub mod cache;
pub mod cli;
pub mod compiler;
pub mod config_file;
pub mod domain;
pub mod project_model;
pub mod utils;

/// The entry point for the execution of the program.
///
/// This module existence is motivated to let us run
/// integration tests for the whole operations of the program
/// without having to do fancy work about checking the
/// data sent to stdout/stderr
pub mod worker {
    use crate::config_file;
    use crate::config_file::ZorkConfigFile;
    use crate::domain::target::Target;
    use crate::project_model;
    use std::path::PathBuf;
    use std::{fs, path::Path, time::Instant};

    use crate::utils::constants::{dir_names, error_messages, ZORK};
    use crate::{
        cache::{self, ZorkCache},
        cli::{
            input::{CliArgs, Command},
            output::executors,
        },
        compiler::generate_commands,
        project_model::{compiler::CppCompiler, ZorkModel},
        utils::{
            self,
            reader::{find_config_files, ConfigFile},
            template::create_templated_project,
        },
    };
    use color_eyre::eyre::ContextCompat;
    use color_eyre::{eyre::Context, Report, Result};

    /// The main work of the project. Runs the tasks
    /// inputted in the CLI
    pub fn run_zork(cli_args: &CliArgs) -> std::result::Result<(), Report> {
        let abs_project_root = determine_absolute_path_of_the_project_root(cli_args)?;

        // If this run is just for create a new C++ project with the given Zork++ projects creation
        // by template, create it and exit
        if is_template_creation_then_create(cli_args, &abs_project_root)? {
            return Ok(());
        };

        let config_files: Vec<ConfigFile> =
            find_config_files(&abs_project_root, &cli_args.match_files)?;

        for config_file in config_files {
            let cfg_path = &config_file.path;
            log::debug!(
                "Launching a Zork++ work event for the configuration file: {:?}",
                cfg_path,
            );
            let raw_file = fs::read_to_string(cfg_path)
                .with_context(|| format!("{}: {:?}", error_messages::READ_CFG_FILE, cfg_path))?;

            let config: ZorkConfigFile<'_> = config_file::zork_cfg_from_file(raw_file.as_str())
                .with_context(|| error_messages::PARSE_CFG_FILE)?;

            create_output_directory(&config, &abs_project_root)?;

            let mut cache: ZorkCache<'_> = cache::load(&config, cli_args, &abs_project_root)?;

            let program_data: ZorkModel<'_> = load_zork_model(
                &mut cache,
                &config_file,
                config,
                cli_args,
                &abs_project_root,
            )?;
            map_model_targets_to_cache(&program_data, &mut cache);

            let generate_commands_ts = Instant::now();
            generate_commands(&program_data, &mut cache, cli_args)
                .with_context(|| error_messages::FAILURE_GENERATING_COMMANDS)?;

            log::debug!(
                "Zork++ took a total of {:?} ms on handling the generated commands",
                generate_commands_ts.elapsed().as_millis()
            );

            // Perform main work
            let cfg_result = perform_main_work(cli_args, &program_data, &mut cache, cfg_path);

            // Now save the cache
            cache.save(&program_data, cli_args)?;

            // Handle the errors after ensure that the cache is saved (if it didn't failed)
            if cfg_result.is_err() {
                log::error!("Failed to complete the job for: {:?}", cfg_path);
                cfg_result?
            }
        }

        Ok(())
    }

    /// Inspects the [`CliArgs`] main passed argument, and if it's [`Command::New`] just creates a
    /// new *C++* project at the *abs_project_root* and exits
    fn is_template_creation_then_create(
        cli_args: &CliArgs,
        abs_project_root: &Path,
    ) -> Result<bool> {
        if let Command::New {
            ref name,
            git,
            compiler,
            template,
        } = cli_args.command
        {
            create_templated_project(abs_project_root, name, git, compiler.into(), template)?;
            return Ok(true);
        };
        Ok(false)
    }

    fn perform_main_work(
        cli_args: &CliArgs,
        program_data: &ZorkModel<'_>,
        cache: &mut ZorkCache<'_>,
        cfg_path: &Path,
    ) -> Result<()> {
        do_main_work_based_on_cli_input(cli_args, program_data, cache).with_context(|| {
            format!(
                "{}: {:?}",
                error_messages::FAILED_BUILD_FOR_CFG_FILE,
                cfg_path
            )
        })
    }

    fn do_main_work_based_on_cli_input(
        cli_args: &CliArgs,
        program_data: &ZorkModel<'_>,
        cache: &mut ZorkCache<'_>,
    ) -> Result<()> {
        let generated_commands = &mut cache.generated_commands;

        let general_args = generated_commands
            .general_args
            .as_mut()
            .with_context(|| error_messages::GENERAL_ARGS_NOT_FOUND)? // TODO: remove the optionals
            // from the shared data on
            // the flyweights
            .get_args();

        let compiler_specific_shared_args = generated_commands
            .compiler_common_args
            .as_mut()
            .with_context(|| error_messages::COMPILER_SPECIFIC_COMMON_ARGS_NOT_FOUND)?
            .get_args();

        let env_vars = match program_data.compiler.cpp_compiler {
            CppCompiler::MSVC => &cache.compilers_metadata.msvc.env_vars,
            CppCompiler::CLANG => &cache.compilers_metadata.clang.env_vars,
            CppCompiler::GCC => &cache.compilers_metadata.gcc.env_vars,
        };

        let modules_time = Instant::now();
        executors::run_modules_generated_commands(
            program_data,
            &general_args,
            &compiler_specific_shared_args,
            &mut generated_commands.modules,
            env_vars,
        )?;
        log::debug!(
            "Took {:?} in analyze and run the generated modules commands",
            modules_time.elapsed()
        );

        match cli_args.command {
            Command::Build => executors::run_targets_generated_commands(
                program_data,
                &general_args,
                &compiler_specific_shared_args,
                &mut generated_commands.targets,
                &generated_commands.modules,
                env_vars,
            ), // TODO: group the duplicated calls

            Command::Run | Command::Test => {
                let rgtct = Instant::now();
                let rgtc = executors::run_targets_generated_commands(
                    program_data,
                    &general_args,
                    &compiler_specific_shared_args,
                    &mut generated_commands.targets,
                    &generated_commands.modules,
                    env_vars,
                );
                log::debug!(
                    "Took {:?} in analyze and run the current target",
                    rgtct.elapsed()
                );

                match rgtc {
                    Ok(_) => {
                        for target_name in generated_commands.targets.keys() {
                            executors::autorun_generated_binary(
                                &program_data.compiler.cpp_compiler,
                                &program_data.build.output_dir,
                                target_name.name(),
                            )?
                        }

                        return Ok(());
                    }
                    Err(e) => Err(e),
                }?
            }
            _ => todo!("{}", error_messages::CLI_ARGS_CMD_NEW_BRANCH),
        }
    }

    /// Resolves the full path of the location of the project's root on the fs. If the `--root`
    /// [`CliArgs`] arg is present, it will be used as the project root path, otherwise, we will
    /// assume that the project root is exactly in the same directory from where the *Zork++*
    /// binary was invoked by the user
    fn determine_absolute_path_of_the_project_root(cli_args: &CliArgs) -> Result<PathBuf> {
        let project_root = cli_args
            .root
            .as_deref()
            .map(Path::new)
            .unwrap_or(Path::new("."));

        utils::fs::get_project_root_absolute_path(project_root)
            .with_context(|| error_messages::FAILURE_GATHERING_PROJECT_ROOT_ABS_PATH)
    }

    /// Helper function to load the data of a concrete [`ZorkConfigFile`] into a [`ZorkModel`],
    /// which is the ultimate data structure that holds the read only information about the user
    /// input in a more concise way that the config file struct.
    fn load_zork_model<'a>(
        cache: &mut ZorkCache<'a>,
        meta_config_file: &ConfigFile,
        zork_config_file: ZorkConfigFile<'a>,
        cli_args: &'a CliArgs,
        abs_project_root: &Path,
    ) -> Result<ZorkModel<'a>> {
        if meta_config_file.last_time_modified > cache.metadata.last_program_execution {
            cache.metadata.save_project_model = true;
            utils::reader::build_model(zork_config_file, cli_args, abs_project_root)
        } else {
            log::debug!("Loading the ZorkModel from the cache");
            project_model::load(cache)
        }
    }

    /// Helper to map the user declared targets on the [`ZorkModel`], previously mapped from the
    /// [`ZorkConfigFile`] into the [`ZorkCache`], in order to avoid later calls with entry or insert
    /// which will be hidden on the code an harder to read for newcomers or after time without
    /// reading the codebase
    fn map_model_targets_to_cache<'a>(program_data: &ZorkModel<'a>, cache: &mut ZorkCache<'a>) {
        for (target_identifier, target_data) in program_data.targets.iter() {
            if !cache
                .generated_commands
                .targets
                .contains_key(target_identifier)
            {
                log::debug!("Adding a new target to the cache: {:?}", target_identifier);
                cache.generated_commands.targets.insert(
                    target_identifier.clone(),
                    Target::new_default_for_kind(target_data.kind),
                );
            }
        }
    }

    /// Creates the directory for output the elements generated
    /// during the build process based on the client specification.
    ///
    /// Also, it will generate the
    /// ['<output_build_dir>'/zork], which is a subfolder
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
    fn create_output_directory(config: &ZorkConfigFile, project_root: &Path) -> Result<()> {
        let compiler: CppCompiler = config.compiler.cpp_compiler.into();
        let compiler_name = compiler.as_ref();
        let binding = config
            .build
            .as_ref()
            .and_then(|build_attr| build_attr.output_dir)
            .unwrap_or("out");
        let out_dir = Path::new(project_root).join(binding);

        if out_dir.exists() {
            return Ok(()); // TODO: remeber that this causes a bug
        } // early guard. If the out_dir already exists, all
          // the sub-structure must exists and be correct.
          // Otherwise, a full out dir wipe will be preferable
          // that checking if they all exists on every run

        // Recursively create the directories below and all of its parent components if they are missing
        let modules_path = out_dir.join(compiler_name).join(dir_names::MODULES);

        let zork_path = out_dir.join(ZORK);
        let zork_cache_path = zork_path.join(dir_names::CACHE);
        let zork_intrinsics_path = zork_path.join(dir_names::INTRINSICS);

        utils::fs::create_directory(&out_dir.join(compiler_name).join(dir_names::OBJECT_FILES))?;

        utils::fs::create_directory(&modules_path.join(dir_names::INTERFACES))?;
        utils::fs::create_directory(&modules_path.join(dir_names::IMPLEMENTATIONS))?;
        utils::fs::create_directory(&modules_path.join(dir_names::STD))?;

        utils::fs::create_directory(&zork_cache_path)?;
        utils::fs::create_directory(&zork_intrinsics_path)?;

        // Pre Clang-18 way
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
        use crate::project_model::compiler::CppCompiler;
        use crate::utils::template::resources::CONFIG_FILE;
        use color_eyre::Result;
        use tempfile::tempdir;

        use crate::config_file::{self, ZorkConfigFile};
        use crate::utils::constants::{dir_names, ZORK};

        #[test]
        fn test_creation_directories() -> Result<()> {
            let temp = tempdir()?;
            let temp_path = temp.path();
            let out_dir = temp_path.join(dir_names::DEFAULT_OUTPUT_DIR);

            let zork_dir = out_dir.join(ZORK);

            let normalized_cfg_file = CONFIG_FILE
                .replace("<compiler>", "clang")
                .replace("<std_lib>", "LIBCPP")
                .replace('\\', "/");
            let zcf: ZorkConfigFile = config_file::zork_cfg_from_file(&normalized_cfg_file)?;

            let compiler: CppCompiler = zcf.compiler.cpp_compiler.into();
            let compiler_folder_dir = out_dir.join(compiler.as_ref());
            let modules_path = compiler_folder_dir.join("modules");

            // This should create and out/ directory at the root of the tmp path
            super::create_output_directory(&zcf, temp_path)?;

            assert!(out_dir.exists());

            assert!(compiler_folder_dir.exists());

            assert!(compiler_folder_dir.join(dir_names::OBJECT_FILES).exists());
            assert!(modules_path.exists());

            assert!(modules_path.join(dir_names::INTERFACES).exists());
            assert!(modules_path.join(dir_names::IMPLEMENTATIONS).exists());
            assert!(modules_path.join(dir_names::STD).exists());

            assert!(zork_dir.exists());
            assert!(zork_dir.join(dir_names::CACHE).exists());
            assert!(zork_dir.join(dir_names::INTRINSICS).exists());

            Ok(())
        }
    }
}
