#[allow(non_camel_case_types)]
pub mod lora_defines {
    /*
     * Register definitions
     */
    pub const REG_FIFO: u8 = 0x00;
    pub const REG_OP_MODE: u8 = 0x01;
    pub const REG_FRF_MSB: u8 = 0x06;
    pub const REG_FRF_MID: u8 = 0x07;
    pub const REG_FRF_LSB: u8 = 0x08;
    pub const REG_PA_CONFIG: u8 = 0x09;
    pub const REG_LNA: u8 = 0x0C;
    pub const REG_FIFO_ADDR_PTR: u8 = 0x0D;
    pub const REG_FIFO_TX_BASE_ADDR: u8 = 0x0E;
    pub const REG_FIFO_RX_BASE_ADDR: u8 = 0x0F;
    pub const REG_FIFO_RX_CURRENT_ADDR: u8 = 0x10;
    pub const REG_IRQ_FLAGS: u8 = 0x12;
    pub const REG_RX_NB_BYTES: u8 = 0x13;
    pub const REG_PKT_SNR_VALUE: u8 = 0x19;
    pub const REG_PKT_RSSI_VALUE: u8 = 0x1A;
    pub const REG_MODEM_CONFIG_1: u8 = 0x1D;
    pub const REG_MODEM_CONFIG_2: u8 = 0x1E;
    pub const REG_PREAMBLE_MSB: u8 = 0x20;
    pub const REG_PREAMBLE_LSB: u8 = 0x21;
    pub const REG_PAYLOAD_LENGTH: u8 = 0x22;
    pub const REG_MODEM_CONFIG_3: u8 = 0x26;
    pub const REG_RSSI_WIDEBAND: u8 = 0x2C;
    pub const REG_DETECTION_OPTIMIZE: u8 = 0x31;
    pub const REG_DETECTION_THRESHOLD: u8 = 0x37;
    pub const REG_SYNC_WORD: u8 = 0x39;
    pub const REG_REG_IRQ_FLAGS_2: u8 = 0x3F;
    pub const REG_DIO_MAPPING_1: u8 = 0x40;
    pub const REG_DIO_MAPPING_2: u8 = 0x41;
    pub const REG_VERSION: u8 = 0x42;

    /*
     * Transceiver modes
     */
    pub const MODE_LONG_RANGE_MODE: u8 = 0x80;
    pub const MODE_SLEEP: u8 = 0x00;
    pub const MODE_STDBY: u8 = 0x01;
    pub const MODE_TX: u8 = 0x03;
    pub const MODE_RX_CONTINUOUS: u8 = 0x05;
    pub const MODE_RX_SINGLE: u8 = 0x06;

    /*
     * PA configuration
     */
    pub const PA_BOOST: u8 = 0x80;

    /*
     * IRQ masks
     */
    pub const IRQ_TX_DONE_MASK: u8 = 0x08;
    pub const IRQ_PAYLOAD_CRC_ERROR_MASK: u8 = 0x20;
    pub const IRQ_RX_DONE_MASK: u8 = 0x40;
    pub const IRQ_PAYLOAD_CRC_ERROR: u8 = 0x20;
    pub const PA_OUTPUT_RFO_PIN: u8 = 0;
    pub const PA_OUTPUT_PA_BOOST_PIN: u8 = 1;

    /*
     * Lora delays
     */
    pub const LORA_DELAY_10MS: u8 = 10;
    pub const LORA_DELAY_20MS: u8 = 20;
    pub const TIMEOUT_RESET: u8 = 100;

    pub const LORA_TAG: &str = "LORA_DRIVER";

    /*
     * Spi defines
     */
    pub const SPI_READ: u8 = 0x00;
    pub const SPI_WRITE: u8 = 0x80;
}

#[allow(non_camel_case_types)]
pub mod api_defines {
    #[derive(Debug)]
    pub enum API_Status {
        API_OK,                     // The operation was successful.
        API_FAILED_SPI_SET_PIN,     // The pin has failed to have been set
        API_FAILED_SPI_SET_LEVEL,   // The spi level failed to be set for a pin
        API_FAILED_SPI_CHIP_SELECT, // The SPI chip select operation failed.
        API_FAILED_SPI_ADD_DEVICE,  // The spi device failed to be added
        API_FAILED_SPI_INIT,        // The SPI initialization failed.
        API_FAILED_SPI_READ,        // The SPI read operation failed.
        API_FAILED_SPI_READ_BUF,    // The SPI read buffer operation failed.
        API_FAILED_SPI_WRITE,       // The SPI write operation failed.
        API_FAILED_SPI_WRITE_BUF,   // The SPI write buffer operation failed.
        API_BUFFER_TOO_LARGE,       // The buffer is to large to assign.
        API_NULL_POINTER_ERROR,     // The pointer is NULL.
        API_SPI_ERROR,              // The SPI operation encountered an error.
    }
}
