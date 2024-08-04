use crate::cli::input::CliArgs;

use crate::config_file::target::TargetAttribute;
use crate::domain::commands::arguments::Argument;
use crate::domain::target::TargetIdentifier;
use crate::project_model::modules::SystemModule;
use crate::project_model::sourceset::SourceFile;
use crate::project_model::target::TargetModel;
use crate::{
    config_file::{
        build::BuildAttribute,
        compiler::CompilerAttribute,
        modules::{ModuleImplementation, ModuleInterface, ModulesAttribute},
        project::ProjectAttribute,
        ZorkConfigFile,
    },
    project_model::{
        build::BuildModel,
        compiler::CompilerModel,
        modules::{
            ModuleImplementationModel, ModuleInterfaceModel, ModulePartitionModel, ModulesModel,
        },
        project::ProjectModel,
        sourceset::{GlobPattern, Source, SourceSet},
        ZorkModel,
    },
    utils,
};
use chrono::{DateTime, Utc};
use color_eyre::{eyre::eyre, Result};
use indexmap::IndexMap;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::constants::dir_names;

/// Details about a found configuration file on the project
///
/// This is just a configuration file with a valid name found
/// at a valid path in some subdirectory
#[derive(Debug)]
pub struct ConfigFile {
    pub path: PathBuf,
    pub last_time_modified: DateTime<Utc>,
}

/// Checks for the existence of the `zork_<any>.toml` configuration files
/// present in the same directory when the binary is called, and
/// returns a collection of the ones found.
///
/// *base_path* - A parameter for receive an input via command line
/// parameter to indicate where the configuration files lives in
/// the client's project. Defaults to `.`
///
/// This function fails if there's no configuration file
/// (or isn't present in any directory of the project)
pub fn find_config_files(
    base_path: &Path, // TODO:(Workspaces) create the cfg arg to specifically receive where's located the
    // user's Zork config files if they're not in the root of the project
    // nor matches the tree structure from user's root cmd arg value
    filename_match: &Option<String>,
) -> Result<Vec<ConfigFile>> {
    log::debug!("Searching for Zork++ configuration files...");
    let mut files = vec![];

    for e in WalkDir::new(base_path)
        .max_depth(2) // TODO:(Workspaces) so, max_depth should be zero when the cfg arg is ready
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let filename = e.file_name().to_str().unwrap();
        let file_match = filename_match
            .as_ref()
            .map(|fm| fm.as_str())
            .unwrap_or(filename);
        if e.metadata().unwrap().is_file()
            && filename.starts_with("zork")
            && filename.ends_with(".toml")
            && filename.contains(file_match)
        {
            files.push(ConfigFile {
                path: e.path().to_path_buf(),
                last_time_modified: DateTime::<Utc>::from(e.metadata()?.modified()?),
            })
        }
    }

    if files.is_empty() {
        Err(eyre!("No configuration files found for the project"))
    } else {
        Ok(files)
    }
}

pub fn build_model<'a>(
    config: ZorkConfigFile<'a>,
    cli_args: &'a CliArgs,
    absolute_project_root: &Path,
) -> Result<ZorkModel<'a>> {
    let proj_name = config.project.name;

    let project = assemble_project_model(config.project);
    let compiler = assemble_compiler_model(config.compiler, cli_args);
    let build = assemble_build_model(config.build, absolute_project_root);

    let code_root = PathBuf::from(absolute_project_root).join(
        project
            .code_root
            .as_ref()
            .map(|pr| pr.to_string())
            .unwrap_or_default(),
    );

    let modules = assemble_modules_model(config.modules, &code_root);
    let targets = assemble_targets_model(config.targets, proj_name, &code_root);

    Ok(ZorkModel {
        project,
        compiler,
        build,
        modules,
        targets,
    })
}

