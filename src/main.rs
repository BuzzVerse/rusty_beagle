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
use packet::Data;
use packet::Packet;
use std::sync::Arc;

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

    let mqtt = Arc::new(handle_error!(Mqtt::new(mqtt_config).await));
    let mqtt_clone = Arc::clone(&mqtt);
    tokio::spawn(async move {
        loop {
            let time_stamp = handle_error!(std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH));
            let packet: Packet = mqtt_queue.take().await;
            let msg = match packet.data {
                Data::Bme280(data) => {
                    format!("{{\"time\": {}, \"humidity\": {}, \"temperature\": {}, \"battery_voltage_mv\": 3000}}", time_stamp.as_secs(), data.humidity, data.temperature)
                },
                _ => "".to_string(),
            };
            handle_error!(mqtt_clone.publish(&msg).await);
            println!("Sent: {}", msg);
        }
    });
    let handle = tokio::spawn(async move {
        handle_error!(lora.start(radio_config, lora_queue).await);
    });
    if let Err(e) = handle.await {
        eprintln!("Task failed: {:?}", e);
    }

    mqtt.shutdown().await;
}
