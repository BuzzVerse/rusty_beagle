use crate::defines::{api_defines::API_Status, lora_defines::*};
use crate::LoRaConfig;
#[allow(unused_imports)] // TODO delete later
use gpio::sysfs::{SysFsGpioInput, SysFsGpioOutput};
use gpio::GpioOut;
#[allow(unused_imports)] // TODO delete later
use log::{debug, error, info, trace, warn};
use spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};
use std::io::Result;

pub struct LoRa {
    spidev: Spidev,
    reset_pin: SysFsGpioOutput,
    //dio0: SysFsGpioInput, // used to read RX_DONE and TX_DONE
}

impl LoRa {
    pub fn from_config(lora_config: LoRaConfig) -> Result<LoRa> {
        let local_spi_config = lora_config.spi_config;
        let mut spidev = Spidev::open(local_spi_config.spidev_path)?;

        let spi_options = SpidevOptions::new()
            .bits_per_word(local_spi_config.bits_per_word)
            .max_speed_hz(local_spi_config.max_speed_hz)
            .mode(SpiModeFlags::from_bits(local_spi_config.spi_mode as u32).unwrap())
            .build();
        spidev.configure(&spi_options)?;

        let reset_pin = gpio::sysfs::SysFsGpioOutput::open(lora_config.reset_gpio).unwrap();

        Ok(LoRa { spidev, reset_pin })
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

    pub fn reset(&mut self) -> API_Status {
        // pull NRST pin low for 5 ms
        self.reset_pin
            .set_low()
            .expect("Could not set reset_pin low.");

        std::thread::sleep(std::time::Duration::from_millis(5));

        self.reset_pin
            .set_high()
            .expect("Could not set reset_pin high.");

        // wait for 10 ms before using the chip
        std::thread::sleep(std::time::Duration::from_millis(10));

        API_Status::API_OK
    }
}
