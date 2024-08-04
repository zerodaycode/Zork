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
    time::Instant,
};

use crate::config_file::ZorkConfigFile;
use crate::domain::commands::command_lines::{Commands, SourceCommandLine};
use crate::domain::target::TargetIdentifier;
use crate::domain::translation_unit::{TranslationUnit, TranslationUnitKind};
use crate::project_model::sourceset::SourceFile;
use crate::utils::constants::{dir_names, error_messages};
use crate::{
    cli::input::CliArgs,
    project_model,
    project_model::{compiler::CppCompiler, ZorkModel},
    utils::{self},
};
use serde::{Deserialize, Serialize};

use crate::project_model::compiler::StdLibMode;
use crate::utils::constants;

/// Standalone utility for load from the file system the Zork++ cache file
/// for the target [`CppCompiler`]
pub fn load<'a>(
    config: &ZorkConfigFile<'a>,
    cli_args: &CliArgs,
    project_root: &Path,
) -> Result<ZorkCache<'a>> {
    let compiler: CppCompiler = config.compiler.cpp_compiler.into();
    let output_dir = Path::new(project_root).join(
        config
            .build
            .as_ref()
            .and_then(|build_attr| build_attr.output_dir)
            .unwrap_or(dir_names::DEFAULT_OUTPUT_DIR),
    );
    let cache_path = output_dir.join(constants::ZORK).join(dir_names::CACHE);

    let cache_file_path = cache_path
        .join(compiler.as_ref())
        .with_extension(constants::CACHE_FILE_EXT);

    let mut cache = if !cache_file_path.exists() {
        File::create(&cache_file_path).with_context(|| error_messages::FAILURE_LOADING_CACHE)?;
        helpers::initialize_cache(cache_path, cache_file_path, compiler, &output_dir)?
    } else if cache_path.exists() && cli_args.clear_cache {
        fs::remove_dir_all(&cache_path).with_context(|| error_messages::FAILURE_CLEANING_CACHE)?;
        fs::create_dir(&cache_path).with_context(|| {
            format!(
                "{} for: {}",
                error_messages::FAILURE_CREATING_COMPILER_CACHE_DIR,
                compiler
            )
        })?;
        File::create(&cache_file_path).with_context(|| {
            format!(
                "{} after cleaning the cache",
                error_messages::FAILURE_LOADING_CACHE
            )
        })?;
        helpers::initialize_cache(cache_path, cache_file_path, compiler, &output_dir)?
    } else {
        log::trace!(
            "Loading Zork++ cache file for {compiler} at: {:?}",
            cache_file_path
        );
        utils::fs::load_and_deserialize(&cache_file_path)
            .with_context(|| "Error loading the Zork++ cache")?
    };

    cache.metadata.process_no += 1;

    Ok(cache)
}

#[derive(Serialize, Deserialize, Default)]
pub struct ZorkCache<'a> {
    pub compilers_metadata: CompilersMetadata<'a>,
    pub generated_commands: Commands<'a>,
    pub metadata: CacheMetadata,
}

impl<'a> ZorkCache<'a> {
    pub fn save(&mut self, program_data: &ZorkModel<'_>, cli_args: &CliArgs) -> Result<()> {
        self.run_final_tasks(program_data, cli_args)?;
        self.metadata.last_program_execution = Utc::now();

        utils::fs::save_file(&self.metadata.cache_file_path, self)
            .with_context(|| error_messages::FAILURE_SAVING_CACHE)
    }

