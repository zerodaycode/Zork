use crate::cache::ZorkCache;
use crate::cli::output::arguments::Arguments;
use crate::cli::output::commands::SourceCommandLine;
use crate::utils;
use crate::utils::constants::COMPILATION_DATABASE;
use color_eyre::eyre::{Context, Result};
use serde::Serialize;
use std::fs::File;
use std::path::{Path, PathBuf};

pub type CompileCommands = Vec<CompileCommand>;

/// Generates the `compile_commands.json` file, that acts as a compilation database
/// for some static analysis external tools, like `clang-tidy`, and populates it with
/// the generated commands for the translation units
pub(crate) fn map_generated_commands_to_compilation_db(
    cache: &ZorkCache,
) -> Result<CompileCommands> {
    log::trace!("Generating the compilation database...");

    let generated_commands = cache.get_all_commands_iter();
    let mut compilation_db_entries: Vec<CompileCommand> =
        Vec::with_capacity(cache.count_total_generated_commands()); // Without the linker one

    for command in generated_commands {
        compilation_db_entries.push(CompileCommand::from(command));
    }

    let compile_commands_path = Path::new(COMPILATION_DATABASE);
    if !Path::new(&compile_commands_path).exists() {
        File::create(compile_commands_path)
            .with_context(|| "Error creating the compilation database")?;
    }

    utils::fs::serialize_object_to_file(Path::new(compile_commands_path), &compilation_db_entries)
        .with_context(move || "Error saving the compilation database")?;

    Ok(compilation_db_entries)
}

/// Data model for serialize the data that will be outputted
/// to the `compile_commands.json` compilation database file
#[derive(Serialize, Debug, Default, Clone)]
pub struct CompileCommand {
    pub directory: PathBuf,
    pub file: String,
    pub arguments: Arguments,
}

impl From<&SourceCommandLine> for CompileCommand {
    fn from(value: &SourceCommandLine) -> Self {
        let value = value.clone();
        Self {
            directory: value.directory,
            file: value.filename,
            arguments: value.args,
        }
    }
}
