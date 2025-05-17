use anyhow::Result;
use csv::Writer;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
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
    lora_config: LoRaConfig,
    path: PathBuf,
}

impl CSVWriter {
    pub fn new(lora_config: &LoRaConfig) -> Self  {
        Self {
            lora_config: lora_config.clone(),
            path: PathBuf::from("/home/debian/rusty-beagle-csv/"),
        }
    }

    fn init_writer(&self) -> Result<Writer<File>> {
        // Generate filename w/ timestamp
        let filename = Self::generate_csv_filename(&self.lora_config);

        // Create a folder for the .csv files if it doesn't exist already
        if !self.path.exists() {
            fs::create_dir(&self.path)?;
        }

        Ok(
            // Append filename to directory path
            Writer::from_path(Path::new(&self.path).join(filename))?
        )
    }

    fn generate_csv_filename(lora_config: &LoRaConfig) -> String {
        let timestamp = format!("{}", chrono::offset::Local::now().format("%Y%m%d%H%M%S"));
        format!("{}-{}-{:?}.csv", timestamp, lora_config.radio_config.frequency, lora_config.mode)
    }

    fn write_packet(&self, packet: CSVPacketWrapper, writer: &mut Writer<File>) -> Result<()> {
        // Millisecond precision
        let timestamp = format!("{}", chrono::offset::Local::now().format("%Y%m%d-%H%M%S%3f"));
        writer.write_record([
            timestamp,
            format!("{:?}", packet),
            format!("{:?}", self.lora_config.radio_config.bandwidth),
            format!("{:?}", self.lora_config.radio_config.coding_rate),
            format!("{:?}", self.lora_config.radio_config.spreading_factor),
            format!("{}", self.lora_config.radio_config.tx_power),
        ])?;
        writer.flush()?;

        Ok(())
    }

    // Logging to CSV files is only needed for LoRa tests, hence the LoRaConfig parameter
    pub fn run_csv_writer(&self, csv_receiver: Receiver<CSVPacketWrapper>) -> Result<()> {
        let mut writer = self.init_writer()?;

        // Headers 
        writer.write_record(["Timestamp", "Packet", "Bandwidth", "Coding rate", "Spreading factor", "TX power"])?;

        loop {
            // Blocks until it gets a packet
            let packet = csv_receiver.recv()?;

            self.write_packet(packet, &mut writer)?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use log::error;
    use std::sync::mpsc::channel;

    use crate::packet::{Data, DataType};

    #[test]
    fn generate_csv_filename_correct() {
        // Frequency & mode taken from the default, valid config (conf.toml in project root)
        let filename_regex = Regex::new(r"^\d{14}-433000000-RX.csv$").unwrap();
        let lora_config = Config::from_file("./conf.toml".to_string()).unwrap().lora_config.unwrap();
        let filename = CSVWriter::generate_csv_filename(&lora_config);
        assert!(filename_regex.is_match(&filename));
    }

    #[test]
    fn init_writer_ok() {
        let lora_config = Config::from_file("./conf.toml".to_string()).unwrap().lora_config.unwrap();
        let mut csv_writer = CSVWriter::new(&lora_config);
        // Different path for testing purposes
        csv_writer.path = PathBuf::from("./tests/rusty-beagle-csv/");

        assert!(csv_writer.init_writer().is_ok());
    }

    #[test]
    fn init_writer_inexistent_path() {
        let lora_config = Config::from_file("./conf.toml".to_string()).unwrap().lora_config.unwrap();
        let mut csv_writer = CSVWriter::new(&lora_config);
        // Different path for testing purposes
        csv_writer.path = PathBuf::from("./tests/inexistent_directory/rusty-beagle-csv/");

        assert!(csv_writer.init_writer().is_err());
    }

    #[test]
    fn write_packet_packet_ok() {
        let lora_config = Config::from_file("./conf.toml".to_string()).unwrap().lora_config.unwrap();
        let mut csv_writer = CSVWriter::new(&lora_config);
        // Different path for testing purposes
        csv_writer.path = PathBuf::from("./tests/rusty-beagle-csv/");
        let mut writer = csv_writer.init_writer().unwrap();
        let packet = CSVPacketWrapper::Packet(Packet {
            version: 1,
            id: 1,
            msg_id: 1,
            msg_count: 1,
            data_type: DataType::Sms,
            data: Data::Sms(String::from("Buzzverse"))
        });

        assert!(csv_writer.write_packet(packet, &mut writer).is_ok());
    }

    #[test]
    fn write_packet_crc_error_ok() {
        let lora_config = Config::from_file("./conf.toml".to_string()).unwrap().lora_config.unwrap();
        let mut csv_writer = CSVWriter::new(&lora_config);
        // Different path for testing purposes
        csv_writer.path = PathBuf::from("./tests/rusty-beagle-csv/");
        let mut writer = csv_writer.init_writer().unwrap();
        let packet = CSVPacketWrapper::CRC_ERROR;

        assert!(csv_writer.write_packet(packet, &mut writer).is_ok());
    }
}
