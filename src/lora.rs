#![allow(unused_imports)]
use core::time;
use std::thread::sleep;

// TODO delete later
use crate::defines::{api_defines::API_Status, lora_defines::*};
use crate::{GPIOPin, LoRaConfig};
use gpiod::{Lines, Output, AsValuesMut, Chip, Masked, Options};
use log::{debug, error, info, trace, warn};
use spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};
use anyhow::{anyhow, Context, Result};

#[cfg(target_arch = "arm")]
pub struct LoRa {
    spidev: Spidev,
    reset_pin: Lines<Output>,
    //dio0: SysFsGpioInput, // used to read RX_DONE and TX_DONE
}

#[cfg(target_arch = "x86_64")]
pub struct LoRa {
    mock_registers: [u8; 112],
}

impl LoRa {
    pub fn sleep(ms: u32) {
        sleep(time::Duration::from_millis(ms.into()));
    }

    #[cfg(target_arch = "x86_64")]
    pub fn from_config(_lora_config: LoRaConfig) -> Result<LoRa> {
        let mock_registers = [1; 112];
        Ok(LoRa { mock_registers })
    }

    #[cfg(target_arch = "x86_64")]
    pub fn spi_read_register(&mut self, register: LoRaRegister, value: &mut u8) -> API_Status {
        *value = self.mock_registers[register as usize];
        API_Status::API_OK
    }

    #[cfg(target_arch = "x86_64")]
    pub fn spi_write_register(&mut self, register: LoRaRegister, value: u8) -> API_Status {
        self.mock_registers[register as usize] = value;
        API_Status::API_OK
    }

    #[cfg(target_arch = "x86_64")]
    pub fn reset(&mut self) -> API_Status {
        self.mock_registers = [1; 112];

        // wait for 10 ms before using the chip
        std::thread::sleep(std::time::Duration::from_millis(10));

        API_Status::API_OK
    }

    #[cfg(target_arch = "arm")]
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

        let chip = match Chip::new(gpio_pin.chip) {
            Ok(chip) => chip,
            Err(e) => return Err(anyhow!("While creating gpio chip got {:#?}", e))
        };
        let opts = Options::output([gpio_pin.offset]);
        let reset_pin = match chip.request_lines(opts) {
            Ok(reset_pin) => reset_pin,
            Err(e) => return Err(anyhow!("While requesting gpio line got {:#?}", e))
        };

        Ok(LoRa { spidev, reset_pin })
    }

    #[cfg(target_arch = "arm")]
    pub fn spi_read_register(&mut self, register: LoRaRegister, value: &mut u8) -> Result<()> {
        let tx_buf: [u8; 2] = [register as u8 | SPIIO::SPI_READ as u8, 0x00];
        let mut rx_buf: [u8; 2] = [0x00, 0x00];
        let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);

        match self.spidev.transfer(&mut transfer) {
            Ok(()) => {
                *value = rx_buf[1];
                Ok(())
            }
            Err(e) => Err(anyhow!("While reading {:#?} got {:#?}", register, e.to_string())),
        }
    }

    #[cfg(target_arch = "arm")]
    pub fn spi_write_register(&mut self, register: LoRaRegister, value: u8) -> Result<()> {
        let tx_buf: [u8; 2] = [register as u8 | SPIIO::SPI_WRITE as u8, value];
        let mut rx_buf: [u8; 2] = [0x00, 0x00];
        let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);

        match self.spidev.transfer(&mut transfer) {
            Ok(()) => Ok(()),
            Err(e) => Err(anyhow!("While writing to {:#?} got {:#?}", register, e.to_string())),
        }
    }

    pub fn standby_mode(&mut self) -> Result<()> {
        self.spi_write_register(LoRaRegister::OP_MODE, LoRaMode::LONG_RANGE as u8 | LoRaMode::STDBY as u8)
            .context("Function - standby_mode: ")?;
        Self::sleep(10);
        Ok(())
    }

    pub fn sleep_mode(&mut self) -> Result<()> {
        self.spi_write_register(LoRaRegister::OP_MODE, LoRaMode::LONG_RANGE as u8 | LoRaMode::SLEEP as u8)
            .context("Function - sleep_mode: ")?;
        Self::sleep(10);
        Ok(())
    }

    pub fn receive_mode(&mut self) -> Result<()> {
        self.spi_write_register(LoRaRegister::OP_MODE, LoRaMode::LONG_RANGE as u8 | LoRaMode::RX_CONTINUOUS as u8)
            .context("Function - recieve_mode: ")?;
        Self::sleep(10);
        Ok(())
    }

    pub fn set_tx_power(&mut self, level: u8) -> Result<()> {
        let correct_level = match level {
            0 | 1 => 2,
            2..=17 => level,
            _ => 17,
        };
        self.spi_write_register(LoRaRegister::PA_CONFIG, PAConfiguration::PA_BOOST as u8 | correct_level)
            .context("Function - set_tx_power: ")?;
        Self::sleep(10);
        Ok(())
    }

    pub fn set_frequency(&mut self, frequency: u64) -> Result<()> {
        let frf = (frequency << 19) / 32_000_000;
        self.spi_write_register(LoRaRegister::FRF_MSB, (frf >> 16) as u8)
            .context("Function - set_frequency ")?;
        self.spi_write_register(LoRaRegister::FRF_MID, (frf >> 8) as u8)
            .context("Function - set_frequency ")?;
        self.spi_write_register(LoRaRegister::FRF_LSB, frf as u8)
            .context("Function - set_frequency ")?;

        Ok(())
    }


    #[cfg(target_arch = "arm")]
    pub fn reset(&mut self) -> Result<()> {
        // pull NRST pin low for 5 ms
        self.reset_pin.set_values(0x00_u8).context("LoRa reset: while setting reset_pin low: ")?;

        Self::sleep(5);

        self.reset_pin.set_values(0x01_u8).context("LoRa reset: while setting reset_pin high: ")?;

        // wait 10 ms before using the chip
        Self::sleep(10);

        Ok(())
    }

}
