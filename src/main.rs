mod config;
mod defines;
mod logging;

extern crate log;

pub use crate::config::{Config, MqttConfig};
pub use crate::defines::{api_defines::API_Status, lora_defines::LoRa_Registers};
pub use crate::logging::start_logger;
//use log::{debug, error, info, trace, warn};
use std::env;

#[cfg(target_arch = "x86_64")]
fn prepare_mocks() {
    println!("prepare_mocks(): Running on x86_64.");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let file_path = match args.len() {
        2 => &args[1],
        _ => {
            eprintln!("Wrong number of arguments!");
            println!("Usage: ./rusty_beagle [config file]");
            std::process::exit(-1);
        }
    };

    start_logger();

    #[cfg(target_arch = "x86_64")]
    prepare_mocks();

    let config = Config::from_file(file_path);
    println!("{:?}", config);
}
