//! The crate responsable for executing the core work of `Zork++`,
// generate command lines and execute them in a shell of the current
// operating system against the designed compilers in the configuration
// file.

use color_eyre::{eyre::Context, Result};
use std::{path::Path, process::Command};

use crate::{
    cli::CliArgs,
    config_file::{modules::ModuleInterface, ZorkConfigFile},
    utils::{self, constants::DEFAULT_OUTPUT_DIR, reader::find_config_file},
};

/// The entry point of the compilation process
///
/// Whenever this process gets triggered, the files declared within the
/// configuration file will be build.
///
/// TODO Decision path for building the executable command line,
/// the tests executable command line, a static lib, a dylib...
pub fn build_project(base_path: &Path, _cli_args: &CliArgs) -> Result<()> {
    let config_file: String =
        find_config_file(base_path).with_context(|| "Failed to read configuration file")?;
    let config: ZorkConfigFile = toml::from_str(config_file.as_str())
        .with_context(|| "Could not parse configuration file")?;

    // Create the directory for dump the generated files
    create_output_directory(base_path, &config)?;
    // 1st - Build the modules
    #[allow(clippy::let_unit_value)]  // TODO Solved when the commands will be returned
    let _modules_commands = build_modules(&config);

    Ok(())
}

/// Triggers the build process for compile the declared modules in the project
///
/// This function acts like a operation result processor, by running instances
/// and parsing the obtained result, handling the flux according to the
/// compiler responses
fn build_modules(config: &ZorkConfigFile) {
    let compiler_driver = &config.compiler.cpp_compiler.get_driver();

    if let Some(modules) = &config.modules {
        if let Some(interfaces) = &modules.interfaces {
            // TODO append to a collection to make them able to be dump into a text file
            let miu_commands = prebuild_module_interfaces(config, interfaces);

            // For debugging purposes, then will be refactored
            for command in miu_commands {
                let output = Command::new(compiler_driver)
                    .args(command)
                    .output()
                    .expect("failed to execute process");
                // TODO If compiler is msvc, we must launch cl

                println!("Command execution result: {:?}", output.status);
                println!(
                    "Command execution stdout: {:?}",
                    String::from_utf16(
                        &output
                            .stdout
                            .iter()
                            .map(|b| *b as u16)
                            .collect::<Vec<u16>>()
                    )
                );
                println!(
                    "Command execution stderr: {:?}",
                    String::from_utf16(
                        &output
                            .stderr
                            .iter()
                            .map(|b| *b as u16)
                            .collect::<Vec<u16>>()
                    )
                );
            }
        }
    }
}

/// Parses the configuration in order to build the BMIs declared for the project,
/// by precompiling the module interface units
fn prebuild_module_interfaces(
    config: &ZorkConfigFile,
    interfaces: &Vec<ModuleInterface>,
) -> Vec<Vec<String>> {
    let mut commands: Vec<Vec<String>> = Vec::with_capacity(interfaces.len());
    interfaces.iter().for_each(|module_interface| {
        commands.push(
            config
                .compiler
                .cpp_compiler
                .get_module_ifc_args(config, module_interface),
        )
    });
    println!("BMIs: {commands:?}");

    commands
}

/// Creates the directory for output the elements generated
/// during the build process. Also, it will generate the
/// ['output_build_dir'/zork], which is a subfolder
/// where Zork dumps the things that needs to work correctly
/// under different conditions.
///
/// Under /zork, some new folders are created:
/// - a /intrinsics folder in created as well,
/// where different specific details of Zork++ are stored
/// related with the C++ compilers
///
/// - a /cache folder, where lives the metadata cached by Zork++
/// in order to track different aspects of the program (last time
/// modified files, last process build time...)
///  
/// TODO Generate the caché process, like last time project build,
/// and only rebuild files that is metadata contains a newer last
/// time modified date that the last Zork++ process
fn create_output_directory(base_path: &Path, config: &ZorkConfigFile) -> Result<()> {
    let out_dir = config
        .build
        .as_ref()
        .and_then(|build| build.output_dir)
        .unwrap_or(DEFAULT_OUTPUT_DIR);

    let compiler = &config.compiler.cpp_compiler;

    // Recursively create a directory and all of its parent components if they are missing
    let modules_path = base_path
        .join(out_dir)
        .join(compiler.to_string())
        .join("modules");
    let zork_path = base_path.join(out_dir).join("zork");

    utils::fs::create_directory(&modules_path.join("interfaces"))?;
    utils::fs::create_directory(&modules_path.join("implementations"))?;
    utils::fs::create_directory(&zork_path.join("cache"))?;
    utils::fs::create_directory(&zork_path.join("intrinsics"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use color_eyre::Result;
    use tempfile::tempdir;

    use crate::utils::template::resources::CONFIG_FILE;

    use super::*;

    #[test]
    fn test_creation_directories() -> Result<()> {
        let temp = tempdir()?;

        let zcf: ZorkConfigFile = toml::from_str(CONFIG_FILE)?;

        // This should create and out/ directory in the ./zork++ folder at the root of this project
        create_output_directory(temp.path(), &zcf)?;

        assert!(temp.path().join("out").exists());
        assert!(temp.path().join("out/zork").exists());
        assert!(temp.path().join("out/zork/cache").exists());
        assert!(temp.path().join("out/zork/intrinsics").exists());

        Ok(())
    }
}
