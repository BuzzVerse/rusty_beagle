use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

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
