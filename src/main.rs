mod config;
mod defines;
mod logging;

extern crate log;

pub use crate::config::*;
pub use crate::defines::{api_defines::API_Status, lora_defines::LoRa_Registers};
pub use crate::logging::start_logger;
use spidev::{SpiModeFlags, SpidevOptions};
//use log::{debug, error, info, trace, warn};

#[cfg(target_arch = "x86_64")]
fn prepare_mocks() {
    println!("prepare_mocks(): Running on x86_64.");
}

fn main() {
    start_logger();

    #[cfg(target_arch = "x86_64")]
    prepare_mocks();

    let config = Config::from_file();
    println!("{:?}", config);

    let spi_options = SpidevOptions::new()
        .bits_per_word(config.spi_config.bits_per_word)
        .max_speed_hz(config.spi_config.max_speed_hz)
        .mode(SpiModeFlags::from_bits(config.spi_config.spi_mode as u32).unwrap())
        .build();
    println!("{:?}", spi_options);
}
