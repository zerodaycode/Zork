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
};
use color_eyre::{eyre::eyre, Result};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

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
pub fn find_config_files(base_path: &Path) -> Result<Vec<ConfigFile>> {
    log::debug!("Searching for Zork++ configuration files...");
    let mut files = vec![];

    for e in WalkDir::new(base_path)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if e.metadata().unwrap().is_file()
            && e.file_name().to_str().unwrap().starts_with("zork")
            && e.file_name().to_str().unwrap().ends_with(".toml")
        {
            log::debug!("Config file found!: {:?}", e.file_name());
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

pub fn build_model<'a>(config: &'a ZorkConfigFile) -> ZorkModel<'a> {
    let project = assemble_project_model(&config.project);
    let compiler = assemble_compiler_model(&config.compiler);
    let build = assemble_build_model(&config.build);
    let executable = assemble_executable_model(project.name, &config.executable);
    let modules = assemble_modules_model(&config.modules);
    let tests = assemble_tests_model(project.name, &config.tests);

    ZorkModel {
        project,
        compiler,
        build,
        executable,
        modules,
        tests,
    }
}

fn assemble_project_model<'a>(config: &'a ProjectAttribute) -> ProjectModel<'a> {
    ProjectModel {
        name: config.name,
        authors: config
            .authors
            .as_ref()
            .map_or_else(|| &[] as &[&str], |auths| auths.as_slice()),
    }
}

fn assemble_compiler_model<'a>(config: &'a CompilerAttribute) -> CompilerModel<'a> {
    let extra_args = config
        .extra_args
        .as_ref()
        .map(|args| args.iter().map(|arg| Argument::from(*arg)).collect())
        .unwrap_or_default();

    CompilerModel {
        cpp_compiler: config.cpp_compiler.clone().into(),
        cpp_standard: config.cpp_standard.clone().into(),
        std_lib: config.std_lib.clone().map(|lib| lib.into()),
        extra_args,
    }
}

fn assemble_build_model<'a>(config: &'a Option<BuildAttribute>) -> BuildModel<'a> {
    let output_dir = config
        .as_ref()
        .and_then(|build| build.output_dir)
        .unwrap_or(DEFAULT_OUTPUT_DIR);

    BuildModel {
        output_dir: Path::new(output_dir),
    }
}

fn assemble_executable_model<'a>(
    project_name: &'a str,
    config: &'a Option<ExecutableAttribute>,
) -> ExecutableModel<'a> {
    let config = config.as_ref();

    let executable_name = config
        .and_then(|exe| exe.executable_name)
        .unwrap_or(project_name);

    let base_path = config.and_then(|exe| exe.sources_base_path).unwrap_or(".");

    let sources = config
        .and_then(|exe| exe.sources.clone())
        .unwrap_or_default()
        .into_iter()
        .map(|source| {
            if source.contains('.') {
                Source::Glob(GlobPattern(source))
            } else {
                Source::File(Path::new(source))
            }
        })
        .collect();

    let sourceset = SourceSet {
        base_path: Path::new(base_path),
        sources,
    };

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

fn assemble_modules_model<'a>(config: &'a Option<ModulesAttribute>) -> ModulesModel<'a> {
    let config = config.as_ref();

    let base_ifcs_dir = config
        .and_then(|modules| modules.base_ifcs_dir)
        .unwrap_or(".");

    let interfaces = config
        .and_then(|modules| modules.interfaces.as_ref())
        .map(|ifcs| ifcs.iter().map(assemble_module_interface_model).collect())
        .unwrap_or_default();

    let base_impls_dir = config
        .and_then(|modules| modules.base_impls_dir)
        .unwrap_or(".");

    let implementations = config
        .and_then(|modules| modules.implementations.as_ref())
        .map(|impls| {
            impls
                .iter()
                .map(assemble_module_implementation_model)
                .collect()
        })
        .unwrap_or_default();

    let sys_modules = config
        .and_then(|modules| modules.sys_modules.as_ref())
        .map_or_else(Default::default, |headers| headers.clone());

    ModulesModel {
        base_ifcs_dir: Path::new(base_ifcs_dir),
        interfaces,
        base_impls_dir: Path::new(base_impls_dir),
        implementations,
        sys_modules,
    }
}

fn assemble_module_interface_model<'a>(config: &'a ModuleInterface) -> ModuleInterfaceModel<'a> {
    let module_name = config
        .module_name
        .unwrap_or_else(|| 
            Path::new(config.file)
                .file_name()
                .map(|f| f.to_str().expect(&format!("Found ill-formed path on: {}", config.file)))
                .unwrap()
        );

    let dependencies = config.dependencies.clone().unwrap_or_default();
    let partition = if config.partition.is_none() {
        None
    } else {
        Some(ModulePartitionModel::from(
            config.partition.as_ref().unwrap(),
        ))
    };

    ModuleInterfaceModel {
        file: Path::new(config.file),
        module_name,
        partition,
        dependencies,
    }
}

