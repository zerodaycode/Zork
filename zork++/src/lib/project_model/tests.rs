#[derive(Debug, PartialEq, Eq)]
pub struct TestsModel<'a> {
    pub test_executable_name: String,
    pub source_base_path: &'a str,
    pub sources: Vec<&'a str>,
    pub extra_args: Vec<&'a str>,
}
