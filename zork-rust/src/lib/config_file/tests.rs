use serde::*;

#[derive(Deserialize, Debug, PartialEq)]
pub struct TestsAttribute {
    pub tests_executable_name: Option< Vec<String> >,
    pub sources_base_path: Option< Vec<String> >,
    pub sources: Option< Vec<String> >,
    pub auto_run_tests: Option< bool >,
    pub extra_args: Option< String >
}