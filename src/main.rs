mod config;
mod conversions;
mod defines;
mod logging;
mod lora;
mod packet;
mod version_tag;

extern crate log;

pub use crate::config::*;
pub use crate::defines::*;
pub use crate::logging::start_logger;
use log::{error, info};
use lora::LoRa;
use std::env;

macro_rules! handle_error {
    ($func:expr) => {
        match $func {
            Err(e) => {
                eprintln!("{:?}", e);
                error!("{:?}", e);
                std::process::exit(-1);
            }
            Ok(s) => s,
        }
    };
}

fn parse_args() -> String {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => "./conf.ron".to_string(),
        2 => args[1].to_string(),
        _ => {
            eprintln!("Wrong number of arguments!");
            println!("Usage: ./rusty_beagle [config file]");
            error!("Wrong number of arguments.");
            std::process::exit(-1);
        }
    }
}

fn main() {
    start_logger();

    let config_path = parse_args();
    let config = Config::from_file(config_path);
    let radio_config = config.lora_config.radio_config.clone();

    let mut lora = match LoRa::from_config(&config.lora_config) {
        Ok(lora) => {
            info!("LoRa object created successfully.");
            lora
        }
        Err(e) => {
            eprintln!("When creating lora object: {:?}", e);
            error!("When creating lora object: {e}");
            std::process::exit(-1);
        }
    };
    handle_error!(lora.start(radio_config));
}
