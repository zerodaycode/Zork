use core::fmt;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

use transient::Transient;

use crate::cli::output::arguments::Argument;
use crate::config_file::modules::ModulePartition;
use crate::domain::target::ExtraArgs;
use crate::domain::translation_unit::TranslationUnit;
use crate::impl_translation_unit_for;

#[derive(Debug, PartialEq, Eq)]
pub struct ModulesModel<'a> {
    pub base_ifcs_dir: &'a Path,
    pub interfaces: Vec<ModuleInterfaceModel<'a>>,
    pub base_impls_dir: &'a Path,
    pub implementations: Vec<ModuleImplementationModel<'a>>,
    pub sys_modules: Vec<SystemModule<'a>>,
    pub extra_args: Vec<Argument<'a>>,
}

impl<'a> ExtraArgs<'a> for ModulesModel<'a> {
    fn extra_args(&'a self) -> &'a [Argument] {
        &self.extra_args
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Transient)]
pub struct ModuleInterfaceModel<'a> {
    pub path: PathBuf,
    pub file_stem: Cow<'a, str>,
    pub extension: Cow<'a, str>,
    pub module_name: Cow<'a, str>,
    pub partition: Option<ModulePartitionModel<'a>>,
    pub dependencies: Vec<Cow<'a, str>>,
}

impl_translation_unit_for!(ModuleInterfaceModel<'a>);

impl<'a> fmt::Display for ModuleInterfaceModel<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:?}, {:?}, {:?}, {:?})",
            self.path(),
            self.module_name,
            self.dependencies,
            self.partition
        )
    }
}

#[derive(Debug, PartialEq, Eq, Transient, Clone)]
pub struct ModulePartitionModel<'a> {
    pub module: Cow<'a, str>,
    pub partition_name: Cow<'a, str>,
    pub is_internal_partition: bool,
}

impl<'a> From<ModulePartition<'a>> for ModulePartitionModel<'a> {
    fn from(value: ModulePartition<'a>) -> Self {
        Self {
            module: Cow::Borrowed(value.module),
            partition_name: Cow::Borrowed(value.partition_name.unwrap_or_default()),
            is_internal_partition: value.is_internal_partition.unwrap_or_default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Transient)]
pub struct ModuleImplementationModel<'a> {
    pub path: PathBuf,
    pub file_stem: Cow<'a, str>,
    pub extension: Cow<'a, str>,
    pub dependencies: Vec<Cow<'a, str>>,
}

impl_translation_unit_for!(ModuleImplementationModel<'a>);

impl<'a> fmt::Display for ModuleImplementationModel<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.path(), self.dependencies)
    }
}

/// Holds the fs information about the `C++` system headers, which they can be built as
/// binary module interface for certain compilers, while allowing to import those system headers
/// as modules
#[derive(Debug, PartialEq, Eq, Default, Transient)]
pub struct SystemModule<'a> {
    pub path: PathBuf,
    pub file_stem: Cow<'a, str>,
    pub extension: Cow<'a, str>,
}

impl_translation_unit_for!(SystemModule<'a>);

impl<'a> fmt::Display for SystemModule<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.path())
    }
}
