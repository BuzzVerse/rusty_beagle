use crate::defines::*;
use crate::sx1278::SX1278;
use crate::{bme280::BME280Sensor, BME280Config, Config, LoRaConfig};
use anyhow::{anyhow, Result};
use log::{error, info};
use ping::ping;
use std::net::Ipv4Addr;
use std::{path::Path, time::Duration};

macro_rules! error_log {
    ($func:expr, $type:expr) => {
        match $func {
            Err(e) => {
                eprintln!("POST: {} not working: {:?}", $type, e);
                error!("POST: {} not working: {:?}", $type, e);
                false
            }
            Ok(_) => true,
        }
    };
}

pub fn post(config: &Config) -> Result<ModulesState> {
    println!("--------------------------------------------------------------------------------");

    let lora;
    let mqtt;
    let bme280;

    if let Some(lora_config) = &config.lora_config {
        lora = error_log!(post_lora(lora_config), "LoRa");
    } else {
        lora = false;
        println!("[ OFF ] SPI POST");
        println!("[ OFF ] GPIO POST");
        info!("[ OFF ] SPI POST");
        info!("[ OFF ] GPIO POST");
    }

    if let Some(bme_config) = &config.bme_config {
        bme280 = error_log!(post_bme(bme_config), "BME280");
    } else {
        bme280 = false;
        println!("[ OFF ] BME280 POST");
        info!("[ OFF ] BME280 POST");
    }

    if config.mqtt_config.is_some() {
        mqtt = error_log!(post_mqtt(), "MQTT");
    } else {
        mqtt = false;
        println!("[ OFF ] MQTT POST");
        info!("[ OFF ] MQTT POST");
    }

    println!("--------------------------------------------------------------------------------");
    println!();

    Ok(ModulesState { lora, mqtt, bme280 })
}

fn post_lora(lora_config: &LoRaConfig) -> Result<()> {
    if !Path::new(&lora_config.spi_config.spidev_path).exists() {
        eprintln!("[ ERR ] SPI POST");
        error!("[ ERR ] SPI POST");
        return Err(anyhow!(
            "Spidev path {} doesn't exist!",
            &lora_config.spi_config.spidev_path
        ));
    }
    match lora_config.chip {
        Chip::SX1278 => {
            let mut lora = SX1278::from_config(lora_config)?;
            let mut mode = 0;
            lora.spi_read_register(SX1278LoRaRegister::OP_MODE, &mut mode)?;
            if mode == 0 {
                eprintln!("[ ERR ] SPI POST");
                error!("[ ERR ] SPI POST");
                return Err(anyhow!("Unable to IO via SPI"));
            }

            lora.standby_mode()?;
            lora.reset()?;
            lora.spi_read_register(SX1278LoRaRegister::OP_MODE, &mut mode)?;

            if mode == 0 {
                eprintln!("[ ERR ] SPI POST");
                error!("[ ERR ] SPI POST");
                return Err(anyhow!("Unable to IO via SPI"));
            } else if mode != 9 {
                eprintln!("[ ERR ] GPIO POST");
                error!("[ ERR ] GPIO POST");
                return Err(anyhow!("Unable to IO via GPIO"));
            }
        }
    }

    println!("[ OK ] SPI POST");
    info!("[ OK ] SPI POST");

    println!("[ OK ] GPIO POST");
    info!("[ OK ] GPIO POST");

    Ok(())
}

fn post_bme(bme_config: &BME280Config) -> Result<()> {
    if !Path::new(&bme_config.i2c_bus_path).exists() {
        eprintln!("[ ERR ] BME280 POST");
        error!("[ ERR ] BME280 POST");
        return Err(anyhow!(
            "i2c bus path {} doesn't exist!",
            &bme_config.i2c_bus_path
        ));
    }

    let mut bme = match BME280Sensor::new(bme_config.clone()) {
        Err(e) => {
            eprintln!("[ ERR ] BME280 POST");
            error!("[ ERR ] BME280 POST");
            return Err(anyhow!("Failed initializing BME280 Sensor: {}", e));
        }
        Ok(bme) => bme,
    };

    if let Err(e) = bme.read_measurements() {
        eprintln!("[ ERR ] BME280 POST");
        error!("[ ERR ] BME280 POST");
        return Err(anyhow!("Failed comunicating via i2c: {}", e));
    }

    println!("[ OK ] BME280 POST");

    Ok(())
}

fn post_mqtt() -> Result<()> {
    let ip = std::net::IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let payload = [1; 24];
    match ping(
        ip,
        Some(Duration::from_secs(1)),
        Some(128),
        None,
        None,
        Some(&payload),
    ) {
        Ok(_) => {
            println!("[ OK ] MQTT POST");
            info!("[ OK ] MQTT POST");
            Ok(())
        }
        Err(e) => {
            eprintln!("[ ERR ] MQTT POST");
            error!("[ ERR ] MQTT POST");
            Err(anyhow!("Unable to connect to internet: {:?}", e))
        }
    }
}

pub struct ModulesState {
    pub lora: bool,
    pub mqtt: bool,
    pub bme280: bool,
}
