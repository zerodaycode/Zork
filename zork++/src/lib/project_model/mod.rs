pub mod build;
pub mod compiler;
pub mod executable;
pub mod modules;
pub mod project;
pub mod sourceset;
pub mod tests;
pub mod workspace;

use std::fmt::Debug;
use crate::project_model::workspace::WorkspaceModel;

use self::{
    build::BuildModel, compiler::CompilerModel, executable::ExecutableModel, modules::ModulesModel,
    project::ProjectModel, tests::TestsModel,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ZorkModel<'a> {
    pub workspace: WorkspaceModel<'a>,
    pub project: ProjectModel<'a>,
    pub compiler: CompilerModel<'a>,
    pub build: BuildModel,
    pub executable: ExecutableModel<'a>,
    pub modules: ModulesModel<'a>,
    pub tests: TestsModel<'a>,
}
