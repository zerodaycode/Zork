use crate::cache::ZorkCache;

use crate::domain::commands::arguments::Argument;
use crate::project_model::ZorkModel;
use crate::utils;
use crate::utils::constants::{error_messages, COMPILATION_DATABASE};
use color_eyre::eyre::{Context, ContextCompat, Result};
use serde::Serialize;
use std::fs::File;
use std::path::{Path, PathBuf};

pub type CompileCommands<'a> = Vec<CompileCommand<'a>>;

/// Generates the `compile_commands.json` file, that acts as a compilation database
/// for some static analysis external tools, like `clang-tidy`, and populates it with
/// the generated commands for the translation units
pub(crate) fn map_generated_commands_to_compilation_db(
    program_data: &ZorkModel,
    cache: &mut ZorkCache,
) -> Result<()> {
    log::debug!("Generating the compilation database...");
    let compiler = program_data.compiler.cpp_compiler;

    let flyweight_data = cache
        .generated_commands
        .flyweight_data
        .as_ref()
        .with_context(|| error_messages::FAILURE_LOADING_FLYWEIGHT_DATA)?;

    let generated_commands = cache.get_all_commands_iter();
    let mut compilation_db_entries: Vec<CompileCommand> =
        Vec::with_capacity(cache.count_total_generated_commands());

    let compiler_driver: [Argument; 1] =
        [Argument::from(compiler.get_driver(&program_data.compiler))];

    for source_command_line in generated_commands {
        let translation_unit_cmd_args = compiler_driver
            .iter()
            .chain(flyweight_data.general_args.as_ref().iter())
            .chain(flyweight_data.shared_args.as_ref().iter())
            .chain(flyweight_data.std_references.iter())
            .chain(flyweight_data.compile_but_dont_link.iter())
            .chain(source_command_line.args.iter())
            .collect::<Vec<&Argument>>();

        let compile_command = CompileCommand {
            directory: &source_command_line.directory,
            file: &source_command_line.filename,
            arguments: translation_unit_cmd_args,
        };
        compilation_db_entries.push(compile_command);
    }

    let compile_commands_path = Path::new(COMPILATION_DATABASE);
    if !Path::new(&compile_commands_path).exists() {
        File::create(compile_commands_path)
            .with_context(|| "Error creating the compilation database")?;
    }

    utils::fs::save_file(Path::new(compile_commands_path), &compilation_db_entries)
        .with_context(move || "Error saving the compilation database")
}

/// Data model for serialize the data that will be outputted
/// to the `compile_commands.json` compilation database file
#[derive(Serialize, Debug)]
pub struct CompileCommand<'a> {
    pub directory: &'a PathBuf,
    pub file: &'a String,
    pub arguments: Vec<&'a Argument<'a>>,
}
