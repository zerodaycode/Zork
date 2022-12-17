///! root file for the crate where the datastructures that holds the TOML
/// parsed data lives.

pub mod project;
pub mod compiler;
pub mod language;
pub mod build;
pub mod executable;
pub mod modules;
pub mod tests;

use serde::Deserialize;

use self::{
    project::ProjectAttribute,
    compiler::CompilerAttribute,
    language::LanguageAttribute,
    build::BuildAttribute,
    executable::ExecutableAttribute,
    modules::ModulesAttribute,
    tests::TestsAttribute
};

// /// ```
// #[test]
// fn load_compiler_config_from_array() {
//     const CONFIG_FILE_MOCK: &str = r#"
//         [project]
//         name = 'Zork++ serde tests'
//         authors = ['zerodaycode.gz@gmail.com']

//         [language]
//         cpp_compiler = 'clang'

//         [compiler]
//         cpp_compiler = 'clang'
//     "#;

//     let config: ZorkConfigFile = toml::from_str(CONFIG_FILE_MOCK)
//         .expect("A failure happened parsing the Zork toml file");

//     let compiler_attribute = &config.compiler;

//     assert_eq!(compiler_attribute.cpp_compiler, CppCompiler::CLANG);
// }
// /// ```
/// The [`ZorkConfigFile`] is the type that holds
/// the whole hierarchy of Zork++ config file attributes
/// and properties
#[derive(Deserialize, Debug)]
pub struct ZorkConfigFile<'a> {
    #[serde(borrow)] pub project: ProjectAttribute<'a>,
    #[serde(borrow)] pub compiler: CompilerAttribute<'a>,
    pub language: LanguageAttribute,
    pub build: Option<BuildAttribute>,
    pub executable: Option<ExecutableAttribute>,
    pub modules: Option<ModulesAttribute>,
    pub tests: Option<TestsAttribute>
}