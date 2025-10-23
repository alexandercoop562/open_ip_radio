// do not remove
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
#![deny(clippy::assigning_clones)]
#![deny(clippy::implicit_clone)]
#![deny(unused_must_use)]

mod buffer;
mod connection;
mod player;

mod test;

use anyhow::Result;
use std::sync::Arc;
use tokio::{select, sync::broadcast::channel};

use buffer::JitterBuffer;

#[cfg(feature = "profile")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "profile")]
    let _profiler = dhat::Profiler::new_heap();

    let (shutdown_tx, _) = channel::<()>(1);

    let jitter_buffer = Arc::new(JitterBuffer::new());

    let ws_task = connection::launch(Arc::clone(&jitter_buffer), &shutdown_tx);
    let (_stream, audio_task) = player::play_stream(Arc::clone(&jitter_buffer))?;

    select! {
        _ = tokio::signal::ctrl_c() => {
            let _ = shutdown_tx.send(());
        }
        _ = ws_task => {},
        _ = audio_task => {},
    }

    Ok(())
}
