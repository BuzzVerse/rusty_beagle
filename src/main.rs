mod bme280;
mod config;
mod conversions;
mod defines;
mod logging;
mod lora;
mod packet;
mod mqtt;
mod version_tag;

extern crate log;

pub use crate::config::*;
pub use crate::defines::*;
pub use crate::logging::start_logger;

use bme280::BME280Sensor;
use log::{error, info};
use lora::LoRa;
use mqtt::BlockingQueue;
use mqtt::Mqtt;
use packet::DataType;
use packet::Packet;
use tokio::time::sleep;
use std::sync::Arc;
use std::env;
use std::time::Duration;


macro_rules! handle_error_exit {
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

macro_rules! handle_error_contiue {
    ($func:expr) => {
        match $func {
            Err(e) => {
                eprintln!("{:?}", e);
                error!("{:?}", e);
                continue;
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

#[tokio::main]
async fn main() {
    start_logger();

    let config_path = parse_args();
    let config = handle_error_exit!(Config::from_file(config_path));
    let radio_config = config.lora_config.radio_config.clone();
    let mqtt_config = config.mqtt_config.clone();
    let bme_config: BME280Config = config.bme_config.clone();

    if bme_config.enabled {
        tokio::spawn(async move {
            let measurement_interval = bme_config.measurement_interval;
            let mut bme280 = handle_error_exit!(BME280Sensor::new(bme_config));

            loop {
                if let Err(e) = bme280.print() {
                    error!("Failed to print BME280 sensor measurements: {:?}", e);
                }
                sleep(Duration::from_secs(measurement_interval)).await;
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
    let queue = BlockingQueue::new(128);
    let lora_queue = queue.clone();
    let mqtt_queue = queue.clone();

    let mqtt = Arc::new(handle_error_exit!(Mqtt::new(mqtt_config.clone()).await));
    let mqtt_clone = Arc::clone(&mqtt);
    let mqtt_handle  = tokio::spawn(async move {
        let mqtt_config = mqtt_config;
        loop {
            let packet: Packet = mqtt_queue.take().await;
            let msg = handle_error_contiue!(packet.to_json());
            match packet.data_type {
                DataType::BME280 => {
                    handle_error_contiue!(mqtt_clone.publish(&mqtt_config.topic, &msg).await)
                },
                _ => continue,
            }
        }
    });
    let lora_handle = tokio::spawn(async move {
        handle_error_exit!(lora.start(radio_config, lora_queue).await);
    });
    handle_error_exit!(lora_handle.await);
    handle_error_exit!(mqtt_handle.await);

    mqtt.shutdown().await;
}
