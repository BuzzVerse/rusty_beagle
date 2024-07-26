use crate::defines::{api_defines::API_Status, lora_defines::*};
use crate::{GPIOPin, LoRaConfig};
#[allow(unused_imports)] // TODO delete later
use gpiod::{AsValuesMut, Chip, Masked, Options};
use gpiod::{Lines, Output};
#[allow(unused_imports)] // TODO delete later
use log::{debug, error, info, trace, warn};
use spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};
use std::io::Result;

pub struct LoRa {
    spidev: Spidev,
    reset_pin: Lines<Output>,
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
        let gpio_pin = GPIOPin::from_gpio_pin_number(lora_config.reset_gpio);

        // let reset_pin = SysFsGpioOutput::open(lora_config.reset_gpio as u16).unwrap();
        let chip = match Chip::new(gpio_pin.chip) {
            Ok(chip) => chip,
            Err(e) => {
                eprintln!("Chip: {}", e.to_string());
                error!("Chip: {e}");
                std::process::exit(-1);
            }
        };
        let opts = Options::output([gpio_pin.offset]);
        let reset_pin = match chip.request_lines(opts) {
            Ok(reset_pin) => reset_pin,
            Err(e) => {
                eprintln!("Reset pin: {}", e.to_string());
                error!("Reset pin: {e}");
                std::process::exit(-1);
            }
        };
        Ok(LoRa { spidev, reset_pin })
    }

    pub fn spi_read_register(&mut self, register: LoRaRegister, value: &mut u8) -> API_Status {
        let tx_buf: [u8; 2] = [register as u8 | SPIIO::SPI_READ as u8, 0x00];
        let mut rx_buf: [u8; 2] = [0x00, 0x00];
        let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);

        match self.spidev.transfer(&mut transfer) {
            Err(e) => {
                eprintln!("{:?}", e.to_string());
                error!("While reading LoRa register {:#?} got {e}", register);
                API_Status::API_SPI_ERROR
            }
            Ok(()) => {
                *value = rx_buf[1];
                API_Status::API_OK
            }
        }
    }

    pub fn spi_write_register(&mut self, register: LoRaRegister, value: u8) -> API_Status {
        let tx_buf: [u8; 2] = [register as u8 | SPIIO::SPI_WRITE as u8, value];
        let mut rx_buf: [u8; 2] = [0x00, 0x00];
        let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);

        match self.spidev.transfer(&mut transfer) {
            Err(e) => {
                eprintln!("{:?}", e.to_string());
                error!("While writing to LoRa register {:#?} got {e}", register);
                API_Status::API_SPI_ERROR
            }
            Ok(()) => API_Status::API_OK,
        }
    }

    pub fn reset(&mut self) -> API_Status {
        // pull NRST pin low for 5 ms
        match self.reset_pin.set_values(0x00 as u8) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("While setting reset_pin low: {}", e.to_string());
                error!("While setting reset_pin low: {e}");
                std::process::exit(-1);
            }
        };
        // .expect("Could not set reset_pin low.");

        std::thread::sleep(std::time::Duration::from_millis(5));

        match self.reset_pin.set_values(0x01 as u8) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("While setting reset_pin high: {}", e.to_string());
                error!("While setting reset_pin high: {e}");
                std::process::exit(-1);
            }
        }

        // wait for 10 ms before using the chip
        std::thread::sleep(std::time::Duration::from_millis(10));

        API_Status::API_OK
    }
}
