pub use crate::lora::LoRa;

use core::time;

use crate::config::{config_input_pin, config_output_pin, RadioConfig};
use crate::csv_writer::CSVPacketWrapper;
use crate::defines::*;
use crate::mqtt::MQTTMessage;
use crate::packet::{Data, DataType, Metadata, Packet, PacketWrapper, BME280};
use crate::{LoRaConfig, Mode};
use anyhow::{anyhow, Context, Result};
use gpiod::{Edge, Input, Lines, Output};
use log::{error, info};
use spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};
use std::sync::mpsc::Sender;

macro_rules! handle_error_continue {
    ($func:expr) => {
        match $func {
            Err(e) => {
                eprintln!("{:?}", e);
                error!("{:?}", e);
                continue;
            }
            Ok(s) => s,
        }
    };
}

#[cfg(target_arch = "arm")]
pub struct SX1278 {
    spidev: Spidev,
    reset_pin: Lines<Output>,
    dio0_pin: Lines<Input>,
    pub mode: Mode,
}

#[cfg(target_arch = "x86_64")]
pub struct SX1278 {
    mock_registers: [u8; 112],
    dio0_pin: MockGPIO,
    pub mode: Mode,
}

#[cfg(target_arch = "x86_64")]
pub struct MockGPIO {}

#[cfg(target_arch = "x86_64")]
impl MockGPIO {
    fn read_event(&mut self) -> Result<MockEvent> {
        let event = MockEvent { edge: Edge::Rising };
        Ok(event)
    }
}

#[cfg(target_arch = "x86_64")]
pub struct MockEvent {
    edge: Edge,
}

impl SX1278 {
    pub fn sleep(ms: u64) {
        std::thread::sleep(time::Duration::from_millis(ms));
    }

    #[cfg(target_arch = "x86_64")]
    pub fn from_config(_lora_config: &LoRaConfig) -> Result<Self> {
        let mock_registers = [1; 112];
        let dio0_pin = MockGPIO {};
        let mode = _lora_config.mode.clone();

        Ok(Self {
            mock_registers,
            dio0_pin,
            mode,
        })
    }

    #[cfg(target_arch = "arm")]
    pub fn from_config(lora_config: &LoRaConfig) -> Result<Self> {
        let local_spi_config = lora_config.spi_config.clone();
        let mut spidev = Spidev::open(local_spi_config.spidev_path.clone())?;

        let spi_options = SpidevOptions::new()
            .bits_per_word(local_spi_config.bits_per_word)
            .max_speed_hz(local_spi_config.max_speed_hz)
            .mode(SpiModeFlags::from_bits(local_spi_config.spi_mode as u32).unwrap())
            .build();
        spidev.configure(&spi_options)?;

        let reset_pin =
            config_output_pin(lora_config.reset_gpio).context("LoRa::from_config")?;
        let dio0_pin =
            config_input_pin(lora_config.dio0_gpio).context("LoRa::from_config")?;

        let mode = lora_config.mode.clone();

        let lora = Self {
            spidev,
            reset_pin,
            dio0_pin,
            mode,
        };

        Ok(lora)
    }

