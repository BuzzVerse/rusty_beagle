mod config;
mod defines;
mod logging;
mod spi;

extern crate log;

pub use crate::config::*;
pub use crate::defines::{api_defines::API_Status, lora_defines::*};
pub use crate::logging::start_logger;
use log::{debug, error, info, trace, warn};
use spi::Lora;

#[cfg(target_arch = "x86_64")]
fn prepare_mocks() {
    println!("prepare_mocks(): Running on x86_64.");
}

fn main() {
    #[cfg(target_arch = "x86_64")]
    prepare_mocks();

    start_logger();

    let config = Config::from_file();

    let mut lora = match Lora::from_config(config.spi_config) {
        Ok(lora) => lora,
        Err(e) => {
            eprintln!("When creating lora object: {:?}", e);
            error!("When creating lora object: {e}");
            std::process::exit(-1);
        }
    };

    let mut value = 0x00;
    lora.spi_read_register(REG_OP_MODE, &mut value);
    println!("value: {:#04x}", value);
}
