use crate::cli::input::CliArgs;
use crate::project_model::sourceset::SourceFile;
use crate::utils::fs::get_project_root_absolute_path;
use crate::{
    cli::output::arguments::Argument,
    config_file::{
        build::BuildAttribute,
        compiler::CompilerAttribute,
        executable::ExecutableAttribute,
        modules::{ModuleImplementation, ModuleInterface, ModulesAttribute},
        project::ProjectAttribute,
        tests::TestsAttribute,
        ZorkConfigFile,
    },
    project_model::{
        build::BuildModel,
        compiler::CompilerModel,
        executable::ExecutableModel,
        modules::{
            ModuleImplementationModel, ModuleInterfaceModel, ModulePartitionModel, ModulesModel,
        },
        project::ProjectModel,
        sourceset::{GlobPattern, Source, SourceSet},
        tests::TestsModel,
        ZorkModel,
    },
    utils,
};
use color_eyre::{eyre::eyre, Result};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};
use crate::config_file::workspace::WorkspaceAttribute;
use crate::project_model::workspace::WorkspaceModel;

use super::constants::DEFAULT_OUTPUT_DIR;

/// Details about a found configuration file on the project
///
/// This is just a configuration file with a valid name found
/// at a valid path in some subdirectory
#[derive(Debug)]
pub struct ConfigFile {
    pub dir_entry: DirEntry,
    pub path: PathBuf,
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
    base_path: &Path,
    filename_match: &Option<String>,
) -> Result<Vec<ConfigFile>> {
    log::debug!("Searching for Zork++ configuration files...");
    let mut files = vec![];

    /*
    Opción A: Matcheamos con depth = 1, con lo cual solo puedes correr zork++ desde mínimo, dentro
    de la raíz del projecto, PEEEERO... habría que buscar de nuevo las config files registradas
    en el workspace

    Opción B: Cargarlas todas. Parsear fuera del bucle principal. Organizar. En caso de haber
    workspace, lógica 1. Else => otra lógica

    Opción C: Buscar siempre con depth = 1. Si cuando salen los resultados, la config file es
    un workspace, volver a buscar. Si no, ya estaríamos compilando el crate al que apunta "el tema"

    Opción D: Al pasar la flag de workspace, sabemos que es de antemano un workspace. Eso implica menos
    lógica de proceso, pero podría haber distintos zork por ahí aunque sea de puto milagro

    */
    for e in WalkDir::new(base_path)
        .max_depth(2)
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
                dir_entry: e.clone(),
                path: e.path().to_path_buf(),
            })
        }
    }

    if files.is_empty() {
        Err(eyre!("No configuration files found for the project"))
    } else {
        Ok(files)
    }
}

pub fn build_model<'a>(config: &'a ZorkConfigFile, cli_args: &'a CliArgs) -> Result<ZorkModel<'a>> {
pub fn build_model<'a>(
    config: &'a ZorkConfigFile,
    project_root_from_cli: &Path,
) -> Result<ZorkModel<'a>> {
    let workspace = assemble_workspace_model(&config.workspace);
    let project = assemble_project_model(&config.project);

    let absolute_project_root = if cli_args.root.is_none() {
        get_project_root_absolute_path(
            project
                .project_root
                .map(Path::new)
                .unwrap_or(Path::new(".")),
        )?
    } else {
        Path::new(&cli_args.root.as_ref().unwrap()).to_path_buf()
    };

    let compiler = assemble_compiler_model(&config.compiler, cli_args);
    let build = assemble_build_model(&config.build, &absolute_project_root);
    let executable =
        assemble_executable_model(project.name, &config.executable, &absolute_project_root);
    let modules = assemble_modules_model(&config.modules, &absolute_project_root);
    let tests = assemble_tests_model(project.name, &config.tests, &absolute_project_root);

    Ok(ZorkModel {
        workspace,
        project,
        compiler,
        build,
        executable,
        modules,
        tests,
    })
}

fn assemble_workspace_model<'a>(config: &'a Option<WorkspaceAttribute>) -> WorkspaceModel<'a> {
    WorkspaceModel {
        members: config.as_ref().unwrap_or(&WorkspaceAttribute::default()).members.clone()
    }
}

