use super::{ExecutableTarget, ExtraArgs};

#[derive(Debug, PartialEq, Eq)]
pub struct ExecutableModel<'a> {
    pub executable_name: &'a str,
    pub sources_base_path: &'a str,
    pub sources: Vec<&'a str>,
    pub extra_args: Vec<&'a str>,
}

impl<'a> ExtraArgs<'a> for ExecutableModel<'a> {
    fn extra_args(&'a self) -> &'a [&'a str] {
        &self.extra_args
    }
}

impl<'a> ExecutableTarget<'a> for ExecutableModel<'a> {
    fn name(&'a self) -> &'a str {
        self.executable_name
    }

    fn sources_base_path(&'a self) -> &'a str {
        self.sources_base_path
    }

    fn sources(&'a self) -> &'a [&'a str] {
        &self.sources
    }
}
