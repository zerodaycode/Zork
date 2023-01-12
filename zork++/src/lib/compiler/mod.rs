//! The crate responsable for executing the core work of `Zork++`,
// generate command lines and execute them in a shell of the current
// operating system against the designed compilers in the configuration
// file.

use crate::{cli::CliArgs, config_file::{ZorkConfigFile, modules::ModuleInterface, compiler::CppCompiler}, utils::constants::DEFAULT_OUTPUT_DIR};
use std::fs;

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

    let base_command_line = get_base_command_line(config);

    // 1st - Build the modules
    build_modules(
        config, &base_command_line
    );
}

/// Generates the base command line that is shared among multiple processes
/// like, for example, generate the command line for the executable and build
/// modules
fn get_base_command_line<'a>(config: &'a ZorkConfigFile) -> Vec<String> {
    let compiler = &config.compiler;
    
    let mut base_command_line = Vec::with_capacity(2);
    base_command_line.push(compiler.cpp_compiler.get_driver().to_string());
    base_command_line.push(
        compiler.cpp_standard.as_cmd_arg(&compiler.cpp_compiler)
    );

    base_command_line
}

/// Triggers the build process for compile the declared modules in the project
/// 
/// This function acts like a operation result processor, by running instances
/// and parsing the obtained result, handling the flux according to the
/// compiler responses
fn build_modules(config: &ZorkConfigFile, bcl: &Vec<String>) {
    if let Some(modules) = &config.modules {
        if let Some(interfaces) = &modules.interfaces {
            prebuild_module_interfaces(config, interfaces, bcl);
        }
    }
}

/// Parses the configuration in order to build the BMIs declared for the project,
/// by precompiling the module interface units
fn prebuild_module_interfaces(
    config: &ZorkConfigFile,
    interfaces: &Vec<ModuleInterface>,
    bcl: &Vec<String>
) {
    // is a base path declared?
    // let base_path = base_path.unwrap_or_default();
    // base cmd args
    let mut bmis_args = Vec::from_iter(bcl.iter());
    let args = config.compiler.cpp_compiler
    .get_module_ifcs_args(config); // longer lived binding for debug
    bmis_args.extend(args.iter());
    // interfaces.iter().for_each()
    println!("BMIs: {:?}", bmis_args)
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
/// TODO Handle error with `color_eyre`
fn create_output_directory(config: &ZorkConfigFile) {
    let out_dir = config.build.as_ref().map_or_else(
        || DEFAULT_OUTPUT_DIR,
        |build| build.output_dir.unwrap_or(DEFAULT_OUTPUT_DIR),
    );

    // Recursively create a directory and all of its parent components if they are missing
    fs::create_dir_all(format!("{out_dir}/zork/cache"))
        .expect("A failure happened creating the cache Zork subdirectory");
    fs::create_dir_all(format!("{out_dir}/zork/intrinsics"))
        .expect("A failure happened creating the cache Zork subdirectory");
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
