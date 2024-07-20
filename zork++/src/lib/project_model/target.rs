use crate::domain::target::TargetKind;
use crate::project_model::sourceset::SourceSet;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TargetModel<'a> {
    pub output_name: Cow<'a, str>,
    pub sources: SourceSet<'a>,
    pub extra_args: Vec<Cow<'a, str>>,
    pub kind: TargetKind,
}
