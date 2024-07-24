#!/bin/bash

# SPIDEV0, /dev/spidev0.#
config-pin p9_17 spi_cs     # NSS
config-pin p9_18 spi        # MOSI
config-pin p9_21 spi        # MISO
config-pin p9_22 spi_sclk   # SCK

# SPIDEV1, /dev/spidev1.#
config-pin p9_28 spi_cs     # NSS
config-pin p9_29 spi        # MISO
config-pin p9_30 spi        # MOSI
config-pin p9_31 spi_sclk   # SCK
