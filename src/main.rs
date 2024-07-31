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

            let mut crc_error = false;

            let received_buffer = match lora.receive_packet(&mut crc_error) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{:?}", e);
                    error!("{:?}", e);
                    std::process::exit(-1);
                }
            };

            if crc_error {
                println!("CRC Error");
            }
            println!("Received {:#?} byte(s): {:#?}", received_buffer.len(), received_buffer);  
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

            let packet = String::from("BUZZVERSE").as_bytes().to_vec();

            handle_error!(lora.send_packet(packet));
        },
    }

    handle_error!(lora.reset());
}
