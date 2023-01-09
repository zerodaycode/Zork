use std::{fs::File, io::Read, path::Path};

use crate::{config_file::ZorkConfigFile, utils::template::resources::CONFIG_FILE_NAME};

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
pub fn find_config_file() -> String {
    log::info!("Sarching for configuration file");

    let path = Path::new(".").join(CONFIG_FILE_NAME);

    path.to_str().expect("Error get path file").to_owned()
}
