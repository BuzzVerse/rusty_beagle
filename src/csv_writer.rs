use anyhow::Result;
use csv::Writer;
use std::fs::File;
use std::sync::mpsc::Receiver;
use crate::config::*;
use crate::packet::Packet;

// Data sent here from the LoRa thread through a channel.
// Either the packet, or information that a CRC error occured.
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum CSVPacketWrapper {
    Packet(Packet),
    CRC_ERROR,
}

pub struct CSVWriter {
    pub writer: Writer<File>,
}

impl CSVWriter {
    pub fn new(lora_config: &LoRaConfig) -> Result<Self>  {
        let filename = Self::generate_csv_filename(lora_config);
        Ok(
            CSVWriter {
                writer: Writer::from_path(filename)?,
            }
        )
    }

    fn generate_csv_filename(lora_config: &LoRaConfig) -> String {
        let timestamp = format!("{}", chrono::offset::Local::now().format("%Y%m%d%H%M%S"));
        format!("{}-{}-{:?}.csv", timestamp, lora_config.radio_config.frequency, lora_config.mode)
    }

    // Logging to CSV files is only needed for LoRa tests, hence the LoRaConfig parameter
    pub fn run_csv_writer(&mut self, lora_config: &LoRaConfig, csv_receiver: Receiver<CSVPacketWrapper>) -> Result<()> {
        // Headers 
        self.writer.write_record(["Timestamp", "Packet", "Bandwidth", "Coding rate", "Spreading factor", "TX power"])?;

        loop {
            // Blocks until it gets a packet
            let packet = csv_receiver.recv()?;
            // Millisecond precision
            let timestamp = format!("{}", chrono::offset::Local::now().format("%Y%m%d-%H%M%S%3f"));
            self.writer.write_record([
                timestamp,
                format!("{:?}", packet),
                format!("{:?}", lora_config.radio_config.bandwidth),
                format!("{:?}", lora_config.radio_config.coding_rate),
                format!("{:?}", lora_config.radio_config.spreading_factor),
                format!("{}", lora_config.radio_config.tx_power),
            ])?;
            self.writer.flush()?;
        }
    }
}
