# rusty_beagle
LoRa Daemon and Buildroot image for Beagle Bone Black

# How to build
1. Install rust through [rustup](https://rustup.rs/)
1. Add these lines to ~/.cargo/config.toml
```
[target.arm-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
rustflags = ["-C", "target-feature=+crt-static"]
```
1. ```sudo apt install g++-arm-linux-gnueabihf```
1. ```rustup target add arm-unknown-linux-gnueabihf```
1. Build with ```cargo build --target=arm-unknown-linux-gnueabihf```

# Logging
1. Ensure you have rsyslog downloaded and started, if not:
    1. ```sudo apt install rsyslog```
    1. ```sudo systemctl enable rsyslog```
    1. ```sudo systemctl start rsyslog```
1. Run rsyslog setup script: rusty_beagle/scripts/setup_rsyslog.sh
