use crate::aws::realtime::volume::Volume;
use chrono::{DateTime, Utc};

/// Represents a chunk of NEXRAD data within a volume.
#[derive(Clone)]
pub struct Chunk {
    volume: Volume,
    identifier: String,
    date_time: DateTime<Utc>,
}

impl Chunk {
    pub(crate) fn new(volume: Volume, identifier: String, date_time: DateTime<Utc>) -> Self {
        Self {
            volume,
            identifier,
            date_time,
        }
    }

    /// The volume containing this chunk.
    pub fn volume(&self) -> Volume {
        self.volume
    }

    /// The unique identifier for this chunk.
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// The date and time this chunk was uploaded.
    pub fn date_time(&self) -> DateTime<Utc> {
        self.date_time
    }
}
