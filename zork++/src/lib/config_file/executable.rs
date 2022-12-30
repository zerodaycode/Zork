///! Specify the execution configuration
use serde::*;

/// [`ExecutableAttribute`] -  The core section to instruct the compiler to work with C++20 modules.
/// The most important are the base path to the interfaces and implementation files
/// * `executable_name`- This is name to output executable file
/// * `sources_base_path` - Define directory source code
/// * `sources` - Define list extension files source code
/// * `auto_execute` - To run zork can execute after build proyect
/// * `extra_args` - Define other params to add execution executable
///
/// ### Tests
///
/// ```rust
/// use zork::config_file::executable::ExecutableAttribute;
/// const CONFIG_FILE_MOCK: &str = r#"
///     #[executable]
///     executable_name = "outputExecutableName"
///     sources_base_path = './src'
///     sources = [
///         '*.cpp'
///     ]
///     auto_execute = true
///     extra_args = 'example'
/// "#;
///
/// let config: ExecutableAttribute = toml::from_str(CONFIG_FILE_MOCK)
///    .expect("A failure happened parsing the Zork toml file");
///
/// assert_eq!(config.executable_name, Some("outputExecutableName"));
/// assert_eq!(config.sources_base_path, Some("./src"));
/// assert_eq!(config.sources, Some(vec!["*.cpp"]));
/// assert_eq!(config.auto_execute, Some(true));
/// assert_eq!(config.extra_args, Some("example"))
/// ```
/// > Note: TOML table are toml commented (#) to allow us to parse
/// the inner attributes as the direct type that they belongs to.
/// That commented tables aren't the real TOML, they are just there
/// for testing and exemplification purposes of the inner attributes
/// of the configuration file.
///
/// For a test over a real example, please look at the
/// [`zork::config_file::ZorkConfigFile`] doc-test
#[derive(Deserialize, Debug, PartialEq)]
pub struct ExecutableAttribute<'a> {
    #[serde(borrow)]
    pub executable_name: Option<&'a str>,
    #[serde(borrow)]
    pub sources_base_path: Option<&'a str>,
    #[serde(borrow)]
    pub sources: Option<Vec<&'a str>>,
    pub auto_execute: Option<bool>,
    #[serde(borrow)]
    pub extra_args: Option<&'a str>,
}
