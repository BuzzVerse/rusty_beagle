use bme280::i2c::BME280;
use linux_embedded_hal::{Delay, I2cdev};

pub struct BME280Sensor {
    bme280: BME280<I2cdev>,
    delay: Delay,
}

impl BME280Sensor {
    pub fn new(i2c_bus_path: &str) -> Self {
        let i2c_bus = I2cdev::new(i2c_bus_path).unwrap();
        let mut delay = Delay {};

        let mut bme280 = BME280::new_primary(i2c_bus);
        bme280.init(&mut delay).unwrap();

        BME280Sensor { bme280, delay }
    }

    pub fn read_measurements(&mut self) -> (f32, f32, f32) {
        let measurements = self.bme280.measure(&mut self.delay).unwrap();
        (
            measurements.temperature,
            measurements.pressure / 100.0,
            measurements.humidity,
        )
    }
}
