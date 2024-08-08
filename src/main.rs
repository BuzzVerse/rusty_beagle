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

fn main() {
    start_logger();

    let config = Config::from_file();
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
