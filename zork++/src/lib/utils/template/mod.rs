pub mod resources;

use crate::cli::input::TemplateValues;
use crate::project_model::compiler::CppCompiler;
use crate::utils;
use color_eyre::eyre::{bail, Context};
use color_eyre::{Report, Result};
use std::path::Path;
use std::process::Command;

/// Generates a new C++ standarized empty base project
/// with a pre-designed structure to organize the
/// user code in a modern fashion way.
///
/// Base template for the project files and folders:
///    - ./ifc
///        - math.<extension>
///    - ./src
///       - math.<extension -> .cpp, .cc, ...>
///       - math2.<extension -> .cpp, .cc, ...>
///    - main.cpp
///    - test/
///    - dependencies/
///
/// Note that this template is just a pnemonic. Any `C++` project can adhere to
/// whatever they feel that suits them better. Even tho, take in consideration
/// that legacy projects (headers + sources) can differ structurally to better reflect
/// what kind of responsabilities a translation unit has in the `C++` modules world.
pub fn create_templated_project(
    base_path: &Path,
    project_name: &str,
    git: bool,
    compiler: CppCompiler,
    template: TemplateValues,
) -> std::result::Result<(), Report> {
    let project_root = base_path.join(project_name);

    let path_ifc = project_root.join("ifc");
    let path_src = project_root.join("src");
    let path_test = project_root.join("test");
    let path_dependencies = project_root.join("deps");

    check_project_root_available(&project_root)?;

    utils::fs::create_directory(&project_root)?;
    utils::fs::create_directory(&path_ifc)?;
    utils::fs::create_directory(&path_src)?;
    utils::fs::create_directory(&path_test)?;
    utils::fs::create_directory(&path_dependencies)?;

    utils::fs::create_file(
        &path_ifc,
        &format!("{}.{}", "math", compiler.get_default_module_extension()),
        resources::IFC_MOD_FILE.as_bytes(),
    )?;

    match template {
        TemplateValues::BASIC => {
            utils::fs::create_file(&project_root, "main.cpp", resources::MAIN_BASIC.as_bytes())?;
        }
        TemplateValues::PARTITIONS => {
            utils::fs::create_file(
                &path_ifc,
                &format!(
                    "{}.{}",
                    "partitions",
                    compiler.get_default_module_extension()
                ),
                resources::IFC_PART_FILE.as_bytes(),
            )?;
            utils::fs::create_file(
                &path_ifc,
                &format!(
                    "{}.{}",
                    "interface_partition",
                    compiler.get_default_module_extension()
                ),
                resources::IFC_PART_PARTITION_FILE.as_bytes(),
            )?;
            utils::fs::create_file(
                &path_ifc,
                &format!("{}.{}", "internal_partition", "cpp"),
                resources::PARTITIONS_INTERNAL_PARTITION_FILE.as_bytes(),
            )?;
            utils::fs::create_file(&project_root, "main.cpp", resources::MAIN.as_bytes())?;
        }
    }

    utils::fs::create_file(&path_src, "math.cpp", resources::SRC_MOD_FILE.as_bytes())?;
    utils::fs::create_file(&path_src, "math2.cpp", resources::SRC_MOD_FILE_2.as_bytes())?;

    let template = match compiler {
        CppCompiler::MSVC => match template {
            TemplateValues::BASIC => resources::CONFIG_FILE_BASIC_MSVC,
            TemplateValues::PARTITIONS => resources::CONFIG_FILE_MSVC,
        },
        CppCompiler::CLANG => match template {
            TemplateValues::BASIC => resources::CONFIG_FILE_BASIC,
            TemplateValues::PARTITIONS => resources::CONFIG_FILE,
        },
        CppCompiler::GCC => match template {
            TemplateValues::BASIC => resources::CONFIG_FILE_BASIC_GCC,
            TemplateValues::PARTITIONS => resources::CONFIG_FILE_GCC,
        },
    }
    .replace("<project_name>", project_name);

    utils::fs::create_file(
        &project_root,
        &format!(
            "{}_{}.{}",
            utils::constants::CONFIG_FILE_NAME,
            compiler.as_ref(),
            utils::constants::CONFIG_FILE_EXT
        ),
        template.as_bytes(),
    )?;

    if git {
        initialize_git_repository(&project_root)?
    }

    Ok(())
}

fn check_project_root_available(project_root: &Path) -> Result<()> {
    if !project_root.exists() {
        // if it doesn't exist, there is nothing that would be overwritten
        return Ok(());
    }

    if !is_empty_directory(project_root)? {
        bail!("Directory {project_root:?} is not empty")
    }

    Ok(())
}

fn is_empty_directory(path: &Path) -> Result<bool> {
    if !path.is_dir() {
        return Ok(false);
    }

    let is_empty = path
        .read_dir()
        .with_context(|| format!("Directory {path:?} is not readable"))?
        .next()
        .is_none();

    Ok(is_empty)
}

fn initialize_git_repository(project_root: &Path) -> Result<()> {
    let exit_status = Command::new("git")
        .current_dir(project_root)
        .arg("init")
        .spawn()
        .with_context(|| "Could not run \"git init\"")?
        .wait()
        .with_context(|| "An error occurred while waiting for \"git init\" to finish")?;

    match exit_status.code() {
        Some(0) => {}
        None => bail!("Process \"git init\" was terminated by external signal"),
        Some(error_code) => bail!("Process \"git init\" returned {}", error_code),
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use color_eyre::Result;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_create_if_root_not_empty() -> Result<()> {
        let temp = tempdir()?;

        const PROJECT_NAME: &str = "example";

        let project_path = temp.path().join(PROJECT_NAME);
        let dummy_path = project_path.join("dummy.txt");

        std::fs::create_dir(project_path)?;
        std::fs::File::create(dummy_path)?;

        let result = create_templated_project(
            temp.path(),
            PROJECT_NAME,
            false,
            CppCompiler::CLANG,
            TemplateValues::BASIC,
        );
        assert!(
            result.is_err(),
            "The project was created, even though the project root is not empty"
        );

        Ok(())
    }
}
