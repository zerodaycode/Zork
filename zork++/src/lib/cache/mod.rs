//! The implementation of the Zork++ cache, for persisting data in between process

pub mod compile_commands;

use chrono::{DateTime, Utc};
use color_eyre::eyre::OptionExt;
use color_eyre::{eyre::Context, Result};
use regex::Regex;
use std::collections::HashMap;
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
            .with_context(|| "Error creating the cache subdir for {compiler}")?;
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
    commands: Commands<'_>,
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

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ZorkCache {
    pub compiler: CppCompiler,
    pub last_program_execution: DateTime<Utc>,
    pub compilers_metadata: CompilersMetadata,
    pub last_generated_commands: HashMap<PathBuf, Vec<String>>, // TODO remove it. Is only valid for translate our cache args fmt
    pub last_generated_linker_commands: HashMap<PathBuf, String>, // TODO set it with the generated_commands
    pub generated_commands: Vec<CommandsDetails>, // TODO meter todo aquí
    // TODO meter al status ::Cached un valor dentro que indica cacheado desde hace cuanto?
    // TODO meter un unique hash para el 'is_file_cached', para evitar matchear strings? Se le puede asignar un hash único desde
    // la creación del project_model => ESO implica cachear el project model, ojo, no nos pasemos de listos, que solo se le puede
    // poner a los que NO estén ya trackeados por el project model de Zork
}

// TODO la idea buena. Diferentes niveles de -c, por ejemplo -c No vuelve a generar los comandos PERO los manda a compilar
// y -cc sería un clear_cache

// ORDEN:
// 1 - Arreglar el formato de entradas, para que tenga todos los datos del json del compile_commands
// 2 - Zumbarse los últimos commands
// 3 - pre_tasks no está mal, pero mejor un named que refleje las librerías standard
// 4 - cachear el project model
// 5 - meter la última del jota-son no del tomto-ml y testear rendimiento serializando todo tipos owned

impl ZorkCache {
    /// Returns a [`Option`] of [`CommandDetails`] if the file is persisted already in the cache
    pub fn is_file_cached(&self, path: impl AsRef<Path>) -> Option<&CommandDetail> {
        let last_iteration_details = self.generated_commands.last();

        if let Some(last_iteration) = last_iteration_details {
            return last_iteration
                .interfaces
                .iter()
                .chain(last_iteration.implementations.iter())
                .chain(last_iteration.sources.iter())
                .find(|comm_det| comm_det.file_path().eq(path.as_ref()));
        }
        None
    }

    /// The tasks associated with the cache after load it from the file system
    pub fn run_tasks(&mut self, program_data: &ZorkModel<'_>) -> Result<()> {
        let compiler = program_data.compiler.cpp_compiler;
        if cfg!(target_os = "windows") && compiler == CppCompiler::MSVC {
            self.load_msvc_metadata(program_data)?
        }

        if compiler != CppCompiler::MSVC {
            let i = Self::track_system_modules(program_data);
            self.compilers_metadata.system_modules.clear();
            self.compilers_metadata.system_modules.extend(i);
        }

        Ok(())
    }

