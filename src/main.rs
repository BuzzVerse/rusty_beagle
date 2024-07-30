mod config;
mod defines;
mod logging;
mod lora;

extern crate log;

pub use crate::config::*;
pub use crate::defines::*;
pub use crate::logging::start_logger;
use log::{error, info};
use lora::LoRa;

macro_rules! handle_error {
    ($func:expr) => {
        if let Err(e) = $func {
            eprintln!("{:?}", e);
            error!("{:?}", e);
            std::process::exit(-1);
        }
    };
}

fn main() {
    start_logger();

    let config = Config::from_file();

    let mut lora = match LoRa::from_config(&config.lora_config) {
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

    handle_error!(lora.reset());
    handle_error!(lora.sleep_mode());

    match lora.mode {
        config::Mode::RX => {
            println!("[MODE]: RX");

            handle_error!(lora.config_radio(config.lora_config.radio_config));

            let mut value = 0x00;
            handle_error!(lora.spi_read_register(LoRaRegister::OP_MODE, &mut value));
            println!("value: {:#04x} (expected 0x80)", value);

            let mut received_value = 0x00;
            let mut return_length = 0x00;
            let mut crc_error = false;

            handle_error!(lora.receive_packet(&mut received_value, &mut return_length, &mut crc_error));

            if crc_error {
                println!("CRC Error");
            }
            println!("Received {:#04x} byte(s): {:#04x}", return_length, received_value);  
        },
        config::Mode::TX => {
            println!("[MODE]: TX");

            let mut value = 0x00;
            handle_error!(lora.spi_read_register(LoRaRegister::OP_MODE, &mut value));
            println!("value: {:#04x} (expected 0x80)", value);

            handle_error!(lora.config_radio(config.lora_config.radio_config));
            let mut lna = 0x00;
            handle_error!(lora.spi_read_register(LoRaRegister::LNA, &mut lna));
            handle_error!(lora.spi_write_register(LoRaRegister::LNA, lna | 0x03));

            handle_error!(lora.standby_mode());

            let packet = 0xAB;

            handle_error!(lora.send_packet(packet));
        },
    }

    handle_error!(lora.reset());
}
