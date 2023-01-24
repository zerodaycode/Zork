//! The implementation of the Zork++ cache, for persisting data in between process

use std::path::Path;

use toml::value::Datetime;

use crate::{cli::output::commands::Commands, bounds::TranslationUnit};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ZorkCache<'a> {
    pub last_program_execution: Datetime,
    pub last_generated_commands: Commands<'a>,
    pub config_files: Vec<ZorkConfigFile<'a>>,
    pub compilers_metadata: CompilersMetadata<'a>, 
}

/// The metadata for a valid an recognized configuration
/// file for the project.
/// 
/// TODO! Currently, we are only supporting one config file, but
/// the idea is support multiple ones, and make them run concurrently,
/// with different options, like having a suffix to distinguish them
/// by compiler, and/or by environment...
#[derive(Deserialize, Serialize, Debug)]
pub struct ZorkConfigFile<'a> {
    pub path: &'a Path,
    pub modified: Option<Datetime>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CompilersMetadata<'a> {
    // pub clang: ClangMetadata<'a>, // NOT yet!
    pub msvc: MsvcMetadata<'a>,
    pub gcc: GccMetadata<'a>
}

// #[derive(Deserialize, Serialize, Debug)]
// pub struct ClangMetadata<'a> {

// }

#[derive(Deserialize, Serialize, Debug)]
pub struct MsvcMetadata<'a> {
    pub dev_commands_prompt: &'a Path
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GccMetadata<'a> {
    pub 
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CachedProjectFiles {
    // pub modules: TranslationUnit 
}