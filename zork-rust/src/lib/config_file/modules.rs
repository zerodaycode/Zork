use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct ModulesAttribute {
    pub base_ifcs_dir: Option< String >,
    pub interfaces: Option< Vec<String> >,
    pub base_impls_dir: Option< String >,
    pub implementations: Option< Vec<String> >
}