use crate::{
    bounds::{ExecutableTarget, ExtraArgs},
    cli::output::arguments::Argument,
};

use super::sourceset::SourceSet;

#[derive(Debug, PartialEq, Eq)]
pub struct TestsModel {
    pub test_executable_name: String,
    pub sourceset: SourceSet,
    pub extra_args: Vec<Argument>,
}

impl<'a> ExtraArgs<'a> for TestsModel {
    fn extra_args(&'a self) -> &'a [Argument] {
        &self.extra_args
    }
}

impl<'a> ExecutableTarget<'a> for TestsModel {
    fn name(&'a self) -> &'a str {
        &self.test_executable_name
    }
    fn sourceset(&'a self) -> &'a SourceSet {
        &self.sourceset
    }
}
