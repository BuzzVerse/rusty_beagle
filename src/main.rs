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
use log::{error, info};
use lora::LoRa;
use mqtt::BlockingQueue;
use mqtt::Mqtt;
use packet::DataType;
use packet::Packet;
use std::sync::Arc;

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

#[tokio::main]
async fn main() {
    start_logger();

    let config = Config::from_file();
    let radio_config = config.lora_config.radio_config.clone();
    let mqtt_config = config.mqtt_config.clone();

    let mut lora = match LoRa::from_config(&config.lora_config).await {
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
