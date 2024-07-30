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

            // receive()
            let mut irq: u8 = 0x00;
            let mut return_length: u8 = 0x00;
            let mut crc_error: bool = false;

            handle_error!(lora.receive_mode());

            handle_error!(lora.spi_read_register(LoRaRegister::OP_MODE, &mut value));
            println!("value: {:#04x} (expected 0x85)", value); 

            loop {
                // has_received()
                handle_error!(lora.spi_read_register(LoRaRegister::IRQ_FLAGS, &mut irq));
                if irq & IRQMask::IRQ_RX_DONE_MASK as u8 == IRQMask::IRQ_RX_DONE_MASK as u8 {
                    if irq & IRQMask::IRQ_PAYLOAD_CRC_ERROR as u8 == IRQMask::IRQ_PAYLOAD_CRC_ERROR as u8 {
                        crc_error = true;
                    }
                    println!("Packet received: IRQMask: {:#04x}", irq);
                    break;
                }
            }

            handle_error!(lora.standby_mode());

            handle_error!(lora.spi_read_register(LoRaRegister::RX_NB_BYTES, &mut return_length));

            let mut received_address = 0x00;
            handle_error!(lora.spi_read_register(LoRaRegister::FIFO_RX_CURRENT_ADDR, &mut received_address));
            handle_error!(lora.spi_write_register(LoRaRegister::FIFO_ADDR_PTR, received_address));

            let mut received_value = 0x00;
            handle_error!(lora.spi_read_register(LoRaRegister::FIFO, &mut received_value));

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
            let mut tx_address = 0x00;
            handle_error!(lora.spi_read_register(LoRaRegister::FIFO_TX_BASE_ADDR, &mut tx_address));
            handle_error!(lora.spi_write_register(LoRaRegister::FIFO_ADDR_PTR, tx_address));

            handle_error!(lora.spi_write_register(LoRaRegister::PAYLOAD_LENGTH, 0x01));

            handle_error!(lora.spi_write_register(LoRaRegister::FIFO, packet));

            // send_packet()
            let mut irq: u8 = 0x00;

            handle_error!(lora.transmit_mode());
            handle_error!(lora.spi_read_register(LoRaRegister::OP_MODE, &mut value));
            println!("value: {:#04x} (expected 0x83)", value); 

            loop {
                handle_error!(lora.spi_read_register(LoRaRegister::IRQ_FLAGS, &mut irq));
                if irq & IRQMask::IRQ_TX_DONE_MASK as u8 == IRQMask::IRQ_TX_DONE_MASK as u8 {
                    println!("Packet sent: IRQMask: {:#04x}", irq);
                    break;
                }
            }
        },
    }

    handle_error!(lora.reset());
}
