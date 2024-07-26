//! The higher abstractions of the program

use crate::domain::commands::arguments::Argument;
use crate::domain::commands::command_lines::{LinkerCommandLine, SourceCommandLine};
use crate::project_model::sourceset::SourceSet;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// The final product that will be made after the building process
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Target<'a> {
    pub sources: Vec<SourceCommandLine<'a>>,
    pub linker: LinkerCommandLine<'a>,
    pub kind: TargetKind,
    #[serde(skip)]
    pub enabled_for_current_program_iteration: bool,
}

impl<'a> Target<'a> {
    /// Defaults initializes a new [`Target`] when the unique data
    /// is the [`TargetKind`]. This is useful in our internals when there's
    /// no entry for this target on the [`ZorkCache`] and we want to create a new
    /// one in place to add a new [`SourceCommandLine`]
    pub fn new_default_for_kind(kind: TargetKind) -> Self {
        Self {
            sources: Vec::default(),
            linker: LinkerCommandLine::default(),
            kind,
            enabled_for_current_program_iteration: true,
        }
    }
}

/// Strong type for storing the target unique identifier, which instead of being
/// composite within the [`Target`] struct, is externalized in this wrapped type, so
/// we can use a strong type on the [`Commands.targets`] container
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default, Hash, Clone)]
pub struct TargetIdentifier<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for TargetIdentifier<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl<'a> TargetIdentifier<'a> {
    pub fn name(&'a self) -> &'a str {
        self.0.as_ref()
    }
}

/// The different types of final products
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default, Copy, Clone)]
pub enum TargetKind {
    #[default]
    #[serde(alias = "Executable", alias = "executable", alias = "exe")]
    Executable,
    #[serde(
        alias = "StaticLib",
        alias = "static lib",
        alias = "static-lib",
        alias = "static_lib",
        alias = "staticlib"
    )]
    StaticLib,
    #[serde(
        alias = "DynamicLib",
        alias = "dynamic lib",
        alias = "dyn-lib",
        alias = "dyn_lib",
        alias = "dylib"
    )]
    DyLib,
}

/// Bound for the user defined arguments that are passed to the compiler
pub trait ExtraArgs<'a> {
    fn extra_args(&'a self) -> &'a [Argument];
}

/// Contracts for the executable operations
pub trait ExecutableTarget<'a>: ExtraArgs<'a> {
    fn name(&'a self) -> &'a str;
    fn sourceset(&'a self) -> &'a SourceSet;
}
