mod defines;
mod logging;

extern crate log;

pub use crate::defines::api_defines::API_Status;
pub use crate::defines::lora_defines::LoRa_Registers;
pub use crate::logging::start_logger;
//use log::{debug, error, info, trace, warn};

#[cfg(target_arch = "x86_64")]
fn prepare_mocks() {
    println!("prepare_mocks(): Running on x86_64.");
}

fn main() {
    #[cfg(target_arch = "x86_64")]
    prepare_mocks();
}
