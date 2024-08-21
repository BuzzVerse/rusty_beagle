mod bme280;
mod config;
mod conversions;
mod defines;
mod logging;
mod lora;
mod mqtt;
mod packet;
mod version_tag;

extern crate log;

pub use crate::config::*;
pub use crate::defines::*;
pub use crate::logging::start_logger;

use bme280::BME280Sensor;
use log::{error, info};
use lora::LoRa;
use mqtt::{Mqtt, mqtt_thread};
use packet::{BME280, Data, DataType, Packet};
use std::env;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender};

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

macro_rules! handle_error_continue {
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

fn main() {
    start_logger();
    let mut threads = Vec::new();

    let config_path = parse_args();
    let config = handle_error_exit!(Config::from_file(config_path));
    let radio_config = config.lora_config.radio_config.clone();
    let mqtt_config = config.mqtt_config.clone();
    let bme_config: BME280Config = config.bme_config.clone();

    let option_sender: Option<Sender<Packet>>;
    let mqtt_enabled = mqtt_config.enabled;

    if mqtt_enabled {
        let (sender, receiver) = channel::<Packet>();
        option_sender = Some(sender);
        let option_receiver = Some(receiver);

        let mqtt = Arc::new(handle_error_exit!(Mqtt::new(mqtt_config.clone())));
        let mqtt_clone = Arc::clone(&mqtt);
        threads.push(thread::spawn(move || {
            mqtt_thread(mqtt_clone, mqtt_config, option_receiver);
        }));
    } else {
        option_sender = None;
    }

    if bme_config.enabled {
        let bme280_sender = match option_sender.clone() {
            Some(sender) => sender,
            None => {
                eprintln!("No sender created");
                error!("No sender created");
                std::process::exit(-1);
            },
        };

        threads.push(thread::spawn(move || {
            let measurement_interval = bme_config.measurement_interval;
            let mut bme280 = handle_error_exit!(BME280Sensor::new(bme_config));

            loop {
                match bme280.read_measurements() {
                    Ok(data) => {
                        bme280
                            .print(&data)
                            .expect("Failed to print BME280 measurements");

                        if mqtt_enabled {
                            // TODO rethink version, msg_id and msg_count values
                            let packet = Packet {
                                version: 0,
                                id: config.mqtt_config.device_id,
                                msg_id: 0,
                                msg_count: 0,
                                data_type: DataType::BME280,
                                data: Data::Bme280(BME280 {
                                    temperature: data.temperature,
                                    humidity: data.humidity,
                                    pressure: data.pressure,
                                })
                            };
                            handle_error_continue!(bme280_sender.send(packet));
                        }
                    }
                    Err(e) => println!("Error reading measurements: {:?}", e),
                }

                thread::sleep(Duration::from_secs(measurement_interval));
            }
        }));
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


    threads.push(thread::spawn(move || {
        handle_error_exit!(lora.start(radio_config, option_sender));
    }));

    for thread in threads {
        handle_error_exit!(thread.join());
    }

}
