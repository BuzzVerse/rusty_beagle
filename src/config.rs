use log::{error, info};
use serde::{Deserialize, Serialize};
use std::env;
use std::{fs, process};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub mqtt_config: MQTTConfig,
    pub lora_config: LoRaConfig,
}

impl Config {
    pub fn from_file() -> Config {
        let config_file = match fs::read_to_string(Config::parse_args()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ERROR {:?}", e.to_string());
                error!("While reading config file: {e}.");
                process::exit(-1)
            }
        };

        let config: Config = match ron::from_str(config_file.as_str()) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("ERROR {:?}", e.to_string());
                error!("While deserializing ron file to config: {e}.");
                process::exit(-1);
            }
        };
        info!("Succesfully read config file.");
        config
    }

    fn parse_args() -> String {
        let args: Vec<String> = env::args().collect();

        match args.len() {
            1 => "./conf.ron".to_string(),
            2 => args[1].to_string(),
            _ => {
                eprintln!("Wrong number of arguments!");
                println!("Usage: ./rusty_beagle [config file]");
                error!("Wrong number of arguments.");
                std::process::exit(-1);
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MQTTConfig {
    pub ip: String,
    pub port: String,
    pub login: String,
    pub password: String,
    pub topic: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SPIConfig {
    pub spidev_path: String,
    pub bits_per_word: u8,
    pub max_speed_hz: u32,
    pub lsb_first: bool,
    pub spi_mode: SpiFlags,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoRaConfig {
    pub spi_config: SPIConfig,
    pub reset_gpio: u16, // TODO only allow usable GPIO pins
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
