mod bme280;
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

use bme280::BME280Sensor;
use log::{error, info};
use lora::LoRa;
use std::env;
use std::thread;
use std::time::Duration;

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
    let config = handle_error!(Config::from_file(config_path));
    let radio_config = config.lora_config.radio_config.clone();
    let bme_config: BME280Config = config.bme_config.clone();

    if bme_config.enabled {
        thread::spawn(move || {
            let measurement_interval = bme_config.measurement_interval;
            let mut bme280 = handle_error!(BME280Sensor::new(bme_config));

            loop {
                if let Err(e) = bme280.print() {
                    error!("Failed to print BME280 sensor measurements: {:?}", e);
                }
                thread::sleep(Duration::from_secs(measurement_interval));
            }
        });
    }

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
