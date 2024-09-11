mod bme280;
mod config;
mod conversions;
mod defines;
mod logging;
mod lora;
mod mqtt;
mod packet;
mod version_tag;
mod post;

extern crate log;

pub use crate::config::*;
pub use crate::defines::*;
pub use crate::logging::start_logger;
pub use crate::post::post;

use bme280::BME280Sensor;
use log::{error, info};
use lora::LoRa;
use mqtt::{Mqtt, MQTTMessage};
use std::env;
use std::thread;
use std::sync::mpsc::channel;

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
        1 => "./conf.toml".to_string(),
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
    let config = handle_error_exit!(Config::from_file(config_path));
    let mod_state = handle_error_exit!(post(&config));

    let mut threads = Vec::new();

    let option_lora_config = config.lora_config;
    let option_mqtt_config = config.mqtt_config;
    let option_bme_config = config.bme_config;

    let option_sender;
    let option_device_id;

    if let (Some(mqtt_config), true) = (option_mqtt_config, mod_state.mqtt) {
        let (sender, receiver) = channel::<MQTTMessage>();
        option_sender = Some(sender);
        option_device_id = Some(mqtt_config.device_id);
        let option_receiver = Some(receiver);

        threads.push(thread::spawn(move || {
            let mqtt = handle_error_exit!(Mqtt::new(mqtt_config.clone()));
            mqtt.thread_run(mqtt_config, option_receiver);
        }));
    } else {
        option_sender = None;
        option_device_id = None;
    }

    if let (Some(bme280_config), true) = (option_bme_config, mod_state.bme280) {
        let option_sender = option_sender.clone();
        threads.push(thread::spawn(move || {
            let mut bme280 = handle_error_exit!(BME280Sensor::new(bme280_config.clone()));
            bme280.thread_run(bme280_config, mod_state.mqtt, option_device_id, option_sender);
        }));
    }

    if let (Some(lora_config), true) = (option_lora_config, mod_state.lora) {
        let radio_config = lora_config.radio_config.clone();
        let mut lora = match LoRa::from_config(&lora_config) {
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
    }



    for thread in threads {
        handle_error_exit!(thread.join());
    }

}

