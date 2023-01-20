pub mod build;
pub mod compiler;
pub mod executable;
pub mod modules;
pub mod project;
pub mod tests;

use std::fmt::Debug;

use self::{
    build::BuildModel, compiler::CompilerModel, executable::ExecutableModel, modules::ModulesModel,
    project::ProjectModel, tests::TestsModel,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ZorkModel {
    pub project: ProjectModel,
    pub compiler: CompilerModel,
    pub build: BuildModel,
    pub executable: ExecutableModel,
    pub modules: ModulesModel,
    pub tests: TestsModel,
}
