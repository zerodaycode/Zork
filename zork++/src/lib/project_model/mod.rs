pub mod build;
pub mod compiler;
pub mod modules;
pub mod project;
pub mod sourceset;
pub mod target;

use std::fmt::Debug;

use color_eyre::eyre::Context;
use color_eyre::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::utils::constants::error_messages;
use crate::{cache::ZorkCache, domain::target::TargetIdentifier};

use crate::utils;

use self::{
    build::BuildModel, compiler::CompilerModel, modules::ModulesModel, project::ProjectModel,
    target::TargetModel,
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ZorkModel<'a> {
    pub project: ProjectModel<'a>,
    pub compiler: CompilerModel<'a>,
    pub build: BuildModel,
    pub modules: ModulesModel<'a>,
    pub targets: IndexMap<TargetIdentifier<'a>, TargetModel<'a>>,
}

/// Loads the mapped [`ZorkModel`] for a concrete [`ZorkConfigFile`] from the [`ZorkCache`]
pub fn load<'a>(cache: &ZorkCache<'a>) -> Result<ZorkModel<'a>> {
    utils::fs::load_and_deserialize::<ZorkModel, _>(&cache.metadata.project_model_file_path)
        .with_context(|| error_messages::PROJECT_MODEL_LOAD)
}

/// Saves the mapped [`ZorkModel`] for a concrete [`ZorkConfigFile`] on the [`ZorkCache`]
pub fn save(program_data: &ZorkModel, cache: &ZorkCache) -> Result<()> {
    utils::fs::save_file(&cache.metadata.project_model_file_path, program_data)
        .with_context(|| error_messages::PROJECT_MODEL_SAVE)
}
