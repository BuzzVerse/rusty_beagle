#[allow(non_camel_case_types)]
pub mod lora_defines {
    #[derive(Debug)]
    pub enum LoRa_Registers {
        REG_FIFO = 0x00,
        REG_OP_MODE = 0x01,
        REG_FRF_MSB = 0x06,
        REG_FRF_MID = 0x07,
        REG_FRF_LSB = 0x08,
        REG_PA_CONFIG = 0x09,
        REG_LNA = 0x0c,
        REG_FIFO_ADDR_PTR = 0x0d,
        REG_FIFO_TX_BASE_ADDR = 0x0e,
        REG_FIFO_RX_BASE_ADDR = 0x0f,
        REG_FIFO_RX_CURRENT_ADDR = 0x10,
        REG_IRQ_FLAGS = 0x12,
        REG_RX_NB_BYTES = 0x13,
        REG_PKT_SNR_VALUE = 0x19,
        REG_PKT_RSSI_VALUE = 0x1a,
        REG_MODEM_CONFIG_1 = 0x1d,
        REG_MODEM_CONFIG_2 = 0x1e,
        REG_PREAMBLE_MSB = 0x20,
        REG_PREAMBLE_LSB = 0x21,
        REG_PAYLOAD_LENGTH = 0x22,
        REG_MODEM_CONFIG_3 = 0x26,
        REG_RSSI_WIDEBAND = 0x2c,
        REG_DETECTION_OPTIMIZE = 0x31,
        REG_DETECTION_THRESHOLD = 0x37,
        REG_SYNC_WORD = 0x39,
        REG_REG_IRQ_FLAGS_2 = 0x3F,
        REG_DIO_MAPPING_1 = 0x40,
        REG_DIO_MAPPING_2 = 0x41,
        REG_VERSION = 0x42,
    }

    #[derive(Debug)]
    pub enum Transceiver_Modes {
        MODE_LONG_RANGE_MODE = 0x80,
        MODE_SLEEP = 0x00,
        MODE_STDBY = 0x01,
        MODE_TX = 0x03,
        MODE_RX_CONTINUOUS = 0x05,
        MODE_RX_SINGLE = 0x06,
    }

    #[derive(Debug)]
    pub enum PA_Configuration {
        PA_BOOST = 0x80,
    }

    #[derive(Debug)]
    pub enum IRQ_Masks {
        IRQ_TX_DONE_MASK = 0x08,
        IRQ_RX_DONE_MASK = 0x40,
        IRQ_PAYLOAD_CRC_ERROR = 0x20,
        PA_OUTPUT_RFO_PIN = 0,
        PA_OUTPUT_PA_BOOST_PIN = 1,
    }

    #[derive(Debug)]
    pub enum LoRa_Delays {
        LORA_DELAY_10MS = 10,
        LORA_DELAY_20MS = 20,
        TIMEOUT_RESET = 100,
    }
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
