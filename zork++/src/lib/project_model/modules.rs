use core::fmt;

use crate::bounds::TranslationUnit;

#[derive(Debug, PartialEq, Eq)]
pub struct ModulesModel {
    pub base_ifcs_dir: String,
    pub interfaces: Vec<ModuleInterfaceModel>,
    pub base_impls_dir: String,
    pub implementations: Vec<ModuleImplementationModel>,
    pub gcc_sys_headers: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleInterfaceModel {
    pub filename: String,
    pub module_name: String,
    pub dependencies: Vec<String>,
}

impl fmt::Display for ModuleInterfaceModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {:?}, {:?})",
            self.filename, self.module_name, self.dependencies
        )
    }
}

impl TranslationUnit for ModuleInterfaceModel {
    fn get_filename(&self) -> String {
        self.filename.to_string()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleImplementationModel {
    pub filename: String,
    pub dependencies: Vec<String>,
}

impl fmt::Display for ModuleImplementationModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {:?})", self.filename, self.dependencies)
    }
}

impl TranslationUnit for ModuleImplementationModel {
    fn get_filename(&self) -> String {
        self.filename.to_string()
    }
}
