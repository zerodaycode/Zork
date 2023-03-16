//! Types and procedures that represents a command line argument,
//! or collections of command line arguments

use std::path::Path;
use std::{borrow::Borrow, ffi::OsStr, path::PathBuf};
use std::ops::Deref;

use serde::{Deserialize, Serialize};

/// Type for represent a command line argument
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Argument<'a> {
    pub value: &'a str,
}

impl<'a> From<&'a str> for Argument<'a> {
    fn from(value: &'a str) -> Self {
        Self { value }
    }
}

impl<'a> From<String> for Argument<'a> {
    fn from(value: String) -> Argument<'a> {
        Self {
            value: Box::leak(value.into_boxed_str()),
        }
    }
}

impl<'a> Deref for Argument<'a> {
    type Target = &'a str;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a> From<&'a Path> for Argument<'a> {
    fn from(value: &'a Path) -> Self {
        Self::from(format!("{}", value.display()))
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

impl<'a> Borrow<str> for Argument<'a> {
    fn borrow(&self) -> &str {
        self.value
    }
}

impl<'a> AsRef<OsStr> for Argument<'a> {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self.value)
    }
}

impl<'a> core::fmt::Display for Argument<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Strong type for represent a linear collection of [`Argument`]
#[derive(Debug, Default, Clone)]
pub struct Arguments<'a>(Vec<Argument<'a>>);
impl<'a> Arguments<'a> {
    /// Wraps an existing [`std::vec::Vec`] of [`Argument`]
    pub fn from_vec(vec: Vec<Argument<'a>>) -> Self { Self(vec) }

    /// Returns a new collection of [`Argument`] with the specified capacity
    pub fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    /// Creates and stores a new [`Argument`] to the end of this collection
    pub fn create_and_push<T>(&mut self, val: T)
        where T: Into<Argument<'a>>
    {
        self.0.push(val.into())
    }

    /// Appends a new [`Argument`] to the end of this collection
    pub fn push(&mut self, arg: Argument<'a>) {
        self.0.push(arg)
    }

    /// Given an optional, adds the wrapper inner value if there's some element,
    /// otherwise leaves
    pub fn push_opt(&mut self, arg: Option<Argument<'a>>) {
        if let Some(val) = arg {
            self.0.push(val)
        }
    }

    /// Extends the underlying collection from a Iterator of [`Argument`]
    pub fn extend(&mut self, iter: impl IntoIterator<Item = Argument<'a>>) {
        self.0.extend(iter);
    }

    /// Extends the underlying collection given a slice of [`Argument`]
    pub fn extend_from_slice(&mut self, slice: &'a [Argument<'a>]) {
        self.0.extend_from_slice(slice);
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

pub mod clang_args {
    use std::path::Path;

    use crate::project_model::compiler::CppCompiler;

    use super::*;

    /// Generates the correct module mapping command line argument for Clang.
    ///
    // The Windows variant is a Zork++ feature to allow the users to write `import std;`
    // under -std=c++20 with clang linking against GCC with
    // some MinGW installation or similar
    pub(crate) fn implicit_module_maps<'a>(out_dir: &Path) -> Argument<'a> {
        if std::env::consts::OS.eq("windows") {
            Argument::from(format!(
                "-fmodule-map-file={}",
                out_dir
                    .join("zork")
                    .join("intrinsics")
                    .join("zork.modulemap")
                    .display()
            ))
        } else {
            Argument::from("-fimplicit-module-maps")
        }
    }

    pub(crate) fn add_prebuilt_module_path(compiler: CppCompiler, out_dir: &Path) -> Argument<'_> {
        Argument::from(format!(
            "-fprebuilt-module-path={}",
            out_dir
                .join(compiler.as_ref())
                .join("modules")
                .join("interfaces")
                .display()
        ))
    }

    pub(crate) fn add_direct_module_interfaces_dependencies(
        dependencies: &[&str],
        compiler: CppCompiler,
        out_dir: &Path,
        arguments: &mut Arguments<'_>,
    ) {
        dependencies.iter().for_each(|ifc_dep| {
            arguments.push(Argument::from(format!(
                "-fmodule-file={}",
                out_dir
                    .join(compiler.as_ref())
                    .join("modules")
                    .join("interfaces")
                    .join(ifc_dep)
                    .with_extension(compiler.get_typical_bmi_extension())
                    .display()
            )))
        });
    }
}
