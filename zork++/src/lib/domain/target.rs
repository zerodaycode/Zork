//! The higher abstractions of the program

use crate::domain::commands::arguments::Argument;
use crate::domain::commands::command_lines::{LinkerCommandLine, SourceCommandLine};
use crate::project_model::sourceset::SourceSet;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// The final product that will be made after the building process
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Target<'a> {
    // pub identifier: Cow<'a, str>,
    pub sources: Vec<SourceCommandLine<'a>>,
    pub linker: LinkerCommandLine<'a>,
    pub kind: TargetKind,
}

/// Strong type for storing the target unique identifier, which instead of being
/// composite within the [`Target`] struct, is externalized in this wrapped type, so
/// we can use a strong type on the [`Commands.targets`] container
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
pub struct TargetIdentifier<'a>(Cow<'a, str>);

/// The different types of final products
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default, Copy, Clone)]
pub enum TargetKind {
    #[default]
    Executable,
    StaticLib,
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
