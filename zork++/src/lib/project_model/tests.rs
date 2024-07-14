use serde::{Deserialize, Serialize};

use crate::{
    cli::output::arguments::Argument,
    domain::target::{ExecutableTarget, ExtraArgs},
};
use std::borrow::Cow;

use super::sourceset::SourceSet;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TestsModel<'a> {
    pub test_executable_name: Cow<'a, str>,
    pub sourceset: SourceSet<'a>,
    pub extra_args: Vec<Argument<'a>>,
}

impl<'a> ExtraArgs<'a> for TestsModel<'a> {
    fn extra_args(&'a self) -> &'a [Argument] {
        &self.extra_args
    }
}

impl<'a> ExecutableTarget<'a> for TestsModel<'a> {
    fn name(&'a self) -> &'a str {
        &self.test_executable_name
    }
    fn sourceset(&'a self) -> &'a SourceSet {
        &self.sourceset
    }
}
