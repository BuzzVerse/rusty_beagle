use crate::mqtt::MQTTMessage;
use crate::sx1278::SX1278;
use crate::{config::RadioConfig, Mode};
use crate::csv_writer::CSVPacketWrapper;
use crate::{defines::*, LoRaConfig};
use anyhow::{Context, Result};
use std::sync::mpsc::Sender;

pub trait LoRa: Send {
    fn get_mode(&self) -> Mode;
    fn display_parameters(&mut self, radio_config: &RadioConfig) -> Result<()>;
    fn configure_lora(&mut self, radio_config: &RadioConfig) -> Result<()>;
    fn receive(&mut self, option_sender: Option<Sender<MQTTMessage>>) -> Result<()>;
    fn transmit(&mut self) -> Result<()>;
    fn rt_receive(&mut self, csv_sender: Sender<CSVPacketWrapper>) -> Result<()>;
    fn rt_transmit(&mut self, csv_sender: Sender<CSVPacketWrapper>) -> Result<()>;
}

pub fn lora_from_config(lora_config: &LoRaConfig) -> Result<Box<dyn LoRa>> {
    let lora: Box<dyn LoRa> = match lora_config.chip {
        Chip::SX1278 => Box::new(SX1278::from_config(lora_config)?),
    };
    Ok(lora)
}

pub fn start_lora(
    lora: &mut Box<dyn LoRa>,
    radio_config: &RadioConfig,
    option_sender: Option<Sender<MQTTMessage>>,
    csv_sender: Option<Sender<CSVPacketWrapper>>,
) -> Result<()> {
    lora.configure_lora(radio_config)
        .context("lora::start_lora")?;
    lora.display_parameters(radio_config)
        .context("lora::start_lora")?;
    match lora.get_mode() {
        Mode::RX => lora.receive(option_sender),
        Mode::TX => lora.transmit(),
        Mode::RX_RANGE_TEST => lora.rt_receive(Option::expect(csv_sender, "CSV sender not found - required for range tests.")),
        Mode::TX_RANGE_TEST => lora.rt_transmit(Option::expect(csv_sender, "CSV sender not found - required for range tests.")),
    }
    .context("lora::start_lora")?;
    Ok(())
}
