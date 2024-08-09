use anyhow::{Context, Result};

pub fn vec_to_u16(vec: &[u8], start: usize) -> Result<u16> {
    let slice = &vec[start..start + 2];
    let array: [u8; 2] = slice.try_into().context("vec_to_u16")?;
    Ok(u16::from_le_bytes(array))
}

pub fn vec_to_u64(vec: &[u8], start: usize) -> Result<u64> {
    let slice = &vec[start..start + 8];
    let array: [u8; 8] = slice.try_into().context("vec_to_u64")?;
    Ok(u64::from_le_bytes(array))
}

pub fn vec_to_u128(vec: &[u8], start: usize) -> Result<u128> {
    let slice = &vec[start..start + 16];
    let array: [u8; 16] = slice.try_into().context("vec_to_u128")?;
    Ok(u128::from_le_bytes(array))
}

pub fn vec_to_i32(vec: &[u8], start: usize) -> Result<i32> {
    let slice = &vec[start..start + 4];
    let array: [u8; 4] = slice.try_into().context("vec_to_i32")?;
    Ok(i32::from_le_bytes(array))
}
