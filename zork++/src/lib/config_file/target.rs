//! Type for holds the Targets build details

use serde::{Deserialize, Serialize};

use crate::domain::target::TargetKind;

/// [`TargetAttribute`] - The type for holding the build details of every
/// user defined target
/// * `output_name`- The name with which the final byproduct will be generated
/// * `sources` - The sources to be included in the compilation of this target
/// * `extra_args` - Holds extra arguments that the user wants to introduce
/// * `kind` - Determined which type of byproduct will be generated (binary, library...)
///
/// ### Tests
///
/// ```rust
/// use zork::config_file::target::TargetAttribute;
/// const CONFIG_FILE_MOCK: &str = r#"
///     #[target.executable]
///     output_name = "some_executable"
///     sources = [ '*.cpp' ]
///     extra_args = ['-Wall']
///     kind = "Executable"
/// "#;
///
/// let config: TargetAttribute = toml::from_str(CONFIG_FILE_MOCK)
///    .expect("A failure happened parsing the Zork toml file");
///
/// assert_eq!(config.executable_name, Some("some_executable"));
/// assert_eq!(config.sources, Some(vec!["*.cpp"]));
/// assert_eq!(config.extra_args, Some(vec!["-Wall"]))
/// assert_eq!(config.kind, TargetKind::Executable)
/// ```
/// > Note: TOML table are toml commented (#) to allow us to parse
/// the inner attributes as the direct type that they belongs to.
/// That commented tables aren't the real TOML, they are just there
/// for testing and exemplification purposes of the inner attributes
/// of the configuration file.
///
/// For a test over a real example, please look at the
/// [`zork::config_file::ZorkConfigFile`] doc-test
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TargetAttribute<'a> {
    pub output_name: Option<&'a str>,
    pub sources: Vec<&'a str>,
    pub extra_args: Option<Vec<&'a str>>,
    pub kind: Option<TargetKind>,
}
