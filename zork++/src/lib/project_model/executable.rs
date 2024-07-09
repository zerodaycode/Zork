use std::borrow::Cow;

use super::sourceset::SourceSet;
use crate::{
    cli::output::arguments::Argument,
    domain::target::{ExecutableTarget, ExtraArgs},
};

#[derive(Debug, PartialEq, Eq)]
pub struct ExecutableModel<'a> {
    pub executable_name: Cow<'a, str>,
    pub sourceset: SourceSet<'a>,
    pub extra_args: Vec<Argument>,
}

impl<'a> ExtraArgs<'a> for ExecutableModel<'a> {
    fn extra_args(&'a self) -> &'a [Argument] {
        &self.extra_args
    }
}

impl<'a> ExecutableTarget<'a> for ExecutableModel<'a> {
    fn name(&'a self) -> &'a str {
        self.executable_name.as_ref()
    }
    fn sourceset(&'a self) -> &'a SourceSet {
        &self.sourceset
    }
}
