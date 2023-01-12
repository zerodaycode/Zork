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
///     base_ifcs_dir = "./ifc"
///     interfaces = [
///         { filename = 'math.cppm' }, { filename = 'some_module.cppm', module_name = 'math' }    
///     ]
///     base_impls_dir = './src'
///     implementations = [
///         'Implementation1', 'Implementation2'
///     ]
/// "#;
///
/// let config: ModulesAttribute = toml::from_str(CONFIG_FILE_MOCK)
///    .expect("A failure happened parsing the Zork toml file");
///
/// assert_eq!(config.base_ifcs_dir, Some("./ifc"));
/// 
/// let ifcs = config.interfaces.unwrap();
/// let ifc_0 = &ifcs[0];
/// assert_eq!(ifc_0.filename, "math.cppm");
/// assert_eq!(ifc_0.module_name, None);
/// let ifc_1 = &ifcs[1];
/// assert_eq!(ifc_1.filename, "some_module.cppm");
/// assert_eq!(ifc_1.module_name, Some("math"));
/// 
/// assert_eq!(config.base_impls_dir, Some("./src"));
/// assert_eq!(config.implementations, Some(vec!["Implementation1","Implementation2"]));
/// ```
#[derive(Deserialize, Debug, PartialEq)]
pub struct ModulesAttribute<'a> {
    #[serde(borrow)]
    pub base_ifcs_dir: Option<&'a str>,
    #[serde(borrow)]
    pub interfaces: Option<Vec<ModuleInterface<'a>>>,
    #[serde(borrow)]
    pub base_impls_dir: Option<&'a str>,
    #[serde(borrow)]
    pub implementations: Option<Vec<&'a str>>,
}

/// [`ModuleInterface`] -  A module interface structure
/// * `filename`- The filename of a primary module interface
/// * `module_name` - An optional field for make an explicit declaration of the
/// C++ module declared on this module interface with the `export module 'module_name'
/// statement. If this attribute isn't present, Zork++ will assume that the
/// C++ module declared within this file is equls to the filename
/// 
/// ### Tests
///
/// ```rust
/// use zork::config_file::modules::ModulesAttribute;
/// use zork::config_file::modules::ModuleInterface;
/// const CONFIG_FILE_MOCK: &str = r#"
///     interfaces = [
///         { filename = 'math.cppm' },
///         { filename = 'some_module.cppm', module_name = 'math' },
///         { filename = 'a.cppm', module_name = 'module', dependencies = ['math', 'type_traits', 'iostream'] }
///     ]
/// "#;
///
/// let config: ModulesAttribute = toml::from_str(CONFIG_FILE_MOCK)
///    .expect("A failure happened parsing the Zork toml file");
/// 
/// let ifcs = config.interfaces.unwrap();
/// 
/// let ifc_0 = &ifcs[0];
/// assert_eq!(ifc_0.filename, "math.cppm");
/// assert_eq!(ifc_0.module_name, None);
/// 
/// let ifc_1 = &ifcs[1];
/// assert_eq!(ifc_1.filename, "some_module.cppm");
/// assert_eq!(ifc_1.module_name, Some("math"));
///
/// let ifc_2 = &ifcs[2];
/// assert_eq!(ifc_2.filename, "a.cppm");
/// assert_eq!(ifc_2.module_name, Some("module"));
/// let deps = ifc_2.dependencies.as_ref().unwrap();
/// 
/// assert_eq!(deps[0], "math");
/// assert_eq!(deps[1], "type_traits");
/// assert_eq!(deps[2], "iostream");
/// ```
#[derive(Deserialize, Debug, PartialEq)]
pub struct ModuleInterface<'a> {
    #[serde(borrow)]
    pub filename: &'a str,
    #[serde(borrow)]
    pub module_name: Option<&'a str>,
    #[serde(borrow)]
    pub dependencies: Option<Vec<&'a str>>
}