    #[cfg(target_arch = "x86_64")]
    pub fn spi_read_register(
        &mut self,
        register: SX1278LoRaRegister,
        value: &mut u8,
    ) -> Result<()> {
        *value = self.mock_registers[register as usize];
        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    pub fn spi_write_register(&mut self, register: SX1278LoRaRegister, value: u8) -> Result<()> {
        self.mock_registers[register as usize] = value;
        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    pub fn reset(&mut self) -> Result<()> {
        self.mock_registers = [1; 112];

        // wait for 10 ms before using the chip
        Self::sleep(10);

        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    pub fn config_dio(&mut self) -> Result<()> {
        Ok(())
    }

    #[cfg(target_arch = "arm")]
    pub fn spi_read_register(
        &mut self,
        register: SX1278LoRaRegister,
        value: &mut u8,
    ) -> Result<()> {
        let tx_buf: [u8; 2] = [register as u8 | SPIIO::SPI_READ as u8, 0x00];
        let mut rx_buf: [u8; 2] = [0x00, 0x00];
        let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);

        match self.spidev.transfer(&mut transfer) {
            Ok(()) => {
                *value = rx_buf[1];
                Ok(())
            }
            Err(e) => Err(anyhow!(
                "While reading {:#?} got {:#?}",
                register,
                e.to_string()
            )),
        }
    }

    pub fn read_fifo(&mut self, buffer: &mut Vec<u8>) -> Result<()> {
        for value in buffer {
            self.spi_read_register(SX1278LoRaRegister::FIFO, value)
                .context("LoRa::read_fifo")?;
        }

        Ok(())
    }

    #[cfg(target_arch = "arm")]
    pub fn spi_write_register(&mut self, register: SX1278LoRaRegister, value: u8) -> Result<()> {
        let tx_buf: [u8; 2] = [register as u8 | SPIIO::SPI_WRITE as u8, value];
        let mut rx_buf: [u8; 2] = [0x00, 0x00];
        let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);

        match self.spidev.transfer(&mut transfer) {
            Ok(()) => Ok(()),
            Err(e) => Err(anyhow!(
                "While writing to {:#?} got {:#?}",
                register,
                e.to_string()
            )),
        }
    }

    pub fn write_fifo(&mut self, buffer: Vec<u8>) -> Result<()> {
        for value in buffer {
            self.spi_write_register(SX1278LoRaRegister::FIFO, value)
                .context("LoRa::write_fifo")?;
        }

        Ok(())
    }

    pub fn standby_mode(&mut self) -> Result<()> {
        self.spi_write_register(
            SX1278LoRaRegister::OP_MODE,
            SX1278LoRaMode::LONG_RANGE as u8 | SX1278LoRaMode::STDBY as u8,
        )
        .context("LoRa::standby_mode")?;
        Self::sleep(10);
        Ok(())
    }

    pub fn sleep_mode(&mut self) -> Result<()> {
        self.spi_write_register(
            SX1278LoRaRegister::OP_MODE,
            SX1278LoRaMode::LONG_RANGE as u8 | SX1278LoRaMode::SLEEP as u8,
        )
        .context("LoRa::sleep_mode")?;
        Self::sleep(10);
        Ok(())
    }

    pub fn receive_mode(&mut self) -> Result<()> {
        self.spi_write_register(
            SX1278LoRaRegister::OP_MODE,
            SX1278LoRaMode::LONG_RANGE as u8 | SX1278LoRaMode::RX_CONTINUOUS as u8,
        )
        .context("LoRa::recieve_mode")?;
        Self::sleep(10);
        Ok(())
    }

    pub fn transmit_mode(&mut self) -> Result<()> {
        self.spi_write_register(
            SX1278LoRaRegister::OP_MODE,
            SX1278LoRaMode::LONG_RANGE as u8 | SX1278LoRaMode::TX as u8,
        )
        .context("LoRa::transmit_mode")?;
        Self::sleep(10);
        Ok(())
    }

    pub fn set_tx_power(&mut self, level: u8) -> Result<()> {
        let correct_level = match level {
            0 | 1 => 2,
            2..=17 => level,
            _ => 17,
        };
        self.spi_write_register(
            SX1278LoRaRegister::PA_CONFIG,
            SX1278PAConfiguration::PA_BOOST as u8 | correct_level,
        )
        .context("LoRa::set_tx_power")?;
        Self::sleep(10);
        Ok(())
    }

    pub fn set_frequency(&mut self, frequency: u64) -> Result<()> {
        let frf = (frequency << 19) / 32_000_000;
        self.spi_write_register(SX1278LoRaRegister::FRF_MSB, (frf >> 16) as u8)
            .context("LoRa::set_frequency ")?;
        self.spi_write_register(SX1278LoRaRegister::FRF_MID, (frf >> 8) as u8)
            .context("LoRa::set_frequency ")?;
        self.spi_write_register(SX1278LoRaRegister::FRF_LSB, frf as u8)
            .context("LoRa::set_frequency ")?;
        Self::sleep(10);

        Ok(())
    }

    pub fn set_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<()> {
        let mut value = 0x00;
        let register = SX1278LoRaRegister::MODEM_CONFIG_1;
        self.spi_read_register(register, &mut value)
            .context("LoRa::set_bandwidth")?;

        let mask = 0x0f;
        self.spi_write_register(register, (value & mask) | ((bandwidth as u8) << 4))
            .context("LoRa::set_bandwidth")?;
        Self::sleep(10);

        Ok(())
    }

    pub fn set_coding_rate(&mut self, coding_rate: CodingRate) -> Result<()> {
        let mut value = 0x00;
        let register = SX1278LoRaRegister::MODEM_CONFIG_1;
        self.spi_read_register(register, &mut value)
            .context("LoRa::set_coding_rate")?;

        let mask = 0xf1;
        let cr = coding_rate as u8 - 4;
        self.spi_write_register(register, (value & mask) | (cr << 1))
            .context("LoRa::set_coding_rate")?;
        Self::sleep(10);

        Ok(())
    }

    pub fn set_spreading_factor(&mut self, spreading_factor: SpreadingFactor) -> Result<()> {
        let mut value = 0x00;
        let register = SX1278LoRaRegister::MODEM_CONFIG_2;
        self.spi_read_register(register, &mut value)
            .context("LoRa::set_spreading_factor")?;

        let reg_mask = 0x0f;
        let val_mask = 0xf0;
        self.spi_write_register(
            register,
            (value & reg_mask) | (((spreading_factor as u8) << 4) & val_mask),
        )
        .context("LoRa::set_spreading_factor")?;
        Self::sleep(10);

        Ok(())
    }

    pub fn enable_crc(&mut self) -> Result<()> {
        let mut value = 0x00;
        let crc_on = 0x04;
        let register = SX1278LoRaRegister::MODEM_CONFIG_2;
        self.spi_read_register(register, &mut value)
            .context("LoRa::enable_crc")?;

        self.spi_write_register(register, value | crc_on)
            .context("LoRa::enable_crc")?;
        Self::sleep(10);

        Ok(())
    }

    pub fn get_bandwidth(&mut self) -> Result<u8> {
        let mut value = 0x00;
        self.spi_read_register(SX1278LoRaRegister::MODEM_CONFIG_1, &mut value)
            .context("LoRa::get_bandwidth")?;

        Ok((value & 0xf0) >> 4)
    }

    pub fn get_coding_rate(&mut self) -> Result<u8> {
        let mut value = 0x00;
        self.spi_read_register(SX1278LoRaRegister::MODEM_CONFIG_1, &mut value)
            .context("LoRa::get_coding_rate")?;

        Ok(((value & 0x0e) >> 1) + 4)
    }

    pub fn get_spreading_factor(&mut self) -> Result<u8> {
        let mut value = 0x00;
        self.spi_read_register(SX1278LoRaRegister::MODEM_CONFIG_1, &mut value)
            .context("LoRa::get_spreading_factor")?;

        Ok((value >> 4) + 8)
    }

    pub fn get_frequency(&mut self) -> Result<u64> {
        let mut values: [u8; 3] = [0, 0, 0];
        self.spi_read_register(SX1278LoRaRegister::FRF_MSB, &mut values[0])
            .context("LoRa::get_frequency")?;
        self.spi_read_register(SX1278LoRaRegister::FRF_MID, &mut values[1])
            .context("LoRa::get_frequency")?;
        self.spi_read_register(SX1278LoRaRegister::FRF_LSB, &mut values[2])
            .context("LoRa::get_frequency")?;

        let msb = (values[0] as u32) << 16;
        let mid = (values[1] as u32) << 8;
        let lsb = values[2] as u32;
        let frf = msb | mid | lsb;
        let frequency = (frf as u64) * (32_000_000 / (0b1 << 19) as u64);

        Ok(frequency)
    }

    pub fn has_crc_error(&mut self, has_crc_error: &mut bool) -> Result<()> {
        let mut irq: u8 = 0x00;

        self.spi_read_register(SX1278LoRaRegister::IRQ_FLAGS, &mut irq)
            .context("LoRa::has_crc_error")?;
        if irq & SX1278IRQMask::IRQ_PAYLOAD_CRC_ERROR as u8
            == SX1278IRQMask::IRQ_PAYLOAD_CRC_ERROR as u8
        {
            *has_crc_error = true;
        }

        Ok(())
    }

    pub fn config_radio(&mut self, radio_config: &RadioConfig) -> Result<()> {
        self.set_frequency(433_000_000)
            .context("LoRa::config_radio")?;
        self.set_bandwidth(radio_config.bandwidth)
            .context("LoRa::config_radio")?;
        self.set_coding_rate(radio_config.coding_rate)
            .context("LoRa::config_radio")?;
        self.set_spreading_factor(radio_config.spreading_factor)
            .context("LoRa::config_radio")?;
        self.enable_crc().context("LoRa::config_radio")?;
        self.set_tx_power(radio_config.tx_power)
            .context("LoRa::config_radio")?;
        self.spi_write_register(SX1278LoRaRegister::MODEM_CONFIG_3, 0x04u8)
            .context("LoRa::config_radio")?;

        Ok(())
    }

    pub fn receive_packet(&mut self, crc_error: &mut bool) -> Result<Vec<u8>> {
        let mut return_length = 0;

        self.receive_mode().context("LoRa::receive_packet")?;

        loop {
            // this blocks and waits for a pin event
            let dio0_event = self.dio0_pin.read_event().context("LoRa::receive_packet")?;

            // packet is received on rising edge of DIO0
            if dio0_event.edge == Edge::Rising {
                let mut has_crc_error = false;
                self.has_crc_error(&mut has_crc_error)
                    .context("LoRa::receive_packet")?;
                if has_crc_error {
                    *crc_error = true;
                }

                break;
            }
        }

        self.standby_mode().context("LoRa::receive_packet")?;

        self.spi_read_register(SX1278LoRaRegister::RX_NB_BYTES, &mut return_length)
            .context("LoRa::receive_packet")?;
        let mut buffer: Vec<u8> = vec![0; return_length.into()];

        let mut received_address = 0x00;
        self.spi_read_register(
            SX1278LoRaRegister::FIFO_RX_CURRENT_ADDR,
            &mut received_address,
        )
        .context("LoRa::receive_packet")?;
        self.spi_write_register(SX1278LoRaRegister::FIFO_ADDR_PTR, received_address)
            .context("LoRa::receive_packet")?;

        self.read_fifo(&mut buffer)
            .context("LoRa::receive_packet")?;

        Ok(buffer)
    }

    pub fn send_packet(&mut self, buffer: Vec<u8>) -> Result<()> {
        let mut tx_address = 0x00;
        self.spi_read_register(SX1278LoRaRegister::FIFO_TX_BASE_ADDR, &mut tx_address)
            .context("LoRa::send_packet")?;
        self.spi_write_register(SX1278LoRaRegister::FIFO_ADDR_PTR, tx_address)
            .context("LoRa::send_packet")?;

        self.spi_write_register(SX1278LoRaRegister::PAYLOAD_LENGTH, buffer.len() as u8)
            .context("LoRa::send_packet")?;
        self.write_fifo(buffer).context("LoRa::send_packet")?;

        self.transmit_mode().context("LoRa::send_packet")?;

        loop {
            let dio0_event = self.dio0_pin.read_event().context("LoRa::send_packet")?;

            if dio0_event.edge == Edge::Rising {
                // rising edge of DIO0 indicates succesful packet send
                println!("Packet sent.");
                break;
            }
        }

        self.sleep_mode().context("LoRa::send_packet")?;

        Ok(())
    }

    #[cfg(target_arch = "arm")]
    pub fn reset(&mut self) -> Result<()> {
        // pull NRST pin low for 5 ms
        self.reset_pin
            .set_values(0x00_u8)
            .context("LoRa::LoRa reset: while setting reset_pin low")?;

        Self::sleep(5);

        self.reset_pin
            .set_values(0x01_u8)
            .context("LoRa::LoRa reset: while setting reset_pin high")?;

        // wait 10 ms before using the chip
        Self::sleep(10);

        Ok(())
    }

    #[cfg(target_arch = "arm")]
    pub fn config_dio(&mut self) -> Result<()> {
        let mut initial_value = 0x00;
        self.spi_read_register(SX1278LoRaRegister::DIO_MAPPING_1, &mut initial_value)
            .context("LoRa::config_dio")?;
        match self.mode {
            Mode::RX | Mode::RX_RANGE_TEST => {}
            Mode::TX | Mode::TX_RANGE_TEST => {
                self.spi_write_register(
                    SX1278LoRaRegister::DIO_MAPPING_1,
                    initial_value | (0b01 << 6),
                )
                .context("LoRa::config_dio")?; // DIO0 TxDone
            }
        }

        Ok(())
    }

    /*
     * Returns SNR[dB] on last packet received
     */
    pub fn get_packet_snr(&mut self) -> Result<u8> {
        let mut value: u8 = 0x00;
        self.spi_read_register(SX1278LoRaRegister::PKT_SNR_VALUE, &mut value)
            .context("LoRa::get_packet_snr")?;

        let snr = value.wrapping_neg() / 4;
        Ok(snr)
    }

    /*
     * Returns RSSI[dBm] of the last packet received
     */
    pub fn get_packet_rssi(&mut self) -> Result<i16> {
        let mut value: u8 = 0x00;
        let frequency = self.get_frequency().context("LoRa::get_packet_rssi")?;
        self.spi_read_register(SX1278LoRaRegister::PKT_RSSI_VALUE, &mut value)
            .context("LoRa::get_packet_rssi")?;

        let rssi = if frequency < 868_000_000 {
            -164 + (value as i16)
        } else {
            -157 + (value as i16)
        };
        Ok(rssi)
    }
}

impl LoRa for SX1278 {
    fn get_mode(&self) -> Mode {
        self.mode.clone()
    }

