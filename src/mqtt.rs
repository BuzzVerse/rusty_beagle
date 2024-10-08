use std::time::Duration;
use anyhow::{Context, Result};
use log::{error, info};
use rumqttc::{Client, Event, MqttOptions, Packet as MqttPacket, Outgoing, QoS};
use std::sync::mpsc::Receiver;
use crate::packet::{Packet, PacketWrapper};
use crate::config::MQTTConfig;
extern crate chrono;

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

/// An enum to represent any message that could be sent through MQTT.
/// It provides an interface to convert different packet types to JSON,
/// and to extract specific fields (e.g. device_id) from different packet types.
pub enum MQTTMessage {
    Packet(Packet),
    PacketWrapper(PacketWrapper),
}

impl MQTTMessage {
    pub fn to_json(&self) -> Result<String> {
        match self {
            MQTTMessage::Packet(packet) 
                => Ok(format!(r#"{{ {} }}"#, packet.to_json()?)),
            MQTTMessage::PacketWrapper(wrapped) => Ok(wrapped.to_json()?),
        }
    }

    pub fn get_device_id(&self) -> u8 {
        match self {
            MQTTMessage::Packet(packet) => packet.id,
            MQTTMessage::PacketWrapper(wrapped) => wrapped.packet.id,
        }
    }
}

/// Generates a practically unique client_id
/// using date and time, millisecond precision
fn generate_client_id() -> String {
    let datetime = format!("{}", chrono::offset::Local::now().format("%Y%m%d-%H%M%S%3f"));
    format!("rustybeagle_{}", datetime)
}

pub struct Mqtt {
    client: Client,
}

impl Mqtt {
    pub fn new(mqtt_config: MQTTConfig) -> Result<Self> {
        let client_id = generate_client_id();
        println!("MQTT: client_id: {}", client_id);
        info!("MQTT: client_id: {}", client_id);
        let mut options = MqttOptions::new(client_id, mqtt_config.ip, mqtt_config.port.parse().context("Mqtt::new")?);
        options.set_credentials(mqtt_config.login, mqtt_config.password);
        options.set_keep_alive(Duration::from_secs(5));
        let reconnect_interval = mqtt_config.reconnect_interval;

        let (client, mut connection) = Client::new(options, 10);

        std::thread::spawn(move || {
            for notification in connection.iter() {
                match notification {
                    Ok(m) => {
                        match m {
                            Event::Incoming(MqttPacket::PingReq) |
                            Event::Incoming(MqttPacket::PingResp) |
                            Event::Incoming(MqttPacket::PubAck(..)) |
                            Event::Outgoing(Outgoing::Publish(..)) |
                            Event::Outgoing(Outgoing::PingReq) |
                            Event::Outgoing(Outgoing::PingResp) => continue,
                            _ => info!("MQTT: {:?}", m)
                        }
                    },
                    Err(e) => {
                        eprintln!("MQTT: {:?}, retrying in {} s...", e, reconnect_interval);
                        error!("MQTT: {:?}, retrying in {} s...", e, reconnect_interval);
                        // retry after [reconnect_interval] seconds
                        std::thread::sleep(Duration::from_secs(reconnect_interval));
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

    pub fn thread_run(&self, mqtt_config: MQTTConfig, option_receiver: Option<Receiver<MQTTMessage>>) {
        let receiver = match option_receiver {
            Some(receiver) => receiver,
            None => {
                eprintln!("No receiver created");
                error!("No receiver created");
                std::process::exit(-1);
            },
        };

        loop {
            let received = handle_error_continue!(receiver.recv());
            let msg = handle_error_continue!(received.to_json());
            let topic = mqtt_config
                .topic
                .replace("{device_id}", &received.get_device_id().to_string());
            handle_error_continue!(self.publish(&topic, &msg));
        }
    }
}
