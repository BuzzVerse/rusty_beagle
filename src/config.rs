use crate::defines::{Bandwidth, CodingRate, SpreadingFactor};
use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub mqtt_config: MQTTConfig,
    pub lora_config: LoRaConfig,
    pub bme_config: BME280Config
}

impl Config {
    pub fn from_file(config_path: String) -> Result<Config> {
        let config_file = fs::read_to_string(config_path).context("Config::from_file")?;
        let config = ron::from_str(config_file.as_str()).context("Config::from_file")?;
        info!("Succesfully read config file.");
        Ok(config)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MQTTConfig {
    pub ip: String,
    pub port: String,
    pub login: String,
    pub password: String,
    pub topic: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BME280Config {
    pub i2c_bus_path: String,
    pub i2c_address: u8,
    pub measurement_interval: u64,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SPIConfig {
    pub spidev_path: String,
    pub bits_per_word: u8,
    pub max_speed_hz: u32,
    pub lsb_first: bool,
    pub spi_mode: SpiFlags,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoRaConfig {
    pub mode: Mode,
    pub reset_gpio: GPIOPinNumber,
    pub dio0_gpio: GPIOPinNumber,
    pub spi_config: SPIConfig,
    pub radio_config: RadioConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RadioConfig {
    pub frequency: u64,
    pub bandwidth: Bandwidth,
    pub coding_rate: CodingRate,
    pub spreading_factor: SpreadingFactor,
    pub tx_power: u8,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Mode {
    RX,
    TX,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum GPIOPinNumber {
    GPIO_26 = 26,
    GPIO_27 = 27,
    GPIO_40 = 40,
    GPIO_44 = 44,
    GPIO_45 = 45,
    GPIO_46 = 46,
    GPIO_47 = 47,
    GPIO_60 = 60,
    GPIO_61 = 61,
    GPIO_65 = 65,
    GPIO_66 = 66,
    GPIO_67 = 67,
    GPIO_68 = 68,
    GPIO_69 = 69,
}

pub struct GPIOPin {
    pub chip: String,
    pub offset: u32,
}

impl GPIOPin {
    pub fn from_gpio_pin_number(gpio_pin_number: GPIOPinNumber) -> GPIOPin {
        let pin_number = gpio_pin_number as u32;
        let chip = match pin_number {
            0..=31 => "gpiochip0".to_string(),
            32..=63 => "gpiochip1".to_string(),
            64..=95 => "gpiochip2".to_string(),
            _ => "gpiochip3".to_string(),
        };

        GPIOPin {
            chip,
            offset: pin_number % 32,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum SpiFlags {
    /// Clock Phase
    SPI_CPHA = 0x01,
    /// Clock Polarity
    SPI_CPOL = 0x02,
    /// Chipselect Active High?
    SPI_CS_HIGH = 0x04,
    /// Per-word Bits On Wire
    SPI_LSB_FIRST = 0x08,
    /// SI/SO Signals Shared
    SPI_3WIRE = 0x10,
    /// Loopback Mode
    SPI_LOOP = 0x20,
    /// 1 dev/bus, no chipselect
    SPI_NO_CS = 0x40,
    /// Slave pulls low to pause
    SPI_READY = 0x80,

    // Common Configurations
    SPI_MODE_0 = 0x00,
    // SPI_MODE_1 = Self::SPI_CPHA.bits(),
    // SPI_MODE_2 = Self::SPI_CPOL.bits(),
    // SPI_MODE_3 = (Self::SPI_CPOL.bits() | Self::SPI_CPHA.bits()),

    // == Only Supported with 32-bits ==
    /// Transmit with 2 wires
    SPI_TX_DUAL = 0x100,
    /// Transmit with 4 wires
    SPI_TX_QUAD = 0x200,
    /// Receive with 2 wires
    SPI_RX_DUAL = 0x400,
    /// Receive with 4 wires
    SPI_RX_QUAD = 0x800,
}

impl SpiFlags {
    pub const SPI_MODE_1: SpiFlags = SpiFlags::SPI_CPHA;
    pub const SPI_MODE_2: SpiFlags = SpiFlags::SPI_CPOL;
    pub const SPI_MODE_3: SpiFlags = SpiFlags::SPI_MODE_0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_correct() {
        assert!(Config::from_file("./tests/configs/conf.ron".to_string()).is_ok());
    }

    #[test]
    fn config_incomplete() {
        assert!(Config::from_file("./tests/configs/incomplete_conf.ron".to_string()).is_err());
    }
}
