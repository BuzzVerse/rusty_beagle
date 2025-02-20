use anyhow::{Context, Result};
use signal_hook::{
    consts::{SIGINT, SIGQUIT, SIGTERM},
    iterator::Signals,
};
use std::sync::mpsc::Sender;
use std::time::Duration;
use std::thread;

use crate::config::config_output_pin;
use crate::GPIOPinNumber;

pub fn run_signal_handler(signal_sender: Sender<i32>) -> Result<()> {
    let mut signals = Signals::new([SIGINT, SIGQUIT, SIGTERM])?;

    for signal in signals.forever() {
        signal_sender.send(signal)?;
    }

    Ok(())
}

/*
 * The point of this function is to be able to reset LoRa
 * without an existing LoRa object - all that is needed
 * is the reset GPIO pin number from the config file
 */
pub fn emergency_reset(reset_pin_number: GPIOPinNumber) -> Result<()> {
    let reset_pin = config_output_pin(reset_pin_number)?;

    // pull NRST pin low for 5 ms
    reset_pin
        .set_values(0x00_u8)
        .context("LoRa::LoRa reset: while setting reset_pin low")?;

    thread::sleep(Duration::from_millis(5));

    reset_pin
        .set_values(0x01_u8)
        .context("LoRa::LoRa reset: while setting reset_pin high")?;

    // wait 10 ms before using the chip
    thread::sleep(Duration::from_millis(10));

    Ok(())
}
