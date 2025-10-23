use std::collections::VecDeque;
use tokio::sync::Mutex;

pub struct JitterBuffer {
    buffer: Mutex<VecDeque<Vec<u8>>>,
}

impl Default for JitterBuffer {
    fn default() -> Self {
        return Self::new();
    }
}

impl JitterBuffer {
    pub fn new() -> Self {
        return Self {
            buffer: Mutex::new(VecDeque::new()),
        };
    }

    pub async fn push(&self, packet: Vec<u8>) {
        let mut buffer = self.buffer.lock().await;
        buffer.push_back(packet);
    }

    pub async fn pop(&self) -> Option<Vec<u8>> {
        let mut buffer = self.buffer.lock().await;
        return buffer.pop_front();
    }

    pub async fn len(&self) -> usize {
        let buffer = self.buffer.lock().await;
        return buffer.len();
    }
}
