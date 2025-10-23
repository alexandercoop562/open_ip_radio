#[cfg(test)]
mod tests {
    #[test]
    fn test_streamer() {
        use crate::network::streamer::Streamer;

        let streamer = Streamer::new();
        assert_eq!(streamer.metadata_tx.receiver_count(), 0);
        assert_eq!(streamer.audio_tx.receiver_count(), 0);

        let streamer = Streamer::default();
        assert_eq!(streamer.metadata_tx.receiver_count(), 0);
    }

    #[test]
    fn test_should_notify_metadata() {
        use crate::{audio::queue::Metadata, network::streamer::Streamer};

        let metadata = Metadata {
            title: "".to_string(),
            artist: "Artist".to_string(),
            sample_rate: 44100,
            channels: 2,
            timestamp: Some(1234567890),
        };
        assert!(!Streamer::should_notify_metadata(&metadata));

        let metadata = Metadata {
            title: "Song Title".to_string(),
            artist: "".to_string(),
            sample_rate: 0,
            channels: 0,
            timestamp: None,
        };
        assert!(Streamer::should_notify_metadata(&metadata));
    }

    #[test]
    fn test_notify_song_playing() {
        use crate::{audio::queue::Queue, audio::song::Song, network::streamer::Streamer};
        use std::{
            sync::{Arc, Mutex},
            time::Duration,
        };

        let streamer = Streamer::new();
        let queue = Arc::new(Mutex::new(Queue::new()));
        let result = streamer.notify_song_playing(queue.clone());
        assert!(result.is_err());

        let song = Song {
            pcm_data: Arc::new(vec![0.1; 1000]),
            position: 0,
            sample_rate: 44100,
            channels: 2,
            title: "Test Song".to_string(),
            artist: "Test Artist".to_string(),
            sleep_duration: Duration::from_millis(10),
        };
        {
            if let Ok(mut q) = queue.lock() {
                q.add(song.clone());
                q.advance_queue();
            }
        }
        let result = streamer.notify_song_playing(queue.clone());
        assert!(result.is_ok());

        let streamer = Streamer::new();
        let _rx = streamer.metadata_tx.subscribe();
        {
            if let Ok(mut q) = queue.lock() {
                q.add(song);
                q.advance_queue();
            }
        }
        let result = streamer.notify_song_playing(queue);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_streamer_receivers() {
        use crate::network::streamer::Streamer;

        let streamer = Streamer::new();
        assert_eq!(streamer.metadata_tx.receiver_count(), 0);
        assert_eq!(streamer.audio_tx.receiver_count(), 0);

        let _metadata_rx = streamer.metadata_tx.subscribe();
        let _audio_rx = streamer.audio_tx.subscribe();

        assert_eq!(streamer.metadata_tx.receiver_count(), 1);
        assert_eq!(streamer.audio_tx.receiver_count(), 1);
    }

    #[test]
    fn test_calculate_effective_sleep() {
        use crate::network::streamer::Streamer;
        use std::time::Duration;

        let base = Duration::from_millis(100);
        let result = Streamer::calculate_effective_sleep(base, None);
        assert_eq!(result, base);

        let override_dur = Duration::from_millis(50);
        let result = Streamer::calculate_effective_sleep(base, Some(override_dur));
        assert_eq!(result, override_dur);
    }

    #[tokio::test]
    async fn test_launch_loop() {
        use crate::{audio::queue::Queue, audio::song::Song, network::streamer::Streamer};
        use std::{
            sync::{Arc, Mutex},
            time::Duration,
        };

        let streamer = Streamer::new();
        let queue = Arc::new(Mutex::new(Queue::new()));

        let song = Song {
            pcm_data: Arc::new(vec![0.1; 8192]),
            position: 0,
            sample_rate: 44100,
            channels: 2,
            title: "Test Song".to_string(),
            artist: "Test Artist".to_string(),
            sleep_duration: Duration::from_millis(1),
        };

        {
            if let Ok(mut q) = queue.lock() {
                q.add(song);
            }
        }

        let result = streamer.launch_loop(queue, Some(3)).await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread")]
    #[allow(clippy::panic)]
    async fn test_launch_loop_with_receivers() {
        use crate::{audio::queue::Queue, audio::song::Song, network::streamer::Streamer};
        use std::{
            sync::{Arc, Mutex},
            time::Duration,
        };

        let streamer = Streamer::new();
        let queue = Arc::new(Mutex::new(Queue::new()));

        let _metadata_rx = streamer.metadata_tx.subscribe();
        let mut audio_rx = streamer.audio_tx.subscribe();

        let song = Song {
            pcm_data: Arc::new(vec![0.2; 8192 * 2]),
            position: 0,
            sample_rate: 48000,
            channels: 2,
            title: "Receiver Test".to_string(),
            artist: "Test Artist".to_string(),
            sleep_duration: Duration::from_millis(1),
        };

        {
            if let Ok(mut q) = queue.lock() {
                q.add(song);
            }
        }

        let queue_clone = Arc::clone(&queue);
        let handle = tokio::spawn(async move {
            let _ = streamer.launch_loop(queue_clone, Some(2)).await;
        });

        tokio::select! {
            result = audio_rx.recv() => {
                assert!(result.is_ok());
                if let Ok(audio) = result {
                    assert!(!audio.is_empty());
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(2)) => {
                panic!("Timeout waiting for audio packet");
            }
        }

        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_launch_loop_no_song_error_path() {
        use crate::{audio::queue::Queue, network::streamer::Streamer};
        use std::sync::{Arc, Mutex};

        let streamer = Streamer::new();
        let queue = Arc::new(Mutex::new(Queue::new()));

        let result = streamer.launch_loop(queue, Some(2)).await;
        assert!(result.is_ok());
    }
}
