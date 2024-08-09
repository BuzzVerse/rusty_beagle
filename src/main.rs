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

    tokio::spawn(async move {
        loop {
            println!("Async took: {:?}", mqtt_queue.take().await);
        }
    });
    let handle = tokio::spawn(async move {
        handle_error!(lora.start(radio_config, lora_queue).await);
    });
    if let Err(e) = handle.await {
        eprintln!("Task failed: {:?}", e);
    }
}
