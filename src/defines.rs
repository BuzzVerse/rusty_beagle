#[allow(non_camel_case_types)]
pub mod lora_defines {
    /*
     * Register definitions
     */
    #[derive(Debug, Clone, Copy)]
    pub enum LoRaRegister {
        FIFO = 0x00,
        OP_MODE = 0x01,
        FRF_MSB = 0x06,
        FRF_MID = 0x07,
        FRF_LSB = 0x08,
        PA_CONFIG = 0x09,
        LNA = 0x0C,
        FIFO_ADDR_PTR = 0x0D,
        FIFO_TX_BASE_ADDR = 0x0E,
        FIFO_RX_BASE_ADDR = 0x0F,
        FIFO_RX_CURRENT_ADDR = 0x10,
        IRQ_FLAGS = 0x12,
        RX_NB_BYTES = 0x13,
        PKT_SNR_VALUE = 0x19,
        PKT_RSSI_VALUE = 0x1A,
        MODEM_CONFIG_1 = 0x1D,
        MODEM_CONFIG_2 = 0x1E,
        PREAMBLE_MSB = 0x20,
        PREAMBLE_LSB = 0x21,
        PAYLOAD_LENGTH = 0x22,
        MODEM_CONFIG_3 = 0x26,
        RSSI_WIDEBAND = 0x2C,
        DETECTION_OPTIMIZE = 0x31,
        DETECTION_THRESHOLD = 0x37,
        SYNC_WORD = 0x39,
        REG_IRQ_FLAGS_2 = 0x3F,
        DIO_MAPPING_1 = 0x40,
        DIO_MAPPING_2 = 0x41,
        VERSION = 0x42,
    }

    /*
     * Transceiver modes
     */
    #[derive(Debug, Clone, Copy)]
    pub enum LoRaMode {
        LONG_RANGE = 0x80,
        SLEEP = 0x00,
        STDBY = 0x01,
        TX = 0x03,
        RX_CONTINUOUS = 0x05,
        RX_SINGLE = 0x06,
    }

    /*
     * PA configuration
     */
    #[derive(Debug, Clone, Copy)]
    pub enum PAConfiguration {
        PA_BOOST = 0x80,
    }

    /*
     * IRQ masks
     */
    #[derive(Debug, Clone, Copy)]
    pub enum IRQMask {
        IRQ_TX_DONE_MASK = 0x08,
        IRQ_RX_DONE_MASK = 0x40,
        IRQ_PAYLOAD_CRC_ERROR = 0x20,
        PA_OUTPUT_RFO_PIN = 0,
        PA_OUTPUT_PA_BOOST_PIN = 1,
    }

    impl IRQMask {
        pub const IRQ_PAYLOAD_CRC_ERROR_MASK: IRQMask = IRQMask::IRQ_PAYLOAD_CRC_ERROR;
    }

    /*
     * Lora delays
     */
    #[derive(Debug, Clone, Copy)]
    pub enum LoRaDelay {
        LORA_DELAY_10MS = 10,
        LORA_DELAY_20MS = 20,
        TIMEOUT_RESET = 100,
    }

    /*
     * Spi defines
     */
    #[derive(Debug, Clone, Copy)]
    pub enum SPIIO {
        SPI_READ = 0x00,
        SPI_WRITE = 0x80,
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