    fn configure_lora(&mut self, radio_config: &RadioConfig) -> Result<()> {
        self.reset().context("LoRa::start")?;
        self.sleep_mode().context("LoRa::start")?;
        self.config_radio(radio_config).context("LoRa::start")?;
        self.config_dio().context("LoRa::start")?;
        self.spi_write_register(SX1278LoRaRegister::MODEM_CONFIG_3, 0x04u8)
            .context("LoRa::start")?;
        Ok(())
    }

    fn display_parameters(&mut self, radio_config: &RadioConfig) -> Result<()> {
        let frequency = radio_config.frequency;
        println!("+-------------------------+");
        println!("| Frequency: {} MHz      |", frequency / 1_000_000);
        println!(
            "| Bandwidth: {}            |",
            self.get_bandwidth().context("LoRa::start")?
        );
        println!(
            "| Coding rate: {}          |",
            self.get_coding_rate().context("LoRa::start")?
        );
        println!(
            "| Spreading factor: {:02}    |",
            self.get_spreading_factor().context("LoRa::start")?
        );
        println!("| Mode: {:?}                |", self.mode);
        println!("+-------------------------+");
        Ok(())
    }

    fn receive(&mut self, option_sender: Option<Sender<MQTTMessage>>) -> Result<()> {
        loop {
            let mut crc_error = false;

            let received_buffer = match self.receive_packet(&mut crc_error) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{:?}", e);
                    error!("{:?}", e);
                    std::process::exit(-1);
                }
            };
            println!();
            println!(
                "--------------------------------------------------------------------------------"
            );
            println!();

