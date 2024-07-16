pub mod build;
pub mod compiler;
pub mod executable;
pub mod modules;
pub mod project;
pub mod sourceset;
pub mod tests;

use std::fmt::Debug;

use color_eyre::eyre::Context;
use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cache::ZorkCache;

use crate::utils;

use self::{
    build::BuildModel, compiler::CompilerModel, executable::ExecutableModel, modules::ModulesModel,
    project::ProjectModel, tests::TestsModel,
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ZorkModel<'a> {
    pub project: ProjectModel<'a>,
    pub compiler: CompilerModel<'a>,
    pub build: BuildModel,
    pub executable: ExecutableModel<'a>,
    pub modules: ModulesModel<'a>,
    pub tests: TestsModel<'a>,
}

/// Loads the mapped [`ZorkModel`] for a concrete [`ZorkConfigFile`] from the [`ZorkCache`]
pub fn load<'a>(cache: &ZorkCache<'a>) -> Result<ZorkModel<'a>> {
    utils::fs::load_and_deserialize::<ZorkModel, _>(&cache.metadata.project_model_file_path)
        .with_context(|| "Error loading the project model")
}

/// Saves the mapped [`ZorkModel`] for a concrete [`ZorkConfigFile`] on the [`ZorkCache`]
pub fn save(program_data: &ZorkModel, cache: &ZorkCache) -> Result<()> {
    utils::fs::save_file(&cache.metadata.project_model_file_path, program_data)
        .with_context(|| "Error saving the project model")
}
