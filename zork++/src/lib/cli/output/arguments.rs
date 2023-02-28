//! Types and procedures that represents a command line argument,
//! or collections of command line arguments

use std::{borrow::Borrow, ffi::OsStr, path::PathBuf};

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

pub mod clang_args {
    use std::path::Path;

    use crate::project_model::{compiler::CppCompiler, ZorkModel};

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

    #[inline(always)]
    pub fn add_std_lib<'a>(model: &'a ZorkModel) -> Option<Argument<'a>> {
        if !cfg!(target_os = "windows") {
            if let Some(arg) = model.compiler.stdlib_arg() {
                return Some(arg);
            }
        }

        None
    }

    pub(crate) fn add_direct_module_interfafces_dependencies(
        dependencies: &[&str],
        compiler: &CppCompiler,
        out_dir: &Path,
        arguments: &mut Vec<Argument>,
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
