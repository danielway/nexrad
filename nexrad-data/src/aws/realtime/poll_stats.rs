use std::time::Duration;

/// Statistics from the polling process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PollStats {
    /// The number of network calls made to find the most recent volume.
    LatestVolumeCalls(usize),
    /// The number of network calls made to find a new volume.
    NewVolumeCalls(usize),
    /// Statistics for a new chunk.
    NewChunk(NewChunkStats),
}

/// Statistics for a new chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NewChunkStats {
    /// The number of network calls made to find a new chunk.
    pub calls: usize,
    /// The latency between when a chunk was uploaded to S3 and when it was downloaded.
    pub latency: Option<Duration>,
}
