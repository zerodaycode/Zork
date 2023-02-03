use std::path::Path;

use clap::Parser;
use color_eyre::Result;
use env_logger::Target;
use zork::{cli::input::CliArgs, utils::logger::config_logger, worker::run_zork};

/// The entry point for the binary generated
/// for the program
fn main() -> Result<()> {
    color_eyre::install()?;
    let cli_args = CliArgs::parse();
    config_logger(cli_args.verbose, Target::Stdout)
        .expect("Error configuring the logger");
    log::info!("Lanuching a new Zork++ program");
    run_zork(&cli_args, Path::new("."))?;
    log::info!("Tasks succesfully finished");

    Ok(())
}