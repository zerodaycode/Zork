//! The implementation of the Zork++ cache, for persisting data in between process

use chrono::{DateTime, Utc};
use color_eyre::{eyre::Context, Result};
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::{
    cli::output::commands::{CommandExecutionResult, Commands},
    project_model::{compiler::CppCompiler, ZorkModel},
    utils::{
        self,
        constants::{self, GCC_CACHE_DIR},
    },
};
use serde::{Deserialize, Serialize};

/// Standalone utility for retrieve the Zork++ cache file
pub fn load(program_data: &ZorkModel<'_>) -> Result<ZorkCache> {
    let cache_path = &Path::new(program_data.build.output_dir)
        .join("zork")
        .join("cache")
        .join(program_data.compiler.cpp_compiler.as_ref());

    let cache_file_path = cache_path.join(constants::ZORK_CACHE_FILENAME);

    if !Path::new(&cache_file_path).exists() {
        File::create(cache_file_path).with_context(|| "Error creating the cache file")?;
    }

    let mut cache: ZorkCache = utils::fs::load_and_deserialize(&cache_path)
        .with_context(|| "Error loading the Zork++ cache")?;

    cache.run_tasks(program_data);

    Ok(cache)
}

/// Standalone utility for persist the cache to the file system
pub fn save(
    program_data: &ZorkModel<'_>,
    mut cache: ZorkCache,
    commands: Commands<'_>,
) -> Result<()> {
    let cache_path = &Path::new(program_data.build.output_dir)
        .join("zork")
        .join("cache")
        .join(program_data.compiler.cpp_compiler.as_ref())
        .join(constants::ZORK_CACHE_FILENAME);

    cache.run_final_tasks(program_data, commands);
    cache.last_program_execution = Utc::now();

    utils::fs::serialize_object_to_file(cache_path, &cache)
        .with_context(move || "Error saving data to the Zork++ cache")
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ZorkCache {
    pub last_program_execution: DateTime<Utc>,
    pub compilers_metadata: CompilersMetadata,
    pub generated_commands: CachedCommands,
}

impl ZorkCache {
    /// Returns a [`Option`] of [`CommandDetails`] if the file is persisted already in the cache
    pub fn is_file_cached(&self, path: &Path) -> Option<&CommandDetail> {
        let last_iteration_details = self.generated_commands.details.last();
        if let Some(last_iteration) = last_iteration_details {
            let found_as_ifc = last_iteration.interfaces.iter().find(|f| {
                path.to_str()
                    .unwrap_or_default()
                    .contains(&f.translation_unit)
            });
            let found_as_impl = last_iteration.implementations.iter().find(|f| {
                path.to_str()
                    .unwrap_or_default()
                    .contains(&f.translation_unit)
            });
            if found_as_ifc.is_some() {
                return found_as_ifc;
            }
            if found_as_impl.is_some() {
                return found_as_impl;
            }
        }
        None
    }

    /// The tasks associated with the cache after load it from the file system
    pub fn run_tasks(&mut self, program_data: &ZorkModel<'_>) {
        let compiler = program_data.compiler.cpp_compiler;
        if cfg!(target_os = "windows") && compiler == CppCompiler::MSVC {
            self.load_msvc_metadata()
        }
        if compiler != CppCompiler::MSVC {
            let i = Self::track_system_modules(program_data);
            self.compilers_metadata.system_modules.clear();
            self.compilers_metadata.system_modules.extend(i);
        }
    }

    /// Runs the tasks just before end the program and save the cache
    pub fn run_final_tasks(&mut self, program_data: &ZorkModel<'_>, commands: Commands<'_>) {
        self.save_generated_commands(commands);

        if !(program_data.compiler.cpp_compiler == CppCompiler::MSVC) {
            self.compilers_metadata.system_modules = program_data
                .modules
                .sys_modules
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>();
        }
    }

    fn save_generated_commands(&mut self, commands: Commands<'_>) {
        self.generated_commands.compiler = commands.compiler;
        let process_no = if !self.generated_commands.details.is_empty() {
            self.generated_commands
                .details
                .last()
                .unwrap()
                .cached_process_num
                + 1
        } else {
            1
        };

        let mut commands_details = CommandsDetails {
            cached_process_num: process_no,
            generated_at: Utc::now(),
            interfaces: Vec::with_capacity(commands.interfaces.len()),
            implementations: Vec::with_capacity(commands.implementations.len()),
            main: MainCommandLineDetail::default(),
        };

        commands_details
            .interfaces
            .extend(commands.interfaces.iter().map(|module_command_line| {
                let details = CommandDetail {
                    translation_unit: String::from(
                        module_command_line
                            .path
                            .as_os_str()
                            .to_str()
                            .unwrap_or_default(),
                    ),
                    execution_result: module_command_line.execution_result.clone(),
                    command: if module_command_line.processed {
                        Vec::with_capacity(0)
                    } else {
                        module_command_line
                            .args
                            .iter()
                            .map(|argument| argument.value.to_string())
                            .collect()
                    },
                };
                details
            }));

        commands_details
            .implementations
            .extend(commands.implementations.iter().map(|module_command_line| {
                let details = CommandDetail {
                    translation_unit: String::from(
                        module_command_line
                            .path
                            .as_os_str()
                            .to_str()
                            .unwrap_or_default(),
                    ),
                    execution_result: module_command_line.execution_result.clone(),
                    command: if module_command_line.processed {
                        Vec::with_capacity(0)
                    } else {
                        module_command_line
                            .args
                            .iter()
                            .map(|argument| argument.value.to_string())
                            .collect()
                    },
                };
                details
            }));

        commands_details.main = MainCommandLineDetail {
            files: commands.sources.sources_paths,
            execution_result: commands.sources.execution_result.clone(),
            command: commands
                .sources
                .args
                .iter()
                .map(|arg| arg.value.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        };

        self.generated_commands.details.push(commands_details)
    }

    /// If Windows is the current OS, and the compiler is MSVC, then we will try
    /// to locate the path os the vcvars64.bat scripts that launches the
    /// Developers Command Prompt
    fn load_msvc_metadata(&mut self) {
        if self.compilers_metadata.msvc.dev_commands_prompt.is_none() {
            self.compilers_metadata.msvc.dev_commands_prompt =
                WalkDir::new(constants::MSVC_BASE_PATH)
                    .into_iter()
                    .filter_map(Result::ok)
                    .find(|file| {
                        file.file_name()
                            .to_str()
                            .map(|filename| filename.eq(constants::MS_DEVS_PROMPT_BAT))
                            .unwrap_or(false)
                    })
                    .map(|e| e.path().display().to_string());
        }
    }

    /// Looks for the already precompiled GCC or Clang system headers, to avoid recompiling
    /// them on every process
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
pub struct CachedCommands {
    compiler: CppCompiler,
    details: Vec<CommandsDetails>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct CommandsDetails {
    cached_process_num: i32,
    generated_at: DateTime<Utc>,
    interfaces: Vec<CommandDetail>,
    implementations: Vec<CommandDetail>,
    main: MainCommandLineDetail,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct CommandDetail {
    translation_unit: String,
    pub execution_result: CommandExecutionResult,
    command: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct MainCommandLineDetail {
    files: Vec<PathBuf>,
    execution_result: CommandExecutionResult,
    command: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct CompilersMetadata {
    pub msvc: MsvcMetadata,
    pub system_modules: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct MsvcMetadata {
    pub dev_commands_prompt: Option<String>,
}
