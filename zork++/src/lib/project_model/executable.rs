use crate::bounds::ExtraArgs;

#[derive(Debug, PartialEq, Eq)]
pub struct ExecutableModel<'a> {
    pub executable_name: &'a str,
    pub sources_base_path: &'a str,
    pub sources: Vec<&'a str>,
    pub extra_args: Vec<&'a str>,
}

impl<'a> ExtraArgs for ExecutableModel<'a> {
    fn get_extra_args(&self) -> Option<Vec<&str>> {
        todo!()
    }

    fn get_extra_args_alloc(&self) -> Vec<String> {
        todo!()
    }
}
