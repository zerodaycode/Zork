//! Metadata about the user's project
use serde::*;

/// [`ProjectAttribute`] - Metadata about the user's project
/// * `name` - The C++ project's name
/// * `authors` - A comma separated list of strings indicating the
/// authors that are responsible for the project
///
/// ### Tests
///
/// ```rust
/// use zork::config_file::project::ProjectAttribute;
///
/// const CONFIG_FILE_MOCK: &str = r#"
///     #[project]
///     name = 'Zork++ serde tests'
///     authors = ['zerodaycode.gz@gmail.com']
///     compilation_db = true
///"#;
///
/// let config: ProjectAttribute = toml::from_str(CONFIG_FILE_MOCK)
///    .expect("A failure happened parsing the Zork toml file");
///
/// assert_eq!(config.name, "Zork++ serde tests");
/// assert_eq!(config.authors, Some(vec!["zerodaycode.gz@gmail.com"]));
/// assert_eq!(config.compilation_db, Some(true));
/// assert_eq!(config.project_root, None);
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
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct ProjectAttribute<'a> {
    pub name: &'a str,
    #[serde(borrow)]
    pub authors: Option<Vec<&'a str>>,
    pub compilation_db: Option<bool>,
    pub project_root: Option<&'a str>,
}
