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

use crate::domain::translation_unit::{TranslationUnit, TranslationUnitKind};
use crate::project_model::sourceset::SourceFile;
use crate::utils::constants::CACHE_FILE_EXT;
use crate::{
    cli::{
        input::CliArgs,
        output::commands::{Commands, SourceCommandLine},
    },
    project_model::{compiler::CppCompiler, ZorkModel},
    utils::{self, constants::GCC_CACHE_DIR},
};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

/// Standalone utility for load from the file system the Zork++ cache file
/// for the target [`CppCompiler`]
pub fn load<'a>(program_data: &'a ZorkModel<'_>, cli_args: &CliArgs) -> Result<ZorkCache<'a>> {
    let compiler = program_data.compiler.cpp_compiler;
    let cache_path = &program_data.build.output_dir.join("zork").join("cache");

    let cache_file_path = cache_path
        .join(compiler.as_ref())
        .with_extension(CACHE_FILE_EXT);

    // TODO: analyze if the clear cache must be performed by target and/or active cfg file(s)
    // TODO: should we just have a cache dir with the <compiler>_<cfg_file>_<target>.json or similar?
    // Or just .../<compiler>/<cfg_file>_<target>.json
    let mut cache = if !Path::new(&cache_file_path).exists() {
        File::create(&cache_file_path).with_context(|| "Error creating the cache file")?;
        helpers::initialize_default_cache(compiler, cache_file_path)?
    } else if Path::new(cache_path).exists() && cli_args.clear_cache {
        fs::remove_dir_all(cache_path).with_context(|| "Error cleaning the Zork++ cache")?;
        fs::create_dir(cache_path)
            .with_context(|| "Error creating the cache subdirectory for {compiler}")?;
        File::create(&cache_file_path)
            .with_context(|| "Error creating the cache file after cleaning the cache")?;
        helpers::initialize_default_cache(compiler, cache_file_path)?
    } else {
        log::trace!(
            "Loading Zork++ cache file for {compiler} at: {:?}",
            cache_file_path
        );
        utils::fs::load_and_deserialize(&cache_file_path)
            .with_context(|| "Error loading the Zork++ cache")?
    };

    cache
        .run_tasks(program_data)
        .with_context(|| "Error running the cache tasks")?;

    Ok(cache)
}

#[derive(Serialize, Deserialize, Default)]
pub struct ZorkCache<'a> {
    pub compiler: CppCompiler,
    pub compilers_metadata: CompilersMetadata<'a>,
    pub generated_commands: Commands,
    pub metadata: CacheMetadata,
}

impl<'a> ZorkCache<'a> {
    pub fn save(&mut self, program_data: &ZorkModel<'_>) -> Result<()> {
        self.run_final_tasks(program_data)?;
        self.metadata.last_program_execution = Utc::now();

        utils::fs::serialize_object_to_file(&self.metadata.cache_file_path, self)
            .with_context(move || "Error saving data to the Zork++ cache")
    }