fn assemble_project_model(config: ProjectAttribute) -> ProjectModel {
    ProjectModel {
        name: Cow::Borrowed(config.name),
        authors: config
            .authors
            .as_ref()
            .map_or_else(Vec::default, |authors| {
                authors
                    .iter()
                    .map(|auth| Cow::Borrowed(*auth))
                    .collect::<Vec<_>>()
            }),
        compilation_db: config.compilation_db.unwrap_or_default(),
        code_root: config.code_root.map(Cow::Borrowed),
    }
}

fn assemble_compiler_model<'a>(
    config: CompilerAttribute<'a>,
    cli_args: &'a CliArgs,
) -> CompilerModel<'a> {
    let extra_args = config
        .extra_args
        .map(|args| args.into_iter().map(Argument::from).collect())
        .unwrap_or_default();

    CompilerModel {
        cpp_compiler: config.cpp_compiler.into(),
        driver_path: if let Some(driver_path) = cli_args.driver_path.as_ref() {
            Cow::Borrowed(driver_path)
        } else {
            Cow::Owned(cli_args.driver_path.clone().unwrap_or_default())
        },
        cpp_standard: config.cpp_standard.clone().into(),
        std_lib: config.std_lib.clone().map(|lib| lib.into()),
        extra_args,
    }
}

fn assemble_build_model(config: Option<BuildAttribute>, project_root: &Path) -> BuildModel {
    let output_dir = config
        .as_ref()
        .and_then(|build| build.output_dir)
        .map(|out_dir| out_dir.strip_prefix("./").unwrap_or(out_dir))
        .unwrap_or(dir_names::DEFAULT_OUTPUT_DIR);

    BuildModel {
        output_dir: Path::new(project_root).join(output_dir),
    }
}

fn assemble_modules_model<'a>(
    config: Option<ModulesAttribute<'a>>,
    code_root: &Path,
) -> ModulesModel<'a> {
    let modules = config.unwrap_or_default();

    let base_ifcs_dir = modules
        .base_ifcs_dir
        .map(Path::new)
        .map(Cow::from)
        .unwrap_or_default();

    let interfaces = modules
        .interfaces
        .map(|ifcs| {
            ifcs.into_iter()
                .map(|m_ifc| -> ModuleInterfaceModel<'_> {
                    assemble_module_interface_model(m_ifc, &base_ifcs_dir, code_root)
                })
                .collect()
        })
        .unwrap_or_default();

    let base_impls_dir = modules
        .base_impls_dir
        .map(Path::new)
        .map(Cow::from)
        .unwrap_or_default();

    let implementations = modules
        .implementations
        .map(|impls| {
            impls
                .into_iter()
                .map(|m_impl| {
                    assemble_module_implementation_model(m_impl, &base_impls_dir, code_root)
                })
                .collect()
        })
        .unwrap_or_default();

    let sys_modules = modules
        .sys_modules
        .as_ref()
        .map_or_else(Default::default, |headers| {
            headers
                .iter()
                .map(|sys_header| SystemModule {
                    file_stem: Cow::from(*sys_header),
                    ..Default::default()
                })
                .collect()
        });

    ModulesModel {
        base_ifcs_dir,
        interfaces,
        base_impls_dir,
        implementations,
        sys_modules,
    }
}

fn assemble_module_interface_model<'a>(
    config: ModuleInterface<'a>,
    base_path: &Path,
    code_root: &Path,
) -> ModuleInterfaceModel<'a> {
    let cfg_file = config.file;

    let file_path = Path::new(code_root).join(base_path).join(cfg_file);

    let module_name = if let Some(mod_name) = config.module_name {
        Cow::Borrowed(mod_name)
    } else {
        Path::new(cfg_file)
            .file_stem()
            .unwrap_or_else(|| panic!("Found ill-formed file_stem data for: {cfg_file}"))
            .to_string_lossy()
    };
    let dependencies = config
        .dependencies
        .map(|deps| deps.into_iter().map(Cow::Borrowed).collect())
        .unwrap_or_default();
    let partition = config.partition.map(ModulePartitionModel::from);

    let file_details = utils::fs::get_file_details(&file_path).unwrap_or_else(|_| {
        panic!("An unexpected error happened getting the file details for {file_path:?}")
    });

    ModuleInterfaceModel {
        path: file_details.0,
        file_stem: Cow::from(file_details.1),
        extension: Cow::from(file_details.2),
        module_name,
        partition,
        dependencies,
    }
}