            match Packet::new(&received_buffer) {
                Ok(packet) => {
                    let snr = self.get_packet_snr().context("LoRa::start")?;
                    let rssi = self.get_packet_rssi().context("LoRa::start")?;

                    if !crc_error {
                        println!(
                            "Received: {:#?}, SNR = {} dB, RSSI = {} dBm",
                            packet, snr, rssi
                        );
                        info!(
                            "Received: {:?}, SNR = {} dB, RSSI = {} dBm",
                            packet, snr, rssi
                        );
                        if let Some(lora_sender) = &option_sender {
                            let wrapped = PacketWrapper {
                                packet,
                                metadata: Metadata { snr, rssi },
                            };
                            handle_error_continue!(
                                lora_sender.send(MQTTMessage::PacketWrapper(wrapped))
                            );
                        }
                    } else {
                        // using ANSI escape codes for colors in terminal
                        println!("\x1b[0;31m[CRC ERROR]\x1b[0m\nReceived: {:#?}, SNR = {} dB, RSSI = {} dBm", packet, snr, rssi);
                        info!(
                            "[CRC ERROR] Received: {:?}, SNR = {} dB, RSSI = {} dBm",
                            packet, snr, rssi
                        );
                    }
                }
                Err(e) => {
                    println!("Bad package: {:?}", e);
                    println!();
                    println!("Received: {:02X?}", received_buffer);
                    self.sleep_mode().context("LoRa::start")?;
                    continue;
                }
            };