    pub fn get_cmd_for_translation_unit_kind<T: TranslationUnit<'a>>(
        &mut self,
        translation_unit: &T,
        translation_unit_kind: &TranslationUnitKind<'a>,
    ) -> Option<&mut SourceCommandLine<'a>> {
        match translation_unit_kind {
            TranslationUnitKind::ModuleInterface => self.get_module_ifc_cmd(translation_unit),
            TranslationUnitKind::ModuleImplementation => self.get_module_impl_cmd(translation_unit),
            TranslationUnitKind::SourceFile(for_target) => {
                self.get_source_cmd(translation_unit, for_target)
            }
            TranslationUnitKind::SystemHeader => self.get_system_module_cmd(translation_unit),
            TranslationUnitKind::ModularStdLib(stdlib_mode) => match stdlib_mode {
                StdLibMode::Cpp => self.get_cpp_stdlib_cmd(),
                StdLibMode::CCompat => self.get_ccompat_stdlib_cmd(),
            },
        }
    }

    fn get_module_ifc_cmd<T: TranslationUnit<'a>>(
        &mut self,
        module_interface: &T,
    ) -> Option<&mut SourceCommandLine<'a>> {
        self.generated_commands
            .modules
            .interfaces
            .iter_mut()
            .find(|cached_tu| module_interface.path().eq(&cached_tu.path()))
    }

    fn get_module_impl_cmd<T: TranslationUnit<'a>>(
        &mut self,
        module_impl: &T,
    ) -> Option<&mut SourceCommandLine<'a>> {
        self.generated_commands
            .modules
            .implementations
            .iter_mut()
            .find(|cached_tu| module_impl.path().eq(&cached_tu.path()))
    }

    fn get_source_cmd<T: TranslationUnit<'a>>(
        &mut self,
        source: &T,
        for_target: &TargetIdentifier<'a>,
    ) -> Option<&mut SourceCommandLine<'a>> {
        self.generated_commands
            .targets
            .get_mut(for_target)
            .and_then(|target| {
                target
                    .sources
                    .iter_mut()
                    .find(|cached_tu| source.path().eq(&cached_tu.path()))
            })
    }

    /// Gets the target [`SystemModule`] generated [`SourceCommandLine`] from the cache
    ///
    /// NOTE: While we don't implement the lookup of the directory of the installed system headers,
    /// we are using some tricks to matching the generated command, but is not robust
    fn get_system_module_cmd<T: TranslationUnit<'a>>(
        &mut self,
        system_module: &T,
    ) -> Option<&mut SourceCommandLine<'a>> {
        self.generated_commands
            .modules
            .system_modules
            .iter_mut()
            .find(|cached_tu| system_module.file_stem().eq(cached_tu.filename()))
    }

    pub fn get_cpp_stdlib_cmd_by_kind(
        &mut self,
        stdlib_mode: StdLibMode,
    ) -> Option<&mut SourceCommandLine<'a>> {
        match stdlib_mode {
            StdLibMode::Cpp => self.generated_commands.modules.cpp_stdlib.as_mut(),
            StdLibMode::CCompat => self.generated_commands.modules.c_compat_stdlib.as_mut(),
        }
    }

    pub fn set_cpp_stdlib_cmd_by_kind(
        &mut self,
        stdlib_mode: StdLibMode,
        cmd_line: Option<SourceCommandLine<'a>>,
    ) {
        match stdlib_mode {
            StdLibMode::Cpp => self.generated_commands.modules.cpp_stdlib = cmd_line,
            StdLibMode::CCompat => self.generated_commands.modules.c_compat_stdlib = cmd_line,
        }
    }
    fn get_cpp_stdlib_cmd(&mut self) -> Option<&mut SourceCommandLine<'a>> {
        self.generated_commands.modules.cpp_stdlib.as_mut()
    }

    fn get_ccompat_stdlib_cmd(&mut self) -> Option<&mut SourceCommandLine<'a>> {
        self.generated_commands.modules.c_compat_stdlib.as_mut()
    }

    /// The tasks associated with the cache after load it from the file system
    pub fn run_tasks(&mut self, compiler: CppCompiler, output_dir: &Path) -> Result<()> {
        if cfg!(target_os = "windows") && compiler.eq(&CppCompiler::MSVC) {
            msvc::load_metadata(self, compiler, output_dir)?
        }

        Ok(())
    }

    /// Runs the tasks just before end the program and save the cache
    fn run_final_tasks(&mut self, program_data: &ZorkModel<'_>, cli_args: &CliArgs) -> Result<()> {
        let process_removals = Instant::now();
        let deletions_on_cfg = helpers::check_user_files_removals(self, program_data, cli_args);
        log::debug!(
            "Zork++ took a total of {:?} ms on checking and process removed items",
            process_removals.elapsed().as_millis()
        );

        if self.metadata.save_project_model {
            project_model::save(program_data, self)?;
        }

        if program_data.project.compilation_db
            && (self.metadata.generate_compilation_database || deletions_on_cfg)
        {
            let compile_commands_time = Instant::now();
            compile_commands::map_generated_commands_to_compilation_db(program_data, self)?;
            log::debug!(
                "Zork++ took a total of {:?} ms on generate the compilation database",
                compile_commands_time.elapsed().as_millis()
            );
        }

        Ok(())
    }

    /// Method that returns the HashMap that holds the environmental variables that must be passed
    /// to the underlying shell
    #[inline(always)]
    pub fn get_process_env_args(&'a mut self, compiler: CppCompiler) -> &'a EnvVars {
        match compiler {
            CppCompiler::MSVC => &self.compilers_metadata.msvc.env_vars,
            CppCompiler::CLANG => &self.compilers_metadata.clang.env_vars,
            CppCompiler::GCC => &self.compilers_metadata.gcc.env_vars,
        }
    }

    /// Returns a view of borrowed data over all the generated commands for a target
    pub fn get_all_commands_iter(&self) -> impl Iterator<Item = &SourceCommandLine> + Debug + '_ {
        let generated_commands = &self.generated_commands;

        generated_commands
            .modules
            .cpp_stdlib
            .as_slice()
            .iter()
            .chain(generated_commands.modules.c_compat_stdlib.as_slice().iter())
            .chain(generated_commands.modules.interfaces.iter())
            .chain(generated_commands.modules.implementations.iter())
            .chain(
                generated_commands
                    .targets
                    .values()
                    .flat_map(|target| target.sources.iter()),
            )
    }

    /// The current integer value that is the total of commands generated for all the
    /// [`TranslationUnit`] declared in the user's configuration file, without counting the linker
    /// one for the current target
    pub fn count_total_generated_commands(&self) -> usize {
        let latest_commands = &self.generated_commands;

        latest_commands.modules.interfaces.len()
            + latest_commands.modules.implementations.len()
            + latest_commands.modules.system_modules.len()
            + 2 // the cpp_stdlib and the c_compat_stdlib
            + latest_commands.targets.values().flat_map(|target| target.sources.iter()).count()
    }
}

