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

# BeagleBone Black and LoRa pinout
1. Connect LoRa GND & 3.3V pins to corresponding pins on BBB
1. Connect LoRa module to BBB via SPI
1. Connect LoRa RST pin to any GPIO pin on BBB
1. Connect LoRa DIO0 pin to any GPIO pin on BBB
1. Include chosen SPIDEV and GPIO pins in a config file

# Running on BeagleBone Black
1. Ensure that HDMI is disabled: in /boot/uEnv.txt uncomment the line:
    - ```disable_uboot_overlay_video=1```
1. Ensure that BeagleBone Black has and uses the provided Device Tree Overlays for SPI found in /lib/firmware: in /boot/uEnv.txt uncomment and modify the lines under ```###Additional custom capes```:
    - ```uboot_overlay_addr4=/lib/firmware/BB-SPIDEV0-00A0.dtbo```
    - ```uboot_overlay_addr5=/lib/firmware/BB-SPIDEV1-00A0.dtbo```
1. Create config for rusty_beagle, example config is available in rusty_beagle/conf.ron
1. Run ```./rusty_beagle <path_to_config>``` 

# How to build on Apple Silicon using Docker

1. In docker directory run commands below
2. Docker build
```bash
docker build --platform linux/amd64 --progress=plain -t rusty_beagle .
```
3. Docker run
```bash
docker run --rm -v $(pwd)/output:/output rusty_beagle
```
4. output of build is in the "output" directory

# Running on BeagleBone Black as a daemon
1. Copy rusty-beagled.service to /etc/systemd/system/
    - make sure the user specified under `User=` has access (permissions) to SPI, network and I2C (if BME is enabled)
    - make sure to provide the path to the rusty_beagle executable and the desired config under `ExecStart=`
    - default .service file config assumes that the user is debian, the executable & config are located in /home/debian/, and the debian user has access (permissions) to /dev/spidev* devices
    - for user's permanent access to SPI devices:
        - create a group: `sudo groupadd spidev`
        - add user to the group: `sudo usermod --append --groups spidev debian`
        - permanently grant spidev permissions to the group: create `/etc/udev/rules.d/90-spi.rules` and write:
            - SUBSYSTEM=="spidev", GROUP="spidev", MODE="0660"
        - sudo udevadm control --reload-rules
        - sudo udevadm trigger
1. Run `systemctl daemon-reload` to make systemd recognize the .service file
1. Manage using systemctl
    - `systemctl [start\stop\restart\status] rusty-beagled`
1. View logs
    - `journalctl -f -u rusty-beagled` for logs & daemon info
        - KNOWN ISSUE: journal entries are sorted by date - if date & time on BeagleBone is incorrect, journal logs will be out of order
    - `tail -f /var/log/rusty_beagle.log` for logs only
1. To start the daemon at boot, run `systemctl enable rusty-beagled`
    - to disable: `systemctl disable rusty-beagled`

# Sharing connection through USB (Linux hosts only)
This section explains how to acquire internet connection on BeagleBone Black, by sharing the connection of a machine, that the BeagleBone is connected to via USB.
The exact steps differ depending on the firewall framework that is used (either ufw & iptables or nftables)

## On BeagleBone Black:
* Specify the host's USB port as a default gateway for the BeagleBone:
    * temporarily (not persistent between reboots):
        * `sudo route add default [ip]`
    * permanently:
        1. edit `/etc/systemd/network/usb1.network` - under "\[Network\]" add: "Gateway=\[ip\]"
        1. restart NetworkManager - `sudo systemctl restart NetworkManager.service`
    * where \[ip\] is the address of BeagleBone's USB interface on the host machine (either 192.168.6.1 or 192.168.7.1)
* In case of permission errors when running Rusty Beagle: change the capabilities of the rusty_beagle executable by running `sudo setcap cap_net_raw+ep [path/to/rusty_beagle]`

## On the host machine (ufw & iptables)
1. Enable packet forwarding for IPv4:
    1. edit `/etc/sysctl.conf` and uncomment `net.ipv4.ip_forward=1`
    1. verify: `sysctl -a | fgrep .forwarding | grep ^net | grep ipv4`
1. Add iptables firewall rules to forward traffic from the USB interface to the WiFi interface:
    1. `sudo iptables --table nat --append POSTROUTING --out-interface [name of host's working network interface] -j MASQUERADE`
    1. `sudo iptables --append FORWARD --in-interface [name of host's interface connected to BeagleBone] -j ACCEPT`

## On the host machine (nftables)
1. Enable packet forwarding for IPv4:
    1. edit `/etc/sysctl.conf` and uncomment `net.ipv4.ip_forward=1`
    1. verify: `sysctl -a | fgrep .forwarding | grep ^net | grep ipv4`
1. Add nftables firewall rules to forward traffic from the USB interface to the WiFi interface:
    1. `sudo nft add table nat`
    1. `sudo nft 'add chain nat postrouting { type nat hook postrouting priority 100 ; }'`
    1. `sudo nft 'add rule nat postrouting oifname "[name of host's working network interface]" counter masquerade'`
    1. `sudo nft 'add rule inet filter forward iifname "[name of host's interface connected to BeagleBone]" counter accept'`