            self.sleep_mode().context("LoRa::start")?;
        }
    }

    fn transmit(&mut self) -> Result<()> {
        loop {
            let mut lna = 0x00;
            self.spi_read_register(SX1278LoRaRegister::LNA, &mut lna)
                .context("LoRa::start")?;
            self.spi_write_register(SX1278LoRaRegister::LNA, lna | 0x03)
                .context("LoRa::start")?;

            self.standby_mode().context("LoRa::start")?;

            let dummy_temperature: f32 = -3.2;
            let dummy_humidity: f32 = 45.6;
            let dummy_pressure: f32 = 996.6;
            let packet = Packet {
                version: 0x33,
                id: 255, // device_id = 255 for tests
                msg_id: 0x11,
                msg_count: 0x00,
                data_type: DataType::BME280,
                data: Data::Bme280(BME280 {
                    temperature: (dummy_temperature * 2.0).round() as i8 as u8,
                    humidity: dummy_humidity.round() as u8,
                    pressure: (dummy_pressure - 1000.0).round() as i8 as u8,
                }),
            };
            self.send_packet(packet.to_bytes()?)
                .context("LoRa::start")?;
            self.sleep_mode()?;
            Self::sleep(2000);
        }
    }

    fn rt_receive(&mut self, csv_sender: Sender<CSVPacketWrapper>) -> Result<()> {
        loop {
            let mut crc_error = false;

            let received_buffer = match self.receive_packet(&mut crc_error) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{:?}", e);
                    error!("{:?}", e);
                    std::process::exit(-1);
                }
            };
            println!();
            println!(
                "--------------------------------------------------------------------------------"
            );
            println!();

            match Packet::new(&received_buffer) {
                Ok(packet) => {
                    let snr = self.get_packet_snr().context("LoRa::start")?;
                    let rssi = self.get_packet_rssi().context("LoRa::start")?;

                    if !crc_error {
                        println!(
                            "Received: {:#?}, SNR = {} dB, RSSI = {} dBm",
                            packet, snr, rssi
                        );
                        info!(
                            "Received: {:?}, SNR = {} dB, RSSI = {} dBm",
                            packet, snr, rssi
                        );

                        // send to the CSV writer
                        let _ = csv_sender.send(CSVPacketWrapper::Packet(packet));
                    } else {
                        // using ANSI escape codes for colors in terminal
                        println!("\x1b[0;31m[CRC ERROR]\x1b[0m\nReceived: {:#?}, SNR = {} dB, RSSI = {} dBm", packet, snr, rssi);
                        info!(
                            "[CRC ERROR] Received: {:?}, SNR = {} dB, RSSI = {} dBm",
                            packet, snr, rssi
                        );
                    }
                }
                Err(e) => {
                    println!("Bad package: {:?}", e);
                    println!();
                    println!("Received: {:02X?}", received_buffer);
                    self.sleep_mode().context("LoRa::start")?;
                    continue;
                }
            };

            self.sleep_mode().context("LoRa::start")?;
        }
    }

    fn rt_transmit(&mut self, csv_sender: Sender<CSVPacketWrapper>) -> Result<()> {
        loop {
            let mut lna = 0x00;
            self.spi_read_register(SX1278LoRaRegister::LNA, &mut lna)
                .context("LoRa::start")?;
            self.spi_write_register(SX1278LoRaRegister::LNA, lna | 0x03)
                .context("LoRa::start")?;

            self.standby_mode().context("LoRa::start")?;

            let dummy_temperature: f32 = -3.2;
            let dummy_humidity: f32 = 45.6;
            let dummy_pressure: f32 = 996.6;
            let packet = Packet {
                version: 0x33,
                id: 255, // device_id = 255 for tests
                msg_id: 0x11,
                msg_count: 0x00,
                data_type: DataType::BME280,
                data: Data::Bme280(BME280 {
                    temperature: (dummy_temperature * 2.0).round() as i8 as u8,
                    humidity: dummy_humidity.round() as u8,
                    pressure: (dummy_pressure - 1000.0).round() as i8 as u8,
                }),
            };
            self.send_packet(packet.to_bytes()?)
                .context("LoRa::start")?;

            // send to the CSV writer
            let _ = csv_sender.send(CSVPacketWrapper::Packet(packet));

            self.sleep_mode()?;
            Self::sleep(2000);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    macro_rules! handle_error {
        ($func:expr) => {
            match $func {
                Err(e) => {
                    eprintln!("{:?}", e);
                    error!("{:?}", e);
                    std::process::exit(-1);
                }
                Ok(s) => s,
            }
        };
    }

    #[test]
    fn spi_read_register_correct() {
        let config = handle_error!(Config::from_file("./conf.toml".to_string()));
        let mut lora = match SX1278::from_config(&config.lora_config.unwrap()) {
            Ok(lora) => lora,
            Err(e) => {
                error!("When creating lora object: {e}");
                std::process::exit(-1);
            }
        };

        let mut value: u8 = 0x00;
        assert!(lora
            .spi_read_register(SX1278LoRaRegister::PAYLOAD_LENGTH, &mut value)
            .is_ok());
    }

    #[test]
    fn spi_write_register_correct() {
        let config = handle_error!(Config::from_file("./conf.toml".to_string()));
        let mut lora = match SX1278::from_config(&config.lora_config.unwrap()) {
            Ok(lora) => lora,
            Err(e) => {
                error!("When creating lora object: {e}");
                std::process::exit(-1);
            }
        };

        let value: u8 = 0xFF;
        assert!(lora
            .spi_write_register(SX1278LoRaRegister::PAYLOAD_LENGTH, value)
            .is_ok());
    }

    #[test]
    fn standby_mode_correct() {
        let config = handle_error!(Config::from_file("./conf.toml".to_string()));
        let mut lora = match SX1278::from_config(&config.lora_config.unwrap()) {
            Ok(lora) => lora,
            Err(e) => {
                error!("When creating lora object: {e}");
                std::process::exit(-1);
            }
        };

        handle_error!(lora.standby_mode());

        let mut mode: u8 = 0x00;
        handle_error!(lora.spi_read_register(SX1278LoRaRegister::OP_MODE, &mut mode));
        assert_eq!(
            (SX1278LoRaMode::LONG_RANGE as u8 | SX1278LoRaMode::STDBY as u8),
            mode
        );
    }

    #[test]
    fn sleep_mode_correct() {
        let config = handle_error!(Config::from_file("./conf.toml".to_string()));
        let mut lora = match SX1278::from_config(&config.lora_config.unwrap()) {
            Ok(lora) => lora,
            Err(e) => {
                error!("When creating lora object: {e}");
                std::process::exit(-1);
            }
        };

        handle_error!(lora.sleep_mode());

        let mut mode: u8 = 0x00;
        handle_error!(lora.spi_read_register(SX1278LoRaRegister::OP_MODE, &mut mode));
        assert_eq!(
            (SX1278LoRaMode::LONG_RANGE as u8 | SX1278LoRaMode::SLEEP as u8),
            mode
        );
    }

    #[test]
    fn receive_mode_correct() {
        let config = handle_error!(Config::from_file("./conf.toml".to_string()));
        let mut lora = match SX1278::from_config(&config.lora_config.unwrap()) {
            Ok(lora) => lora,
            Err(e) => {
                error!("When creating lora object: {e}");
                std::process::exit(-1);
            }
        };

        handle_error!(lora.receive_mode());

        let mut mode: u8 = 0x00;
        handle_error!(lora.spi_read_register(SX1278LoRaRegister::OP_MODE, &mut mode));
        assert_eq!(
            (SX1278LoRaMode::LONG_RANGE as u8 | SX1278LoRaMode::RX_CONTINUOUS as u8),
            mode
        );
    }

    #[test]
    fn transmit_mode_correct() {
        let config = handle_error!(Config::from_file("./conf.toml".to_string()));
        let mut lora = match SX1278::from_config(&config.lora_config.unwrap()) {
            Ok(lora) => lora,
            Err(e) => {
                error!("When creating lora object: {e}");
                std::process::exit(-1);
            }
        };

        handle_error!(lora.transmit_mode());

        let mut mode: u8 = 0x00;
        handle_error!(lora.spi_read_register(SX1278LoRaRegister::OP_MODE, &mut mode));
        assert_eq!(
            (SX1278LoRaMode::LONG_RANGE as u8 | SX1278LoRaMode::TX as u8),
            mode
        );
    }
}
