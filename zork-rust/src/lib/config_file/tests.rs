///! Tests attribute allows you to run the tests written for your application in a convenient way


use serde::*;


/// [`TestsAttribute`] - Tests attribute allows you to run the tests written for your application in a convenient way
/// * `tests_executable_name` - This is names executions test
/// * `sources_base_path` - Directoties source tests
/// * `sources` - Define extension files to tests
/// * `auto_run_tests` - Define if required run test after build 
/// * `extra_args`- Define command to added properties test execution
/// 
/// ### Tests
/// 
/// ```rust
/// use zork::config_file::tests::TestsAttribute;
/// 
/// const CONFIG_FILE_MOCK: &str = r#"
///     #[tests]
///     test_executable_name = 'Zork++ tests'
///     source_base_path = 'path_test'
///     sources = [ '*.cpp' ]
///     auto_run_tests = true
///     extra_args = 'extra_argument to run test'
///"#;
///
/// let config: TestsAttribute = toml::from_str(CONFIG_FILE_MOCK)
///    .expect("A failure happened parsing the Zork toml file");
///
/// assert_eq!(config.test_executable_name, Some("Zork++ tests"));
/// assert_eq!(config.source_base_path, Some("path_test"));
/// assert_eq!(config.sources, Some(vec!["*.cpp"]));
/// assert_eq!(config.auto_run_tests, Some(true));
/// assert_eq!(config.extra_args, Some("extra_argument to run test"));
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
#[derive(Deserialize, Debug, PartialEq)]
pub struct TestsAttribute<'a> {
    #[serde(borrow)] pub test_executable_name: Option<&'a str>,
    #[serde(borrow)] pub source_base_path: Option<&'a str>,
    #[serde(borrow)] pub sources: Option<Vec<&'a str>>,
    pub auto_run_tests: Option<bool>,
    #[serde(borrow)] pub extra_args: Option<&'a str>
}