use core::fmt;
use crate::conversions::*;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

pub const DATA_SIZE: usize = 59;
pub const META_DATA_SIZE: usize = 5;
pub const PACKET_SIZE: usize = DATA_SIZE + META_DATA_SIZE;

pub const PACKET_VERSION_IDX: usize = 0;
pub const PACKET_ID_IDX: usize = 1;
pub const PACKET_MSG_ID_IDX: usize = 2;
pub const PACKET_MSG_COUNT_IDX: usize = 3;
pub const PACKET_DATA_TYPE_IDX: usize = 4;

#[repr(u8)]
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Hash)]
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
            _ => Err(anyhow!("Invalid data type")).context("DataType::new"),
        }
    }
}

#[derive(Deserialize, Serialize, Hash)]
pub enum Data {
    Bme280(BME280),
    Bma400(BMA400),
    Mq2(MQ2),
    Gps(Gps),
    Sms(String),
}

#[derive(Debug, Deserialize, Serialize, Hash)]
pub struct BME280 {
    pub temperature: u8,
    pub humidity: u8,
    pub pressure: u8,
}

#[derive(Debug, Deserialize, Serialize, Hash)]
pub struct BMA400 {
    pub x: u64,
    pub y: u64,
    pub z: u64,
}

#[derive(Debug, Deserialize, Serialize, Hash)]
pub struct MQ2 {
    pub gas_type: u8,
    pub value: u128,
}

#[derive(Debug, Deserialize, Serialize, Hash)]
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
                if bytes.len() != 8 {
                    return Err(anyhow!(
                        "Incorrect length, was {}, should be 8",
                        bytes.len()
                    ))
                    .context("Data::from_bytes");
                }
                Ok(Data::Bme280(BME280 {
                    temperature: bytes[META_DATA_SIZE],
                    humidity: bytes[META_DATA_SIZE + 1],
                    pressure: bytes[META_DATA_SIZE + 2],
                }))
            }
            DataType::BMA400 => {
                if bytes.len() != 29 {
                    return Err(anyhow!(
                        "Incorrect length, was {}, should be 29",
                        bytes.len()
                    ))
                    .context("Data::from_bytes");
                }
                Ok(Data::Bma400(BMA400 {
                    x: vec_to_u64(bytes, META_DATA_SIZE).context("Data::from_bytes")?,
                    y: vec_to_u64(bytes, META_DATA_SIZE + 8).context("Data::from_bytes")?,
                    z: vec_to_u64(bytes, META_DATA_SIZE + 16).context("Data::from_bytes")?,
                }))
            }
            DataType::MQ2 => {
                if bytes.len() != 22 {
                    return Err(anyhow!(
                        "Incorrect length, was {}, should be 22",
                        bytes.len()
                    ))
                    .context("Data::from_bytes");
                }
                Ok(Data::Mq2(MQ2 {
                    gas_type: bytes[META_DATA_SIZE],
                    value: vec_to_u128(bytes, META_DATA_SIZE + 1).context("Data::from_bytes")?,
                }))
            }
            DataType::Gps => {
                if bytes.len() != 16 {
                    return Err(anyhow!(
                        "Incorrect length, was {}, should be 16",
                        bytes.len()
                    ))
                    .context("Data::from_bytes");
                }
                Ok(Data::Gps(Gps {
                    status: bytes[META_DATA_SIZE],
                    altitude: vec_to_u16(bytes, META_DATA_SIZE + 1).context("Data::from_bytes")?,
                    latitude: vec_to_i32(bytes, META_DATA_SIZE + 3).context("Data::from_bytes")?,
                    longitude: vec_to_i32(bytes, META_DATA_SIZE + 7).context("Data::from_bytes")?,
                }))
            }
            DataType::Sms => {
                if bytes.len() < 6 {
                    return Err(anyhow!(
                        "Incorrect length, was {}, should be at least 6",
                        bytes.len()
                    ))
                    .context("Data::from_bytes");
                }
                Ok(Data::Sms(
                    String::from_utf8(bytes[META_DATA_SIZE..bytes.len()].to_vec())
                        .context("Data::from_bytes")?,
                ))
            }
        }
    }
}

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Bme280(data) => write!(
                f,
                "{{ temperature: {}, humidity: {}, pressure: {} }}",
                data.temperature, data.humidity, data.pressure
            ),
            Data::Bma400(data) => write!(f, "{{ x: {}, y: {}, z: {} }}", data.x, data.y, data.z),
            Data::Mq2(data) => write!(
                f,
                "{{ gas_type: {}, value: {} }}",
                data.gas_type, data.value
            ),
            Data::Gps(data) => write!(
                f,
                "{{ status: {}, altitude: {}, latitude: {}, longitude: {} }}",
                data.status, data.altitude, data.latitude, data.longitude
            ),
            Data::Sms(data) => write!(f, "\"{}\"", *data),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Hash)]
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
        if bytes.len() < META_DATA_SIZE || bytes.len() > PACKET_SIZE {
            return Err(anyhow!("Incorrect length, was {}", bytes.len()));
        }
        let version = bytes[PACKET_VERSION_IDX];
        let id = bytes[PACKET_ID_IDX];
        let msg_id = bytes[PACKET_MSG_ID_IDX];
        let msg_count = bytes[PACKET_MSG_COUNT_IDX];
        let data_type = DataType::new(bytes[PACKET_DATA_TYPE_IDX]).context("Packet::new")?;
        let data = Data::from_bytes(bytes).context("Packet::new")?;

        Ok(Self {
            version,
            id,
            msg_id,
            msg_count,
            data_type,
            data,
        })
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut packet = vec![
            self.version,
            self.id,
            self.msg_id,
            self.msg_count,
            self.data_type as u8,
        ];
        let mut data = match &self.data {
            Data::Bme280(data) => bincode::serialize(data).context("Packet::to_bytes")?,
            Data::Bma400(data) => bincode::serialize(data).context("Packet::to_bytes")?,
            Data::Mq2(data) => bincode::serialize(data).context("Packet::to_bytes")?,
            Data::Gps(data) => bincode::serialize(data).context("Packet::to_bytes")?,
            Data::Sms(data) => data.as_bytes().to_vec(),
        };

        packet.append(&mut data);
        Ok(packet)
    }

    pub fn to_json(&self) -> Result<String> {
        match &self.data {
            Data::Bme280(data) => Ok(format!(
                r#""BME280": {{ "temperature": {}, "humidity": {}, "pressure": {} }}"#,
                data.temperature as f32 / 2.0, data.humidity as f32, data.pressure as f32 + 1000.0
            )),
            Data::Bma400(data) => Ok(format!(
                r#""BMA400": {{ "x": {}, "y": {}, "z": {} }}"#,
                data.x, data.y, data.z
            )),
            Data::Mq2(data) => Ok(format!(
                r#""MQ2": {{ "gas_type": {}, "value": {} }}"#,
                data.gas_type, data.value
            )),
            Data::Gps(data) => Ok(format!(
                r#""GPS": {{ "status": {}, "altitude": {}, "latitude": {}, "longitude": {} }}"#,
                data.status, data.altitude, 
                (data.latitude as f64) / 100_000f64, 
                (data.longitude as f64) / 100_000f64
            )),
            Data::Sms(data) => Ok(format!(
                r#""SMS": {{ "text": "{}" }}"#,
                *data
            )),
        }
    }
}

