//! The crate responsable for executing the core work of `Zork++`,
// generate command lines and execute them in a shell of the current
// operating system against the designed compilers in the configuration
// file.

use crate::{
    cli::CliArgs,
    config_file::{modules::ModuleInterface, ZorkConfigFile},
    utils::constants::DEFAULT_OUTPUT_DIR,
};
use std::{
    fs,
    process::Command,
};

/// The entry point of the compilation process
///
/// Whenever this process gets triggered, the files declared within the
/// configuration file will be build.
///
/// TODO Decision path for building the executable command line,
/// the tests executable command line, a static lib, a dylib...
pub fn build_project(config: &ZorkConfigFile, _cli_args: &CliArgs) {
    // Create the directory for dump the generated files
    create_output_directory(config);
    // 1st - Build the modules
    let _modules_commands = build_modules(config);
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
///
/// TODO Handle error with `color_eyre`. Subdirs as constants?=!
fn create_output_directory(config: &ZorkConfigFile) {
    let out_dir = config.build.as_ref().map_or_else(
        || DEFAULT_OUTPUT_DIR,
        |build| build.output_dir.unwrap_or(DEFAULT_OUTPUT_DIR),
    );
    let compiler = &config.compiler.cpp_compiler;

    // Recursively create a directory and all of its parent components if they are missing
    fs::create_dir_all(format!("{out_dir}/{compiler}/modules/interfaces"))
        .expect("A failure happened creating the module interfaces dir");
    fs::create_dir_all(format!("{out_dir}/{compiler}/modules/implementations"))
        .expect("A failure happened creating the module interfaces dir");
    fs::create_dir_all(format!("{out_dir}/{compiler}/zork/cache"))
        .expect("A failure happened creating the cache Zork dir");
    fs::create_dir_all(format!("{out_dir}/{compiler}/zork/intrinsics"))
        .expect("A failure happened creating the intrinsics Zork dir");
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::utils::template::resources::CONFIG_FILE;

    use super::*;

    #[test]
    fn test_creation_directories() {
        let zcf: ZorkConfigFile = toml::from_str(CONFIG_FILE).unwrap();

        // This should create and out/ directory in the ./zork++ folder at the root of this project
        create_output_directory(&zcf);

        assert!(Path::new("./out").exists());
        assert!(Path::new("./out/zork").exists());
        assert!(Path::new("./out/zork/cache").exists());
        assert!(Path::new("./out/zork/intrinsics").exists());

        // Clean up the out directory created for testing purposes
        assert!(fs::remove_dir_all("./out").is_ok())
    }
}
