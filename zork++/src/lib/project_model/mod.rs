pub mod build;
pub mod compiler;
pub mod executable;
pub mod modules;
pub mod project;
pub mod sourceset;
pub mod tests;

use std::{fmt::Debug, path::Path};

use color_eyre::eyre::Context;
use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::{
    cli::input::CliArgs,
    config_file::ZorkConfigFile,
    utils::{
        self,
        constants::{error_messages, CACHE_FILE_EXT},
        reader,
    },
};

use self::{
    build::BuildModel,
    compiler::{CompilerModel, CppCompiler},
    executable::ExecutableModel,
    modules::ModulesModel,
    project::ProjectModel,
    tests::TestsModel,
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

/// Loads the mapped [`ZorkModel`] for a concrete [`ZorkConfigFile`] if a save file exists,
/// otherwise, calls the mapping processor to load the data from the configuration file
pub fn load<'a>(
    config: ZorkConfigFile<'a>,
    cli_args: &'a CliArgs,
    absolute_project_root: &Path,
) -> Result<ZorkModel<'a>> {
    let compiler: CppCompiler = config.compiler.cpp_compiler.into();
    let cache_path = Path::new(
        &config
            .build
            .as_ref()
            .and_then(|build_attr| build_attr.output_dir)
            .unwrap_or("out"),
    )
    .join("zork")
    .join("cache");

    let cached_project_model_path = cache_path
        .join(format!("{}_pm", compiler.as_ref()))
        .with_extension(CACHE_FILE_EXT);

    utils::fs::load_and_deserialize::<ZorkModel, _>(&cached_project_model_path)
        .or_else(|_| {
            log::debug!("Proceding to map the configuration file to the ZorkModel entity, since no cached project model was found");
            let program_data: ZorkModel = reader::build_model(config, cli_args, absolute_project_root)?;
            utils::fs::serialize_object_to_file::<ZorkModel>(&cached_project_model_path, &program_data)
                .with_context(|| error_messages::PROJECT_MODEL_SAVE)?;

            Ok::<ZorkModel<'_>, color_eyre::eyre::Error>(program_data)
        })
        .with_context(|| "Error loading the project model")
}