fn assemble_module_implementation_model<'a>(
    config: ModuleImplementation<'a>,
    base_path: &Path,
    code_root: &Path,
) -> ModuleImplementationModel<'a> {
    let mut dependencies = config
        .dependencies
        .unwrap_or_default()
        .into_iter()
        .map(Cow::Borrowed)
        .collect::<Vec<Cow<str>>>();

    let file_path = Path::new(code_root).join(base_path).join(config.file);

    if dependencies.is_empty() {
        let last_dot_index = config.file.rfind('.');
        if let Some(idx) = last_dot_index {
            let implicit_dependency = config.file.split_at(idx);
            dependencies.push(Cow::Owned(implicit_dependency.0.to_owned()))
        } else {
            dependencies.push(Cow::Borrowed(config.file));
        }
    }

    let file_details = utils::fs::get_file_details(&file_path).unwrap_or_else(|_| {
        panic!("An unexpected error happened getting the file details for {file_path:?}")
    });

    ModuleImplementationModel {
        path: file_details.0,
        file_stem: Cow::Owned(file_details.1),
        extension: Cow::Owned(file_details.2),
        dependencies,
    }
}

fn assemble_targets_model<'a>(
    targets: IndexMap<&'a str, TargetAttribute<'a>>,
    project_name: &'a str,
    code_root: &Path,
) -> IndexMap<TargetIdentifier<'a>, TargetModel<'a>> {
    targets
        .into_iter()
        .map(|(k, v)| {
            (
                TargetIdentifier(Cow::Borrowed(k)),
                assemble_target_model(v, project_name, code_root),
            )
        })
        .collect()
}

fn assemble_target_model<'a>(
    target_config: TargetAttribute<'a>,
    project_name: &'a str,
    code_root: &Path,
) -> TargetModel<'a> {
    let sources = target_config
        .sources
        .into_iter()
        .map(Cow::Borrowed)
        .collect();

    let sources = get_sources_for_target(sources, code_root);

    let extra_args = target_config
        .extra_args
        .map(|args| args.iter().map(|arg| Argument::from(*arg)).collect())
        .unwrap_or_default();

    TargetModel {
        output_name: Cow::Borrowed(target_config.output_name.unwrap_or(project_name)),
        sources,
        extra_args,
        kind: target_config.kind.unwrap_or_default(),
        enabled_for_current_program_iteration: true, // NOTE: For now, it can only be manually
                                                     // disabled by cli args
    }
}

