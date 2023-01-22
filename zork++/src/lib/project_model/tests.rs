use crate::bounds::{ExecutableTarget, ExtraArgs};

use super::{arguments::Argument, sourceset::SourceSet};

#[derive(Debug, PartialEq, Eq)]
pub struct TestsModel<'a> {
    pub test_executable_name: String,
    pub sourceset: SourceSet<'a>,
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

    fn sourceset(&'a self) -> &'a SourceSet<'a> {
        &self.sourceset
    }
}
