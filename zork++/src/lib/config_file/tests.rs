///! Tests attribute allows you to run the tests written for your application in a convenient way
use serde::*;

/// [`TestsAttribute`] - Tests attribute allows you to run the tests written for your application in a convenient way
/// * `tests_executable_name` - The name of the generated binary
/// * `sources_base_path` - Base common path for the source files
/// * `sources` - All the source files that must be included
/// * `extra_args`- Extra arguments to add to the generated command line(s)
///
/// ### Tests
///
/// ```rust
/// use zork::config_file::tests::TestsAttribute;
///
/// const CONFIG_FILE_MOCK: &str = r#"
///     #[tests]
///     test_executable_name = 'Zork++ tests'
///     sources_base_path = 'path_test'
///     sources = [ '*.cpp' ]
///     extra_args = ['extra_argument to run test']
///"#;
///
/// let config: TestsAttribute = toml::from_str(CONFIG_FILE_MOCK)
///    .expect("A failure happened parsing the Zork toml file");
///
/// assert_eq!(config.test_executable_name, Some("Zork++ tests"));
/// assert_eq!(config.sources_base_path, Some("path_test"));
/// assert_eq!(config.sources, Some(vec!["*.cpp"]));
/// assert_eq!(config.extra_args, Some(vec!["extra_argument to run test"]));
///
/// ```
///
/// > Note: TOML table are toml commented (#) to allow us to parse
/// the inner attributes as the direct type that they belongs to.
/// That commented tables aren't the real TOML, they are just there
/// for testing and exemplification purposes of the inner attributes
/// of the configuration file.
///
/// For a test over a real example, please look at the
/// [`zork::config_file::ZorkConfigFile`] doc-test
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct TestsAttribute<'a> {
    #[serde(borrow)]
    pub test_executable_name: Option<&'a str>,
    #[serde(borrow)]
    pub sources_base_path: Option<&'a str>,
    #[serde(borrow)]
    pub sources: Option<Vec<&'a str>>,
    #[serde(borrow)]
    pub extra_args: Option<Vec<&'a str>>,
}
