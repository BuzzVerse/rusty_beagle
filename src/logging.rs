extern crate syslog;

use syslog::{BasicLogger, Facility, Formatter3164};

pub fn start_logger() {
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
}
