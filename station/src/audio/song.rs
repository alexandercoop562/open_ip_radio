use std::sync::Arc;
use std::time::Duration;

use super::codec::{get_data, parse_audio_data};
use crate::MainErr;

#[derive(Clone, Default)]
pub struct Song {
    pub pcm_data: Arc<Vec<f32>>,
    pub position: usize,
    pub sample_rate: u32,
    pub channels: u32,
    pub title: String,
    pub artist: String,
    pub sleep_duration: Duration,
}

pub const BUFFER_SIZE: usize = 8192;

pub fn calculate_sleep_duration(sample_rate: u32, channels: u32) -> Duration {
    return Duration::from_micros((1_000_000 * BUFFER_SIZE as u64 / channels as u64) / sample_rate as u64);
}

// Requires audio files - Integration test level
#[cfg(not(tarpaulin_include))]
pub async fn get_song(filename: String) -> Result<Song, MainErr> {
    let (artist, title, format) = get_data(filename)?;
    let (pcm_data, sample_rate, channels) = parse_audio_data(format)?;

    let sleep_duration = calculate_sleep_duration(sample_rate, channels);

    return Ok(Song {
        pcm_data: Arc::new(pcm_data),
        position: 0,
        sample_rate,
        channels,
        title,
        artist,
        sleep_duration,
    });
}
