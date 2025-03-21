use chrono::{DateTime, TimeDelta, Utc};

use super::ChunkTimingStats;

/// Statistics from the polling process.
#[derive(Debug, Clone)]
pub enum PollStats {
    /// The number of network calls made to find the most recent volume.
    LatestVolumeCalls(usize),
    /// The number of network calls made to find a new volume.
    NewVolumeCalls(usize),
    /// Statistics for a new chunk.
    NewChunk(NewChunkStats),
    /// Perodic timing statistics for chunks by-type.
    ChunkTimings(ChunkTimingStats),
}

/// Statistics for a new chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NewChunkStats {
    /// The number of network calls made to find a new chunk.
    pub calls: usize,
    /// The time when the chunk was downloaded.
    pub download_time: Option<DateTime<Utc>>,
    /// The time when the chunk was uploaded to S3.
    pub upload_time: Option<DateTime<Utc>>,
}

impl NewChunkStats {
    /// The latency between when a chunk was downloaded and when it was uploaded to S3.
    pub fn latency(&self) -> Option<TimeDelta> {
        self.download_time.and_then(|download_time| {
            self.upload_time
                .map(|upload_time| upload_time.signed_duration_since(download_time))
        })
    }
}
