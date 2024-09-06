/// Statistics from the polling process.
#[derive(Debug)]
pub enum PollStats {
    /// The number of network calls made to find the most recent volume.
    LatestVolumeCalls(usize),
    /// The number of network calls made to find a new volume.
    NewVolumeCalls(usize),
    /// The number of network calls made to find a new chunk.
    NewChunkCalls(usize),
}
