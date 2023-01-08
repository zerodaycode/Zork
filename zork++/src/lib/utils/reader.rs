use std::{fs::File, io::Read, path::Path};

use crate::{
    cli::CliArgs,
    config_file::ZorkConfigFile,
    utils::template::resources::{CONFIG_FILE_NAME, ROOT_PATH_NAME},
};

//TODO
pub fn get_config_file<'a>(path: &'a str, buffer: &'a mut String) -> ZorkConfigFile<'a> {
    log::info!("Getting configuration");

    let mut file = File::open(path) // TODO Extract from cli_args alternative path?
        .expect("Error open file");
    file.read_to_string(buffer).expect("Error read");
    let zork_conf: ZorkConfigFile = toml::from_str(buffer).expect("Can deserialize config file");
    zork_conf
}

//TODO
pub fn find_config_file(cli_args: &CliArgs) -> String {
    log::info!("Sarching for configuration file");

    let mut path = Path::new(".").join(CONFIG_FILE_NAME);
    if cli_args.new_template {
        path = Path::new(".").join(ROOT_PATH_NAME).join(CONFIG_FILE_NAME);
    }

    path.to_str().expect("Error get path file").to_owned()
}
