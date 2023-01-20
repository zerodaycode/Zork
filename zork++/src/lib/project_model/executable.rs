use crate::bounds::ExtraArgs;

#[derive(Debug, PartialEq, Eq)]
pub struct ExecutableModel {
    pub executable_name: String,
    pub sources_base_path: String,
    pub sources: Vec<String>,
    pub extra_args: Vec<String>,
}

impl ExtraArgs for ExecutableModel {
    fn get_extra_args(&self) -> Option<Vec<&str>> {
        todo!()
    }

    fn get_extra_args_alloc(&self) -> Vec<String> {
        self.extra_args.clone()
    }
}