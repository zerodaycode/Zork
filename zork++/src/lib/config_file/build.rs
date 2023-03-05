///! file that contains the configuration options available
/// within Zork++ to configure the way of how it works
/// the build process
use serde::*;

/// [`BuildAttribute`] - Stores build process specific configuration
///
/// * `output_dir` - An string representing a relative to the root path
/// where the compiler should dump the files generated in the build process.
/// If isn't specified, `Zork++` will generate an `./out/...` folder
/// by default
///
/// ```rust
/// use zork::config_file::build::{BuildAttribute};
///
/// const CONFIG_FILE_MOCK: &str = r#"
///     #[build]
///     output_dir = 'out'
///"#;
///
/// let config: BuildAttribute = toml::from_str(CONFIG_FILE_MOCK)
///    .expect("A failure happened parsing the Zork toml file");
///
/// assert_eq!(config.output_dir, Some("out"));
/// ```
#[derive(Deserialize, Debug, PartialEq)]
#[serde (deny_unknown_fields)]
pub struct BuildAttribute<'a> {
    #[serde(borrow)]
    pub output_dir: Option<&'a str>,
}
