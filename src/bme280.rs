use std::time::Duration;
use std::thread;
use anyhow::{Context, Result};
use bme280::i2c::BME280;
use crate::packet::{BME280 as PacketBME280, Data, DataType, Packet};
use linux_embedded_hal::{Delay, I2cdev};
use log::{error, info};
use crate::BME280Config;
use std::sync::mpsc::Sender;

macro_rules! handle_error_exit {
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

pub struct BME280Sensor {
    bme280: BME280<I2cdev>,
    delay: Delay,
}

impl BME280Sensor {
    pub fn new(config: BME280Config) -> Result<Self> {
        let i2c_bus = I2cdev::new(&config.i2c_bus_path)
            .context("Failed to initialize I2C bus in BME280Sensor::new")?;

        let mut delay = Delay {};

        let mut bme280 = BME280::new(i2c_bus, config.i2c_address);
        bme280
            .init(&mut delay)
            .map_err(|e| anyhow::anyhow!("BME280 initialization failed: {:?}", e))?;

        info!("BME280 initialized successfully");
        Ok(BME280Sensor { bme280, delay })
    }

    pub fn read_measurements(&mut self) -> Result<PacketBME280> {
        let measurements = self
            .bme280
            .measure(&mut self.delay)
            .map_err(|e| anyhow::anyhow!("Failed to read BME280 sensor: {:?}", e))?;

        let temperature = (measurements.temperature * 2.0).round() as u8;
        let pressure = ((measurements.pressure / 100.0) - 1000.0).round() as u8;
        let humidity = measurements.humidity.round() as u8;
    
        Ok(PacketBME280 {
            temperature,
            pressure,
            humidity,
        })
    }

    pub fn print(&self, data: &PacketBME280) -> Result<()> {
        let temperature = data.temperature as f32 / 2.0;
        let pressure = (data.pressure as f32 + 1000.0) * 100.0;
        let humidity = data.humidity as f32;

        info!("BME280 Sensor Measurements:");
        info!("Temperature: {:.1} °C", temperature);
        info!("Pressure:    {:.1} hPa", pressure / 100.0);
        info!("Humidity:    {:.1} %", humidity);
        Ok(())
    }

    pub fn thread_run(&mut self, bme280_config: BME280Config, mqtt_enabled: bool, device_id: u8, option_sender: Option<Sender<Packet>>) {
        let measurement_interval = bme280_config.measurement_interval;

        let bme280_sender = match option_sender.clone() {
            Some(sender) => sender,
            None => {
                eprintln!("No sender created");
                error!("No sender created");
                std::process::exit(-1);
            },
        };

        loop {
            match self.read_measurements() {
                Ok(data) => {
                    self
                        .print(&data)
                        .expect("Failed to print BME280 measurements");

                    if mqtt_enabled {
                        // TODO rethink version, msg_id and msg_count values
                        let packet = Packet {
                            version: 0,
                            id: device_id,
                            msg_id: 0,
                            msg_count: 0,
                            data_type: DataType::BME280,
                            data: Data::Bme280(PacketBME280 {
                                temperature: data.temperature,
                                humidity: data.humidity,
                                pressure: data.pressure,
                            })
                        };
                        handle_error_continue!(bme280_sender.send(packet));
                    }
                }
                Err(e) => println!("Error reading measurements: {:?}", e),
            }

            thread::sleep(Duration::from_secs(measurement_interval));
        }
    }
}
