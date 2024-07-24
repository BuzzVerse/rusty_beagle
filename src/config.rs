use log::{error, info};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::{fs, process};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub mqtt_config: MQTTConfig,
}

impl Config {
    pub fn from_file() -> Config {
        let config_file = match fs::read_to_string(Config::parse_args()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ERROR {:?}", e);
                error!("While reading config file: {e}.");
                process::exit(-1)
            }
        };

        let config: Config = match ron::from_str(config_file.as_str()) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("ERROR {:?}", e);
                error!("While deserializing ron file to config: {e}.");
                process::exit(-1);
            }
        };
        info!("Succesfully read config file.");
        config
    }

    fn parse_args() -> String {
        let args: Vec<String> = env::args().collect();

        let file_path = match args.len() {
            1 => "./conf.ron".to_string(),
            2 => args[1].to_string(),
            _ => {
                eprintln!("Wrong number of arguments!");
                println!("Usage: ./rusty_beagle [config file]");
                error!("Wrong number of arguments.");
                std::process::exit(-1);
            }
        };
        file_path
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MQTTConfig {
    pub ip: String,
    pub port: String,
    pub login: String,
    pub password: String,
    pub topic: String,
}
