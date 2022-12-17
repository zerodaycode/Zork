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
    pub compiler: Compiler,
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