fn assemble_project_model<'a>(config: &'a ProjectAttribute) -> ProjectModel<'a> {
    ProjectModel {
        name: config.name,
        authors: config
            .authors
            .as_ref()
            .map_or_else(|| &[] as &[&str], |auths| auths.as_slice()),
        compilation_db: config.compilation_db.unwrap_or_default(),
        project_root: config.project_root,
    }
}

fn assemble_compiler_model<'a>(
    config: &'a CompilerAttribute,
    cli_args: &'a CliArgs,
) -> CompilerModel<'a> {
    let extra_args = config
        .extra_args
        .as_ref()
        .map(|args| args.iter().map(|arg| Argument::from(*arg)).collect())
        .unwrap_or_default();

    let cli_driver_path = cli_args.driver_path.as_ref();

    CompilerModel {
        cpp_compiler: config.cpp_compiler.clone().into(),
        driver_path: if let Some(driver_path) = cli_driver_path {
            driver_path.as_str()
        } else {
            config.driver_path.unwrap_or_default()
        },
        cpp_standard: config.cpp_standard.clone().into(),
        std_lib: config.std_lib.clone().map(|lib| lib.into()),
        extra_args,
    }
}

fn assemble_build_model(config: &Option<BuildAttribute>, project_root: &Path) -> BuildModel {
    let output_dir = config
        .as_ref()
        .and_then(|build| build.output_dir)
        .map(|out_dir| out_dir.strip_prefix("./").unwrap_or(out_dir))
        .unwrap_or(DEFAULT_OUTPUT_DIR);

    BuildModel {
        output_dir: Path::new(project_root).join(output_dir),
    }
}

//noinspection ALL
fn assemble_executable_model<'a>(
    project_name: &'a str,
    config: &'a Option<ExecutableAttribute>,
    project_root: &Path,
) -> ExecutableModel<'a> {
    let config = config.as_ref();

    let executable_name = config
        .and_then(|exe| exe.executable_name)
        .unwrap_or(project_name);

    let sources = config
        .and_then(|exe| exe.sources.clone())
        .unwrap_or_default();

    let sourceset = get_sourceset_for(sources, project_root);

    let extra_args = config
        .and_then(|exe| exe.extra_args.as_ref())
        .map(|args| args.iter().map(|arg| Argument::from(*arg)).collect())
        .unwrap_or_default();

    ExecutableModel {
        executable_name,
        sourceset,
        extra_args,
    }
}

fn assemble_modules_model<'a>(
    config: &'a Option<ModulesAttribute>,
    project_root: &Path,
) -> ModulesModel<'a> {
    let config = config.as_ref();

    let base_ifcs_dir = config
        .and_then(|modules| modules.base_ifcs_dir)
        .unwrap_or(".");

    let interfaces = config
        .and_then(|modules| modules.interfaces.as_ref())
        .map(|ifcs| {
            ifcs.iter()
                .map(|m_ifc| assemble_module_interface_model(m_ifc, base_ifcs_dir, project_root))
                .collect()
        })
        .unwrap_or_default();

    let base_impls_dir = config
        .and_then(|modules| modules.base_impls_dir)
        .unwrap_or(".");

    let implementations = config
        .and_then(|modules| modules.implementations.as_ref())
        .map(|impls| {
            impls
                .iter()
                .map(|m_impl| {
                    assemble_module_implementation_model(m_impl, base_impls_dir, project_root)
                })
                .collect()
        })
        .unwrap_or_default();

    let sys_modules = config
        .and_then(|modules| modules.sys_modules.as_ref())
        .map_or_else(Default::default, |headers| headers.clone());

    let extra_args = config
        .and_then(|mod_attr| mod_attr.extra_args.as_ref())
        .map(|args| args.iter().map(|arg| Argument::from(*arg)).collect())
        .unwrap_or_default();

    ModulesModel {
        base_ifcs_dir: Path::new(base_ifcs_dir),
        interfaces,
        base_impls_dir: Path::new(base_impls_dir),
        implementations,
        sys_modules,
        extra_args,
    }
}

