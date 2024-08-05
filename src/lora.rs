#![allow(unused_imports)]
use core::time;
use std::thread::sleep;

use crate::config::RadioConfig;
use crate::conversions::*;
use crate::defines::*;
use crate::{GPIOPin, GPIOPinNumber, LoRaConfig, Mode};
use anyhow::{anyhow, Context, Result};
use gpiod::{AsValuesMut, Chip, Lines, Masked, Options, Output, Input, Edge, EdgeDetect};
use gpiod::DirectionType;
use log::{debug, error, info, trace, warn};
use spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};

#[cfg(target_arch = "arm")]
pub struct LoRa {
    spidev: Spidev,
    reset_pin: Lines<Output>,
    dio0_pin: Lines<Input>,
    pub mode: Mode,
}

#[cfg(target_arch = "x86_64")]
pub struct LoRa {
    mock_registers: [u8; 112],
}

impl LoRa {
    pub fn sleep(ms: u64) {
        sleep(time::Duration::from_millis(ms));
    }

    #[cfg(target_arch = "x86_64")]
    pub fn from_config(_lora_config: &LoRaConfig) -> Result<LoRa> {
        let mock_registers = [1; 112];
        Ok(LoRa { mock_registers })
    }

    #[cfg(target_arch = "x86_64")]
    pub fn spi_read_register(&mut self, register: LoRaRegister, value: &mut u8) -> Result<()> {
        *value = self.mock_registers[register as usize];
        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    pub fn spi_write_register(&mut self, register: LoRaRegister, value: u8) -> Result<()> {
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

    #[cfg(target_arch = "arm")]
    pub fn from_config(lora_config: &LoRaConfig) -> Result<LoRa> {
        let local_spi_config = lora_config.spi_config.clone();
        let mut spidev = Spidev::open(local_spi_config.spidev_path.clone())?;

        let spi_options = SpidevOptions::new()
            .bits_per_word(local_spi_config.bits_per_word)
            .max_speed_hz(local_spi_config.max_speed_hz)
            .mode(SpiModeFlags::from_bits(local_spi_config.spi_mode as u32).unwrap())
            .build();
        spidev.configure(&spi_options)?;

        let reset_pin = Self::config_output_pin(lora_config.reset_gpio).context("from_config: ")?;
        let dio0_pin = Self::config_input_pin(lora_config.dio0_gpio).context("from_config: ")?;

        let mode = lora_config.mode.clone();

        let lora = LoRa {
            spidev,
            reset_pin,
            dio0_pin,
            mode,
        };

        Ok(lora)
    }

    #[cfg(target_arch = "arm")]
    fn config_output_pin(pin_number: GPIOPinNumber) -> Result<gpiod::Lines<Output>> {

        let pin = GPIOPin::from_gpio_pin_number(pin_number);

        let chip = match Chip::new(pin.chip) {
            Ok(chip) => chip,
            Err(e) => return Err(anyhow!("While creating gpio chip got {:#?}", e)),
        };

        let opts = Options::output([pin.offset]);

        let line = match chip.request_lines(opts) {
            Ok(line) => line,
            Err(e) => return Err(anyhow!("While requesting gpio line got {:#?}", e)),
        };

        Ok(line)
    }

    #[cfg(target_arch = "arm")]
    fn config_input_pin(pin_number: GPIOPinNumber) -> Result<gpiod::Lines<Input>> {

        let pin = GPIOPin::from_gpio_pin_number(pin_number);

        let chip = match Chip::new(pin.chip) {
            Ok(chip) => chip,
            Err(e) => return Err(anyhow!("While creating gpio chip got {:#?}", e)),
        };

        let opts = Options::input([pin.offset])
            .edge(EdgeDetect::Rising)
            .consumer("dio0_pin");

        let line = match chip.request_lines(opts) {
            Ok(line) => line,
            Err(e) => return Err(anyhow!("While requesting gpio line got {:#?}", e)),
        };

        Ok(line)
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
            Err(e) => Err(anyhow!(
                "While reading {:#?} got {:#?}",
                register,
                e.to_string()
            )),
        }
    }

    pub fn read_fifo(&mut self, buffer: &mut Vec<u8>) -> Result<()> {
        for value in buffer {
            self.spi_read_register(LoRaRegister::FIFO, value)
                .context("read_fifo: ")?;
        }

        Ok(())
    }

    #[cfg(target_arch = "arm")]
    pub fn spi_write_register(&mut self, register: LoRaRegister, value: u8) -> Result<()> {
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
            self.spi_write_register(LoRaRegister::FIFO, value)
                .context("write_fifo: ")?;
        }

        Ok(())
    }

