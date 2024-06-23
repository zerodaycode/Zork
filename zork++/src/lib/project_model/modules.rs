use core::fmt;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

use crate::bounds::ExtraArgs;
use crate::cli::output::arguments::Argument;
use crate::{bounds::TranslationUnit, config_file::modules::ModulePartition};

#[derive(Debug, PartialEq, Eq)]
pub struct ModulesModel<'a> {
    pub base_ifcs_dir: &'a Path,
    pub interfaces: Vec<ModuleInterfaceModel<'a>>,
    pub base_impls_dir: &'a Path,
    pub implementations: Vec<ModuleImplementationModel<'a>>,
    pub sys_modules: Vec<Cow<'a, str>>,
    pub extra_args: Vec<Argument>,
}

impl<'a> ExtraArgs<'a> for ModulesModel<'a> {
    fn extra_args(&'a self) -> &'a [Argument] {
        &self.extra_args
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleInterfaceModel<'a> {
    pub path: PathBuf,
    pub file_stem: Cow<'a, str>,
    pub extension: Cow<'a, str>,
    pub module_name: Cow<'a, str>,
    pub partition: Option<ModulePartitionModel<'a>>,
    pub dependencies: Vec<Cow<'a, str>>,
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
        self.path
            .join::<&str>(&self.file_stem)
            .join::<&str>(&self.extension)
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn file_stem(&self) -> Cow<'_, str> {
        self.file_stem.clone()
    }

    fn extension(&self) -> Cow<'_, str> {
        self.extension.clone()
    }
}

impl<'a> TranslationUnit for &'a ModuleInterfaceModel<'a> {
    fn file(&self) -> PathBuf {
        self.path
            .join::<&str>(&self.file_stem)
            .join::<&str>(&self.extension)
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn file_stem(&self) -> Cow<'_, str> {
        self.file_stem.clone()
    }

    fn extension(&self) -> Cow<'a, str> {
        self.extension.clone()
    }
}

#[derive(Debug, PartialEq, Eq)]
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

// impl<'a> From<&ModulePartition<'a>> for ModulePartitionModel<'a> {
//     fn from(value: &ModulePartition<'a>) -> &'a Self {
//         Self {
//             module: value.module.,
//             partition_name: value.partition_name.unwrap_or_default(),
//             is_internal_partition: value.is_internal_partition.unwrap_or_default(),
//         }
//     }
// }

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleImplementationModel<'a> {
    pub path: PathBuf,
    pub file_stem: Cow<'a, str>,
    pub extension: Cow<'a, str>,
    pub dependencies: Vec<Cow<'a, str>>,
}

impl<'a> fmt::Display for ModuleImplementationModel<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.path, self.dependencies)
    }
}

impl<'a> TranslationUnit for &'a ModuleImplementationModel<'a> {
    fn file(&self) -> PathBuf {
        self.path
            .join::<&str>(&self.file_stem)
            .with_extension::<&str>(&self.extension)
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn file_stem(&self) -> Cow<'_, str> {
        self.file_stem.clone()
    }

    fn extension(&self) -> Cow<'_, str> {
        self.extension.clone()
    }
}
