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
use mqtt::Mqtt;
use packet::Packet;
use std::env;
use std::thread;
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
    let bme280_config: BME280Config = config.bme_config.clone();

    let option_sender: Option<Sender<Packet>>;
    let mqtt_enabled = mqtt_config.enabled;
    let device_id = mqtt_config.device_id;

    if mqtt_enabled {
        let (sender, receiver) = channel::<Packet>();
        option_sender = Some(sender);
        let option_receiver = Some(receiver);

        threads.push(thread::spawn(move || {
            let mqtt = handle_error_exit!(Mqtt::new(mqtt_config.clone()));
            mqtt.thread_run(mqtt_config, option_receiver);
        }));
    } else {
        option_sender = None;
    }

    if bme280_config.enabled {
        let option_sender = option_sender.clone();
        threads.push(thread::spawn(move || {
            let mut bme280 = handle_error_exit!(BME280Sensor::new(bme280_config.clone()));
            bme280.thread_run(bme280_config, mqtt_enabled, device_id, option_sender);
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
