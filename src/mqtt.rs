use std::time::Duration;
use anyhow::{Context, Result};
use log::{error, info};
use rumqttc::{Client, MqttOptions, QoS};
use std::sync::mpsc::Receiver;
use crate::packet::Packet;
use crate::config::MQTTConfig;

macro_rules! handle_error_continue {
    ($func:expr) => {
        match $func {
            Err(e) => {
                eprintln!("{:?}", e);
                error!("{:?}", e);
                continue;
            }
            Ok(s) => s,
        }
    };
}

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

    pub fn thread_run(&self, mqtt_config: MQTTConfig, option_receiver: Option<Receiver<Packet>>) {
        let receiver = match option_receiver {
            Some(receiver) => receiver,
            None => {
                eprintln!("No receiver created");
                error!("No receiver created");
                std::process::exit(-1);
            },
        };

        loop {
            let packet: Packet = handle_error_continue!(receiver.recv());
            let msg = handle_error_continue!(packet.to_json());
            let topic = mqtt_config
                .topic
                .replace("{device_id}", &packet.id.to_string());
            handle_error_continue!(self.publish(&topic, &msg));
        }
    }
}
