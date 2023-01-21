pub mod arguments;
pub mod build;
pub mod compiler;
pub mod executable;
pub mod modules;
pub mod project;
pub mod sourceset;
pub mod tests;

use std::fmt::{Debug, Display};

use self::{
    arguments::Argument, build::BuildModel, compiler::CompilerModel, executable::ExecutableModel,
    modules::ModulesModel, project::ProjectModel, sourceset::SourceSet, tests::TestsModel,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ZorkModel<'a> {
    pub project: ProjectModel<'a>,
    pub compiler: CompilerModel<'a>,
    pub build: BuildModel<'a>,
    pub executable: ExecutableModel<'a>,
    pub modules: ModulesModel<'a>,
    pub tests: TestsModel<'a>,
}

pub trait ExtraArgs<'a> {
    fn extra_args(&'a self) -> &'a [Argument<'a>];
}

pub trait ExecutableTarget<'a>: ExtraArgs<'a> {
    fn name(&'a self) -> &'a str;
    fn sourceset(&'a self) -> &'a SourceSet<'a>;
}

/// Represents any kind of translation unit and the generic operations
/// applicable to all the implementors
pub trait TranslationUnit: Display + Debug {
    /// Outputs the declared filename for `self` being the translation unit
    fn filename(&self) -> &str;
}

impl TranslationUnit for &str {
    fn filename(&self) -> &str {
        self
    }
}

impl TranslationUnit for String {
    fn filename(&self) -> &str {
        self
    }
}
