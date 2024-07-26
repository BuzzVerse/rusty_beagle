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

# Running on BeagleBone Black
1. Ensure that HDMI is disabled: in /boot/uEnv.txt uncomment the line:
    - ```disable_uboot_overlay_video=1```
1. Ensure that BeagleBone Black has and uses the provided Device Tree Overlays for SPI found in /lib/firmware: in /boot/uEnv.txt uncomment and modify the lines under ```###Additional custom capes```:
    - ```uboot_overlay_addr4=/lib/firmware/BB-SPIDEV0-00A0.dtbo```
    - ```uboot_overlay_addr5=/lib/firmware/BB-SPIDEV1-00A0.dtbo```
