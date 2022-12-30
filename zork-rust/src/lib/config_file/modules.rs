///!  The core section to instruct the compiler to work with C++20 modules. The most important are the base path to the interfaces and implementation files


use serde::Deserialize;

/// [`ModulesAttribute`] -  The core section to instruct the compiler to work with C++20 modules. The most important are the base path to the interfaces and implementation files
/// * `base_ifcs_dir`- Can use to define base interfaces directory
/// * `interfaces` - Can use to define a list with extension interface files
/// * `base_impls_dir` - This is base directory implementation
/// * `implementations` - This a list to define with implementations files
/// 
/// ### Tests
/// 
/// ```rust
/// use zork::config_file::modules::ModulesAttribute;
/// const CONFIG_FILE_MOCK: &str = r#"
///     #[module]
///     base_ifcs_dir = "./ifc"
///     interfaces = [
///         'Interface1', 'Interface2'    
///     ]
///     base_impls_dir = './src' 
///     implementations = [
///         'Implementation1','Implementation2'
///     ]
/// "#;
/// 
/// let config: ModulesAttribute = toml::from_str(CONFIG_FILE_MOCK)
///    .expect("A failure happened parsing the Zork toml file");
///
/// assert_eq!(config.base_ifcs_dir, Some("./ifc"));
/// assert_eq!(config.interfaces, Some(vec!["Interface1","Interface2"]));
/// assert_eq!(config.base_impls_dir, Some("./src"));
/// assert_eq!(config.implementations, Some(vec!["Implementation1","Implementation2"]));
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
pub struct ModulesAttribute<'a> {
    #[serde(borrow)] pub base_ifcs_dir: Option<&'a str>,
    #[serde(borrow)] pub interfaces: Option<Vec<&'a str>>,
    #[serde(borrow)] pub base_impls_dir: Option<&'a str>,
    #[serde(borrow)] pub implementations: Option<Vec<&'a str>>
}