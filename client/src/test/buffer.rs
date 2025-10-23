#[cfg(test)]
mod test {
    use crate::buffer::*;
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_jitter_buffer() {
        let buffer = JitterBuffer::new();
        assert_eq!(buffer.len().await, 0);

        let buffer_default = JitterBuffer::default();
        assert_eq!(buffer_default.len().await, 0);

        // Test basic push/pop and FIFO
        let buffer = JitterBuffer::new();
        buffer.push(vec![1u8]).await;
        buffer.push(vec![2u8]).await;
        buffer.push(vec![3u8]).await;
        assert_eq!(buffer.len().await, 3);
        if let Some(data) = buffer.pop().await {
            assert_eq!(data[0], 1u8);
        }
        if let Some(data) = buffer.pop().await {
            assert_eq!(data[0], 2u8);
        }
        assert_eq!(buffer.len().await, 1);

        // Test empty pop
        let buffer = JitterBuffer::new();
        assert!(buffer.pop().await.is_none());

        // Test large packets
        let buffer = JitterBuffer::new();
        let large_packet = vec![128u8; 8192 * 4];
        buffer.push(large_packet.clone()).await;
        if let Some(data) = buffer.pop().await {
            assert_eq!(data, large_packet);
        }
    }

    #[tokio::test]
    async fn test_jitter_buffer_concurrent() {
        let buffer = Arc::new(JitterBuffer::new());

        // Test concurrent producers
        let mut handles = vec![];
        for i in 0..5 {
            let buffer_clone = Arc::clone(&buffer);
            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    buffer_clone.push(vec![(i * 10 + j) as u8]).await;
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            });
            handles.push(handle);
        }
        for handle in handles {
            let _ = handle.await;
        }
        assert_eq!(buffer.len().await, 50);
    }

    #[tokio::test]
    async fn test_jitter_buffer_producer_consumer() {
        let buffer = Arc::new(JitterBuffer::new());
        let buffer_producer = Arc::clone(&buffer);
        let buffer_consumer = Arc::clone(&buffer);

        // Producer task
        let producer = tokio::spawn(async move {
            for i in 0..20 {
                let packet = vec![i as u8; 400];
                buffer_producer.push(packet).await;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        // Consumer task
        let consumer = tokio::spawn(async move {
            let mut consumed = 0;
            while consumed < 20 {
                if let Some(packet) = buffer_consumer.pop().await {
                    assert_eq!(packet[0] as usize, consumed);
                    consumed += 1;
                } else {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            }
        });

        let _ = tokio::join!(producer, consumer);
    }

    #[tokio::test]
    async fn test_jitter_buffer_stress() {
        let buffer = Arc::new(JitterBuffer::new());
        let buffer_producer = Arc::clone(&buffer);
        let buffer_consumer = Arc::clone(&buffer);

        let producer = tokio::spawn(async move {
            for i in 0..1000 {
                buffer_producer.push(vec![i as u8; 4096]).await;
            }
        });

        let consumer = tokio::spawn(async move {
            let mut count = 0;
            while count < 1000 {
                if buffer_consumer.pop().await.is_some() {
                    count += 1;
                } else {
                    tokio::time::sleep(Duration::from_micros(100)).await;
                }
            }
            return count;
        });

        let (prod_result, cons_result) = tokio::join!(producer, consumer);
        assert!(prod_result.is_ok());
        if let Ok(count) = cons_result {
            assert_eq!(count, 1000);
        }
    }
}