fn assemble_module_interface_model<'a>(
    config: &'a ModuleInterface,
    base_path: &str,
    project_root: &Path,
) -> ModuleInterfaceModel<'a> {
    let file_path = Path::new(project_root).join(base_path).join(config.file);
    let module_name = config.module_name.unwrap_or_else(|| {
        Path::new(config.file)
            .file_stem()
            .unwrap_or_else(|| panic!("Found ill-formed path on: {}", config.file))
            .to_str()
            .unwrap()
    });

    let dependencies = config.dependencies.clone().unwrap_or_default();
    let partition = if config.partition.is_none() {
        None
    } else {
        Some(ModulePartitionModel::from(
            config.partition.as_ref().unwrap(),
        ))
    };

    let file_details = utils::fs::get_file_details(&file_path).unwrap_or_else(|_| {
        panic!("An unexpected error happened getting the file details for {file_path:?}")
    });
    ModuleInterfaceModel {
        path: file_details.0,
        file_stem: file_details.1,
        extension: file_details.2,
        module_name,
        partition,
        dependencies,
    }
}

fn assemble_module_implementation_model<'a>(
    config: &'a ModuleImplementation,
    base_path: &str,
    project_root: &Path,
) -> ModuleImplementationModel<'a> {
    let file_path = Path::new(project_root).join(base_path).join(config.file);
    let mut dependencies = config.dependencies.clone().unwrap_or_default();
    if dependencies.is_empty() {
        let last_dot_index = config.file.rfind('.');
        if let Some(idx) = last_dot_index {
            let implicit_dependency = config.file.split_at(idx);
            dependencies.push(implicit_dependency.0)
        } else {
            dependencies.push(config.file);
        }
    }

    let file_details = utils::fs::get_file_details(&file_path).unwrap_or_else(|_| {
        panic!("An unexpected error happened getting the file details for {file_path:?}")
    });

    ModuleImplementationModel {
        path: file_details.0,
        file_stem: file_details.1,
        extension: file_details.2,
        dependencies,
    }
}

fn assemble_tests_model<'a>(
    project_name: &'a str,
    config: &'a Option<TestsAttribute>,
    project_root: &Path,
) -> TestsModel<'a> {
    let config = config.as_ref();

    let test_executable_name = config.and_then(|exe| exe.test_executable_name).map_or_else(
        || format!("{project_name}_test"),
        |exe_name| exe_name.to_owned(),
    );

    let sources = config
        .and_then(|exe| exe.sources.clone())
        .unwrap_or_default();
    let sourceset = get_sourceset_for(sources, project_root);

    let extra_args = config
        .and_then(|test| test.extra_args.as_ref())
        .map(|args| args.iter().map(|arg| Argument::from(*arg)).collect())
        .unwrap_or_default();

    TestsModel {
        test_executable_name,
        sourceset,
        extra_args,
    }
}

fn get_sourceset_for(srcs: Vec<&str>, project_root: &Path) -> SourceSet {
    let sources = srcs
        .iter()
        .map(|src| {
            let target_src = project_root.join(src);
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
                file_stem: file_details.1,
                extension: file_details.2,
            }
        })
        .collect();

    SourceSet { sources }
}

#[cfg(test)]
mod test {
    use crate::utils::fs;
    use crate::{
        project_model::compiler::{CppCompiler, LanguageLevel, StdLib},
        utils,
    };
    use clap::Parser;

    use super::*;

