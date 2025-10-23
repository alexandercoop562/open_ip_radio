use crate::buffer::JitterBuffer;

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

use tokio::{net::TcpStream, sync::broadcast::Sender, task::JoinHandle};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

const WEBSOCKET_URL: &str = "ws://127.0.0.1:8082/";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Metadata {
    title: String,
    artist: String,
    sample_rate: u32,
    channels: u32,
    timestamp: Option<u64>,
}

type WSStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub fn binary_to_samples(data: &[u8]) -> Vec<f32> {
    let num_samples = data.len() / 4;
    let mut samples = Vec::with_capacity(num_samples);

    samples.extend(data.chunks_exact(4).map(|chunk| {
        let bytes: [u8; 4] = chunk.try_into().unwrap_or([0, 0, 0, 0]);
        return f32::from_le_bytes(bytes);
    }));

    return samples;
}

// Integration test level - Async message loop
#[cfg(not(tarpaulin_include))]
async fn message_handler(ws_stream: WSStream, jitter_buffer: Arc<JitterBuffer>, shutdown_tx: &Sender<()>) -> bool {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut shutdown_rx = shutdown_tx.subscribe();

    loop {
        tokio::select! {
            Some(msg) = ws_receiver.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(metadata) = serde_json::from_str::<Metadata>(text.as_ref()) {
                            println!("Now Playing: {} - {}", metadata.artist, metadata.title);
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        jitter_buffer.push(data.to_vec()).await;
                    }
                    Ok(Message::Close(_)) => {
                        println!("WebSocket connection closed");
                        return true;
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                        return true;
                    }
                    _ => {}
                }
            }
            _ = shutdown_rx.recv() => {
                let _ = ws_sender.send(Message::Close(None)).await;
                return false;
            }
        }
    }
}

pub fn launch(jitter_buffer: Arc<JitterBuffer>, shutdown_tx: &Sender<()>) -> JoinHandle<()> {
    return launch_with_url(WEBSOCKET_URL, jitter_buffer, shutdown_tx);
}

// Integration test level - WebSocket connection spawner
#[cfg(not(tarpaulin_include))]
pub fn launch_with_url(url: &str, jitter_buffer: Arc<JitterBuffer>, shutdown_tx: &Sender<()>) -> JoinHandle<()> {
    let shutdown_tx = shutdown_tx.clone();
    let url = url.to_string();

    return tokio::spawn(async move {
        const MAX_RETRIES: u32 = 3;
        let mut retry_count = 0;

        loop {
            let ws_stream = match connect_async(&url).await {
                Ok((stream, _)) => stream,
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= MAX_RETRIES {
                        eprintln!("Failed to connect to station: {}", e);
                        return;
                    }

                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }
            };

            let should_reconnect = message_handler(ws_stream, Arc::clone(&jitter_buffer), &shutdown_tx).await;

            if !should_reconnect {
                return;
            }

            retry_count += 1;
            if retry_count >= MAX_RETRIES {
                eprintln!("Connection lost after {} attempts. Giving up.", MAX_RETRIES);
                return;
            }
        }
    });
}
