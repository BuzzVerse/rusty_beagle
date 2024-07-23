mod defines;

pub use crate::defines::api_defines::API_Status;
pub use crate::defines::lora_defines::LoRa_Registers;

fn main() {
    println!("Hello, world! {:?}", API_Status::API_OK as u8);
}
