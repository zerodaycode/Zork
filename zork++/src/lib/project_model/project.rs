use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProjectModel<'a> {
    pub name: Cow<'a, str>,
    pub authors: Vec<Cow<'a, str>>,
    pub compilation_db: bool,
    pub code_root: Option<Cow<'a, str>>,
}
