//! The module which holds the higher and generic abstractions over a source file

use crate::domain::target::TargetIdentifier;
use crate::project_model::compiler::StdLibMode;
use color_eyre::Report;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Debug, Display};
use std::path::PathBuf;
use std::process::ExitStatus;
use transient::{Any, Inv};

/// Represents any kind of translation unit and the generic operations
/// applicable to all the implementors
pub trait TranslationUnit<'a>: AsTranslationUnit<'a> + Any<Inv<'a>> + Display + Debug {
    /// Returns the full path of the [`TranslationUnit`] behind the invocation, including
    /// the file stem and the extension
    ///
    /// # Examples
    ///
    /// ```
    /// use std::borrow::Cow;
    /// use std::path::PathBuf;
    /// use zork::domain::translation_unit::TranslationUnit;
    /// use zork::project_model::sourceset::SourceFile;
    ///
    /// let source_file = SourceFile {
    ///     path: PathBuf::from("/usr/include"),
    ///     file_stem: Cow::from("std"),
    ///     extension: Cow::from("h"),
    /// };
    ///
    /// assert_eq!(source_file.path(), PathBuf::from("/usr/include/std.h"));
    ///
    /// let source_file_compat = SourceFile {
    ///     path: PathBuf::from("/usr/include"),
    ///     file_stem: Cow::from("std.compat"),
    ///     extension: Cow::from("h"),
    /// };
    ///
    /// assert_eq!(source_file_compat.path(), PathBuf::from("/usr/include/std.compat.h"));
    /// ```
    fn path(&self) -> PathBuf {
        self.parent().join(self.filename())
    }

    /// Returns only the path to the directory where the translation unit lives on the fs
    fn parent(&self) -> &PathBuf;

    /// Outputs the declared file stem (filename without extension) for this translation unit
    fn file_stem(&self) -> &Cow<'_, str>;

    /// Outputs the declared extension for `self`
    fn extension(&self) -> &Cow<'_, str>;

    /// Outputs the file stem concatenated with the extension for a given tu
    fn filename(&self) -> String {
        format!("{}.{}", self.file_stem(), self.extension())
    }
}

/// Base trait for downcasting all the implementors of [`TranslationUnit`] when they are hidden
/// behind an opaque type
pub trait AsTranslationUnit<'a> {
    fn as_any(&self) -> &dyn Any<Inv<'a>>;
}

// Blanket implementation of [`AsTranslationUnit`] for all types implementing TranslationUnit
impl<'a, T: TranslationUnit<'a> + 'a> AsTranslationUnit<'a> for T {
    fn as_any(&self) -> &dyn Any<Inv<'a>> {
        self
    }
}

#[macro_export]
macro_rules! impl_translation_unit_for {
    ($t:ty) => {
        impl<'a> TranslationUnit<'a> for $t {
            fn parent(&self) -> &PathBuf {
                &self.path
            }

            fn file_stem(&self) -> &Cow<'_, str> {
                &self.file_stem
            }

            fn extension(&self) -> &Cow<'_, str> {
                &self.extension
            }
        }
    };
}

/// The different type of translation units that `Zork++` is able to deal with
#[derive(Debug)]
pub enum TranslationUnitKind<'a> {
    ModuleInterface,
    ModuleImplementation,
    SourceFile(TargetIdentifier<'a>),
    ModularStdLib(StdLibMode),
    SystemHeader,
}

/// The different states of a translation unit in the whole lifecycle of
/// the build process and across different iterations of the same
#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
pub enum TranslationUnitStatus {
    /// A command that is executed correctly
    Success,
    /// A skipped command due to previous successful iterations
    Cached,
    /// A command which is return code indicates an unsuccessful execution
    Failed,
    /// Whenever a translation unit must be rebuilt
    #[default]
    PendingToBuild,
    /// The associated [`TranslationUnit`] has been deleted from the user's configuration and therefore,
    /// it should be removed from the cache as well as its generated byproducts
    ToDelete,
    /// The execution failed, returning a [`color_eyre::Result`] with the Err variant
    Error,
}

impl From<color_eyre::Result<ExitStatus, Report>> for TranslationUnitStatus {
    fn from(value: color_eyre::Result<ExitStatus, Report>) -> Self {
        helpers::handle_command_execution_result(&value)
    }
}

impl From<&color_eyre::Result<ExitStatus, Report>> for TranslationUnitStatus {
    fn from(value: &color_eyre::Result<ExitStatus, Report>) -> Self {
        helpers::handle_command_execution_result(value)
    }
}

mod helpers {
    use crate::domain::translation_unit::TranslationUnitStatus;
    use std::process::ExitStatus;

    /// Convenient way of handle a command execution result avoiding duplicate code
    pub(crate) fn handle_command_execution_result(
        value: &color_eyre::Result<ExitStatus>,
    ) -> TranslationUnitStatus {
        match value {
            Ok(r) => {
                if r.success() {
                    TranslationUnitStatus::Success
                } else {
                    TranslationUnitStatus::Failed
                }
            }
            Err(_) => TranslationUnitStatus::Error,
        }
    }
}
