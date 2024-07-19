//! The higher abstractions of the program

use crate::domain::commands::arguments::Argument;
use crate::project_model::sourceset::SourceSet;

/// Bound for the user defined arguments that are passed to the compiler
pub trait ExtraArgs<'a> {
    fn extra_args(&'a self) -> &'a [Argument];
}

/// Contracts for the executable operations
pub trait ExecutableTarget<'a>: ExtraArgs<'a> {
    fn name(&'a self) -> &'a str;
    fn sourceset(&'a self) -> &'a SourceSet;
}
