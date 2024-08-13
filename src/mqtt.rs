use std::{collections::VecDeque, time::Duration};
use std::sync::Arc;
use anyhow::{Context, Result};
use log::{error, info};
use tokio::sync::{Mutex, Notify};
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use tokio::time::sleep;
use crate::config::MQTTConfig;

pub struct BlockingQueue<T> {
    queue: Arc<Mutex<VecDeque<T>>>,
    notify: Arc<Notify>,
    capacity: usize,
}

impl<T> BlockingQueue<T> {
    pub fn new(capacity: usize) -> Self {
        BlockingQueue {
            queue: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            notify: Arc::new(Notify::new()),
            capacity,
        }
    }

    pub async fn put(&self, item: T) {
        let mut queue = self.queue.lock().await;

        // Wait if the queue is full
        while queue.len() >= self.capacity {
            drop(queue); // Release the lock before waiting
            self.notify.notified().await; // Wait until notified
            queue = self.queue.lock().await; // Re-acquire the lock
        }

        queue.push_back(item);
        self.notify.notify_one(); // Notify one waiting task
    }

    pub async fn take(&self) -> T {
        let mut queue = self.queue.lock().await;

        // Wait if the queue is empty
        while queue.is_empty() {
            drop(queue); // Release the lock before waiting
            self.notify.notified().await; // Wait until notified
            queue = self.queue.lock().await; // Re-acquire the lock
        }

        let item = queue.pop_front().expect("Queue should not be empty");
        self.notify.notify_one(); // Notify one waiting task
        item
    }
}

impl<T> Clone for BlockingQueue<T> {
    fn clone(&self) -> Self {
        BlockingQueue {
            queue: Arc::clone(&self.queue),
            notify: Arc::clone(&self.notify),
            capacity: self.capacity,
        }
    }
}

pub struct Mqtt {
    client: AsyncClient,
    eventloop_handle: tokio::task::JoinHandle<()>,
}

impl Mqtt {
    pub async fn new(mqtt_config: MQTTConfig) -> Result<Self> {
        let mut options = MqttOptions::new("RustyBeagle", mqtt_config.ip, mqtt_config.port.parse().context("Mqtt::new")?);
        options.set_credentials(mqtt_config.login, mqtt_config.password);
        options.set_keep_alive(Duration::from_secs(5));

        let (client, mut event_loop) = AsyncClient::new(options, 10);
        let eventloop_handle = tokio::spawn(async move {
            loop {
                match event_loop.poll().await {
                    Ok(m) => info!("MQTT: {:?}", m),
                    Err(e) => {
                        eprintln!("{:?}", e);
                        error!("MQTT: {:?}", e);
                    }

                }
            }
        });
        Ok(Self {client, eventloop_handle})
    }

    pub async fn publish(&self, topic: &str, msg: &str) -> Result<()> {
        self.client.publish(topic, QoS::AtLeastOnce, false, msg).await?;
        Ok(())
    }

    pub async fn shutdown(&self) {
        self.eventloop_handle.abort();
        sleep(Duration::from_secs(1)).await;
    }
}
