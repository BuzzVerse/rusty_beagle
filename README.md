# rusty_beagle
LoRa Daemon and Buildroot image for Beagle Bone Black

# How to build
1. Install rust through rustup
1. ```sudo apt install g++-arm-linux-gnueabihf```
1. ```rustup target add arm-unknown-linux-gnueabihf```
1. Build with ```cargo build --target=arm-unknown-linux-gnueabihf```