    pub fn standby_mode(&mut self) -> Result<()> {
        self.spi_write_register(
            LoRaRegister::OP_MODE,
            LoRaMode::LONG_RANGE as u8 | LoRaMode::STDBY as u8,
        )
        .context("standby_mode: ")?;
        Self::sleep(10);
        Ok(())
    }

    pub fn sleep_mode(&mut self) -> Result<()> {
        self.spi_write_register(
            LoRaRegister::OP_MODE,
            LoRaMode::LONG_RANGE as u8 | LoRaMode::SLEEP as u8,
        )
        .context("sleep_mode: ")?;
        Self::sleep(10);
        Ok(())
    }

    pub fn receive_mode(&mut self) -> Result<()> {
        self.spi_write_register(
            LoRaRegister::OP_MODE,
            LoRaMode::LONG_RANGE as u8 | LoRaMode::RX_CONTINUOUS as u8,
        )
        .context("recieve_mode: ")?;
        Self::sleep(10);
        Ok(())
    }

    pub fn transmit_mode(&mut self) -> Result<()> {
        self.spi_write_register(
            LoRaRegister::OP_MODE,
            LoRaMode::LONG_RANGE as u8 | LoRaMode::TX as u8,
        )
        .context("transmit_mode: ")?;
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
            LoRaRegister::PA_CONFIG,
            PAConfiguration::PA_BOOST as u8 | correct_level,
        )
        .context("set_tx_power: ")?;
        Self::sleep(10);
        Ok(())
    }

    pub fn set_frequency(&mut self, frequency: u64) -> Result<()> {
        let frf = (frequency << 19) / 32_000_000;
        self.spi_write_register(LoRaRegister::FRF_MSB, (frf >> 16) as u8)
            .context("set_frequency ")?;
        self.spi_write_register(LoRaRegister::FRF_MID, (frf >> 8) as u8)
            .context("set_frequency ")?;
        self.spi_write_register(LoRaRegister::FRF_LSB, frf as u8)
            .context("set_frequency ")?;
        Self::sleep(10);

        Ok(())
    }

    pub fn set_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<()> {
        let mut value = 0x00;
        let register = LoRaRegister::MODEM_CONFIG_1;
        self.spi_read_register(register, &mut value)
            .context("set_bandwidth: ")?;

        let mask = 0x0f;
        self.spi_write_register(register, (value & mask) | ((bandwidth as u8) << 4))
            .context("set_bandwidth: ")?;
        Self::sleep(10);

        Ok(())
    }

    pub fn set_coding_rate(&mut self, coding_rate: CodingRate) -> Result<()> {
        let mut value = 0x00;
        let register = LoRaRegister::MODEM_CONFIG_1;
        self.spi_read_register(register, &mut value)
            .context("set_coding_rate: ")?;

        let mask = 0xf1;
        let cr = coding_rate as u8 - 4;
        self.spi_write_register(register, (value & mask) | (cr << 1))
            .context("set_coding_rate: ")?;
        Self::sleep(10);

        Ok(())
    }

    pub fn set_spreading_factor(&mut self, spreading_factor: SpreadingFactor) -> Result<()> {
        let mut value = 0x00;
        let register = LoRaRegister::MODEM_CONFIG_2;
        self.spi_read_register(register, &mut value)
            .context("set_spreading_factor: ")?;

        let reg_mask = 0x0f;
        let val_mask = 0xf0;
        self.spi_write_register(
            register,
            (value & reg_mask) | (((spreading_factor as u8) << 4) & val_mask),
        )
        .context("set_spreading_factor: ")?;
        Self::sleep(10);

        Ok(())
    }

    pub fn enable_crc(&mut self) -> Result<()> {
        let mut value = 0x00;
        let crc_on = 0x04;
        let register = LoRaRegister::MODEM_CONFIG_2;
        self.spi_read_register(register, &mut value)
            .context("enable_crc: ")?;

        self.spi_write_register(register, value | crc_on)
            .context("enable_crc: ")?;
        Self::sleep(10);

        Ok(())
    }
    pub fn get_bandwidth(&mut self) -> Result<u8> {
        let mut value = 0x00;
        self.spi_read_register(LoRaRegister::MODEM_CONFIG_1, &mut value)
            .context("get_bandwidth: ")?;

        Ok((value & 0xf0) >> 4)
    }

    pub fn get_coding_rate(&mut self) -> Result<u8> {
        let mut value = 0x00;
        self.spi_read_register(LoRaRegister::MODEM_CONFIG_1, &mut value)
            .context("get_coding_rate: ")?;

        Ok(((value & 0x0e) >> 1) + 4)
    }

    pub fn get_spreading_factor(&mut self) -> Result<u8> {
        let mut value = 0x00;
        self.spi_read_register(LoRaRegister::MODEM_CONFIG_1, &mut value)
            .context("get_spreading_factor: ")?;

        Ok((value >> 4) + 9)
    }

    pub fn get_frequency(&mut self) -> Result<[u8; 3]> {
        let mut values: [u8; 3] = [0, 0, 0];
        self.spi_read_register(LoRaRegister::FRF_MSB, &mut values[0])
            .context("get_frequency: ")?;
        self.spi_read_register(LoRaRegister::FRF_MID, &mut values[1])
            .context("get_frequency: ")?;
        self.spi_read_register(LoRaRegister::FRF_LSB, &mut values[2])
            .context("get_frequency: ")?;

        Ok(values)
    }

    pub fn config_radio(&mut self, radio_config: RadioConfig) -> Result<()> {
        self.set_frequency(433_000_000).context("config_radio: ")?;
        self.set_bandwidth(radio_config.bandwidth)
            .context("config_radio: ")?;
        self.set_coding_rate(radio_config.coding_rate)
            .context("config_radio: ")?;
        self.set_spreading_factor(radio_config.spreading_factor)
            .context("config_radio: ")?;
        self.enable_crc().context("config_radio: ")?;
        self.set_tx_power(radio_config.tx_power)
            .context("config_radio: ")?;

        Ok(())
    }

    fn has_crc_error(&mut self, has_crc_error: &mut bool) -> Result<()> {
        let mut irq: u8 = 0x00;

        self.spi_read_register(LoRaRegister::IRQ_FLAGS, &mut irq)
            .context("has_crc_error: ")?;
        if irq & IRQMask::IRQ_PAYLOAD_CRC_ERROR as u8 == IRQMask::IRQ_PAYLOAD_CRC_ERROR as u8 {
            *has_crc_error = true;
        }

        Ok(())
    }

    pub fn receive_packet(&mut self, crc_error: &mut bool) -> Result<Vec<u8>> {
        let mut return_length = 0;

        self.receive_mode().context("receive_packet: ")?;

        loop {
            let dio0_event = self.dio0_pin.read_event().context("receive_packet: ")?;

            if dio0_event.edge == Edge::Rising { // packet is received on rising edge of DIO0
                let mut has_crc_error = false;
                self.has_crc_error(&mut has_crc_error)
                    .context("receive_packet: ")?;
                if has_crc_error {
                    *crc_error = true;
                }

                break;
            }
        }

        self.standby_mode().context("receive_packet: ")?;

        self.spi_read_register(LoRaRegister::RX_NB_BYTES, &mut return_length)
            .context("receive_packet: ")?;
        let mut buffer: Vec<u8> = vec![0; return_length.into()];

        let mut received_address = 0x00;
        self.spi_read_register(LoRaRegister::FIFO_RX_CURRENT_ADDR, &mut received_address)
            .context("receive_packet: ")?;
        self.spi_write_register(LoRaRegister::FIFO_ADDR_PTR, received_address)
            .context("receive_packet: ")?;

        self.read_fifo(&mut buffer).context("receive_packet: ")?;

        Ok(buffer)
    }

    pub fn send_packet(&mut self, buffer: Vec<u8>) -> Result<()> {
        // TODO rework to send buffers instead of single bytes, related issue: [RB-8]
        let mut tx_address = 0x00;
        self.spi_read_register(LoRaRegister::FIFO_TX_BASE_ADDR, &mut tx_address)
            .context("send_packet: ")?;
        self.spi_write_register(LoRaRegister::FIFO_ADDR_PTR, tx_address)
            .context("send_packet: ")?;

        self.spi_write_register(LoRaRegister::PAYLOAD_LENGTH, buffer.len() as u8)
            .context("send_packet: ")?;
        self.write_fifo(buffer).context("send_packet: ")?;

        // send_packet()
        let mut irq: u8 = 0x00;

        self.transmit_mode().context("send_packet: ")?;

        loop {
            self.spi_read_register(LoRaRegister::IRQ_FLAGS, &mut irq)
                .context("send_packet: ")?;
            if irq & IRQMask::IRQ_TX_DONE_MASK as u8 == IRQMask::IRQ_TX_DONE_MASK as u8 {
                println!("Packet sent: IRQMask: {:#04x}", irq);
                break;
            }
        }

        self.sleep_mode().context("send_packet: ")?;

        Ok(())
    }

    pub fn start(&mut self, radio_config: RadioConfig) -> Result<()> {
        self.reset().context("start: ")?;
        self.sleep_mode().context("start: ")?;
        self.config_radio(radio_config).context("start: ")?;
        self.spi_write_register(LoRaRegister::MODEM_CONFIG_3, 0x04u8)
            .context("start: ")?;
        println!("Bandwidth: {}", self.get_bandwidth().unwrap());
        println!("Coding rate: {}", self.get_coding_rate().unwrap());
        println!("Spreading factor: {}", self.get_spreading_factor().unwrap());
        println!("Frequency: {:?}", self.get_frequency().unwrap());
        for _ in 0..10000 {
            match self.mode {
                Mode::RX => {
                    println!("[MODE]: RX");

                    let mut value = 0x00;
                    self.spi_read_register(LoRaRegister::OP_MODE, &mut value)
                        .context("start: ")?;
                    println!("value: {:#04x} (expected 0x80)", value);

                    let mut crc_error = false;

                    let received_buffer = match self.receive_packet(&mut crc_error) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("{:?}", e);
                            error!("{:?}", e);
                            std::process::exit(-1);
                        }
                    };

                    if crc_error {
                        println!("CRC Error");
                    }
                    println!(
                        "Received {:?} byte(s): {:?}",
                        received_buffer.len(),
                        String::from_utf8(received_buffer)
                    );
                    self.sleep_mode().context("start: ")?;
                }
                Mode::TX => {
                    println!("[MODE]: TX");

                    let mut value = 0x00;
                    self.spi_read_register(LoRaRegister::OP_MODE, &mut value)
                        .context("start: ")?;
                    println!("value: {:#04x} (expected 0x80)", value);

                    let mut lna = 0x00;
                    self.spi_read_register(LoRaRegister::LNA, &mut lna)
                        .context("start: ")?;
                    self.spi_write_register(LoRaRegister::LNA, lna | 0x03)
                        .context("start: ")?;

                    self.standby_mode().context("start: ")?;

                    let packet = String::from("BUZZVERSE").as_bytes().to_vec();

                    self.send_packet(packet).context("start: ")?;
                    self.sleep_mode()?;
                    Self::sleep(2000);
                }
            }
        }

        self.reset().context("start: ")?;
        Ok(())
    }

    #[cfg(target_arch = "arm")]
    pub fn reset(&mut self) -> Result<()> {
        // pull NRST pin low for 5 ms
        self.reset_pin
            .set_values(0x00_u8)
            .context("LoRa reset: while setting reset_pin low: ")?;

        Self::sleep(5);

        self.reset_pin
            .set_values(0x01_u8)
            .context("LoRa reset: while setting reset_pin high: ")?;

        // wait 10 ms before using the chip
        Self::sleep(10);

        Ok(())
    }
}
