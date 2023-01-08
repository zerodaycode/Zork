use std::{fs::File, io::Read};

use clap::Parser;
use env_logger::Target;
use zork::{
    config_cli::CliArgs,
    config_file::ZorkConfigFile,
    utils::{logger::config_logger, template::create_templated_project},
};

fn main() {
    let cli_args = CliArgs::parse_from(vec!["", "-vv"]);
    config_logger(cli_args.verbose, Target::Stdout).expect("Error configure logger");

    if cli_args.new_template {
        create_templated_project(&cli_args);
    }

    // TODO Impl the program's main logic
    build_project(&cli_args)
}

/// Computes the logic written for the build process
fn build_project(_cli_args: &CliArgs) {
    let mut buffer = String::new();
    let mut file = File::open(".") // TODO Extract from cli_args alternative path?
        .expect("Error open file");
    file.read_to_string(&mut buffer).expect("Error read");
    let _zork_conf: ZorkConfigFile = toml::from_str(&buffer).expect("Can deserialize config file");

    log::info!("Finished procces"); // TODO Time calculations at info level
}
