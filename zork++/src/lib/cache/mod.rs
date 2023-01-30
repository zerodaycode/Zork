//! The implementation of the Zork++ cache, for persisting data in between process

use chrono::{DateTime, Utc};
use color_eyre::{eyre::Context, Result};
use std::{fs::File, path::Path};
use walkdir::WalkDir;

use crate::{
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
        .join("cache");

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
pub fn save(program_data: &ZorkModel<'_>, mut cache: ZorkCache) -> Result<()> {
    let cache_path = &Path::new(program_data.build.output_dir)
        .join("zork")
        .join("cache")
        .join(constants::ZORK_CACHE_FILENAME);

    cache.run_final_tasks(program_data);
    cache.last_program_execution = Utc::now();

    utils::fs::serialize_object_to_file(cache_path, &cache)
        .with_context(|| "Error saving data to the Zork++ cache")
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ZorkCache {
    pub last_program_execution: DateTime<Utc>,
    pub last_generated_commands: CachedCommands,
    pub compilers_metadata: CompilersMetadata,
}

impl ZorkCache {
    /// The tasks associated with the cache after load it from the file system
    pub fn run_tasks(&mut self, program_data: &ZorkModel<'_>) {
        let compiler = program_data.compiler.cpp_compiler;
        if cfg!(target_os = "windows") && compiler == CppCompiler::MSVC {
            self.load_msvc_metadata()
        }
        if compiler == CppCompiler::GCC {
            let i = Self::track_gcc_system_modules(program_data);
            self.compilers_metadata.gcc.system_modules.clear();
            self.compilers_metadata.gcc.system_modules.extend(i);
        }
    }

    /// Runs the tasks just before end the program and save the cache
    pub fn run_final_tasks(&mut self, program_data: &ZorkModel<'_>) {
        if program_data.compiler.cpp_compiler == CppCompiler::GCC {
            self.compilers_metadata.gcc.system_modules = program_data
                .modules
                .gcc_sys_modules
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>();
        }
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

    /// Looks for the already precompiled GCC system headers, to avoid recompiling
    /// them on every process
    fn track_gcc_system_modules<'a>(
        program_data: &'a ZorkModel<'a>,
    ) -> impl Iterator<Item = String> + 'a {
        WalkDir::new(GCC_CACHE_DIR)
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
                        .gcc_sys_modules
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

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct CachedCommands {
    compiler: CppCompiler,
    interfaces: Vec<String>,
    implementations: Vec<String>,
    main: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct CompilersMetadata {
    pub msvc: MsvcMetadata,
    pub gcc: GccMetadata,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct MsvcMetadata {
    pub dev_commands_prompt: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct GccMetadata {
    pub system_modules: Vec<String>,
}
