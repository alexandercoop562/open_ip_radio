use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::{
    sync::broadcast::{self, Sender},
    task,
    time::sleep,
};

use tracing::{info, warn};

use super::packets::ws_handler;
use crate::audio::queue::{Metadata, Queue};
use crate::{MainErr, get_lock};

#[derive(Clone)]
pub struct Streamer {
    pub metadata_tx: Arc<Sender<Metadata>>,
    pub audio_tx: Arc<Sender<Arc<Vec<u8>>>>,
}

impl Default for Streamer {
    fn default() -> Self {
        return Self::new();
    }
}

impl Streamer {
    pub fn new() -> Self {
        let metadata_tx = Arc::new(broadcast::channel::<Metadata>(16).0);
        let audio_tx = Arc::new(broadcast::channel::<Arc<Vec<u8>>>(16).0);

        return Self { metadata_tx, audio_tx };
    }

    pub fn should_notify_metadata(metadata: &Metadata) -> bool {
        return !metadata.title.is_empty();
    }

    #[cfg(test)]
    pub fn calculate_effective_sleep(base_duration: Duration, override_sleep: Option<Duration>) -> Duration {
        return override_sleep.unwrap_or(base_duration);
    }

    pub fn notify_song_playing(&self, queue: Arc<Mutex<Queue>>) -> Result<(), MainErr> {
        let metadata = { get_lock(&queue)?.get_metadata() };

        if !Self::should_notify_metadata(&metadata) {
            return Err("Song not Playing".into());
        };

        info!("Now Playing: {} - {}", metadata.artist, metadata.title);
        if self.metadata_tx.receiver_count() > 0 {
            self.metadata_tx.send(metadata)?;
        }

        return Ok(());
    }

    // Integration test level - Main streaming loop
    #[cfg(not(tarpaulin_include))]
    pub async fn launch(&self, queue: Arc<Mutex<Queue>>) -> Result<(), MainErr> {
        let metadata_tx = Arc::clone(&self.metadata_tx);
        let audio_tx = Arc::clone(&self.audio_tx);
        let queue_clone = Arc::clone(&queue);

        task::spawn(async move {
            if let Err(e) = ws_handler(metadata_tx, audio_tx, queue_clone).await {
                warn!("ws_handler exited with error: {}", e);
            }
        });

        return self.launch_loop(queue, None).await;
    }

    pub async fn launch_loop(
        &self,
        queue: Arc<Mutex<Queue>>,
        #[allow(unused_variables)] max_loops_for_test: Option<u64>,
    ) -> Result<(), MainErr> {
        let mut song_start = true;
        #[cfg(test)]
        let mut loop_count: u64 = 0;

        loop {
            #[cfg(test)]
            {
                loop_count += 1;
                if let Some(max) = max_loops_for_test
                    && loop_count >= max
                {
                    break;
                }
            }

            if song_start {
                {
                    get_lock(&queue)?.advance_queue();
                }

                if let Err(e) = self.notify_song_playing(Arc::clone(&queue)) {
                    warn!("No song to play: {}", e);
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
                song_start = false;
            }

            let (audio_packet, sleep_duration) = { get_lock(&queue)?.get_audio_packet() };

            if audio_packet.is_empty() {
                #[cfg(feature = "profile")]
                break;

                #[cfg(not(feature = "profile"))]
                {
                    song_start = true;
                    continue;
                }
            }

            if self.audio_tx.receiver_count() > 0
                && let Err(e) = self.audio_tx.send(audio_packet)
            {
                #[cfg(not(tarpaulin_include))]
                warn!("(Streamer): Error sending audio packet: {}", e);
            }

            sleep(sleep_duration).await;
        }

        #[allow(unreachable_code)]
        return Ok(());
    }
}
