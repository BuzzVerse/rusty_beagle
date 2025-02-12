use log::info;
use signal_hook::{
    consts::{SIGINT, SIGQUIT, SIGTERM},
    iterator::Signals,
};
use std::error::Error;

fn graceful_shutdown(sig: i32) {
    println!(" Received signal {:?}, shutting down gracefully...", sig);
    info!(" Received signal {:?}, shutting down gracefully...", sig);

    // Unlike std::process::exit, this is async-signal-safe:
    signal_hook::low_level::exit(0);
}

pub fn run_signal_handler() -> Result<(), Box<dyn Error>> {
    let mut signals = Signals::new([SIGINT, SIGQUIT, SIGTERM])?;

    for sig in signals.forever() {
        graceful_shutdown(sig);
    }

    Ok(())
}
