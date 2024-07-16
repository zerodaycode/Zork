use crate::cache::ZorkCache;
use crate::cli::output::arguments::Argument;

use crate::project_model::compiler::CppCompiler;
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

    let generated_commands = cache.get_all_commands_iter();
    let mut compilation_db_entries: Vec<CompileCommand> =
        Vec::with_capacity(cache.count_total_generated_commands());

    let general_args = cache
        .generated_commands
        .general_args
        .as_ref()
        .expect(error_messages::GENERAL_ARGS_NOT_FOUND)
        .get_args();

    let compiler_specific_shared_args = cache
        .generated_commands
        .compiler_common_args
        .as_ref()
        .with_context(|| error_messages::COMPILER_SPECIFIC_COMMON_ARGS_NOT_FOUND)?
        .get_args();

    let compile_but_dont_link: [Argument; 1] =
        [Argument::from(match program_data.compiler.cpp_compiler {
            CppCompiler::CLANG | CppCompiler::GCC => "-c",
            CppCompiler::MSVC => "/c",
        })];

    for source_command_line in generated_commands {
        let translation_unit_cmd_args = general_args
            .iter()
            .chain(compiler_specific_shared_args.iter())
            .chain(&compile_but_dont_link)
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
