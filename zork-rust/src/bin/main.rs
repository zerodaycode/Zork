use env_logger::Target;
use zork::{config_cli::CliArgs, utils::logger::config_logger};
use clap::Parser;


fn main() {
    let parser_cli = CliArgs::parse_from([""]);

    config_logger(parser_cli.verbose, Target::Stdout)
        .expect("Error configure logger");


    
    log::warn!("warn");
    log::info!("info");
    log::error!("error");

}




