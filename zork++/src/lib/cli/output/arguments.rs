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
pub struct Argument<'a>(Cow<'a, str>);

impl<'a> Argument<'a> {
    pub fn value(&self) -> &Cow<'a, str> {
        &self.0
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> From<&'a str> for Argument<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl<'a> From<&'a String> for Argument<'a> {
    fn from(value: &'a String) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl<'a> From<Cow<'a, str>> for Argument<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}

impl<'a> From<&Cow<'a, str>> for Argument<'a> {
    fn from(value: &Cow<'a, str>) -> Self {
        Self(value.clone())
    }
}

impl<'a> From<String> for Argument<'a> {
    fn from(value: String) -> Argument<'a> {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<&'a Path> for Argument<'a> {
    fn from(value: &'a Path) -> Self {
        Self::from(value.to_string_lossy())
    }
}

impl<'a> From<PathBuf> for Argument<'a> {
    fn from(value: PathBuf) -> Self {
        Self::from(format!("{}", value.display()))
    }
}

impl<'a> From<&PathBuf> for Argument<'a> {
    fn from(value: &PathBuf) -> Self {
        Self::from(format!("{}", value.display()))
    }
}

impl<'a> From<LanguageLevel> for Argument<'a> {
    fn from(value: LanguageLevel) -> Self {
        Self::from(value.as_ref().to_string())
    }
}

impl<'a> Borrow<str> for Argument<'a> {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl<'a> AsRef<OsStr> for Argument<'a> {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self.0.as_ref())
    }
}

impl<'a> AsRef<str> for Argument<'a> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'a> core::fmt::Display for Argument<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Strong type for represent a linear collection of [`Argument`]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Arguments<'a>(Vec<Argument<'a>>);

impl<'a> core::fmt::Display for Arguments<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().try_for_each(|arg| write!(f, "{} ", arg))
        // TODO: there's an ugly space at the end of every command line when Display is invoked
        // :) Just fix it
    }
}

impl<'a> Arguments<'a> {
    /// Wraps an existing [`std::vec::Vec`] of [`Argument`]
    pub fn from_vec(vec: Vec<Argument<'a>>) -> Self {
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
        T: Into<Argument<'a>>,
    {
        self.0.push(val.into())
    }

    /// Given an optional, adds the inner value if there's Some(<[Argument]>)
    pub fn push_opt(&mut self, arg: Option<Argument<'a>>) {
        if let Some(val) = arg {
            self.0.push(val)
        }
    }

    /// Extends the underlying collection from an Iterator of [`Argument`]
    pub fn extend(&mut self, iter: impl IntoIterator<Item = Argument<'a>>) {
        self.0.extend(iter);
    }

    /// Extends the underlying collection given a slice of [`Argument`]
    pub fn extend_from_slice(&mut self, slice: &'a [Argument]) {
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

impl<'a> Deref for Arguments<'a> {
    type Target = [Argument<'a>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for Arguments<'a> {
    type Item = Argument<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &Arguments<'a> {
    type Item = Argument<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.clone().into_iter()
    }
}

impl<'a> FromIterator<Argument<'a>> for Arguments<'a> {
    fn from_iter<I: IntoIterator<Item = Argument<'a>>>(iter: I) -> Self {
        let mut vec = Vec::new();
        for item in iter {
            vec.push(item);
        }
        Arguments(vec)
    }
}

impl<'a> FromIterator<&'a Argument<'a>> for Arguments<'a> {
    fn from_iter<I: IntoIterator<Item = &'a Argument<'a>>>(iter: I) -> Arguments<'a> {
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

    pub(crate) fn generate_std_cmd<'a>(
        cache: &ZorkCache<'a>,
        stdlib_mode: StdLibMode,
    ) -> SourceCommandLine<'a> {
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
