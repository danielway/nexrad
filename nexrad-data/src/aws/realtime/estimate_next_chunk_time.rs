use crate::aws::realtime::ChunkIdentifier;
use chrono::{DateTime, Utc};
use std::ops::Add;
use std::time::Duration;

/// Attempts to estimate the time at which the next chunk will be available given the previous chunk.
/// A None result indicates that the chunk should already be available.
pub fn estimate_next_chunk_time(previous_chunk: &ChunkIdentifier) -> DateTime<Utc> {
    let mut estimated_wait_time = Duration::from_secs(4);
    if let Some(sequence) = previous_chunk.sequence() {
        match sequence + 1 {
            1..=7 | 14..=19 | 26..=31 => {
                estimated_wait_time = Duration::from_secs(12);
            }
            _ => {}
        };
    }

    let previous_time = previous_chunk.date_time().unwrap_or_else(|| Utc::now());
    previous_time.add(Duration::from_millis(estimated_wait_time.as_millis() as u64))
}
