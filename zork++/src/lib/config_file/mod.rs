//! root file for the crate where the datastructures that holds the TOML
//! parsed data lives.
pub mod project;
pub mod compiler;
pub mod build;
pub mod modules;
pub mod target;
pub mod executable;
pub mod tests;

use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use self::{
    build::BuildAttribute, compiler::CompilerAttribute, executable::ExecutableAttribute,
    modules::ModulesAttribute, target::TargetAttribute, project::ProjectAttribute, tests::TestsAttribute
};

/// ```rust
/// use zork::config_file::{
///     ZorkConfigFile,
///     compiler::{CppCompiler, LanguageLevel},
///     target::TargetAttribute
/// };
/// use zork::domain::target::TargetKind;
/// use std::collections::HashMap;
///
/// const CONFIG_FILE_MOCK: &str = r#"
///     [project]
///     name = 'Zork++ serde tests'
///     authors = ['zerodaycode.gz@gmail.com']
///
///     [compiler]
///     cpp_compiler = 'clang'
///     cpp_standard = '20'
///
///     [targets.executable]
///     output_name = 'final binary'
///     sources = [ 'main.cpp' ]
///     extra_args = [ '-Wall' ]
///
///     [targets.tests]
///     sources = [ 'tests_main.cpp' ]
///     target_kind = 'executable'

///     [targets.other_tests]
///     sources = [ 'other_tests_main.cpp' ]
///     target_kind = 'executable'
/// "#;
///
/// let config: ZorkConfigFile = toml::from_str(CONFIG_FILE_MOCK)
///     .expect("A failure happened parsing the Zork toml file");
///
/// let compiler_attribute = &config.compiler;
/// assert_eq!(compiler_attribute.cpp_compiler, CppCompiler::CLANG);
/// assert_eq!(compiler_attribute.cpp_standard, LanguageLevel::CPP20);
///
/// let targets: &HashMap<&str, TargetAttribute<'_>> = &config.targets;
/// assert!(!targets.is_empty());
///
/// let executable_target: &TargetAttribute<'_> = targets.get("executable").expect("Target named
///     'executable' not found on the configuration");
/// assert!(executable_target.output_name.unwrap().contains("final binary"));
/// assert!(executable_target.sources.contains(&"main.cpp"));
/// assert!(executable_target.extra_args.as_ref().unwrap().contains(&"-Wall"));
/// assert!(executable_target.kind.unwrap_or_default().eq(&TargetKind::Executable));
///
/// let tests_target: &TargetAttribute<'_> = targets.get("tests").expect("Target named
///     'tests' not found on the configuration");
/// assert!(tests_target.sources.contains(&"tests_main.cpp"));
/// assert!(tests_target.extra_args.is_none());
/// assert!(tests_target.kind.unwrap_or_default().eq(&TargetKind::Executable));
///
/// let other_tests_target: &TargetAttribute<'_> = targets.get("other_tests").expect("Target named
///     'other_tests' not found on the configuration");
/// assert!(other_tests_target.sources.contains(&"other_tests_main.cpp"));
/// assert!(other_tests_target.extra_args.is_none());
/// assert!(other_tests_target.kind.unwrap_or_default().eq(&TargetKind::Executable));
/// ```
/// The [`ZorkConfigFile`] is the type that holds
/// the whole hierarchy of Zork++ config file attributes
/// and properties
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ZorkConfigFile<'a> {
    #[serde(borrow)]
    pub project: ProjectAttribute<'a>,
    #[serde(borrow)]
    pub compiler: CompilerAttribute<'a>,
    #[serde(borrow)]
    pub build: Option<BuildAttribute<'a>>,
    #[serde(borrow)]
    pub modules: Option<ModulesAttribute<'a>>,
    #[serde(borrow)]
    pub targets: HashMap<&'a str, TargetAttribute<'a>>,
    #[serde(borrow)]
    pub executable: Option<ExecutableAttribute<'a>>,
    #[serde(borrow)]
    pub tests: Option<TestsAttribute<'a>>,
}

pub fn zork_cfg_from_file(cfg: &'_ str) -> Result<ZorkConfigFile<'_>, toml::de::Error> {
    <ZorkConfigFile>::deserialize(&mut toml::Deserializer::new(cfg))
}