    #[test]
    fn test_project_model_with_minimal_config() -> Result<()> {
        const CONFIG_FILE_MOCK: &str = r#"
            [project]
            name = 'Zork++'
            authors = ['zerodaycode.gz@gmail.com']

            [compiler]
            cpp_compiler = 'clang'
            cpp_standard = '20'
        "#;

        let config: ZorkConfigFile = toml::from_str(CONFIG_FILE_MOCK)?;
        let cli_args = CliArgs::parse_from(["", "-vv", "run"]);
        let model = build_model(&config, &cli_args);

        let abs_path_for_mock = fs::get_project_root_absolute_path(Path::new("."))?;

        let expected = ZorkModel {
            workspace: WorkspaceModel {
                members: vec![],
            },
            project: ProjectModel {
                name: "Zork++",
                authors: &["zerodaycode.gz@gmail.com"],
                compilation_db: false,
                project_root: None,
            },
            compiler: CompilerModel {
                cpp_compiler: CppCompiler::CLANG,
                driver_path: "",
                cpp_standard: LanguageLevel::CPP20,
                std_lib: None,
                extra_args: vec![],
            },
            build: BuildModel {
                output_dir: abs_path_for_mock.join("out"),
            },
            executable: ExecutableModel {
                executable_name: "Zork++",
                sourceset: SourceSet { sources: vec![] },
                extra_args: vec![],
            },
            modules: ModulesModel {
                base_ifcs_dir: Path::new("."),
                interfaces: vec![],
                base_impls_dir: Path::new("."),
                implementations: vec![],
                sys_modules: vec![],
                extra_args: vec![],
            },
            tests: TestsModel {
                test_executable_name: "Zork++_test".to_string(),
                sourceset: SourceSet { sources: vec![] },
                extra_args: vec![],
            },
        };

        assert_eq!(model.unwrap(), expected);

        Ok(())
    }

    #[test]
    fn test_project_model_with_full_config() -> Result<()> {
        let config: ZorkConfigFile = toml::from_str(utils::constants::CONFIG_FILE_MOCK)?;
        let cli_args = CliArgs::parse_from(["", "-vv", "run"]);
        let model = build_model(&config, &cli_args);

        let abs_path_for_mock = fs::get_project_root_absolute_path(Path::new("."))?;

        let expected = ZorkModel {
            workspace: WorkspaceModel {
                members: vec![],
            },
            project: ProjectModel {
                name: "Zork++",
                authors: &["zerodaycode.gz@gmail.com"],
                compilation_db: true,
                project_root: None,
            },
            compiler: CompilerModel {
                cpp_compiler: CppCompiler::CLANG,
                driver_path: "",
                cpp_standard: LanguageLevel::CPP2B,
                std_lib: Some(StdLib::LIBCPP),
                extra_args: vec![Argument::from("-Wall")],
            },
            build: BuildModel {
                output_dir: abs_path_for_mock.clone(),
            },
            executable: ExecutableModel {
                executable_name: "zork",
                sourceset: SourceSet { sources: vec![] },
                extra_args: vec![Argument::from("-Werr")],
            },
            modules: ModulesModel {
                base_ifcs_dir: Path::new("ifcs"),
                interfaces: vec![
                    ModuleInterfaceModel {
                        path: abs_path_for_mock.join("ifcs"),
                        file_stem: String::from("maths"),
                        extension: String::from("cppm"),
                        module_name: "maths",
                        partition: None,
                        dependencies: vec![],
                    },
                    ModuleInterfaceModel {
                        path: abs_path_for_mock.join("ifcs"),
                        file_stem: String::from("some_module"),
                        extension: String::from("cppm"),
                        module_name: "maths",
                        partition: None,
                        dependencies: vec![],
                    },
                ],
                base_impls_dir: Path::new("srcs"),
                implementations: vec![
                    ModuleImplementationModel {
                        path: abs_path_for_mock.join("srcs"),
                        file_stem: String::from("maths"),
                        extension: String::from("cpp"),
                        dependencies: vec!["maths"],
                    },
                    ModuleImplementationModel {
                        path: abs_path_for_mock.join("srcs"),
                        file_stem: String::from("some_module_impl"),
                        extension: String::from("cpp"),
                        dependencies: vec!["iostream"],
                    },
                ],
                sys_modules: vec!["iostream"],
                extra_args: vec![Argument::from("-Wall")],
            },
            tests: TestsModel {
                test_executable_name: "zork_check".to_string(),
                sourceset: SourceSet { sources: vec![] },
                extra_args: vec![Argument::from("-pedantic")],
            },
        };

        assert_eq!(model.unwrap(), expected);

        Ok(())
    }
}
