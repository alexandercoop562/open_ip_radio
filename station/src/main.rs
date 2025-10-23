// do not remove
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
#![deny(clippy::assigning_clones)]
#![deny(clippy::implicit_clone)]
#![deny(unused_must_use)]

use std::{
    error::Error,
    sync::{Arc, Mutex, MutexGuard},
};

mod audio;
mod network;

mod test;

use audio::queue::Queue;

type MainErr = Box<dyn Error + Send + Sync>;

fn get_lock(queue: &Arc<Mutex<Queue>>) -> Result<MutexGuard<'_, Queue>, MainErr> {
    return Ok(queue.lock().map_err(|e| return e.to_string())?);
}

trait LogError<T> {
    fn log_error(self, from: &str) -> Result<T, MainErr>;
}

impl<T: Default> LogError<T> for Result<T, MainErr> {
    fn log_error(self, from: &str) -> Result<T, MainErr> {
        use tracing::error;
        return match self {
            Ok(_) => self,
            Err(e) => {
                error!("({}): {}", from, e.to_string());
                Err("An error was logged.".into())
            }
        };
    }
}

use tracing::Level;
use tracing_subscriber::{FmtSubscriber, fmt::time::ChronoLocal};

use audio::song::get_song;
use network::streamer::Streamer;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> Result<(), MainErr> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_timer(ChronoLocal::new("%Y-%m-%d @%I:%M %p (utc: %:z)".to_string()))
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let queue = Arc::new(Mutex::new(Queue::new()));

    let scarlet_fire = get_song("songs/Scarlet_Fire.m4a".to_string())
        .await
        .log_error("get song")?;
    let over_the_horizon = get_song("songs/Over_the_Horizon.mp3".to_string())
        .await
        .log_error("get song")?;

    {
        let mut queue = get_lock(&queue)?;
        queue.add(scarlet_fire);
        queue.add(over_the_horizon);
    }

    let streamer = Streamer::default();
    streamer.launch(Arc::clone(&queue)).await.log_error("Streamer")?;

    get_lock(&queue)?.clear();
    return Ok(());
}
