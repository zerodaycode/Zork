//! The implementation of the Zork++ cache, for persisting data in between process

pub mod compile_commands;

use chrono::{DateTime, Utc};
use color_eyre::{eyre::Context, Result};

use std::collections::HashMap;
use std::fmt::Debug;
use std::{
    fs,
    fs::File,
    path::{Path, PathBuf},
};

use crate::project_model::sourceset::SourceFile;
use crate::{
    cli::{
        input::CliArgs,
        output::commands::{CommandExecutionResult, Commands, SourceCommandLine},
    },
    project_model::{compiler::CppCompiler, ZorkModel},
    utils::{
        self,
        constants::{self, GCC_CACHE_DIR},
    },
};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

/// Standalone utility for retrieve the Zork++ cache file
pub fn load(program_data: &ZorkModel<'_>, cli_args: &CliArgs) -> Result<ZorkCache> {
    let compiler = program_data.compiler.cpp_compiler;
    let cache_path = &program_data
        .build
        .output_dir
        .join("zork")
        .join("cache")
        .join(compiler.as_ref());

    let cache_file_path = cache_path.join(constants::ZORK_CACHE_FILENAME);

    if !Path::new(&cache_file_path).exists() {
        File::create(cache_file_path).with_context(|| "Error creating the cache file")?;
    } else if Path::new(cache_path).exists() && cli_args.clear_cache {
        fs::remove_dir_all(cache_path).with_context(|| "Error cleaning the Zork++ cache")?;
        fs::create_dir(cache_path)
            .with_context(|| "Error creating the cache subdirectory for {compiler}")?;
        File::create(cache_file_path)
            .with_context(|| "Error creating the cache file after cleaning the cache")?;
    }

    let mut cache: ZorkCache = utils::fs::load_and_deserialize(&cache_path)
        .with_context(|| "Error loading the Zork++ cache")?;
    cache.compiler = compiler;

    cache
        .run_tasks(program_data)
        .with_context(|| "Error running the cache tasks")?;

    Ok(cache)
}

