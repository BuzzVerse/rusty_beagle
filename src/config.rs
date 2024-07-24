use std::path::Path;

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::{fs, process};

pub fn read_config(file_path: &str) -> Config {
    let config_file = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR {:?}", e);
            error!("{e}");
            process::exit(-1)
        }
    };

    let config: Config = match ron::from_str(config_file.as_str()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("ERROR {:?}", e);
            error!("{e}");
            process::exit(-1);
        }
    };
    info!("Succesfully read config file.");
    config
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub mqtt_config: MqttConfig,
}

impl Config {
    pub fn from_file(file_path: &str) -> Config {
        let config_file = match fs::read_to_string(file_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ERROR {:?}", e);
                error!("While reading config file: {e}");
                process::exit(-1)
            }
        };

        let config: Config = match ron::from_str(config_file.as_str()) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("ERROR {:?}", e);
                error!("While deserializing ron file to config: {e}");
                process::exit(-1);
            }
        };
        info!("Succesfully read config file.");
        config
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MqttConfig {
    pub ip: String,
    pub port: String,
    pub login: String,
    pub password: String,
    pub topic: String,
}
