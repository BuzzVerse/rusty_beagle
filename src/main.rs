mod defines;
mod logging;

extern crate log;

pub use crate::defines::api_defines::API_Status;
pub use crate::defines::lora_defines::LoRa_Registers;
pub use crate::logging::start_logger;
use log::{debug, error, info, trace, warn};

fn main() {
    start_logger();

    error!("this is an error message");
    warn!("this is a warning message");
    info!("this is an info message");
    debug!("this is a debug message");
    trace!("this is a trace message");
}
