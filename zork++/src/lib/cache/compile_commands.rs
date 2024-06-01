use crate::cache::ZorkCache;
use crate::cli::output::arguments::Arguments;
use crate::cli::output::commands::SourceCommandLine;
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
    let generated_commands = cache.get_all_commands_iter();
    let mut compilation_db_entries: Vec<CompileCommands> =
        Vec::with_capacity(cache.count_total_generated_commands());
    // compilation_db_entries.push(latest_commands.linker)

    for command in generated_commands {
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
    pub directory: PathBuf,
    pub file: String,
    pub arguments: Arguments,
}

impl From<&SourceCommandLine> for CompileCommands {
    fn from(value: &SourceCommandLine) -> Self {
        let value = value.clone();
        Self {
            directory: value.directory,
            file: value.filename,
            arguments: value.args,
        }
    }
}

// TODO review how the linker command line must be specified for the compile_commands.json
// impl From<&LinkerCommandLine> for CompileCommands {
//     fn from(value: &LinkerCommandLine) -> Self {
//         let value = value.clone();
//         Self {
//             directory: value.directory,
//             file: value.filename,
//             arguments: value.args,
//         }
//     }
// }
