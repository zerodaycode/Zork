use std::path::Path;

use clap::Parser;
use color_eyre::eyre::Context;
use color_eyre::Result;
use env_logger::Target;
use zork::{cli::input::CliArgs, utils::logger::config_logger, worker::run_zork};

/// The entry point for the binary generated
/// for the program
fn main() -> Result<()> {
    color_eyre::install()?;
    let process_start_time = std::time::Instant::now();

    let cli_args = CliArgs::parse();
    config_logger(cli_args.verbose, Target::Stdout)
        .with_context(|| "Error configuring the logger")?;

    log::info!("Launching a new Zork++ program");
    match run_zork(&cli_args, Path::new(".")) {
        Ok(_) => {
            log::info!(
                "[SUCCESS] - The process ended successfully, taking a total time in complete of: {:?} ms",
                process_start_time.elapsed().as_millis()
            );
            Ok(())
        }
        Err(err) => {
            log::error!(
                "[FAILED] - The process failed, taking a total time in complete of: {:?} ms",
                process_start_time.elapsed().as_millis()
            );
            Err(err)
        }
    }?;

    Ok(())
}
