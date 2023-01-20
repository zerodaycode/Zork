use crate::{
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

pub fn load_model(base_path: &Path) -> Result<ZorkModel> {
    let config_file: String =
        find_config_file(base_path).with_context(|| "Failed to read configuration file")?;
    let config: ZorkConfigFile = toml::from_str(config_file.as_str())
        .with_context(|| "Could not parse configuration file")?;

    Ok(build_model(&config))
}

pub fn build_model(config: &ZorkConfigFile) -> ZorkModel {
    let project = assemble_project_model(&config.project);
    let compiler = assemble_compiler_model(&config.compiler);
    let build = assemble_build_model(&config.build);
    let executable = assemble_executable_model(&project.name, &config.executable);
    let modules = assemble_modules_model(&config.modules);
    let tests = assemble_tests_model(&project.name, &config.tests);

    ZorkModel {
        project,
        compiler,
        build,
        executable,
        modules,
        tests,
    }
}

fn assemble_project_model(config: &ProjectAttribute) -> ProjectModel {
    ProjectModel {
        name: config.name.to_owned(),
        authors: extract_vec_or_empty(&config.authors.as_ref()),
    }
}

fn assemble_compiler_model(config: &CompilerAttribute) -> CompilerModel {
    CompilerModel {
        cpp_compiler: config.cpp_compiler.clone().into(),
        cpp_standard: config.cpp_standard.clone().into(),
        std_lib: config.std_lib.clone().map(|lib| lib.into()),
        extra_args: extract_vec_or_empty(&config.extra_args.as_ref()),
        system_headers_path: config.system_headers_path.map(str::to_owned),
    }
}

fn assemble_build_model(config: &Option<BuildAttribute>) -> BuildModel {
    let output_dir = config
        .as_ref()
        .and_then(|build| build.output_dir)
        .unwrap_or(DEFAULT_OUTPUT_DIR)
        .into();

    BuildModel { output_dir }
}

fn assemble_executable_model(
    project_name: &str,
    config: &Option<ExecutableAttribute>,
) -> ExecutableModel {
    let config = config.as_ref();

    let executable_name = config
        .and_then(|exe| exe.executable_name)
        .unwrap_or(project_name)
        .into();

    let sources_base_path = config
        .and_then(|exe| exe.sources_base_path)
        .unwrap_or(".")
        .into();

    let sources = config.and_then(|exe| exe.sources.as_ref());
    let sources = extract_vec_or_empty(&sources);

    let extra_args = config.and_then(|exe| exe.extra_args.as_ref());
    let extra_args = extract_vec_or_empty(&extra_args);

    ExecutableModel {
        executable_name,
        sources_base_path,
        sources,
        extra_args,
    }
}

fn assemble_modules_model(config: &Option<ModulesAttribute>) -> ModulesModel {
    let config = config.as_ref();

    let base_ifcs_dir = config
        .and_then(|modules| modules.base_ifcs_dir)
        .unwrap_or(".")
        .into();

    let interfaces = config
        .and_then(|modules| modules.interfaces.as_ref())
        .unwrap_or(&Vec::new())
        .iter()
        .map(assemble_module_interface_model)
        .collect();

    let base_impls_dir = config
        .and_then(|modules| modules.base_impls_dir)
        .unwrap_or(".")
        .into();

    let implementations = config
        .and_then(|modules| modules.implementations.as_ref())
        .unwrap_or(&Vec::new())
        .iter()
        .map(assemble_module_implementation_model)
        .collect();

    let gcc_sys_headers = config.and_then(|modules| modules.gcc_sys_headers.as_ref());
    let gcc_sys_headers = extract_vec_or_empty(&gcc_sys_headers);

    ModulesModel {
        base_ifcs_dir,
        interfaces,
        base_impls_dir,
        implementations,
        gcc_sys_headers,
    }
}

fn assemble_module_interface_model(config: &ModuleInterface) -> ModuleInterfaceModel {
    let filename: String = config.filename.into();

    let module_name = config
        .module_name
        .unwrap_or_else(|| filename.split('.').collect::<Vec<_>>()[0])
        .into();

    let dependencies = extract_vec_or_empty(&config.dependencies.as_ref());

    ModuleInterfaceModel {
        filename,
        module_name,
        dependencies,
    }
}

fn assemble_module_implementation_model(
    config: &ModuleImplementation,
) -> ModuleImplementationModel {
    let filename: String = config.filename.into();

    let mut dependencies = extract_vec_or_empty(&config.dependencies.as_ref());
    if dependencies.is_empty() {
        let implicit_dependency = filename.split('.').collect::<Vec<_>>()[0];
        dependencies.push(implicit_dependency.into());
    }

    ModuleImplementationModel {
        filename,
        dependencies,
    }
}

fn assemble_tests_model(project_name: &str, config: &Option<TestsAttribute>) -> TestsModel {
    let config = config.as_ref();

    let test_executable_name = config
        .and_then(|exe| exe.test_executable_name)
        .map(|exe_name| exe_name.into())
        .unwrap_or(format!("{project_name}_test"));

    let source_base_path = config
        .and_then(|exe| exe.source_base_path)
        .unwrap_or(".")
        .into();

    let sources = config.and_then(|exe| exe.sources.as_ref());
    let sources = extract_vec_or_empty(&sources);

    let extra_args = config
        .and_then(|exe| exe.extra_args)
        .map(|arg| vec![arg.into()])
        .unwrap_or_else(Vec::new);

    TestsModel {
        test_executable_name,
        source_base_path,
        sources,
        extra_args,
    }
}

fn extract_vec_or_empty(opt_vec: &Option<&Vec<&str>>) -> Vec<String> {
    opt_vec
        .map(|vec| vec.iter().map(|element| (*element).to_owned()).collect())
        .unwrap_or_default()
}
