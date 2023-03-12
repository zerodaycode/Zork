use crate::cache::ZorkCache;
use crate::utils;
use crate::utils::constants::COMPILATION_DATABASE;
use color_eyre::eyre::{Context, Result};
use serde::Serialize;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Generates the `compile_commands.json` file, that acts as a compilation database
/// for some static analysis external tools, like `clang-tidy`, and populates it with
/// the generated commands for the translation units
pub(crate) fn map_generated_commands_to_compilation_db(cache: &ZorkCache) -> Result<()> {
    log::trace!("Generating the compilation database...");
    let mut compilation_db_entries = Vec::with_capacity(cache.last_generated_commands.len());

    for command in cache.last_generated_commands.iter() {
        compilation_db_entries.push(CompileCommands::from(command));
    }

    let compile_commands_path = Path::new(COMPILATION_DATABASE);
    if !Path::new(&compile_commands_path).exists() {
        File::create(compile_commands_path)
            .with_context(|| "Error creating the compilation database")?;
    }
    utils::fs::serialize_object_to_file(Path::new(compile_commands_path), &compilation_db_entries)
        .with_context(move || "Error saving the compilation database")
}

/// Data model for serialize the data that will be outputted
/// to the `compile_commands.json` compilation database file
#[derive(Serialize, Debug, Default, Clone)]
pub struct CompileCommands {
    pub directory: String,
    pub file: String,
    pub arguments: Vec<String>,
}

impl From<(&'_ PathBuf, &'_ Vec<String>)> for CompileCommands {
    fn from(value: (&PathBuf, &Vec<String>)) -> Self {
        let dir = value.0.parent().unwrap_or(Path::new("."));
        let mut file = value.0.file_stem().unwrap_or_default().to_os_string();
        file.push(".");
        file.push(value.0.extension().unwrap_or_default());

        Self {
            directory: dir.to_str().unwrap_or_default().to_string(),
            file: file.to_str().unwrap_or_default().to_string(),
            arguments: value.1.clone(),
        }
    }
}