    /// Runs the tasks just before end the program and save the cache
    pub fn run_final_tasks(
        &mut self,
        program_data: &ZorkModel<'_>,
        mut commands: Commands<'_>,
        test_mode: bool,
    ) -> Result<()> {
        if self.save_generated_commands(&mut commands, program_data, test_mode)
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

    fn save_generated_commands(
        &mut self,
        commands: &mut Commands<'_>,
        model: &ZorkModel,
        test_mode: bool,
    ) -> bool {
        log::debug!("Storing in the cache the last generated command lines...");
        self.compiler = commands.compiler;
        let process_no = if !self.generated_commands.is_empty() {
            self.generated_commands.last().unwrap().cached_process_num + 1
        } else {
            1
        };

        let mut commands_details = CommandsDetails {
            cached_process_num: process_no,
            generated_at: Utc::now(),
            interfaces: Vec::with_capacity(commands.interfaces.len()),
            implementations: Vec::with_capacity(commands.implementations.len()),
            sources: Vec::with_capacity(commands.sources.len()),
            pre_tasks: Vec::with_capacity(commands.pre_tasks.len()),
            main: MainCommandLineDetail::default(),
        };

        let mut are_new_commands = Vec::with_capacity(4);
        let pre_tasks_has_new_commands = self.extend_collection_of_source_file_details(
            model,
            &mut commands_details.pre_tasks,
            &mut commands.pre_tasks,
            commands.compiler,
        );
        are_new_commands.push(pre_tasks_has_new_commands);
        let interfaces_has_new_commands = self.extend_collection_of_source_file_details(
            model,
            &mut commands_details.interfaces,
            &mut commands.interfaces,
            commands.compiler,
        );
        are_new_commands.push(interfaces_has_new_commands);
        let implementations_has_new_commands = self.extend_collection_of_source_file_details(
            model,
            &mut commands_details.implementations,
            &mut commands.implementations,
            commands.compiler,
        );
        are_new_commands.push(implementations_has_new_commands);
        let sources_has_new_commands = self.extend_collection_of_source_file_details(
            model,
            &mut commands_details.sources,
            &mut commands.sources,
            commands.compiler,
        );
        are_new_commands.push(sources_has_new_commands);

        commands_details.main = MainCommandLineDetail {
            files: commands.main.sources_paths.clone(),
            execution_result: commands.main.execution_result,
            command: commands
                .main
                .args
                .iter()
                .map(|arg| arg.value.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        };

        let named_target = if test_mode { "test_main" } else { "main" };
        self.last_generated_linker_commands
            .entry(PathBuf::from(named_target))
            .and_modify(|e| {
                if !(*e).eq(&commands_details.main.command) {
                    e.clone_from(&commands_details.main.command)
                }
            })
            .or_insert(commands_details.main.command.clone());

        self.generated_commands.push(commands_details);

        are_new_commands.iter().any(|b| *b)
    }

    fn normalize_execution_result_status(
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

    fn extend_collection_of_source_file_details(
        &mut self,
        model: &ZorkModel,
        collection: &mut Vec<CommandDetail>,
        target: &mut [SourceCommandLine],
        compiler: CppCompiler,
    ) -> bool {
        let mut new_commands = false;
        collection.extend(target.iter().map(|source_command_line| {
            self.last_generated_commands
                .entry(source_command_line.path())
                .or_insert_with(|| {
                    new_commands = true;
                    let mut arguments = Vec::with_capacity(source_command_line.args.len() + 1);
                    arguments.push(compiler.get_driver(&model.compiler).to_string());
                    arguments.extend(source_command_line.args.iter().map(|e| e.value.to_string()));
                    arguments
                });
            CommandDetail {
                directory: source_command_line
                    .directory
                    .to_str()
                    .unwrap_or_default()
                    .to_string(),
                file: source_command_line.filename.clone(),
                execution_result: self.normalize_execution_result_status(source_command_line),
            }
        }));

        new_commands
    }

    /// Method that returns the HashMap that holds the enviromental variables that must be passed
    /// to the underlying shell
    pub fn get_process_env_args(&self) -> &EnvVars {
        match self.compiler {
            CppCompiler::MSVC => &self.compilers_metadata.msvc.env_vars,
            CppCompiler::CLANG => &self.compilers_metadata.clang.env_vars,
            CppCompiler::GCC => &self.compilers_metadata.gcc.env_vars,
        }
    }

    /// Looks for the already precompiled `GCC` or `Clang` system headers,
    /// to avoid recompiling them on every process
    /// NOTE: This feature should be deprecated an therefore, removed from Zork++ when GCC and
    /// Clang fully implement the required procedures to build the C++ std library as a module
    fn track_system_modules<'a>(
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
    pub execution_result: CommandExecutionResult,
}

impl CommandDetail {
    #[inline(always)]
    pub fn file_path(&self) -> PathBuf {
        Path::new(&self.directory).join(&self.file)
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
    use std::collections::HashMap;
    use std::path::Path;
    use color_eyre::eyre::{Context, OptionExt};
    use regex::Regex;
    use crate::cache::ZorkCache;
    use crate::project_model::sourceset::SourceFile;
    use crate::project_model::ZorkModel;
    use crate::utils;
    use crate::utils::constants;

    /// If Windows is the current OS, and the compiler is MSVC, then we will try
    /// to locate the path of the `vcvars64.bat` script that will set a set of environmental
    /// variables that are required to work effortlessly with the Microsoft's compiler.
    ///
    /// After such effort, we will dump those env vars to a custom temporary file where every
    /// env var is registered there in a key-value format, so we can load it into the cache and
    /// run this process once per new cache created (cache action 1)
    fn load_metadata(cache: &mut ZorkCache, program_data: &ZorkModel<'_>) -> color_eyre::Result<()> {
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

            msvc.env_vars = Self::load_env_vars_from_cmd_output(&output.stdout)?;
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
