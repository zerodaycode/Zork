//! Types and procedures that represents a command line argument,
//! or collections of command line arguments

use std::borrow::Cow;
use std::ops::Deref;
use std::path::Path;
use std::{borrow::Borrow, ffi::OsStr, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::project_model::compiler::LanguageLevel;

/// Wrapper type for represent and storing a command line argument
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Argument(String);

impl Argument {
    pub fn value(&self) -> &String {
        &self.0
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<&str> for Argument {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl From<&String> for Argument {
    fn from(value: &String) -> Self {
        Self(value.into())
    }
}

impl From<Cow<'_, str>> for Argument {
    fn from(value: Cow<'_, str>) -> Self {
        Self(value.into())
    }
}

impl From<&Cow<'_, str>> for Argument {
    fn from(value: &Cow<'_, str>) -> Self {
        Self(value.clone().into())
    }
}

impl From<String> for Argument {
    fn from(value: String) -> Argument {
        Self(value)
    }
}

impl From<&Path> for Argument {
    fn from(value: &Path) -> Self {
        Self::from(format!("{}", value.display()))
    }
}

impl From<PathBuf> for Argument {
    fn from(value: PathBuf) -> Self {
        Self::from(format!("{}", value.display()))
    }
}

impl From<&PathBuf> for Argument {
    fn from(value: &PathBuf) -> Self {
        Self::from(format!("{}", value.display()))
    }
}

impl From<LanguageLevel> for Argument {
    fn from(value: LanguageLevel) -> Self {
        Self::from(value.as_ref().to_string())
    }
}

impl Borrow<str> for Argument {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl AsRef<OsStr> for Argument {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.0)
    }
}

impl AsRef<str> for Argument {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Strong type for represent a linear collection of [`Argument`]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Arguments(Vec<Argument>);

impl core::fmt::Display for Arguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().try_for_each(|arg| write!(f, "{} ", arg))
        // TODO: there's an ugly space at the end of every command line when Display is invoked
        // :) Just fix it
    }
}

impl Arguments {
    /// Wraps an existing [`std::vec::Vec`] of [`Argument`]
    pub fn from_vec(vec: Vec<Argument>) -> Self {
        Self(vec)
    }

    /// Returns a new collection of [`Argument`] with the specified capacity
    pub fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    /// Creates and stores a new [`Argument`] to the end of this collection
    /// from any type *T* that can be coerced into an [`Argument`] type
    pub fn push<T>(&mut self, val: T)
    where
        T: Into<Argument>,
    {
        self.0.push(val.into())
    }

    /// Given an optional, adds the inner value if there's Some(<[Argument]>)
    pub fn push_opt(&mut self, arg: Option<Argument>) {
        if let Some(val) = arg {
            self.0.push(val)
        }
    }

    /// Extends the underlying collection from an Iterator of [`Argument`]
    pub fn extend(&mut self, iter: impl IntoIterator<Item = Argument>) {
        self.0.extend(iter);
    }

    /// Extends the underlying collection given a slice of [`Argument`]
    pub fn extend_from_slice(&mut self, slice: &[Argument]) {
        self.0.extend_from_slice(slice);
    }

    pub fn as_slice(&self) -> &[Argument] {
        &self.0
    }

    /// Clears the contained values of the wrapped [`std::vec::Vec`]
    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl Deref for Arguments {
    type Target = [Argument];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for Arguments {
    type Item = Argument;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl IntoIterator for &Arguments {
    type Item = Argument;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.clone().into_iter()
    }
}

impl FromIterator<Argument> for Arguments {
    fn from_iter<I: IntoIterator<Item = Argument>>(iter: I) -> Self {
        let mut vec = Vec::new();
        for item in iter {
            vec.push(item);
        }
        Arguments(vec)
    }
}

impl<'a> FromIterator<&'a Argument> for Arguments {
    fn from_iter<I: IntoIterator<Item = &'a Argument>>(iter: I) -> Arguments {
        let mut vec = Vec::new();
        for item in iter {
            vec.push(item.clone());
        }
        Arguments(vec)
    }
}

/// Isolated module to storing custom procedures to easily create and add new command line arguments
/// or flags specific to Clang, that otherwise, will be bloating the main procedures with a lot
/// of cognitive complexity
pub mod clang_args {
    use std::path::Path;

    use crate::project_model::compiler::CppCompiler;

    use super::*;

    /// Generates the correct module mapping command line argument for Clang.
    ///
    // The Windows variant is a Zork++ feature to allow the users to write `import std;`
    // under -std=c++20 with clang linking against GCC with
    // some MinGW installation or similar
    pub(crate) fn implicit_module_map<'a>(out_dir: &Path) -> Cow<'a, str> {
        if std::env::consts::OS.eq("windows") {
            Cow::Owned(format!(
                "-fmodule-map-file={}",
                out_dir
                    .join("zork")
                    .join("intrinsics")
                    .join("zork.modulemap")
                    .display()
            ))
        } else {
            Cow::Borrowed("-fimplicit-module-maps")
        }
    }

    pub(crate) fn add_prebuilt_module_path(compiler: CppCompiler, out_dir: &Path) -> String {
        format!(
            "-fprebuilt-module-path={}",
            out_dir
                .join(compiler.as_ref())
                .join("modules")
                .join("interfaces")
                .display()
        )
    }

    pub(crate) fn add_direct_module_interfaces_dependencies(
        dependencies: &[Cow<str>],
        compiler: CppCompiler,
        out_dir: &Path,
        arguments: &mut Arguments,
    ) {
        dependencies.iter().for_each(|ifc_dep| {
            arguments.push(Argument::from(format!(
                "-fmodule-file={}",
                out_dir
                    .join(compiler.as_ref())
                    .join("modules")
                    .join("interfaces")
                    .join::<&str>(ifc_dep)
                    .with_extension(compiler.get_typical_bmi_extension())
                    .display()
            )))
        });
    }
}

pub mod msvc_args {
    use crate::domain::translation_unit::TranslationUnit;
    use crate::{
        cache::ZorkCache, cli::output::commands::SourceCommandLine,
        project_model::compiler::StdLibMode,
    };

    use super::Arguments;

    pub(crate) fn generate_std_cmd(
        cache: &ZorkCache,
        stdlib_mode: StdLibMode,
    ) -> SourceCommandLine {
        let mut arguments = Arguments::default();
        let msvc = &cache.compilers_metadata.msvc;

        let (stdlib_sf, stdlib_bmi_path, stdlib_obj_path) = if stdlib_mode.eq(&StdLibMode::Cpp) {
            (
                &msvc.vs_stdlib_path,
                &msvc.stdlib_bmi_path,
                &msvc.stdlib_obj_path,
            )
        } else {
            (
                &msvc.vs_ccompat_stdlib_path,
                &msvc.ccompat_stdlib_bmi_path,
                &msvc.ccompat_stdlib_obj_path,
            )
        };

        arguments.push(stdlib_sf.path());
        arguments.push("/ifcOutput");
        arguments.push(format! {
            "{}", stdlib_bmi_path.display()
        });
        arguments.push(format! {
            "/Fo{}", stdlib_obj_path.display()
        });

        SourceCommandLine::new(stdlib_sf, arguments)
    }
}
