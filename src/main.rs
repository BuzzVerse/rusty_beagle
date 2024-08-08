mod config;
mod conversions;
mod defines;
mod logging;
mod lora;
mod packet;
mod version_tag;
mod bme280;

extern crate log;

pub use crate::config::*;
pub use crate::defines::*;
pub use crate::logging::start_logger;

use std::thread;
use std::time::Duration;
use bme280::BME280Sensor;
use log::{error, info};
use lora::LoRa;

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

    thread::spawn(move || {
        let mut bme280 = BME280Sensor::new("/dev/i2c-2");

        loop {
            let (temperature, pressure, humidity) = bme280.read_measurements();

            println!("Temperature: {:.1} °C", temperature);
            println!("Pressure: {:.1} hPa", pressure);
            println!("Humidity: {:.1} %", humidity);

            thread::sleep(Duration::from_secs(10));
        }
    });

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