/// A struct for holding Zork++ internal details about its configuration, procedures or runtime
/// statuses
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct CacheMetadata {
    pub process_no: i32,
    pub last_program_execution: DateTime<Utc>,
    pub cache_file_path: PathBuf,
    pub project_model_file_path: PathBuf,
    #[serde(skip)]
    pub generate_compilation_database: bool,
    #[serde(skip)]
    pub save_project_model: bool,
}

/// Type alias for the underlying key-value based collection of environmental variables
pub type EnvVars = HashMap<String, String>;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct CompilersMetadata<'a> {
    pub msvc: MsvcMetadata<'a>,
    pub clang: ClangMetadata,
    pub gcc: GccMetadata,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct MsvcMetadata<'a> {
    pub compiler_version: Option<String>,
    pub dev_commands_prompt: Option<String>,
    pub vs_stdlib_path: SourceFile<'a>, // std.ixx path for the MSVC std lib location
    pub vs_ccompat_stdlib_path: SourceFile<'a>, // std.compat.ixx path for the MSVC std lib location
    pub stdlib_bmi_path: PathBuf,       // BMI byproduct after build in it at the target out dir of
    // the user
    pub stdlib_obj_path: PathBuf, // Same for the .obj file
    // Same as the ones defined for the C++ std lib, but for the C std lib
    pub ccompat_stdlib_bmi_path: PathBuf,
    pub ccompat_stdlib_obj_path: PathBuf,
    // The environmental variables that will be injected to the underlying invoking shell
    pub env_vars: EnvVars,
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
    use crate::cache::ZorkCache;
    use crate::project_model::compiler::CppCompiler;
    use crate::project_model::sourceset::SourceFile;
    use crate::utils;
    use crate::utils::constants::{self, dir_names};
    use crate::utils::constants::{env_vars, error_messages};
    use color_eyre::eyre::{eyre, Context, ContextCompat, OptionExt};
    use regex::Regex;
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::path::Path;

    /// If *Windows* is the current OS, and the compiler is *MSVC*, then we will try
    /// to locate the path of the `vcvars64.bat` script that will set a set of environmental
    /// variables that are required to work effortlessly with the Microsoft's compiler.
    ///
    /// After such effort, we will dump those env vars to a custom temporary file where every
    /// env var is registered there in a key-value format, so we can load it into the cache and
    /// run this process once per new cache created (cache action 1)
    pub(crate) fn load_metadata(
        cache: &mut ZorkCache,
        compiler: CppCompiler,
        output_dir: &Path,
    ) -> color_eyre::Result<()> {
        let msvc = &mut cache.compilers_metadata.msvc;

        if msvc.dev_commands_prompt.is_none() {
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
                .arg(msvc.dev_commands_prompt.as_ref().ok_or_eyre(
                    error_messages::msvc::MISSING_OR_CORRUPTED_MSVC_DEV_COMMAND_PROMPT,
                )?)
                .arg("&&")
                .arg("set")
                .output()
                .with_context(|| error_messages::msvc::FAILURE_LOADING_VS_ENV_VARS)?;

            msvc.env_vars = load_env_vars_from_cmd_output(&output.stdout)?;
            // Cloning the useful ones for quick access at call site
            msvc.compiler_version = msvc.env_vars.get(env_vars::VS_VERSION).cloned();

            // Check the existence of the VCtools
            let vctools_dir = msvc
                .env_vars
                .get(env_vars::VC_TOOLS_INSTALL_DIR)
                .with_context(|| error_messages::msvc::MISSING_VCTOOLS_DIR)?;

            let vs_stdlib_path = Path::new(vctools_dir).join(dir_names::MODULES);
            if !vs_stdlib_path.exists() {
                return Err(eyre!(error_messages::msvc::STDLIB_MODULES_NOT_FOUND));
            }

            msvc.vs_stdlib_path = SourceFile {
                path: vs_stdlib_path.clone(),
                file_stem: Cow::Borrowed("std"),
                extension: compiler.default_module_extension(),
            };
            msvc.vs_ccompat_stdlib_path = SourceFile {
                path: vs_stdlib_path,
                file_stem: Cow::Borrowed("std.compat"),
                extension: compiler.default_module_extension(),
            };
            let modular_stdlib_byproducts_path = Path::new(output_dir)
                .join(compiler.as_ref())
                .join(dir_names::MODULES)
                .join(dir_names::STD) // folder
                .join("std"); // filename

            // Saving the paths to the precompiled bmi and obj files of the MSVC std implementation
            // that will be used to reference the build of the std as a module
            msvc.stdlib_bmi_path =
                modular_stdlib_byproducts_path.with_extension(compiler.get_typical_bmi_extension());
            msvc.stdlib_obj_path =
                modular_stdlib_byproducts_path.with_extension(compiler.get_obj_file_extension());

            let c_modular_stdlib_byproducts_path = modular_stdlib_byproducts_path;
            let compat = String::from("compat.");
            msvc.ccompat_stdlib_bmi_path = c_modular_stdlib_byproducts_path
                .with_extension(compat.clone() + compiler.get_typical_bmi_extension());
            msvc.ccompat_stdlib_obj_path = c_modular_stdlib_byproducts_path
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
            let key = parts
                .next()
                .expect(error_messages::msvc::ILL_FORMED_KEY_ON_ENV_VARS_PARSING)
                .trim();

            if filter.is_match(key) {
                let value = parts.next().unwrap_or_default().trim().to_string();
                env_vars.insert(key.to_string(), value);
            }
        }

        Ok(env_vars)
    }
}

