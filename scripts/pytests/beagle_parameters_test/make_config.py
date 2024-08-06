import sys

if len(sys.argv) != 4:
    print("Wrong number of arguments\n")
    print("python3 make_config.py [bandwidth] [coding_rate] [spreading_factor]\n")
    sys.exit(-1)

bandwidth = sys.argv[1]
coding_rate = sys.argv[2]
spreading_factor = sys.argv[3]

tx_conf = f"""Config(
    mqtt_config: MQTTConfig(
        ip: "192.168.6.2",
        port: "1234",
        login: "admin",
        password: "verysecurepassword",
        topic: "lora/sensor"
    ),
    lora_config: LoRaConfig(
        mode: TX,
        reset_gpio: GPIO_66,
        dio0_gpio: GPIO_69,
        spi_config: SPIConfig(
            spidev_path: "/dev/spidev0.0",
            bits_per_word: 8,
            max_speed_hz: 500000,
            lsb_first: false,
            spi_mode: SPI_MODE_0,
        ),
        radio_config: RadioConfig(
            bandwidth: {bandwidth},
            coding_rate: {coding_rate},
            spreading_factor: {spreading_factor},
            tx_power: 17,
        ),
    ),
)
"""

rx_conf = f"""Config(
    mqtt_config: MQTTConfig(
        ip: "192.168.6.2",
        port: "1234",
        login: "admin",
        password: "verysecurepassword",
        topic: "lora/sensor"
    ),
    lora_config: LoRaConfig(
        mode: RX,
        reset_gpio: GPIO_67,
        dio0_gpio: GPIO_68,
        spi_config: SPIConfig(
            spidev_path: "/dev/spidev1.0",
            bits_per_word: 8,
            max_speed_hz: 500000,
            lsb_first: false,
            spi_mode: SPI_MODE_0,
        ),
        radio_config: RadioConfig(
            bandwidth: {bandwidth},
            coding_rate: {coding_rate},
            spreading_factor: {spreading_factor},
            tx_power: 17,
        ),
    ),
)
"""

tx_f = open("./tx_conf.ron", "w")
rx_f = open("./rx_conf.ron", "w")
tx_f.write(tx_conf)
rx_f.write(rx_conf)
tx_f.close()
rx_f.close()
