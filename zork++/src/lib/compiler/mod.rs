use std::fs;

use crate::{cli::CliArgs, config_file::ZorkConfigFile, utils::constants::DEFAULT_OUTPUT_DIR};

///! The crate responsable for executing the core work of `Zork++`,
/// generate command lines and execute them in a shell of the current
/// operating system against the designed compilers in the configuration
/// file.

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
fn create_output_directory(config: &ZorkConfigFile) {
    let out_dir = config.build.as_ref().map_or_else(
        || DEFAULT_OUTPUT_DIR,
        |build| build.output_dir.unwrap_or(DEFAULT_OUTPUT_DIR),
    );
    let _zork_intrinsics_dir: &str = &format!("{out_dir}/zork/intrinsics");

    if !is_out_dir_already_created(out_dir) { // Generate the output directory structure
    }
}

/// Little helper for determine if the output directory wasn't been created yet
fn is_out_dir_already_created(out_dir: &str) -> bool {
    fs::read_dir("./")
        .expect("Failed to read the directories of the project")
        .into_iter()
        .any(|dir_entry| dir_entry.unwrap().path().ends_with(out_dir))
}