fn assemble_module_implementation_model<'a>(
    config: &'a ModuleImplementation,
) -> ModuleImplementationModel<'a> {
    let mut dependencies = config.dependencies.clone().unwrap_or_default();
    if dependencies.is_empty() {
        let implicit_dependency = config.file.split('.').collect::<Vec<_>>()[0];
        dependencies.push(implicit_dependency);
    }

    ModuleImplementationModel {
        file: Path::new(config.file),
        dependencies,
    }
}

fn assemble_tests_model<'a>(
    project_name: &'a str,
    config: &'a Option<TestsAttribute>,
) -> TestsModel<'a> {
    let config = config.as_ref();

    let test_executable_name = config.and_then(|exe| exe.test_executable_name).map_or_else(
        || format!("{project_name}_test"),
        |exe_name| exe_name.to_owned(),
    );

    let base_path = config.and_then(|exe| exe.sources_base_path).unwrap_or(".");

    let sources = config
        .and_then(|exe| exe.sources.clone())
        .unwrap_or_default()
        .into_iter()
        .map(|source| {
            if source.contains('.') {
                Source::Glob(GlobPattern(source))
            } else {
                Source::File(Path::new(source))
            }
        })
        .collect();

    let sourceset = SourceSet {
        base_path: Path::new(base_path),
        sources,
    };

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

#[cfg(test)]
mod test {
    use crate::{
        project_model::compiler::{CppCompiler, LanguageLevel, StdLib},
        utils,
    };

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
        let model = build_model(&config);

        let expected = ZorkModel {
            project: ProjectModel {
                name: "Zork++",
                authors: &["zerodaycode.gz@gmail.com"],
            },
            compiler: CompilerModel {
                cpp_compiler: CppCompiler::CLANG,
                cpp_standard: LanguageLevel::CPP20,
                std_lib: None,
                extra_args: vec![],
            },
            build: BuildModel {
                output_dir: Path::new("./out"),
            },
            executable: ExecutableModel {
                executable_name: "Zork++",
                sourceset: SourceSet {
                    base_path: Path::new("."),
                    sources: vec![],
                },
                extra_args: vec![],
            },
            modules: ModulesModel {
                base_ifcs_dir: Path::new("."),
                interfaces: vec![],
                base_impls_dir: Path::new("."),
                implementations: vec![],
                sys_modules: vec![],
            },
            tests: TestsModel {
                test_executable_name: "Zork++_test".to_string(),
                sourceset: SourceSet {
                    base_path: Path::new("."),
                    sources: vec![],
                },
                extra_args: vec![],
            },
        };

        assert_eq!(model, expected);

        Ok(())
    }

    #[test]
    fn test_project_model_with_full_config() -> Result<()> {
        let config: ZorkConfigFile = toml::from_str(utils::constants::CONFIG_FILE_MOCK)?;
        let model = build_model(&config);

        let expected = ZorkModel {
            project: ProjectModel {
                name: "Zork++",
                authors: &["zerodaycode.gz@gmail.com"],
            },
            compiler: CompilerModel {
                cpp_compiler: CppCompiler::CLANG,
                cpp_standard: LanguageLevel::CPP20,
                std_lib: Some(StdLib::LIBCPP),
                extra_args: vec![Argument::from("-Wall")],
            },
            build: BuildModel {
                output_dir: Path::new("build"),
            },
            executable: ExecutableModel {
                executable_name: "zork",
                sourceset: SourceSet {
                    base_path: Path::new("bin"),
                    sources: vec![Source::Glob(GlobPattern("*.cpp"))],
                },
                extra_args: vec![Argument::from("-Werr")],
            },
            modules: ModulesModel {
                base_ifcs_dir: Path::new("ifc"),
                interfaces: vec![
                    ModuleInterfaceModel {
                        file: Path::new("math.cppm"),
                        module_name: "math",
                        partition: None,
                        dependencies: vec![],
                    },
                    ModuleInterfaceModel {
                        file: Path::new("some_module.cppm"),
                        module_name: "math",
                        partition: None,
                        dependencies: vec![],
                    },
                ],
                base_impls_dir: Path::new("src"),
                implementations: vec![
                    ModuleImplementationModel {
                        file: Path::new("math.cpp"),
                        dependencies: vec!["math"],
                    },
                    ModuleImplementationModel {
                        file: Path::new("some_module_impl.cpp"),
                        dependencies: vec!["iostream"],
                    },
                ],
                sys_modules: vec!["iostream"],
            },
            tests: TestsModel {
                test_executable_name: "zork_check".to_string(),
                sourceset: SourceSet {
                    base_path: Path::new("test"),
                    sources: vec![Source::Glob(GlobPattern("*.cpp"))],
                },
                extra_args: vec![Argument::from("-pedantic")],
            },
        };

        assert_eq!(model, expected);

        Ok(())
    }
}
