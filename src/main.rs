mod defines;
extern crate syslog;

pub use crate::defines::api_defines::API_Status;
pub use crate::defines::lora_defines::LoRa_Registers;
use log::{debug, error, info, trace, warn};
use syslog::{BasicLogger, Facility, Formatter3164};

fn main() {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "rusty_beagle".into(),
        pid: 0,
    };

    match syslog::unix(formatter) {
        Err(e) => eprintln!("impossible to connect to syslog: {:?}", e),
        Ok(writer) => {
            // Initialize the logger
            log::set_boxed_logger(Box::new(BasicLogger::new(writer)))
                .map(|()| log::set_max_level(log::LevelFilter::Debug))
                .expect("could not set logger");
        }
    }
    error!("this is an error message");
    warn!("this is a warning message");
    info!("this is an info message");
    debug!("this is a debug message");
    trace!("this is a trace message");
}
