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
        modules::{ModuleImplementationModel, ModuleInterfaceModel, ModulesModel},
        project::ProjectModel,
        sourceset::{GlobPattern, Source, SourceSet},
        tests::TestsModel,
        ZorkModel,
    },
    utils::constants::CONFIG_FILE_NAME,
};
use color_eyre::{eyre::Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use super::constants::DEFAULT_OUTPUT_DIR;

/// Checks for the existence of the `zork.toml` configuration files.
/// For now, when finds the first one, stops. Pending to decide if
/// Zork++ should support multiple configuration files (for nested projects)
/// or just by parsing one config file
///
/// This function panics if there's no configuration file
/// (or isn't present in any directory of the project)
pub fn find_config_file(base_path: &Path) -> Result<String> {
    let mut path: PathBuf = base_path.into();

    for e in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        if e.metadata().unwrap().is_file() && e.path().ends_with(CONFIG_FILE_NAME) {
            path.push(e.path());
            break;
        }
    }

    fs::read_to_string(&path).with_context(|| format!("Could not read {path:?}"))
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

    let gcc_sys_modules = config
        .and_then(|modules| modules.gcc_sys_modules.as_ref())
        .map_or_else(Default::default, |headers| headers.clone());

    ModulesModel {
        base_ifcs_dir: Path::new(base_ifcs_dir),
        interfaces,
        base_impls_dir: Path::new(base_impls_dir),
        implementations,
        gcc_sys_modules,
    }
}

fn assemble_module_interface_model<'a>(config: &'a ModuleInterface) -> ModuleInterfaceModel<'a> {
    let module_name = config
        .module_name
        .unwrap_or_else(|| config.file.split('.').collect::<Vec<_>>()[0]);

    let dependencies = config.dependencies.clone().unwrap_or_default();

    ModuleInterfaceModel {
        file: Path::new(config.file),
        module_name,
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
    use crate::project_model::compiler::{CppCompiler, LanguageLevel, StdLib};

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
                gcc_sys_modules: vec![],
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
        const CONFIG_FILE_MOCK: &str = r#"
            [project]
            name = "Zork++"
            authors = ["zerodaycode.gz@gmail.com"]

            [compiler]
            cpp_compiler = "clang"
            cpp_standard = "20"
            std_lib = "libc++"
            extra_args = [ "-Wall" ]

            [build]
            output_dir = "build"

            [executable]
            executable_name = "zork"
            sources_base_path = "bin"
            sources = [
                "*.cpp"
            ]
            extra_args = [ "-Werr" ]

            [tests]
            test_executable_name = "zork_check"
            sources_base_path = "test"
            sources = [
                "*.cpp"
            ]
            extra_args = [ "-pedantic" ]

            [modules]
            base_ifcs_dir = "ifc"
            interfaces = [
                { file = "math.cppm" },
                { file = 'some_module.cppm', module_name = 'math' }
            ]

            base_impls_dir = "src"
            implementations = [
                { file = "math.cpp" },
                { file = 'some_module_impl.cpp', dependencies = ['iostream'] }
            ]
            gcc_sys_modules = [ "iostream" ]
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
                        dependencies: vec![],
                    },
                    ModuleInterfaceModel {
                        file: Path::new("some_module.cppm"),
                        module_name: "math",
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
                gcc_sys_modules: vec!["iostream"],
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
