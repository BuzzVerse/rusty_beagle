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

#[derive(Deserialize, Serialize)]
pub struct BME280 {
    pub temperature: u8,
    pub humidity: u8,
    pub pressure: u8,
}

#[derive(Deserialize, Serialize)]
pub struct BMA400 {
    x: u64,
    y: u64,
    z: u64,
}

#[derive(Deserialize, Serialize)]
pub struct MQ2 {
    gas_type: u8,
    value: u128,
}

#[derive(Deserialize, Serialize)]
pub struct Gps {
    status: u8,
    altitude: u16,
    latitude: u32,
    longitude: u32,
}

impl Data {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let data_type = DataType::new(bytes[PACKET_DATA_TYPE_IDX]).unwrap();
        match data_type {
            DataType::BME280 => Ok(Data::Bme280(BME280 {
                temperature: bytes[META_DATA_SIZE],
                humidity: bytes[META_DATA_SIZE + 1],
                pressure: bytes[META_DATA_SIZE + 2],
            })),
            DataType::BMA400 => Ok(Data::Bma400(BMA400 {
                x: vec_to_u64(&bytes, META_DATA_SIZE)?,
                y: vec_to_u64(&bytes, META_DATA_SIZE + 8)?,
                z: vec_to_u64(&bytes, META_DATA_SIZE + 16)?,
            })),
            DataType::MQ2 => Ok(Data::Mq2(MQ2 {
                gas_type: bytes[META_DATA_SIZE],
                value: vec_to_u128(&bytes, META_DATA_SIZE + 1)?,
            })),
            DataType::Gps => Ok(Data::Gps(Gps {
                status: bytes[META_DATA_SIZE],
                altitude: vec_to_u16(&bytes, META_DATA_SIZE + 1)?,
                latitude: vec_to_u32(&bytes, META_DATA_SIZE + 3)?,
                longitude: vec_to_u32(&bytes, META_DATA_SIZE + 7)?,
            })),
            DataType::Sms => Ok(Data::Sms(String::from_utf8(bytes[META_DATA_SIZE..bytes.len()].to_vec())?)),
        }
    }
}

pub struct Packet {
    pub version: u8,
    pub id: u8,
    pub msg_id: u8,
    pub msg_count: u8,
    pub data_type: DataType,
    pub data: Data,
}

impl Packet {
    fn to_bytes(self) -> Result<Vec<u8>> {
        match self.data {
            Data::Bme280(data) => Ok(
                bincode::serialize(&data).context("Packet::to_bytes: ")?,
            ),
            Data::Bma400(data) => Ok(
                bincode::serialize(&data).context("Packet::to_bytes: ")?,
            ),
            Data::Mq2(data) => Ok(
                bincode::serialize(&data).context("Packet::to_bytes: ")?,
            ),
            Data::Gps(data) => Ok(
                bincode::serialize(&data).context("Packet::to_bytes: ")?,
            ),
            Data::Sms(data) => Ok(data.as_bytes().to_vec()),
        }
    }
}
