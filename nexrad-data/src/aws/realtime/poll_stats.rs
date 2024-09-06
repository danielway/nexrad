/// Statistics from the polling process.
#[derive(Debug)]
pub enum PollStats {
    /// The number of network calls made to find the most recent volume.
    LatestVolumeCalls(usize),
}