/// Standalone utility for persist the cache to the file system
pub fn save(
    program_data: &ZorkModel<'_>,
    cache: &mut ZorkCache,
    commands: Commands,
    test_mode: bool,
) -> Result<()> {
    let cache_path = &program_data
        .build
        .output_dir
        .join("zork")
        .join("cache")
        .join(program_data.compiler.cpp_compiler.as_ref())
        .join(constants::ZORK_CACHE_FILENAME);

    cache.run_final_tasks(program_data, commands, test_mode)?;
    cache.last_program_execution = Utc::now();

    utils::fs::serialize_object_to_file(cache_path, cache)
        .with_context(move || "Error saving data to the Zork++ cache")
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ZorkCache {
    pub compiler: CppCompiler,
    pub last_program_execution: DateTime<Utc>,
    pub compilers_metadata: CompilersMetadata,
    pub generated_commands: Vec<Commands>,
}

impl ZorkCache {
    /// Returns a [`Option`] of [`CommandDetails`] if the file is persisted already in the cache
    pub fn is_file_cached(&self, _path: impl AsRef<Path>) -> Option<&CommandDetail> {
        // let last_iteration_details = self.generated_commands.last();

        // TODO: what a cost. We need to join them for every iteration and every file
        // if let Some(last_iteration) = last_iteration_details {
        //     return last_iteration
        //         .interfaces
        //         .iter()
        //         .chain(last_iteration.implementations.iter())
        //         .chain(last_iteration.sources.iter())
        //         .find(|comm_det| comm_det.file_path().eq(path.as_ref()));
        // }
        None
    }

    /// The tasks associated with the cache after load it from the file system
    pub fn run_tasks(&mut self, program_data: &ZorkModel<'_>) -> Result<()> {
        let compiler = program_data.compiler.cpp_compiler;
        if cfg!(target_os = "windows") && compiler == CppCompiler::MSVC {
            msvc::load_metadata(self, program_data)?
        }

        if compiler != CppCompiler::MSVC {
            let i = Self::track_system_modules(program_data);
            self.compilers_metadata.system_modules.clear();
            self.compilers_metadata.system_modules.extend(i);
        }

        Ok(())
    }

    /// Runs the tasks just before end the program and save the cache
    fn run_final_tasks(
        &mut self,
        program_data: &ZorkModel<'_>,
        commands: Commands,
        test_mode: bool,
    ) -> Result<()> {
        if self.save_generated_commands(commands, program_data, test_mode)
            && program_data.project.compilation_db
        {
            compile_commands::map_generated_commands_to_compilation_db(self)?;
        }

        if !(program_data.compiler.cpp_compiler == CppCompiler::MSVC) {
            self.compilers_metadata.system_modules = program_data
                .modules
                .sys_modules
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>();
        }

        Ok(())
    }

    /// Stores the generated commands for the process in the Cache.
    /// ### Return:
    /// a boolean indicating whether there's new generated commands (non cached), so
    /// the compile commands must be regenerated
    fn save_generated_commands(
        &mut self,
        commands: Commands,
        _model: &ZorkModel,
        _test_mode: bool, // TODO: tests linker cmd?
    ) -> bool {
        log::debug!("Storing in the cache the last generated command lines...");
        self.compiler = commands.compiler;
        let _process_no = if !self.generated_commands.is_empty() {
            // TODO: do we now need this one?
            // self.generated_commands.last().unwrap().cached_process_num + 1
            0
        } else {
            1
        };

        // TODO missing the one that determines if there's a new compilation database that must be generated
        // something like and iter that counts if at least one has been modified ??
        // let at_least_one_changed = commands.
        self.generated_commands.push(commands);

        self.get_all_commands_iter()// TODO: Review the conditions and ensure that are the ones that we're looking for
            .any(|cmd| cmd.processed || cmd.execution_result.eq(&CommandExecutionResult::Success))
        
        // INSTEAD OF THIS, we just can return an Optional with the compilation database, so we can serialize the args in the compile_commands.json
        // format and then join them in a one-liner string, so they're easy to read and/or copy
    }

    fn _normalize_execution_result_status(
        // TODO: pending to re-implement it
        &self,
        module_command_line: &SourceCommandLine,
    ) -> CommandExecutionResult {
        if module_command_line
            .execution_result
            .eq(&CommandExecutionResult::Unprocessed)
        {
            if let Some(prev_entry) = self.is_file_cached(module_command_line.path()) {
                prev_entry.execution_result
            } else {
                module_command_line.execution_result
            }
        } else {
            module_command_line.execution_result
        }
    }

    /// Method that returns the HashMap that holds the environmental variables that must be passed
    /// to the underlying shell
    pub fn get_process_env_args(&self) -> &EnvVars {
        match self.compiler {
            CppCompiler::MSVC => &self.compilers_metadata.msvc.env_vars,
            CppCompiler::CLANG => &self.compilers_metadata.clang.env_vars,
            CppCompiler::GCC => &self.compilers_metadata.gcc.env_vars,
        }
    }

    // TODO:
    pub fn get_all_commands_iter(&self) -> impl Iterator<Item = &SourceCommandLine> + Debug + '_ {
        let latest = self.generated_commands.last().unwrap();
        latest
            .pre_tasks
            .iter()
            .chain(latest.interfaces.iter())
            .chain(latest.implementations.iter())
            .chain(latest.sources.iter())
    }

    pub fn count_total_generated_commands(&self) -> usize {
        let latest_commands = self.generated_commands.last().unwrap();

        latest_commands.interfaces.len()
            + latest_commands.implementations.len()
            + latest_commands.sources.len()
            + latest_commands.pre_tasks.len()
            + 1 // TODO: the linker one
    }

    /// Looks for the already precompiled `GCC` or `Clang` system headers,
    /// to avoid recompiling them on every process
    /// NOTE: This feature should be deprecated an therefore, removed from Zork++ when GCC and
    /// Clang fully implement the required procedures to build the C++ std library as a module
    fn track_system_modules<'a>(
        // TODO move it to helpers
        program_data: &'a ZorkModel<'_>,
    ) -> impl Iterator<Item = String> + 'a {
        let root = if program_data.compiler.cpp_compiler == CppCompiler::GCC {
            Path::new(GCC_CACHE_DIR).to_path_buf()
        } else {
            program_data
                .build
                .output_dir
                .join("clang")
                .join("modules")
                .join("interfaces")
        };

        WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|file| {
                if file
                    .metadata()
                    .expect("Error retrieving metadata")
                    .is_file()
                {
                    program_data
                        .modules
                        .sys_modules
                        .iter()
                        .any(|sys_mod| file.file_name().to_str().unwrap().starts_with(sys_mod))
                } else {
                    false
                }
            })
            .map(|dir_entry| {
                dir_entry
                    .file_name()
                    .to_str()
                    .unwrap()
                    .split('.')
                    .collect::<Vec<_>>()[0]
                    .to_string()
            })
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct CommandsDetails {
    cached_process_num: i32,
    generated_at: DateTime<Utc>,
    interfaces: Vec<CommandDetail>,
    implementations: Vec<CommandDetail>,
    sources: Vec<CommandDetail>,
    pre_tasks: Vec<CommandDetail>,
    main: MainCommandLineDetail,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct CommandDetail {
    directory: String,
    file: String,
    command_line: String,
    execution_result: CommandExecutionResult,
}

impl CommandDetail {
    #[inline(always)]
    pub fn file_path(&self) -> PathBuf {
        Path::new(&self.directory).join(&self.file)
    }

    #[inline]
    pub fn execution_result(&self) -> CommandExecutionResult {
        self.execution_result
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct MainCommandLineDetail {
    files: Vec<PathBuf>,
    execution_result: CommandExecutionResult,
    command: String,
}

/// Type alias for the underlying key-value based collection of environmental variables
pub type EnvVars = HashMap<String, String>;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct CompilersMetadata {
    // ALL of them must be optional, since only exists
    pub msvc: MsvcMetadata,
    pub clang: ClangMetadata,
    pub gcc: GccMetadata,
    pub system_modules: Vec<String>, // TODO: This hopefully will dissappear soon
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct MsvcMetadata {
    pub compiler_version: Option<String>,
    pub dev_commands_prompt: Option<String>,
    pub vs_stdlib_path: Option<SourceFile>, // std.ixx path for the MSVC std lib location
    pub vs_c_stdlib_path: Option<SourceFile>, // std.compat.ixx path for the MSVC std lib location
    pub stdlib_bmi_path: PathBuf, // BMI byproduct after build in it at the target out dir of
    // the user
    pub stdlib_obj_path: PathBuf, // Same for the .obj file
    // Same as the ones defined for the C++ std lib, but for the C std lib
    pub c_stdlib_bmi_path: PathBuf,
    pub c_stdlib_obj_path: PathBuf,
    // The environmental variables that will be injected to the underlying invoking shell
    pub env_vars: EnvVars,
}

impl MsvcMetadata {
    pub fn is_loaded(&self) -> bool {
        self.dev_commands_prompt.is_some() && self.vs_stdlib_path.is_some()
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ClangMetadata {
    pub env_vars: EnvVars,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct GccMetadata {
    pub env_vars: EnvVars,
}

/// Helper procedures to process cache data for Microsoft's MSVC
mod msvc {
    use crate::cache::{msvc, ZorkCache};
    use crate::project_model::sourceset::SourceFile;
    use crate::project_model::ZorkModel;
    use crate::utils;
    use crate::utils::constants;
    use color_eyre::eyre::{Context, OptionExt};
    use regex::Regex;
    use std::collections::HashMap;
    use std::path::Path;

    /// If Windows is the current OS, and the compiler is MSVC, then we will try
    /// to locate the path of the `vcvars64.bat` script that will set a set of environmental
    /// variables that are required to work effortlessly with the Microsoft's compiler.
    ///
    /// After such effort, we will dump those env vars to a custom temporary file where every
    /// env var is registered there in a key-value format, so we can load it into the cache and
    /// run this process once per new cache created (cache action 1)
    pub(crate) fn load_metadata(
        cache: &mut ZorkCache,
        program_data: &ZorkModel<'_>,
    ) -> color_eyre::Result<()> {
        let msvc = &mut cache.compilers_metadata.msvc;

        if msvc.dev_commands_prompt.is_none() {
            let compiler = program_data.compiler.cpp_compiler;

            msvc.dev_commands_prompt = utils::fs::find_file(
                Path::new(constants::MSVC_REGULAR_BASE_PATH),
                constants::MS_ENV_VARS_BAT,
            )
            .map(|walkdir_entry| {
                walkdir_entry.path().to_string_lossy().replace(
                    constants::MSVC_REGULAR_BASE_PATH,
                    constants::MSVC_REGULAR_BASE_SCAPED_PATH,
                )
            });
            let output = std::process::Command::new(constants::WIN_CMD)
                .arg("/c")
                .arg(msvc.dev_commands_prompt.as_ref().ok_or_eyre("Zork++ wasn't unable to find the VS env vars")?)
                .arg("&&")
                .arg("set")
                .output()
                .with_context(|| "Unable to load MSVC pre-requisites. Please, open an issue with the details on upstream")?;

            msvc.env_vars = msvc::load_env_vars_from_cmd_output(&output.stdout)?;
            // Cloning the useful ones for quick access at call site
            msvc.compiler_version = msvc.env_vars.get("VisualStudioVersion").cloned();

            let vs_stdlib_path =
                Path::new(msvc.env_vars.get("VCToolsInstallDir").unwrap()).join("modules");
            msvc.vs_stdlib_path = Some(SourceFile {
                path: vs_stdlib_path.clone(),
                file_stem: String::from("std"),
                extension: compiler.get_default_module_extension().to_string(),
            });
            msvc.vs_c_stdlib_path = Some(SourceFile {
                path: vs_stdlib_path,
                file_stem: String::from("std.compat"),
                extension: compiler.get_default_module_extension().to_string(),
            });
            let modular_stdlib_byproducts_path = Path::new(&program_data.build.output_dir)
                .join(compiler.as_ref())
                .join("modules")
                .join("std") // folder
                .join("std"); // filename

            // Saving the paths to the precompiled bmi and obj files of the MSVC std implementation
            // that will be used to reference the build of the std as a module
            msvc.stdlib_bmi_path =
                modular_stdlib_byproducts_path.with_extension(compiler.get_typical_bmi_extension());
            msvc.stdlib_obj_path =
                modular_stdlib_byproducts_path.with_extension(compiler.get_obj_file_extension());

            let c_modular_stdlib_byproducts_path = modular_stdlib_byproducts_path;
            let compat = String::from("compat."); // TODO: find a better way
            msvc.c_stdlib_bmi_path = c_modular_stdlib_byproducts_path
                .with_extension(compat.clone() + compiler.get_typical_bmi_extension());
            msvc.c_stdlib_obj_path = c_modular_stdlib_byproducts_path
                .with_extension(compat + compiler.get_obj_file_extension());
        }

        Ok(())
    }

    /// Convenient helper to manipulate and store the environmental variables as result of invoking
    /// the Windows `SET` cmd command
    fn load_env_vars_from_cmd_output(stdout: &[u8]) -> color_eyre::Result<HashMap<String, String>> {
        let env_vars_str = std::str::from_utf8(stdout)?;
        let filter = Regex::new(r"^[a-zA-Z_]+$").unwrap();

        let mut env_vars: HashMap<String, String> = HashMap::new();
        for line in env_vars_str.lines() {
            // Parse the key-value pair from each line
            let mut parts = line.splitn(2, '=');
            let key = parts.next().expect("Failed to get key").trim();

            if filter.is_match(key) {
                let value = parts.next().unwrap_or_default().trim().to_string();
                env_vars.insert(key.to_string(), value);
            }
        }

        Ok(env_vars)
    }
}
