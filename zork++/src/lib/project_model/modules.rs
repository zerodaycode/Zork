use core::fmt;
use std::path::{Path, PathBuf};

use crate::{bounds::TranslationUnit, config_file::modules::ModulePartition};

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
    pub path: PathBuf,
    pub file_stem: String,
    pub extension: String,
    pub module_name: &'a str,
    pub partition: Option<ModulePartitionModel<'a>>,
    pub dependencies: Vec<&'a str>,
}

impl<'a> fmt::Display for ModuleInterfaceModel<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:?}.{:?}., {:?}, {:?}, {:?})",
            self.path, self.file_stem, self.module_name, self.dependencies, self.partition
        )
    }
}

impl<'a> TranslationUnit for ModuleInterfaceModel<'a> {
    fn file(&self) -> PathBuf {
        let mut tmp = self.path.join(&self.file_stem).into_os_string();
        tmp.push(".");
        tmp.push(&self.extension);
        PathBuf::from(tmp)
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn file_stem(&self) -> String {
        self.file_stem.clone()
    }

    fn extension(&self) -> String {
        self.extension.clone()
    }
}

impl<'a> TranslationUnit for &'a ModuleInterfaceModel<'a> {
    fn file(&self) -> PathBuf {
        let mut tmp = self.path.join(&self.file_stem).into_os_string();
        tmp.push(".");
        tmp.push(&self.extension);
        PathBuf::from(tmp)
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn file_stem(&self) -> String {
        self.file_stem.clone()
    }

    fn extension(&self) -> String {
        self.extension.clone()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModulePartitionModel<'a> {
    pub module: &'a str,
    pub partition_name: &'a str,
    pub is_internal_partition: bool,
}

impl<'a> From<&ModulePartition<'a>> for ModulePartitionModel<'a> {
    fn from(value: &ModulePartition<'a>) -> Self {
        Self {
            module: value.module,
            partition_name: value.partition_name.unwrap_or_default(),
            is_internal_partition: value.is_internal_partition.unwrap_or_default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleImplementationModel<'a> {
    pub path: PathBuf,
    pub file_stem: String,
    pub extension: String,
    pub dependencies: Vec<&'a str>,
}

impl<'a> fmt::Display for ModuleImplementationModel<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.path, self.dependencies)
    }
}

impl<'a> TranslationUnit for ModuleImplementationModel<'a> {
    fn file(&self) -> PathBuf {
        let mut tmp = self.path.join(&self.file_stem).into_os_string();
        tmp.push(".");
        tmp.push(&self.extension);
        PathBuf::from(tmp)
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn file_stem(&self) -> String {
        self.file_stem.clone()
    }

    fn extension(&self) -> String {
        self.extension.clone()
    }
}

impl<'a> TranslationUnit for &'a ModuleImplementationModel<'a> {
    fn file(&self) -> PathBuf {
        let mut tmp = self.path.join(&self.file_stem).into_os_string();
        tmp.push(".");
        tmp.push(&self.extension);
        PathBuf::from(tmp)
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn file_stem(&self) -> String {
        self.file_stem.clone()
    }

    fn extension(&self) -> String {
        self.extension.clone()
    }
}
