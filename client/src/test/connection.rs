#[cfg(test)]
mod test {
    use crate::connection::binary_to_samples;

    fn samples_to_binary(samples: &[f32]) -> Vec<u8> {
        let mut audio_bytes = Vec::with_capacity(samples.len() * 4);
        for sample in samples {
            audio_bytes.extend_from_slice(&sample.to_le_bytes());
        }
        return audio_bytes;
    }

    #[test]
    fn test_binary_to_samples() {
        let sample1: f32 = 0.5;
        let sample2: f32 = -0.25;
        let mut binary = Vec::new();
        binary.extend_from_slice(&sample1.to_le_bytes());
        binary.extend_from_slice(&sample2.to_le_bytes());
        let samples = binary_to_samples(&binary);
        assert_eq!(samples, vec![sample1, sample2]);

        // Test empty and large buffer
        assert_eq!(binary_to_samples(&[]).len(), 0);
        let samples_orig: Vec<f32> = (0..8192).map(|i| return (i as f32) / 8192.0).collect();
        let binary = samples_to_binary(&samples_orig);
        assert_eq!(binary_to_samples(&binary), samples_orig);
    }

    #[test]
    fn test_binary_to_samples_unaligned() {
        let sample1: f32 = 0.5;
        let sample2: f32 = -0.25;
        let mut unaligned = vec![0u8];
        unaligned.extend_from_slice(&sample1.to_le_bytes());
        unaligned.extend_from_slice(&sample2.to_le_bytes());

        let samples = binary_to_samples(&unaligned[1..]);
        assert_eq!(samples, vec![sample1, sample2]);
    }

    #[test]
    fn test_samples_to_binary() {
        let samples = vec![0.5, -0.25, 0.75];
        let binary = samples_to_binary(&samples);
        assert_eq!(binary.len(), 12);
        assert_eq!(binary_to_samples(&binary), samples);

        assert_eq!(samples_to_binary(&[]).len(), 0);
    }

    #[tokio::test]
    async fn test_launch() {
        use crate::{buffer::JitterBuffer, connection::launch};
        use std::sync::Arc;
        use tokio::sync::broadcast::channel;

        let (shutdown_tx, _) = channel::<()>(1);
        let jitter = Arc::new(JitterBuffer::new());
        let handle = launch(Arc::clone(&jitter), &shutdown_tx);
        let _ = shutdown_tx.send(());
        let _ = handle.await;
    }
}
