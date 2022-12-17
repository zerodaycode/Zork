use serde::*;


#[derive(Deserialize, Debug, PartialEq)]
pub struct ExecutableAttribute {
    pub executable_name: Option< String >,
    pub sources_base_path: Option< String >,
    pub sources: Option< Vec<String> >,
    pub auto_execute: Option< bool >,
    pub extra_args: Option< String >
}