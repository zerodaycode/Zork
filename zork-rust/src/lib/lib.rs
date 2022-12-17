use serde::*;

#[test]
fn load_compiler_config_from_array() {
    const CONFIG_FILE_MOCK: &str = r#"
        [compiler]
        cpp_compiler = 'clang'
    "#;

    let config: ZorkConfig = toml::from_str(CONFIG_FILE_MOCK)
        .expect("A failure happened parsing the Zork toml file");

    let compiler_attribute = &config.compiler;

    assert_eq!(compiler_attribute.cpp_compiler, CppCompiler::CLANG);
}

#[derive(Deserialize, Debug, Clone)]
pub struct ZorkConfig {
    pub project: ProjectAttribute,
    pub compiler: CompilerAttibute,
    pub language: LanguageAttribute,
    pub build: Option< BuildAttribute >,
    pub executable: Option< ExecutableAttribute >,
    pub modules: Option< ModulesAttribute >,
    pub tests: Option< TestAttribute >,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ProjectAttribute {
    pub name: String,
    pub authors: Option< Vec< String> >
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct CompilerAttibute {
    pub cpp_compiler: SupportCompiler,
    pub extra_args: Option<String>,
    pub system_headers_path: Option<String>
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum SupportCompiler {
    #[serde(alias = "CLANG++", alias = "clang++",alias = "CLANG", alias = "clang")]
    CLANG,
    #[serde(alias = "GCC", alias = "GCC")]
    GCC,
    #[serde(alias = "MSVC", alias = "msvc")]
    MSVC
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct LanguageAttribute {
    pub cpp_standard: u8,
    pub std_lib: Option< String >,
    pub modules: Option< bool >
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct BuildAttribute {
    pub output_dir: Option< String >
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ExecutableAttribute {
    pub executable_name: Option< String >,
    pub sources_base_path: Option< String >,
    pub sources: Option< Vec<String> >,
    pub auto_execute: Option< bool >,
    pub extra_args: Option< String >
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ModulesAttribute {
    pub base_ifcs_dir: Option< String >,
    pub interfaces: Option< Vec<String> >,
    pub base_impls_dir: Option< String >,
    pub implementations: Option< Vec<String> >
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct TestAttribute {
    pub tests_executable_name: Option< Vec<String> >,
    pub sources_base_path: Option< Vec<String> >,
    pub sources: Option< Vec<String> >,
    pub auto_run_tests: Option< bool >,
    pub extra_args: Option< String >
}


#[derive(Deserialize, Debug, Clone)]
pub struct Compiler {
    pub cpp_compiler: CppCompiler,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum CppCompiler {
    #[serde(alias="CLANG", alias="Clang", alias="clang")]
    CLANG,
    #[serde(alias="MSVC", alias="Msvc", alias="msvc")]
    MSVC,
    #[serde(alias="GCC", alias="Gcc", alias="gcc")]
    GCC
}