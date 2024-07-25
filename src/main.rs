mod config;
mod defines;
mod logging;

extern crate log;

pub use crate::config::*;
pub use crate::defines::{api_defines::API_Status, lora_defines::*};
pub use crate::logging::start_logger;
use spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};
//use log::{debug, error, info, trace, warn};

#[cfg(target_arch = "x86_64")]
fn prepare_mocks() {
    println!("prepare_mocks(): Running on x86_64.");
}

fn spi_read_register(spidev: &mut Spidev, register: u8, value: &mut u8) -> API_Status {
    let tx: [u8; 2] = [register | SPI_READ, 0x00];
    let mut rx: [u8; 2] = [0x00, 0x00];
    let mut transfer = SpidevTransfer::read_write(&tx, &mut rx);

    spidev.transfer(&mut transfer).unwrap();
    *value = rx[1];

    API_Status::API_OK
}

fn main() {
    start_logger();

    #[cfg(target_arch = "x86_64")]
    prepare_mocks();

    let config = Config::from_file();

    let spi_options = SpidevOptions::new()
        .bits_per_word(config.spi_config.bits_per_word)
        .max_speed_hz(config.spi_config.max_speed_hz)
        .mode(SpiModeFlags::from_bits(config.spi_config.spi_mode as u32).unwrap())
        .build();

    let mut spidev = Spidev::open("/dev/spidev0.0").unwrap();
    spidev.configure(&spi_options).unwrap();

    let mut value = 0x00;
    spi_read_register(&mut spidev, REG_OP_MODE, &mut value);
    println!("value: {}", value);
}
