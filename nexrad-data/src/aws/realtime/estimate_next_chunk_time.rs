use crate::aws::realtime::ChunkIdentifier;
use chrono::{DateTime, TimeDelta, Utc};
use std::ops::Add;

/// Attempts to estimate the time at which the next chunk will be available given the previous.
pub fn estimate_next_chunk_time(previous_chunk: &ChunkIdentifier) -> Option<DateTime<Utc>> {
    match (previous_chunk.sequence(), previous_chunk.date_time()) {
        (Some(sequence), Some(date_time)) => {
            return match sequence + 1 {
                1..=7 | 14..=19 | 26..=31 => Some(date_time.add(TimeDelta::seconds(12))),
                _ => Some(date_time.add(TimeDelta::seconds(4))),
            }
        }
        _ => {}
    }

    None
}
