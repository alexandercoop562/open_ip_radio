#[cfg(test)]
mod test {
    use crate::audio::{queue::*, song::Song};
    use std::{sync::Arc, time::Duration};

    #[test]
    fn test_queue() {
        let queue = Queue::new();
        let metadata = queue.get_metadata();
        assert_eq!(metadata.title, "");

        let mut queue = Queue::new();
        let song = Song {
            pcm_data: Arc::new(vec![0.1, 0.2, 0.3, 0.4]),
            position: 0,
            sample_rate: 44100,
            channels: 2,
            title: "Test Song".to_string(),
            artist: "Test Artist".to_string(),
            sleep_duration: Duration::from_millis(10),
        };
        queue.add(song);

        let metadata = queue.get_metadata();
        assert_eq!(metadata.title, "");

        queue.advance_queue();
        let metadata = queue.get_metadata();
        assert_eq!(metadata.title, "Test Song");
        assert!(metadata.timestamp.is_some());

        let (audio_packet, _) = queue.get_audio_packet();
        assert_eq!(audio_packet.len(), 16);
        let first_sample = f32::from_le_bytes([audio_packet[0], audio_packet[1], audio_packet[2], audio_packet[3]]);
        assert_eq!(first_sample, 0.1);

        let mut queue = Queue::new();
        let (audio_packet, duration) = queue.get_audio_packet();
        assert!(audio_packet.is_empty());
        assert_eq!(duration, Duration::new(0, 0));
    }

    #[test]
    fn test_queue_multiple_songs() {
        let mut queue = Queue::new();
        let song1 = Song {
            pcm_data: Arc::new(vec![0.1; 1000]),
            position: 0,
            sample_rate: 44100,
            channels: 2,
            title: "Song 1".to_string(),
            artist: "Artist 1".to_string(),
            sleep_duration: Duration::from_millis(10),
        };
        let song2 = Song {
            pcm_data: Arc::new(vec![0.2; 1000]),
            position: 0,
            sample_rate: 48000,
            channels: 2,
            title: "Song 2".to_string(),
            artist: "Artist 2".to_string(),
            sleep_duration: Duration::from_millis(10),
        };

        queue.add(song1);
        queue.add(song2);
        queue.advance_queue();

        let metadata = queue.get_metadata();
        assert_eq!(metadata.title, "Song 1");

        queue.advance_queue();
        let metadata = queue.get_metadata();
        assert_eq!(metadata.title, "Song 2");
        assert_eq!(metadata.sample_rate, 48000);
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = Metadata {
            title: "Test".to_string(),
            artist: "Artist".to_string(),
            sample_rate: 44100,
            channels: 2,
            timestamp: Some(1234567890),
        };

        if let Ok(json) = serde_json::to_string(&metadata)
            && let Ok(deserialized) = serde_json::from_str::<Metadata>(&json)
        {
            assert_eq!(deserialized.title, metadata.title);
            assert_eq!(deserialized.timestamp, metadata.timestamp);
        }
    }

    #[test]
    fn test_get_current_timestamp() {
        let timestamp = Queue::get_current_timestamp();
        assert!(timestamp.is_some());

        if let Some(ts) = timestamp {
            assert!(ts > 1_577_836_800);
            assert!(ts < 32_503_680_000);
        }
    }

    #[test]
    fn test_clear() {
        let mut queue = Queue::new();
        let song1 = Song {
            pcm_data: Arc::new(vec![0.1; 1000]),
            position: 0,
            sample_rate: 44100,
            channels: 2,
            title: "Song 1".to_string(),
            artist: "Artist 1".to_string(),
            sleep_duration: Duration::from_millis(10),
        };

        queue.add(song1);
        queue.advance_queue();

        let metadata = queue.get_metadata();
        assert_eq!(metadata.title, "Song 1");

        queue.clear();

        let metadata = queue.get_metadata();
        assert_eq!(metadata.title, "");

        let (packet, _) = queue.get_audio_packet();
        assert!(packet.is_empty());
    }
}
