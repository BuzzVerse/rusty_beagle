use crate::config::SPIConfig;
use crate::defines::{api_defines::API_Status, lora_defines::*};
use log::{debug, error, info, trace, warn};
use spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};
use std::io::Result;

pub struct Lora {
    spidev: Spidev,
}

impl Lora {
    pub fn from_config(spi_config: SPIConfig) -> Result<Lora> {
        let mut spidev = Spidev::open(spi_config.spidev_path)?;

        let spi_options = SpidevOptions::new()
            .bits_per_word(spi_config.bits_per_word)
            .max_speed_hz(spi_config.max_speed_hz)
            .mode(SpiModeFlags::from_bits(spi_config.spi_mode as u32).unwrap())
            .build();
        spidev.configure(&spi_options)?;

        Ok(Lora { spidev })
    }

    pub fn spi_read_register(&mut self, register: u8, value: &mut u8) -> API_Status {
        let tx_buf: [u8; 2] = [register | SPI_READ, 0x00];
        let mut rx_buf: [u8; 2] = [0x00, 0x00];
        let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);

        match self.spidev.transfer(&mut transfer) {
            Err(e) => {
                eprintln!("{:?}", e.to_string());
                error!("While reading LoRa register {register} got {e}");
                API_Status::API_SPI_ERROR
            }
            Ok(()) => {
                *value = rx_buf[1];
                API_Status::API_OK
            }
        }
    }

    pub fn spi_write_register(&mut self, register: u8, value: u8) -> API_Status {
        let tx_buf: [u8; 2] = [register | SPI_WRITE, value];
        let mut rx_buf: [u8; 2] = [0x00, 0x00];
        let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);

        match self.spidev.transfer(&mut transfer) {
            Err(e) => {
                eprintln!("{:?}", e.to_string());
                error!("While writing to LoRa register {register} got {e}");
                API_Status::API_SPI_ERROR
            }
            Ok(()) => API_Status::API_OK,
        }
    }
}
