//! Specify the execution configuration
use serde::*;

/// [`ExecutableAttribute`] -  The core section to instruct the compiler to work with C++20 modules.
/// The most important are the base path to the interfaces and implementation files
/// * `executable_name`- The name that the final binary is to be generated with
/// * `sources_base_path` - Optional param for specify where the non-modules source files lives
/// * `sources` - The sources to be included in the project
/// * `extra_args` - Holds extra arguments that the user wants to introduce
/// in the build process
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
///     extra_args = ['example']
/// "#;
///
/// let config: ExecutableAttribute = toml::from_str(CONFIG_FILE_MOCK)
///    .expect("A failure happened parsing the Zork toml file");
///
/// assert_eq!(config.executable_name, Some("outputExecutableName"));
/// assert_eq!(config.sources_base_path, Some("./src"));
/// assert_eq!(config.sources, Some(vec!["*.cpp"]));
/// assert_eq!(config.extra_args, Some(vec!["example"]))
/// ```
/// > Note: TOML table are toml commented (#) to allow us to parse
/// the inner attributes as the direct type that they belongs to.
/// That commented tables aren't the real TOML, they are just there
/// for testing and exemplification purposes of the inner attributes
/// of the configuration file.
///
/// For a test over a real example, please look at the
/// [`zork::config_file::ZorkConfigFile`] doc-test
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ExecutableAttribute<'a> {
    #[serde(borrow)]
    pub executable_name: Option<&'a str>,
    #[serde(borrow)]
    pub sources_base_path: Option<&'a str>,
    #[serde(borrow)]
    pub sources: Option<Vec<&'a str>>,
    #[serde(borrow)]
    pub extra_args: Option<Vec<&'a str>>,
}
