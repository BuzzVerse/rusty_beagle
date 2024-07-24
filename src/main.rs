mod config;
mod defines;
mod logging;

extern crate log;

pub use crate::config::{read_config, Config, MqttConfig};
pub use crate::defines::{api_defines::API_Status, lora_defines::LoRa_Registers};
pub use crate::logging::start_logger;
use log::{debug, error, info, trace, warn};

fn main() {
    start_logger();
    let config = Config::from_file("./mqtt_conf.ron");
    println!("{:?}", config);
}