pub struct Metadata {
    pub snr: u8,
    pub rssi: i16,
}

impl Metadata {
    pub fn to_json(&self) -> Result<String> {
        Ok(format!(
                r#""META": {{ "snr": {}, "rssi": {} }}"#,
                self.snr, self.rssi
        ))
    }
}

/// A struct to wrap a LoRa packet with additional data about
/// SNR (signal to noise ratio)
/// and RSSI (received signal strength indicator)
pub struct PacketWrapper {
    pub packet: Packet,
    pub metadata: Metadata,
}

impl PacketWrapper {
    pub fn to_json(&self) -> Result<String> {
        Ok(format!(
            r#"{{ {}, {} }}"#,
            self.packet.to_json()?, self.metadata.to_json()?
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::{DefaultHasher, Hasher};

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn serialize_bme280_correct() {
        let packet = Packet {
            version: 0x33,
            id: 0x22,
            msg_id: 0x11,
            msg_count: 0x00,
            data_type: DataType::BME280,
            data: Data::Bme280(BME280 {
                temperature: 23,
                humidity: 45,
                pressure: 67,
            }),
        };

        let expected_data: Vec<u8> = vec![0x33, 0x22, 0x11, 0x00, 0x01, 0x17, 0x2D, 0x43];
        let serialized_packet = packet.to_bytes().unwrap();

        assert_eq!(serialized_packet, expected_data);
    }

    #[test]
    fn deserialize_bme280_correct() {
        let bytes: Vec<u8> = vec![0x33, 0x22, 0x11, 0x00, 0x01, 0x17, 0x2D, 0x43];

        let expected_packet = Packet {
            version: 0x33,
            id: 0x22,
            msg_id: 0x11,
            msg_count: 0x00,
            data_type: DataType::BME280,
            data: Data::Bme280(BME280 {
                temperature: 23,
                humidity: 45,
                pressure: 67,
            }),
        };

        let deserialized_data = Packet::new(&bytes).unwrap();

        assert_eq!(
            calculate_hash(&deserialized_data),
            calculate_hash(&expected_packet)
        );
    }

    #[test]
    fn deserialize_bme280_data_too_short() {
        let bytes: Vec<u8> = vec![0x33, 0x22, 0x11, 0x00, 0x01, 0x17, 0x2D];
        assert!(Packet::new(&bytes).is_err());
    }

    #[test]
    fn deserialize_bme280_packet_too_long() {
        let bytes: Vec<u8> = vec![0x33, 0x22, 0x11, 0x00, 0x01, 0x17, 0x2D, 0x43, 0xFF];
        assert!(Packet::new(&bytes).is_err());
    }

    #[test]
    fn deserialize_bme280_packet_too_short() {
        let bytes: Vec<u8> = vec![0x33, 0x22, 0x11];
        assert!(Packet::new(&bytes).is_err());
    }

    #[test]
    fn serialize_bma400_correct() {
        let packet = Packet {
            version: 0x33,
            id: 0x22,
            msg_id: 0x11,
            msg_count: 0x00,
            data_type: DataType::BMA400,
            data: Data::Bma400(BMA400 {
                x: 255,
                y: 256,
                z: 1024,
            }),
        };

        let expected_data: Vec<u8> = vec![
            0x33, 0x22, 0x11, 0x00, 0x02, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00,
        ];
        let serialized_packet = packet.to_bytes().unwrap();

        assert_eq!(serialized_packet, expected_data);
    }

    #[test]
    fn deserialize_bma400_correct() {
        let bytes: Vec<u8> = vec![
            0x33, 0x22, 0x11, 0x00, 0x02, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00,
        ];

        let expected_packet = Packet {
            version: 0x33,
            id: 0x22,
            msg_id: 0x11,
            msg_count: 0x00,
            data_type: DataType::BMA400,
            data: Data::Bma400(BMA400 {
                x: 255,
                y: 256,
                z: 1024,
            }),
        };

        let deserialized_data = Packet::new(&bytes).unwrap();

        assert_eq!(
            calculate_hash(&deserialized_data),
            calculate_hash(&expected_packet)
        );
    }

    #[test]
    fn deserialize_bma400_data_too_short() {
        let bytes: Vec<u8> = vec![
            0x33, 0x22, 0x11, 0x00, 0x02, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert!(Packet::new(&bytes).is_err());
    }

    #[test]
    fn deserialize_bma400_packet_too_long() {
        let bytes: Vec<u8> = vec![
            0x33, 0x22, 0x11, 0x00, 0x02, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xFF,
        ];
        assert!(Packet::new(&bytes).is_err());
    }

    #[test]
    fn deserialize_bme400_packet_too_short() {
        let bytes: Vec<u8> = vec![0x33, 0x22, 0x11];
        assert!(Packet::new(&bytes).is_err());
    }

    #[test]
    fn serialize_mq2_correct() {
        let packet = Packet {
            version: 0x33,
            id: 0x22,
            msg_id: 0x11,
            msg_count: 0x00,
            data_type: DataType::MQ2,
            data: Data::Mq2(MQ2 {
                gas_type: 0x01,
                value: u128::MAX,
            }),
        };

        let expected_data: Vec<u8> = vec![
            0x33, 0x22, 0x11, 0x00, 0x03, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let serialized_packet = packet.to_bytes().unwrap();

        assert_eq!(serialized_packet, expected_data);
    }

    #[test]
    fn deserialize_mq2_correct() {
        let bytes: Vec<u8> = vec![
            0x33, 0x22, 0x11, 0x00, 0x03, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let expected_packet = Packet {
            version: 0x33,
            id: 0x22,
            msg_id: 0x11,
            msg_count: 0x00,
            data_type: DataType::MQ2,
            data: Data::Mq2(MQ2 {
                gas_type: 0x01,
                value: u128::MAX,
            }),
        };

        let deserialized_data = Packet::new(&bytes).unwrap();

        assert_eq!(
            calculate_hash(&deserialized_data),
            calculate_hash(&expected_packet)
        );
    }

    #[test]
    fn deserialize_mq2_data_too_short() {
        let bytes: Vec<u8> = vec![
            0x33, 0x22, 0x11, 0x00, 0x03, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        assert!(Packet::new(&bytes).is_err());
    }

    #[test]
    fn deserialize_mq2_packet_too_long() {
        let bytes: Vec<u8> = vec![
            0x33, 0x22, 0x11, 0x00, 0x03, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        assert!(Packet::new(&bytes).is_err());
    }

    #[test]
    fn deserialize_mq2_packet_too_short() {
        let bytes: Vec<u8> = vec![0x33, 0x22, 0x11];
        assert!(Packet::new(&bytes).is_err());
    }

    #[test]
    fn serialize_gps_correct() {
        let packet = Packet {
            version: 0x33,
            id: 0x22,
            msg_id: 0x11,
            msg_count: 0x00,
            data_type: DataType::Gps,
            data: Data::Gps(Gps {
                status: u8::MAX,
                altitude: u16::MAX,
                latitude: i32::MAX,
                longitude: i32::MAX,
            }),
        };

        let expected_data: Vec<u8> = vec![
            0x33, 0x22, 0x11, 0x00, 0x04, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F, 0xFF, 0xFF,
            0xFF, 0x7F,
        ];
        let serialized_packet = packet.to_bytes().unwrap();

        assert_eq!(serialized_packet, expected_data);
    }

    #[test]
    fn deserialize_gps_correct() {
        let bytes: Vec<u8> = vec![
            0x33, 0x22, 0x11, 0x00, 0x04, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F, 0xFF, 0xFF,
            0xFF, 0x7F,
        ];

        let expected_packet = Packet {
            version: 0x33,
            id: 0x22,
            msg_id: 0x11,
            msg_count: 0x00,
            data_type: DataType::Gps,
            data: Data::Gps(Gps {
                status: u8::MAX,
                altitude: u16::MAX,
                latitude: i32::MAX,
                longitude: i32::MAX,
            }),
        };

        let deserialized_data = Packet::new(&bytes).unwrap();

        assert_eq!(
            calculate_hash(&deserialized_data),
            calculate_hash(&expected_packet)
        );
    }

    #[test]
    fn deserialize_gps_data_too_short() {
        let bytes: Vec<u8> = vec![
            0x33, 0x22, 0x11, 0x00, 0x04, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F, 0xFF, 0xFF,
            0xFF,
        ];
        assert!(Packet::new(&bytes).is_err());
    }

    #[test]
    fn deserialize_gps_packet_too_long() {
        let bytes: Vec<u8> = vec![
            0x33, 0x22, 0x11, 0x00, 0x04, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F, 0xFF, 0xFF,
            0xFF, 0x7F, 0xFF,
        ];
        assert!(Packet::new(&bytes).is_err());
    }

    #[test]
    fn deserialize_gps_packet_too_short() {
        let bytes: Vec<u8> = vec![0x33, 0x22, 0x11];
        assert!(Packet::new(&bytes).is_err());
    }

    #[test]
    fn serialize_sms_correct() {
        let packet = Packet {
            version: 0x33,
            id: 0x22,
            msg_id: 0x11,
            msg_count: 0x00,
            data_type: DataType::Sms,
            data: Data::Sms(String::from("AB")),
        };

        let expected_data: Vec<u8> = vec![0x33, 0x22, 0x11, 0x00, 0x20, 0x41, 0x42];
        let serialized_packet = packet.to_bytes().unwrap();

        assert_eq!(serialized_packet, expected_data);
    }

    #[test]
    fn deserialize_sms_correct() {
        let bytes: Vec<u8> = vec![0x33, 0x22, 0x11, 0x00, 0x20, 0x41, 0x42];

        let expected_packet = Packet {
            version: 0x33,
            id: 0x22,
            msg_id: 0x11,
            msg_count: 0x00,
            data_type: DataType::Sms,
            data: Data::Sms(String::from("AB")),
        };

        let deserialized_data = Packet::new(&bytes).unwrap();

        assert_eq!(
            calculate_hash(&deserialized_data),
            calculate_hash(&expected_packet)
        );
    }

    #[test]
    fn deserialize_sms_data_too_short() {
        let bytes: Vec<u8> = vec![0x33, 0x22, 0x11, 0x00, 0x20]; // 0 byte string
        assert!(Packet::new(&bytes).is_err());
    }

    #[test]
    fn deserialize_sms_packet_too_long() {
        let bytes: Vec<u8> = vec![
            0x33, 0x22, 0x11, 0x00, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
        ];
        assert!(Packet::new(&bytes).is_err());
    }

    #[test]
    fn deserialize_sms_packet_too_short() {
        let bytes: Vec<u8> = vec![0x33, 0x22, 0x11];
        assert!(Packet::new(&bytes).is_err());
    }
}
