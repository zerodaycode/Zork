#[derive(Debug, PartialEq, Eq)]
pub struct TestsModel {
    pub test_executable_name: String,
    pub source_base_path: String,
    pub sources: Vec<String>,
    pub extra_args: Vec<String>,
}
