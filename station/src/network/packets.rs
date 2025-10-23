use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast::{Receiver, Sender},
    task,
};
use tokio_tungstenite::{
    WebSocketStream, accept_hdr_async,
    tungstenite::{handshake::server::Request, protocol::Message},
};

use tracing::info;

use crate::audio::queue::{Metadata, Queue};
use crate::{MainErr, get_lock};

const HOST: &str = "0.0.0.0";
const WEBSOCKET_PORT: u16 = 8082;

type WSSender = SplitSink<WebSocketStream<TcpStream>, Message>;
type WSReceiver = SplitStream<WebSocketStream<TcpStream>>;

// Integration test level - WebSocket I/O
#[cfg(not(tarpaulin_include))]
async fn ws_connection(stream: TcpStream) -> Result<WebSocketStream<TcpStream>, MainErr> {
    let callback = |_req: &Request, response| return Ok(response);

    return match accept_hdr_async(stream, callback).await {
        Ok(s) => Ok(s),
        Err(e) => Err(e.into()),
    };
}

// Integration test level - Async message handling
#[cfg(not(tarpaulin_include))]
async fn message_handler(
    mut metadata_rx: Receiver<Metadata>,
    mut audio_rx: Receiver<Arc<Vec<u8>>>,
    ws_sender: &mut WSSender,
    ws_receiver: &mut WSReceiver,
) {
    loop {
        tokio::select! {
            result = metadata_rx.recv() => {
                if let Ok(metadata) = result {
                    if let Ok(json) = serde_json::to_string(&metadata)
                        && ws_sender.send(Message::Text(json.into())).await.is_err()
                    {
                        break;
                    }
                } else {
                    break;
                }
            }
            result = audio_rx.recv() => {
                if let Ok(audio_bytes) = result {
                    let bytes = (*audio_bytes).clone();
                    if ws_sender.send(Message::Binary(bytes.into())).await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
            Some(msg) = ws_receiver.next() => {
                match msg {
                    Ok(Message::Close(_)) => break,
                    Err(_) => break,
                    _ => {}
                }
            }
        }
    }
}

// Integration test level - WebSocket connection lifecycle
#[cfg(not(tarpaulin_include))]
pub async fn connection_handler(
    stream: TcpStream,
    queue: Arc<Mutex<Queue>>,
    metadata_rx: Receiver<Metadata>,
    audio_rx: Receiver<Arc<Vec<u8>>>,
) -> Result<(), MainErr> {
    let ws_stream = ws_connection(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let current_metadata = { get_lock(&queue)?.get_metadata() };

    if let Ok(json) = serde_json::to_string(&current_metadata)
        && current_metadata.title != *""
    {
        ws_sender.send(Message::Text(json.into())).await?;
    }

    message_handler(metadata_rx, audio_rx, &mut ws_sender, &mut ws_receiver).await;

    return Ok(());
}

// Integration test level - WebSocket server
#[cfg(not(tarpaulin_include))]
pub async fn ws_handler(
    metadata_tx: Arc<Sender<Metadata>>,
    audio_tx: Arc<Sender<Arc<Vec<u8>>>>,
    queue: Arc<Mutex<Queue>>,
) -> Result<(), MainErr> {
    let addr: SocketAddr = format!("{}:{}", HOST, WEBSOCKET_PORT).parse()?;
    let listener = TcpListener::bind(addr).await?;
    let bound_port = listener.local_addr()?.port();

    info!("WebSocket server listening on ws://{}:{}/", HOST, bound_port);

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            let metadata_rx = metadata_tx.subscribe();
            let audio_rx = audio_tx.subscribe();
            let queue = Arc::clone(&queue);
            task::spawn(connection_handler(stream, queue, metadata_rx, audio_rx));
        }
    }
}
