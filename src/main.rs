mod config;
mod defines;
mod logging;

extern crate log;

pub use crate::config::{read_config, Config, MqttConfig};
pub use crate::defines::{api_defines::API_Status, lora_defines::LoRa_Registers};
pub use crate::logging::start_logger;
//use log::{debug, error, info, trace, warn};

#[cfg(target_arch = "x86_64")]
fn prepare_mocks() {
    println!("prepare_mocks(): Running on x86_64.");
}

fn main() {
    start_logger();
    #[cfg(target_arch = "x86_64")]
    prepare_mocks();
    let config = Config::from_file("./mqtt_conf.ron");
    println!("{:?}", config);
}
