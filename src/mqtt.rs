use std::time::Duration;
use anyhow::{Context, Result};
use log::{error, info};
use rumqttc::{Client, MqttOptions, QoS};
use crate::config::MQTTConfig;

pub struct Mqtt {
    client: Client,
}

impl Mqtt {
    pub fn new(mqtt_config: MQTTConfig) -> Result<Self> {
        let mut options = MqttOptions::new("RustyBeagle", mqtt_config.ip, mqtt_config.port.parse().context("Mqtt::new")?);
        options.set_credentials(mqtt_config.login, mqtt_config.password);
        options.set_keep_alive(Duration::from_secs(5));

        let (client, mut connection) = Client::new(options, 10);

        std::thread::spawn(move || {
            for notification in connection.iter() {
                match notification {
                    Ok(m) => info!("MQTT: {:?}", m),
                    Err(e) => {
                        eprintln!("{:?}", e);
                        error!("MQTT: {:?}", e);
                        break;
                    }
                }
            }
        });

        Ok(Self { client })
    }

    pub fn publish(&self, topic: &str, msg: &str) -> Result<()> {
        self.client.publish(topic, QoS::AtLeastOnce, false, msg)?;
        Ok(())
    }
}
