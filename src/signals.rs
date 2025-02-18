use signal_hook::{
    consts::{SIGINT, SIGQUIT, SIGTERM},
    iterator::Signals,
};
use std::error::Error;
use std::sync::mpsc::Sender;

pub fn run_signal_handler(signal_sender: Sender<i32>) -> Result<(), Box<dyn Error>> {
    let mut signals = Signals::new([SIGINT, SIGQUIT, SIGTERM])?;

    for signal in signals.forever() {
        signal_sender.send(signal)?;
    }

    Ok(())
}
