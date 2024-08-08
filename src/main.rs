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

    if bme_config.enabled {
        thread::spawn(move || {
            let mut bme280 = BME280Sensor::new(bme_config);

            loop {
                let (temperature, pressure, humidity) = bme280.read_measurements();

                println!("Temperature: {:.1} °C", temperature);
                println!("Pressure: {:.1} hPa", pressure);
                println!("Humidity: {:.1} %", humidity);

                thread::sleep(Duration::from_secs(10));
            }
        });
    }
}
