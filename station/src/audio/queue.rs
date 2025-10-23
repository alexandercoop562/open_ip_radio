use std::{
    cmp::min,
    collections::VecDeque,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use super::song::{BUFFER_SIZE, Song};

#[derive(Default)]
pub struct Queue {
    queue: VecDeque<Song>,
    active_song: Option<Song>, // will store songs somewhere else so this can be a refrence
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    pub title: String,
    pub artist: String,
    pub sample_rate: u32,
    pub channels: u32,
    pub timestamp: Option<u64>,
}

impl Queue {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn add(&mut self, song: Song) {
        self.queue.push_back(song);
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.active_song = None;
    }

    pub fn advance_queue(&mut self) {
        if let Some(mut song) = self.active_song.take() {
            song.position = 0;
            self.queue.push_back(song);
        }

        self.active_song = self.queue.pop_front();
    }

    pub fn get_audio_packet(&mut self) -> (Arc<Vec<u8>>, Duration) {
        return match &mut self.active_song {
            Some(song) => {
                let remaining_samples = song.pcm_data.len() - song.position;
                let samples_to_send = min(BUFFER_SIZE, remaining_samples);

                let mut audio_bytes = Vec::with_capacity(samples_to_send * 4);
                for i in song.position..song.position + samples_to_send {
                    audio_bytes.extend_from_slice(&song.pcm_data[i].to_le_bytes());
                }

                song.position += samples_to_send;

                (Arc::new(audio_bytes), song.sleep_duration)
            }
            None => (Arc::new(vec![]), Duration::new(0, 0)),
        };
    }

    pub fn get_current_timestamp() -> Option<u64> {
        return Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    pub fn get_metadata(&self) -> Metadata {
        let timestamp = Self::get_current_timestamp();

        let matadata = match &self.active_song {
            Some(song) => Metadata {
                title: song.title.clone(),
                artist: song.artist.clone(),
                sample_rate: song.sample_rate,
                channels: song.channels,
                timestamp,
            },
            None => Metadata::default(),
        };

        return matadata;
    }
}