/// Utilery function to map all the source files declared on the [`ZorkConfigFile::targets`]
/// attribute to the domain model entity, including resolving any [`GlobPattern`] declared as
/// any file on the input collection
fn get_sources_for_target<'a>(srcs: Vec<Cow<str>>, code_root: &Path) -> SourceSet<'a> {
    let sources = srcs
        .iter()
        .map(|src| {
            let target_src = code_root.join(src.as_ref());
            if src.contains('*') {
                Source::Glob(GlobPattern(target_src))
            } else {
                Source::File(target_src)
            }
        })
        .flat_map(|source| {
            source
                .paths()
                .expect("Error getting the declared paths for the source files")
        })
        .map(|pb| {
            let file_details = utils::fs::get_file_details(&pb).unwrap_or_else(|_| {
                panic!("An unexpected error happened getting the file details for {pb:?}")
            });
            SourceFile {
                path: file_details.0,
                file_stem: Cow::Owned(file_details.1),
                extension: Cow::Owned(file_details.2),
            }
        })
        .collect();

    SourceSet { sources }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use crate::config_file;
    use crate::domain::target::TargetKind;
    use crate::utils::fs;
    use crate::{
        project_model::compiler::{CppCompiler, LanguageLevel, StdLib},
        utils,
    };
    use clap::Parser;

    use super::*;

    #[test]
    fn test_project_model_with_full_config() -> Result<()> {
        let config: ZorkConfigFile =
            config_file::zork_cfg_from_file(utils::constants::CONFIG_FILE_MOCK)?;
        let cli_args = CliArgs::parse_from(["", "-vv", "run"]);
        let abs_path_for_mock = fs::get_project_root_absolute_path(Path::new("."))?;
        let model = build_model(config, &cli_args, &abs_path_for_mock);

        let mut targets = IndexMap::new();
        targets.insert(
            TargetIdentifier::from("executable"),
            TargetModel {
                output_name: "zork".into(),
                sources: SourceSet {
                    sources: vec![SourceFile {
                        path: abs_path_for_mock.clone(),
                        file_stem: Cow::Borrowed("main"),
                        extension: Cow::Borrowed("cpp"),
                    }],
                },
                extra_args: vec!["-Werr".into()],
                kind: TargetKind::Executable,
                enabled_for_current_program_iteration: true,
            },
        );
        targets.insert(
            TargetIdentifier::from("tests"),
            TargetModel {
                output_name: "zork_tests".into(),
                sources: SourceSet {
                    sources: vec![SourceFile {
                        path: abs_path_for_mock.clone(),
                        file_stem: Cow::Borrowed("tests_main"),
                        extension: Cow::Borrowed("cpp"),
                    }],
                },
                extra_args: vec![],
                kind: TargetKind::Executable,
                enabled_for_current_program_iteration: true,
            },
        );

        let expected = ZorkModel {
            project: ProjectModel {
                name: "Zork++".into(),
                authors: vec!["zerodaycode.gz@gmail.com".into()],
                compilation_db: true,
                code_root: None,
            },
            compiler: CompilerModel {
                cpp_compiler: CppCompiler::CLANG,
                driver_path: Cow::Borrowed(""),
                cpp_standard: LanguageLevel::CPP2B,
                std_lib: Some(StdLib::LIBCPP),
                extra_args: vec![Argument::from("-Wall")],
            },
            build: BuildModel {
                output_dir: abs_path_for_mock.clone(),
            },
            modules: ModulesModel {
                base_ifcs_dir: Cow::Borrowed(Path::new("ifcs")),
                interfaces: vec![
                    ModuleInterfaceModel {
                        path: abs_path_for_mock.join("ifcs"),
                        file_stem: Cow::Borrowed("maths"),
                        extension: Cow::Borrowed("cppm"),
                        module_name: "maths".into(),
                        partition: None,
                        dependencies: vec![],
                    },
                    ModuleInterfaceModel {
                        path: abs_path_for_mock.join("ifcs"),
                        file_stem: Cow::Borrowed("some_module"),
                        extension: Cow::Borrowed("cppm"),
                        module_name: "maths".into(),
                        partition: None,
                        dependencies: vec![],
                    },
                ],
                base_impls_dir: Cow::Borrowed(Path::new("srcs")),
                implementations: vec![
                    ModuleImplementationModel {
                        path: abs_path_for_mock.join("srcs"),
                        file_stem: Cow::from("maths"),
                        extension: Cow::from("cpp"),
                        dependencies: vec!["maths".into()],
                    },
                    ModuleImplementationModel {
                        path: abs_path_for_mock.join("srcs"),
                        file_stem: Cow::from("some_module_impl"),
                        extension: Cow::from("cpp"),
                        dependencies: vec!["iostream".into()],
                    },
                ],
                sys_modules: vec![SystemModule {
                    file_stem: Cow::Borrowed("iostream"),
                    ..Default::default()
                }],
            },
            targets,
        };

        assert_eq!(model.unwrap(), expected);

        Ok(())
    }
}
