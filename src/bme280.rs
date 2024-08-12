use anyhow::{Context, Result};
use bme280::i2c::BME280;
use linux_embedded_hal::{Delay, I2cdev};
use log::info;

use crate::BME280Config;

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

    pub fn read_measurements(&mut self) -> Result<(f32, f32, f32)> {
        let measurements = self
            .bme280
            .measure(&mut self.delay)
            .map_err(|e| anyhow::anyhow!("Failed to read BME280 sensor: {:?}", e))?;

        Ok((
            measurements.temperature,
            measurements.pressure / 100.0,
            measurements.humidity,
        ))
    }

    pub fn print(&mut self) -> Result<()> {
        match self.read_measurements() {
            Ok((temperature, pressure, humidity)) => {
                info!("BME280 Sensor Measurements:");
                info!("Temperature: {:>6.1} °C", temperature);
                info!("Pressure:    {:>7.1} hPa", pressure);
                info!("Humidity:    {:>6.1} %", humidity);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
