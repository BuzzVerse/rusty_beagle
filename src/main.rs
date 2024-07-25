mod config;
mod defines;
mod logging;
mod lora;

extern crate log;

pub use crate::config::*;
pub use crate::defines::{api_defines::API_Status, lora_defines::*};
pub use crate::logging::start_logger;
#[allow(unused_imports)] // TODO delete later
use log::{debug, error, info, trace, warn};
use lora::LoRa;

#[cfg(target_arch = "x86_64")]
fn prepare_mocks() {
    println!("prepare_mocks(): Running on x86_64.");
}

fn main() {
    #[cfg(target_arch = "x86_64")]
    prepare_mocks();

    start_logger();

    let config = Config::from_file();

    let mut lora = match LoRa::from_config(config.lora_config) {
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

    lora.reset();

    let mut value = 0x00;
    lora.spi_read_register(REG_OP_MODE, &mut value);
    println!("value: {:#04x}", value); // expected: 0x09

    lora.spi_write_register(REG_OP_MODE, 0x08);

    lora.spi_read_register(REG_OP_MODE, &mut value);
    println!("value: {:#04x}", value); // expected: 0x08

    lora.reset();

    lora.spi_read_register(REG_OP_MODE, &mut value);
    println!("value: {:#04x}", value); // expected: 0x09
}
