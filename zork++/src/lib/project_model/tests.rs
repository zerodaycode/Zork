use super::{arguments::Argument, ExecutableTarget, ExtraArgs};

#[derive(Debug, PartialEq, Eq)]
pub struct TestsModel<'a> {
    pub test_executable_name: String,
    pub source_base_path: &'a str,
    pub sources: Vec<&'a str>,
    pub extra_args: Vec<Argument<'a>>,
}

impl<'a> ExtraArgs<'a> for TestsModel<'a> {
    fn extra_args(&'a self) -> &'a [Argument<'a>] {
        &self.extra_args
    }
}

impl<'a> ExecutableTarget<'a> for TestsModel<'a> {
    fn name(&'a self) -> &'a str {
        &self.test_executable_name
    }

    fn sources_base_path(&'a self) -> &'a str {
        self.source_base_path
    }

    fn sources(&'a self) -> &'a [&'a str] {
        &self.sources
    }
}
