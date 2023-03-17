#[derive(Debug, PartialEq, Eq)]
pub struct ProjectModel<'a> {
    pub name: &'a str,
    pub authors: &'a [&'a str],
    pub compilation_db: bool,
    pub project_root: Option<&'a str>,
}
