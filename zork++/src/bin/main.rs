use std::path::Path;

use clap::Parser;
use color_eyre::{eyre::Context, Result};
use env_logger::Target;
use zork::{
    cli::{input::{CliArgs, Command}, output::commands::autorun_generated_binary},
    compiler::build_project,
    utils::{logger::config_logger, template::create_templated_project, reader::{find_config_file, build_model}}, config_file::ZorkConfigFile
};

fn main() -> Result<()> {
    color_eyre::install()?;

    let base_path = Path::new(".");
    let cli_args = CliArgs::parse();
    config_logger(cli_args.verbose, Target::Stdout).expect("Error configuring the logger");

    let config_file: String =
        find_config_file(base_path).with_context(|| "Failed to read configuration file")?;
    let config: ZorkConfigFile = toml::from_str(config_file.as_str())
        .with_context(|| "Could not parse configuration file")?;

    let program_data = build_model(&config);

    match cli_args.command {
        Command::Build => {
            build_project(base_path, &program_data, &cli_args)
                .with_context(|| "Failed to build project")
                .map(|_| Ok(()))?
        },
        Command::Run => {
            build_project(base_path, &program_data, &cli_args)
                .with_context(|| "Failed to build project")?;
            autorun_generated_binary(
                &program_data.compiler.cpp_compiler,
                program_data.build.output_dir,
                program_data.executable.executable_name // TODO if tests then other logic way around
            )
        },
        Command::Test => todo!("Implement test routine"),
        Command::New {
            name,
            git,
            compiler,
        } => create_templated_project(base_path, &name, git, compiler.into())
            .with_context(|| "Failed to create new project"),
        Command::Cache => todo!(),
    }
}
