use std::{fs::File, io::Read, path::Path};

use clap::Parser;
use env_logger::Target;
use zork::{
    config_cli::{CliArgs, CppCompiler},
    config_file::ZorkConfigFile,
    utils::{logger::config_logger, template::create_template_project},
};

fn main() {
    let parser_cli = CliArgs::parse_from([""]);

    config_logger(parser_cli.verbose, Target::Stdout).expect("Error configure logger");

    log::warn!("warn");
    log::info!("info");
    log::error!("error");

    create_template_project(&Some("proyect".to_owned()), true, &Some(CppCompiler::CLANG));

    let path_file = Path::new("example").join("zork.conf");

    let mut file_conf = File::open(path_file).unwrap();

    let mut buffer: String = String::new();
    file_conf
        .read_to_string(&mut buffer)
        .expect("Error read file");

    let zork_config: Result<ZorkConfigFile, toml::de::Error> = toml::from_str(&buffer);

    println!("{:?}", zork_config);
}
