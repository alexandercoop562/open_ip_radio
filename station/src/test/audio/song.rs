#[cfg(test)]
mod test {
    use crate::audio::song::{BUFFER_SIZE, Song, calculate_sleep_duration, get_song};
    use std::{sync::Arc, time::Duration};

    #[test]
    fn test_song() {
        let song = Song {
            pcm_data: Arc::new(vec![0.1, 0.2, 0.3]),
            position: 0,
            sample_rate: 44100,
            channels: 2,
            title: "Test".to_string(),
            artist: "Artist".to_string(),
            sleep_duration: Duration::from_millis(10),
        };
        assert_eq!(song.pcm_data.len(), 3);
        assert_eq!(song.sample_rate, 44100);

        let song = Song::default();
        assert!(song.pcm_data.is_empty());
        assert_eq!(song.sample_rate, 0);

        let song1 = Song {
            pcm_data: Arc::new(vec![0.1, 0.2]),
            position: 5,
            sample_rate: 44100,
            channels: 2,
            title: "Test".to_string(),
            artist: "Artist".to_string(),
            sleep_duration: Duration::from_millis(10),
        };
        let song2 = song1.clone();
        assert!(Arc::ptr_eq(&song1.pcm_data, &song2.pcm_data));
    }

    #[test]
    fn test_buffer_size_constant() {
        assert_eq!(BUFFER_SIZE, 8192);
    }

    #[test]
    fn test_calculate_sleep_duration() {
        let duration = calculate_sleep_duration(44100, 2);
        assert!(duration.as_micros() > 90_000 && duration.as_micros() < 95_000);

        let duration = calculate_sleep_duration(48000, 2);
        assert!(duration.as_micros() > 80_000 && duration.as_micros() < 90_000);

        let duration = calculate_sleep_duration(44100, 1);
        assert!(duration.as_micros() > 180_000);
    }

    #[tokio::test]
    async fn test_get_song_invalid_file() {
        let result = get_song("nonexistent_file.mp3".to_string()).await;
        assert!(result.is_err());
    }
}
