use crate::buffer::JitterBuffer;
use crate::connection::binary_to_samples;

use anyhow::Result;
use rodio::{OutputStream, OutputStreamBuilder, buffer::SamplesBuffer};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

const BUFFER_TARGET: usize = 10;

// Hardware I/O - Audio device operations
#[cfg(not(tarpaulin_include))]
pub fn play_stream(jitter_buffer: Arc<JitterBuffer>) -> Result<(OutputStream, JoinHandle<()>)> {
    let stream = OutputStreamBuilder::open_default_stream()?;
    let mixer = stream.mixer().clone();

    let handle = tokio::spawn(async move {
        while jitter_buffer.len().await < BUFFER_TARGET {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        loop {
            if let Some(data) = jitter_buffer.pop().await {
                let samples = binary_to_samples(&data);
                let source = SamplesBuffer::new(2, 44100, samples);
                mixer.add(source);
            } else {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    });

    return Ok((stream, handle));
}
