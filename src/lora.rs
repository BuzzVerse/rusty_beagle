use crate::defines::*;
use crate::mqtt::MQTTMessage;
use crate::{config::RadioConfig, Mode};
use anyhow::{Context, Result};
use std::sync::mpsc::Sender;

pub trait LoRa: Send {
    fn spi_read_register(&mut self, register: LoRaRegister, value: &mut u8) -> Result<()>;
    fn spi_write_register(&mut self, register: LoRaRegister, value: u8) -> Result<()>;
    fn reset(&mut self) -> Result<()>;
    fn read_fifo(&mut self, buffer: &mut Vec<u8>) -> Result<()>;
    fn write_fifo(&mut self, buffer: Vec<u8>) -> Result<()>;
    fn standby_mode(&mut self) -> Result<()>;
    fn sleep_mode(&mut self) -> Result<()>;
    fn receive_mode(&mut self) -> Result<()>;
    fn transmit_mode(&mut self) -> Result<()>;
    fn set_tx_power(&mut self, level: u8) -> Result<()>;
    fn set_frequency(&mut self, frequency: u64) -> Result<()>;
    fn set_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<()>;
    fn set_coding_rate(&mut self, coding_rate: CodingRate) -> Result<()>;
    fn set_spreading_factor(&mut self, spreading_factor: SpreadingFactor) -> Result<()>;
    fn enable_crc(&mut self) -> Result<()>;
    fn get_bandwidth(&mut self) -> Result<u8>;
    fn get_coding_rate(&mut self) -> Result<u8>;
    fn get_spreading_factor(&mut self) -> Result<u8>;
    fn get_frequency(&mut self) -> Result<u64>;
    fn get_mode(&self) -> Mode;
    fn has_crc_error(&mut self, has_crc_error: &mut bool) -> Result<()>;
    fn config_radio(&mut self, radio_config: &RadioConfig) -> Result<()>;
    fn receive_packet(&mut self, crc_error: &mut bool) -> Result<Vec<u8>>;
    fn send_packet(&mut self, buffer: Vec<u8>) -> Result<()>;
    fn config_dio(&mut self) -> Result<()>;
    fn get_packet_snr(&mut self) -> Result<u8>;
    fn get_packet_rssi(&mut self) -> Result<i16>;
    fn display_parameters(&mut self, radio_config: &RadioConfig) -> Result<()>;
    fn configure_lora(&mut self, radio_config: &RadioConfig) -> Result<()>;
    fn receive(&mut self, option_sender: Option<Sender<MQTTMessage>>) -> Result<()>;
    fn transmit(&mut self) -> Result<()>;
    fn rt_receive(&mut self, option_sender: Option<Sender<MQTTMessage>>) -> Result<()>;
    fn rt_transmit(&mut self) -> Result<()>;
}

pub fn start_lora(
    lora: &mut Box<dyn LoRa>,
    radio_config: &RadioConfig,
    option_sender: Option<Sender<MQTTMessage>>,
) -> Result<()> {
    lora.configure_lora(radio_config)
        .context("lora::start_lora")?;
    lora.display_parameters(radio_config)
        .context("lora::start_lora")?;
    match lora.get_mode() {
        Mode::RX => lora.receive(option_sender),
        Mode::TX => lora.transmit(),
    }
    .context("lora::start_lora")?;
    Ok(())
}
