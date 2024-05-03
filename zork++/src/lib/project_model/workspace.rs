#[derive(Debug, PartialEq, Eq)]
pub struct WorkspaceModel<'a> {
    pub members: Vec<&'a str>,
}
