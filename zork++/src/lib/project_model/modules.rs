use core::fmt;
use std::path::Path;

use crate::bounds::TranslationUnit;

#[derive(Debug, PartialEq, Eq)]
pub struct ModulesModel<'a> {
    pub base_ifcs_dir: &'a Path,
    pub interfaces: Vec<ModuleInterfaceModel<'a>>,
    pub base_impls_dir: &'a Path,
    pub implementations: Vec<ModuleImplementationModel<'a>>,
    pub sys_modules: Vec<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleInterfaceModel<'a> {
    pub file: &'a Path,
    pub module_name: &'a str,
    pub dependencies: Vec<&'a str>,
}

impl<'a> fmt::Display for ModuleInterfaceModel<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:?}, {:?}, {:?})",
            self.file, self.module_name, self.dependencies
        )
    }
}

impl<'a> TranslationUnit for ModuleInterfaceModel<'a> {
    fn file(&self) -> &Path {
        self.file
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleImplementationModel<'a> {
    pub file: &'a Path,
    pub dependencies: Vec<&'a str>,
}

impl<'a> fmt::Display for ModuleImplementationModel<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.file, self.dependencies)
    }
}

impl<'a> TranslationUnit for ModuleImplementationModel<'a> {
    fn file(&self) -> &Path {
        self.file
    }
}
