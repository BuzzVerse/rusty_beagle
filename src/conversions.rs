use anyhow::{anyhow, Context, Result};

pub fn array_to_f32(array: [u8; 4]) -> f32 {
    f32::from_le_bytes(array)
}

pub fn array_to_f64(array: [u8; 8]) -> f64 {
    f64::from_le_bytes(array)
}

pub fn array_to_u32(array: [u8; 4]) -> u32 {
    u32::from_le_bytes(array)
}

pub fn array_to_u64(array: [u8; 8]) -> u64 {
    u64::from_le_bytes(array)
}

pub fn array_to_i32(array: [u8; 4]) -> i32 {
    i32::from_le_bytes(array)
}
