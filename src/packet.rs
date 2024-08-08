use core::fmt;

use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Context, Result};
use crate::conversions::*;

pub const DATA_SIZE: usize = 59;
pub const META_DATA_SIZE: usize = 5;
pub const PACKET_SIZE: usize = DATA_SIZE + META_DATA_SIZE;

pub const PACKET_VERSION_IDX: usize = 0;
pub const PACKET_ID_IDX: usize = 1;
pub const PACKET_MSG_ID_IDX: usize = 2;
pub const PACKET_MSG_COUNT_IDX: usize = 3;
pub const PACKET_DATA_TYPE_IDX: usize = 4;

#[repr(u8)]
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum DataType {
    BME280 = 1,
    BMA400 = 2,
    MQ2 = 3,
    Gps = 4,
    Sms = 32,
}

impl DataType {
    pub fn new(byte: u8) -> Result<Self> {
        match byte {
            1 => Ok(Self::BME280),
            2 => Ok(Self::BMA400),
            3 => Ok(Self::MQ2),
            4 => Ok(Self::Gps),
            32 => Ok(Self::Sms),
            _ => Err(anyhow!("Invalid data type")).context("DataType::new: "),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum Data {
    Bme280(BME280),
    Bma400(BMA400),
    Mq2(MQ2),
    Gps(Gps),
    Sms(String),
}

pub trait GetData<T> {
    fn get_data(&self) -> Option<&T>;
}

impl GetData<BME280> for Data {
    fn get_data(&self) -> Option<&BME280> {
        if let Data::Bme280(ref inner) = *self {
            Some(inner)
        } else {
            None
        }
    }
}

impl GetData<BMA400> for Data {
    fn get_data(&self) -> Option<&BMA400> {
        if let Data::Bma400(ref inner) = *self {
            Some(inner)
        } else {
            None
        }
    }
}

impl GetData<MQ2> for Data {
    fn get_data(&self) -> Option<&MQ2> {
        if let Data::Mq2(ref inner) = *self {
            Some(inner)
        } else {
            None
        }
    }
}

impl GetData<Gps> for Data {
    fn get_data(&self) -> Option<&Gps> {
        if let Data::Gps(ref inner) = *self {
            Some(inner)
        } else {
            None
        }
    }
}

impl GetData<String> for Data {
    fn get_data(&self) -> Option<&String> {
        if let Data::Sms(ref inner) = *self {
            Some(inner)
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BME280 {
    pub temperature: u8,
    pub humidity: u8,
    pub pressure: u8,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BMA400 {
    pub x: u64,
    pub y: u64,
    pub z: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MQ2 {
    pub gas_type: u8,
    pub value: u128,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Gps {
    pub status: u8,
    pub altitude: u16,
    pub latitude: i32,
    pub longitude: i32,
}

impl Data {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let data_type = DataType::new(bytes[PACKET_DATA_TYPE_IDX]).unwrap();
        match data_type {
            DataType::BME280 => {
                if bytes.len() != 8 { return Err(anyhow!("Incorrect length, was {}, should be 8", bytes.len())) }
                Ok(Data::Bme280(BME280 {
                    temperature: bytes[META_DATA_SIZE],
                    humidity: bytes[META_DATA_SIZE + 1],
                    pressure: bytes[META_DATA_SIZE + 2],
                }))
            },
            DataType::BMA400 => {
                if bytes.len() != 29 { return Err(anyhow!("Incorrect length, was {}, should be 29", bytes.len())) }
                Ok(Data::Bma400(BMA400 {
                    x: vec_to_u64(bytes, META_DATA_SIZE).context("Data::from_bytes: ")?,
                    y: vec_to_u64(bytes, META_DATA_SIZE + 8).context("Data::from_bytes: ")?,
                    z: vec_to_u64(bytes, META_DATA_SIZE + 16).context("Data::from_bytes: ")?,
                }))
            },
            DataType::MQ2 => {
                if bytes.len() != 22 { return Err(anyhow!("Incorrect length, was {}, should be 22", bytes.len())) }
                Ok(Data::Mq2(MQ2 {
                    gas_type: bytes[META_DATA_SIZE],
                    value: vec_to_u128(bytes, META_DATA_SIZE + 1).context("Data::from_bytes: ")?,
                })) 
            },
            DataType::Gps => {
                if bytes.len() != 16 { return Err(anyhow!("Incorrect length, was {}, should be 16", bytes.len())) }
                Ok(Data::Gps(Gps {
                    status: bytes[META_DATA_SIZE],
                    altitude: vec_to_u16(bytes, META_DATA_SIZE + 1).context("Data::from_bytes: ")?,
                    latitude: vec_to_i32(bytes, META_DATA_SIZE + 3).context("Data::from_bytes: ")?,
                    longitude: vec_to_i32(bytes, META_DATA_SIZE + 7).context("Data::from_bytes: ")?,
                }))
            },
            DataType::Sms => {
                if bytes.len() < 6 { return Err(anyhow!("Incorrect length, was {}, should be at least 6", bytes.len())) }
                Ok(Data::Sms(String::from_utf8(bytes[META_DATA_SIZE..bytes.len()].to_vec()).context("Data::from_bytes: ")?))
            },
        }
    }
}

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Bme280(data) => write!(f, "{{ temperature: {}, humidity: {}, pressure: {} }}", data.temperature, data.humidity, data.pressure),
            Data::Bma400(data) => write!(f, "{{ x: {}, y: {}, z: {} }}", data.x, data.y, data.z),
            Data::Mq2(data) => write!(f, "{{ gas_type: {}, value: {} }}", data.gas_type, data.value),
            Data::Gps(data) => write!(f, "{{ status: {}, altitude: {}, latitude: {}, longitude: {} }}", data.status, data.altitude, data.latitude, data.longitude),
            Data::Sms(data) => write!(f, "\"{}\"", *data),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Packet {
    pub version: u8,
    pub id: u8,
    pub msg_id: u8,
    pub msg_count: u8,
    pub data_type: DataType,
    pub data: Data,
}

impl Packet {
    pub fn new(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 5 { return Err(anyhow!("Incorrect length, was {}", bytes.len())) }
        let version = bytes[PACKET_VERSION_IDX];
        let id = bytes[PACKET_ID_IDX];
        let msg_id = bytes[PACKET_MSG_ID_IDX];
        let msg_count = bytes[PACKET_MSG_COUNT_IDX];
        let data_type = DataType::new(bytes[PACKET_DATA_TYPE_IDX]).context("Packet::new: ")?; 
        let data = Data::from_bytes(bytes).context("Packet::new: ")?;

        Ok(Self { version, id, msg_id, msg_count, data_type, data })
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut packet = vec![self.version, self.id, self.msg_id, self.msg_count, self.data_type as u8];
        let mut data = match &self.data {
            Data::Bme280(data) => bincode::serialize(data).context("Packet::to_bytes: ")?,
            Data::Bma400(data) => bincode::serialize(data).context("Packet::to_bytes: ")?,
            Data::Mq2(data) => bincode::serialize(data).context("Packet::to_bytes: ")?,
            Data::Gps(data) => bincode::serialize(data).context("Packet::to_bytes: ")?,
            Data::Sms(data) => data.as_bytes().to_vec(),
        };

        packet.append(&mut data);
        Ok(packet)
    }
}
