use core::fmt;

use super::TranslationUnit;

#[derive(Debug, PartialEq, Eq)]
pub struct ModulesModel<'a> {
    pub base_ifcs_dir: &'a str,
    pub interfaces: Vec<ModuleInterfaceModel<'a>>,
    pub base_impls_dir: &'a str,
    pub implementations: Vec<ModuleImplementationModel<'a>>,
    pub gcc_sys_headers: Vec<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleInterfaceModel<'a> {
    pub filename: &'a str,
    pub module_name: &'a str,
    pub dependencies: Vec<&'a str>,
}

impl<'a> fmt::Display for ModuleInterfaceModel<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {:?}, {:?})",
            self.filename, self.module_name, self.dependencies
        )
    }
}

impl<'a> TranslationUnit for ModuleInterfaceModel<'a> {
    fn filename(&self) -> &str {
        self.filename
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleImplementationModel<'a> {
    pub filename: &'a str,
    pub dependencies: Vec<&'a str>,
}

impl<'a> fmt::Display for ModuleImplementationModel<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {:?})", self.filename, self.dependencies)
    }
}

impl<'a> TranslationUnit for ModuleImplementationModel<'a> {
    fn filename(&self) -> &str {
        self.filename
    }
}