    pub fn get_cmd_for_translation_unit_kind<T: TranslationUnit<'a>>(
        &mut self,
        translation_unit: &T,
        translation_unit_kind: &TranslationUnitKind,
    ) -> Option<&mut SourceCommandLine> {
        match translation_unit_kind {
            TranslationUnitKind::ModuleInterface => self.get_module_ifc_cmd(translation_unit),
            TranslationUnitKind::ModuleImplementation => self.get_module_impl_cmd(translation_unit),
            TranslationUnitKind::SourceFile => self.get_source_cmd(translation_unit),
            TranslationUnitKind::ModularStdLib => todo!(),
            TranslationUnitKind::HeaderFile => todo!(),
        }
    }

    fn get_module_ifc_cmd<T: TranslationUnit<'a>>(
        &mut self,
        module_interface_model: &T,
    ) -> Option<&mut SourceCommandLine> {
        self.generated_commands
            .interfaces
            .iter_mut()
            .find(|mi| module_interface_model.path().eq(&mi.path()))
    }

    fn get_module_impl_cmd<T: TranslationUnit<'a>>(
        &mut self,
        module_impl_model: &T,
    ) -> Option<&mut SourceCommandLine> {
        self.generated_commands
            .implementations
            .iter_mut()
            .find(|mi| *module_impl_model.path() == (*mi).path())
    }

    fn get_source_cmd<T: TranslationUnit<'a>>(
        &mut self,
        module_impl_model: &T,
    ) -> Option<&mut SourceCommandLine> {
        self.generated_commands
            .sources
            .iter_mut()
            .find(|mi| module_impl_model.path() == (*mi).path())
    }

    /// The tasks associated with the cache after load it from the file system
    pub fn run_tasks(&mut self, program_data: &'a ZorkModel<'_>) -> Result<()> {
        let compiler = program_data.compiler.cpp_compiler;
        if cfg!(target_os = "windows") && compiler == CppCompiler::MSVC {
            msvc::load_metadata(self, program_data)?
        }

        if compiler != CppCompiler::MSVC
            && helpers::user_declared_system_headers_to_build(program_data)
        {
            let i = Self::track_system_modules(program_data);
            self.compilers_metadata.system_modules.clear();
            self.compilers_metadata.system_modules.extend(i);
        }

        Ok(())
    }

    /// Runs the tasks just before end the program and save the cache
    fn run_final_tasks(&mut self, program_data: &ZorkModel<'_>) -> Result<()> {
        if program_data.project.compilation_db && self.metadata.regenerate_compilation_database {
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

    /// Method that returns the HashMap that holds the environmental variables that must be passed
    /// to the underlying shell
    pub fn get_process_env_args(&self) -> &EnvVars {
        match self.compiler {
            CppCompiler::MSVC => &self.compilers_metadata.msvc.env_vars,
            CppCompiler::CLANG => &self.compilers_metadata.clang.env_vars,
            CppCompiler::GCC => &self.compilers_metadata.gcc.env_vars,
        }
    }

    // TODO: read_only_iterator (better name) and docs pls
    pub fn get_all_commands_iter(&self) -> impl Iterator<Item = &SourceCommandLine> + Debug + '_ {
        let generated_commands = &self.generated_commands;

        generated_commands
            .cpp_stdlib
            .as_slice()
            .iter()
            .chain(generated_commands.c_compat_stdlib.as_slice().iter())
            .chain(generated_commands.interfaces.iter())
            .chain(generated_commands.implementations.iter())
            .chain(generated_commands.sources.iter())
    }

    pub fn count_total_generated_commands(&self) -> usize {
        let latest_commands = &self.generated_commands;

        latest_commands.interfaces.len()
            + latest_commands.implementations.len()
            + latest_commands.sources.len()
            // + latest_commands.pre_tasks.len()
            + 2 // the cpp_stdlib and the c_compat_stdlib
                // + 1 // TODO: the linker one? Does it supports it clangd?
    }

    /// Looks for the already precompiled `GCC` or `Clang` system headers,
    /// to avoid recompiling them on every process
    /// NOTE: This feature should be deprecated and therefore, removed from Zork++ when GCC and
    /// Clang fully implement the required procedures to build the C++ std library as a module
    fn track_system_modules<'b: 'a>(
        // TODO move it to helpers
        program_data: &'b ZorkModel<'b>,
    ) -> impl Iterator<Item = String> + 'b {
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
                    program_data // TODO: review this, since it's too late and I am just satisfying the borrow checker
                        .modules
                        .sys_modules
                        .iter()
                        .any(|sys_mod| {
                            file.file_name()
                                .to_str()
                                .unwrap()
                                .starts_with(&sys_mod.to_string())
                        })
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
pub struct CacheMetadata {
    pub process_no: i32,
    pub last_program_execution: DateTime<Utc>,
    pub cache_file_path: PathBuf,
    #[serde(skip)]
    pub regenerate_compilation_database: bool,
}

/// Type alias for the underlying key-value based collection of environmental variables
pub type EnvVars = HashMap<String, String>;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct CompilersMetadata<'a> {
    // TODO: apply the same solution: have a fat pointer or better convert them into a Union/enum?
    pub msvc: MsvcMetadata<'a>,
    pub clang: ClangMetadata,
    pub gcc: GccMetadata,
    pub system_modules: Vec<String>, // TODO: This hopefully will dissappear soon
                                     // TODO: Vec of Cow
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct MsvcMetadata<'a> {
    pub compiler_version: Option<String>,
    pub dev_commands_prompt: Option<String>,
    pub vs_stdlib_path: Option<SourceFile<'a>>, // std.ixx path for the MSVC std lib location
    pub vs_c_stdlib_path: Option<SourceFile<'a>>, // std.compat.ixx path for the MSVC std lib location
    pub stdlib_bmi_path: PathBuf, // BMI byproduct after build in it at the target out dir of
    // the user
    pub stdlib_obj_path: PathBuf, // Same for the .obj file
    // Same as the ones defined for the C++ std lib, but for the C std lib
    pub c_stdlib_bmi_path: PathBuf,
    pub c_stdlib_obj_path: PathBuf,
    // The environmental variables that will be injected to the underlying invoking shell
    pub env_vars: EnvVars,
}

impl<'a> MsvcMetadata<'_> {
    pub fn is_loaded(&'a self) -> bool {
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
    use std::borrow::Cow;
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
                file_stem: Cow::Borrowed("std"),
                extension: compiler.default_module_extension(),
            });
            msvc.vs_c_stdlib_path = Some(SourceFile {
                path: vs_stdlib_path,
                file_stem: Cow::Borrowed("std.compat"),
                extension: compiler.default_module_extension(),
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

mod helpers {
    use super::*;
    use crate::project_model::ZorkModel;
    use std::path::PathBuf;

    // TODO: this can be used also on the compiler/mod.rs
    pub(crate) fn user_declared_system_headers_to_build(program_data: &ZorkModel<'_>) -> bool {
        !program_data.modules.sys_modules.is_empty()
    }

    pub(crate) fn initialize_default_cache<'a>(
        compiler: CppCompiler,
        cache_file_path: PathBuf,
    ) -> Result<ZorkCache<'a>> {
        let default_initialized = ZorkCache {
            compiler,
            metadata: CacheMetadata {
                cache_file_path: cache_file_path.clone(),
                ..Default::default()
            },
            ..Default::default()
        };

        utils::fs::serialize_object_to_file(&cache_file_path, &default_initialized)
            .with_context(move || "Error saving data to the Zork++ cache")?;
        Ok(default_initialized)
    }
}
