use std::{path::Path, fs::File, io::Read};

use clap::Parser;
use env_logger::Target;
use zork::{
    config_cli::CliArgs,
    config_file::ZorkConfigFile,
    utils::{
        logger::config_logger,
        template::create_template_project, constans::{DEFAULT_CONFIG_NAME, DEFAULT_DIRECTORY},
        
    },
};



fn main() {
    let parser_cli = CliArgs::parse_from(vec!["","-vv"]);
    config_logger(parser_cli.verbose, Target::Stdout).expect("Error configure logger");

    if parser_cli.new_template {
        create_template_project(DEFAULT_CONFIG_NAME,Path::new(DEFAULT_DIRECTORY), true, parser_cli.compiler);
    }

    if parser_cli.command.is_some() {
        match parser_cli.command.unwrap() {
            zork::config_cli::Command::Tests => todo!(),
        }
    }else{
        execution_procces(Path::new(".").join(DEFAULT_DIRECTORY).join(DEFAULT_CONFIG_NAME).as_path());
    }


}

fn execution_procces(path: &Path) {
    
    log::info!("Find config file: {:?}",path);
    let mut buffer = String::new();
    let mut file = File::open(path).expect("Error open file");
    file.read_to_string(&mut buffer).expect("Error read");
    let zork_conf: ZorkConfigFile = toml::from_str(&buffer).expect("Can deserialize config file");

    if zork_conf.executable.is_some() {

    }

    log::info!("Finish procces");
}
