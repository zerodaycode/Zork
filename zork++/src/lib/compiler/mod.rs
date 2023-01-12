//! The crate responsable for executing the core work of `Zork++`,
// generate command lines and execute them in a shell of the current
// operating system against the designed compilers in the configuration
// file.

use color_eyre::{eyre::Context, Result};

use crate::{
    cli::CliArgs,
    config_file::ZorkConfigFile,
    utils::{self, constants::DEFAULT_OUTPUT_DIR, reader::find_config_file},
};
use std::path::Path;

/// The entry point of the compilation process
///
/// Whenever this process gets triggered, the files declared within the
/// configuration file will be build.
///
/// TODO Decision path for building the executable command line,
/// the tests executable command line, a static lib, a dylib...
pub fn build_project(_cli_args: &CliArgs) -> Result<()> {
    let config_file: String =
        find_config_file().with_context(|| "Failed to read configuration file")?;
    let config: ZorkConfigFile = toml::from_str(config_file.as_str())
        .with_context(|| "Could not parse configuration file")?;

    // Create the directory for dump the generated files
    create_output_directory(&config)?;

    Ok(())
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
fn create_output_directory(config: &ZorkConfigFile) -> Result<()> {
    let out_dir = config.build.as_ref().map_or_else(
        || DEFAULT_OUTPUT_DIR,
        |build| build.output_dir.unwrap_or(DEFAULT_OUTPUT_DIR),
    );

    // Recursively create a directory and all of its parent components if they are missing
    let zork_path = Path::new(out_dir).join("zork");

    utils::fs::create_directory(&zork_path.join("cache"))?;
    utils::fs::create_directory(&zork_path.join("intrinsics"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use color_eyre::Result;

    use crate::utils::template::resources::CONFIG_FILE;
    use crate::utils::test;

    use super::*;

    #[test]
    fn test_creation_directories() -> Result<()> {
        test::in_temp_dir(|_temp| {
            let zcf: ZorkConfigFile = toml::from_str(CONFIG_FILE).unwrap();

            // This should create and out/ directory in the ./zork++ folder at the root of this project
            create_output_directory(&zcf)?;

            assert!(Path::new("./out").exists());
            assert!(Path::new("./out/zork").exists());
            assert!(Path::new("./out/zork/cache").exists());
            assert!(Path::new("./out/zork/intrinsics").exists());

            // Clean up the out directory created for testing purposes
            assert!(std::fs::remove_dir_all("./out").is_ok());

            Ok(())
        })
    }
}
