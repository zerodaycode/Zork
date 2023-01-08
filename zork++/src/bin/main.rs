use clap::Parser;
use env_logger::Target;
use zork::{
    compiler::build_project,
    config_file::ZorkConfigFile, 
    cli::CliArgs,
    utils::{
        logger::config_logger,
        template::create_templated_project,
        reader::find_config_file
    }, 
};

fn main() {
    let cli_args = CliArgs::parse_from(vec!["", "-vv", "-n"]);
    config_logger(cli_args.verbose, Target::Stdout).expect("Error configuring the logger");

    if cli_args.new_template {
        create_templated_project(&cli_args);
    }

    let config_file: String = find_config_file();
    let _config: ZorkConfigFile = toml::from_str(&config_file.as_str())
        .expect("Error generating the configuration for Canyon");

    build_project(&_config, &cli_args);
}
