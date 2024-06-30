use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq)]
pub struct ProjectModel<'a> {
    pub name: Cow<'a, str>,
    pub authors: Vec<Cow<'a, str>>,
    pub compilation_db: bool,
    pub project_root: Option<Cow<'a, str>>,
}