mod helpers {
    use self::utils::constants::error_messages;
    use super::*;
    use crate::domain::translation_unit::TranslationUnitStatus;
    use std::path::PathBuf;

    pub(crate) fn initialize_cache<'a>(
        cache_path: PathBuf,
        cache_file_path: PathBuf,
        compiler: CppCompiler,
        output_dir: &Path,
    ) -> Result<ZorkCache<'a>> {
        let project_model_file_path = cache_path
            .join(format!("{}_pm", compiler.as_ref()))
            .with_extension(constants::CACHE_FILE_EXT);

        let mut cache = ZorkCache {
            metadata: CacheMetadata {
                cache_file_path,
                project_model_file_path,
                ..Default::default()
            },
            ..Default::default()
        };

        cache
            .run_tasks(compiler, output_dir)
            .with_context(|| error_messages::FAILURE_LOADING_INITIAL_CACHE_DATA)?;

        Ok(cache)
    }

    /// Checks for those translation units that the process detected that must be deleted from the
    /// cache -> [`TranslationUnitStatus::ToDelete`] or if the file has been removed from the
    /// Zork++ configuration file or if it has been removed from the fs
    ///
    /// Can we only call this when we know that the user modified the ZorkCache file for the current iteration?
    pub(crate) fn check_user_files_removals(
        cache: &mut ZorkCache,
        program_data: &ZorkModel<'_>,
        _cli_args: &CliArgs,
    ) -> bool {
        remove_if_needed_from_cache_and_count_changes(
            &mut cache.generated_commands.modules.interfaces,
            &program_data.modules.interfaces,
        ) || remove_if_needed_from_cache_and_count_changes(
            &mut cache.generated_commands.modules.implementations,
            &program_data.modules.implementations,
        ) || {
            for (target_name, target_data) in cache.generated_commands.targets.iter_mut() {
                let changes = remove_if_needed_from_cache_and_count_changes(
                    &mut target_data.sources,
                    program_data
                        .targets
                        .get(target_name)
                        .unwrap()
                        .sources
                        .as_slice(),
                );
                if changes {
                    return true;
                }
            }
            return false;
        } || remove_if_needed_from_cache_and_count_changes(
            &mut cache.generated_commands.modules.system_modules,
            &program_data.modules.sys_modules,
        )
    }

    fn remove_if_needed_from_cache_and_count_changes<'a, T: TranslationUnit<'a>>(
        cached_commands: &mut Vec<SourceCommandLine>,
        user_declared_translation_units: &[T],
    ) -> bool {
        let removal_conditions = |scl: &SourceCommandLine| {
            scl.status.eq(&TranslationUnitStatus::ToDelete) || {
                let r = user_declared_translation_units
                    .iter()
                    .any(|cc| cc.path().eq(&scl.path()));

                if !r {
                    log::debug!("Found translation_unit removed from cfg: {:?}", scl);
                }
                r
            }
        };

        let total_cached_source_command_lines = cached_commands.len();
        cached_commands.retain(removal_conditions);
        // TODO: remove them also from the linker

        total_cached_source_command_lines > cached_commands.len()
    }
}
