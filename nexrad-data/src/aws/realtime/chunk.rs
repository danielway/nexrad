use crate::aws::realtime::volume::Volume;
use chrono::{DateTime, Utc};

/// Represents a chunk of NEXRAD data within a volume.
#[derive(Clone, Debug)]
pub struct Chunk {
    volume: Volume,
    key: String,
    date_time: DateTime<Utc>,
}

impl Chunk {
    pub(crate) fn new(volume: Volume, key: String, date_time: DateTime<Utc>) -> Self {
        Self {
            volume,
            key,
            date_time,
        }
    }

    /// The volume containing this chunk.
    pub fn volume(&self) -> Volume {
        self.volume
    }

    /// The unique key for this chunk.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The identifier for this chunk.
    pub fn identifier(&self) -> Option<&str> {
        self.key.split('/').last()
    }

    /// The date and time this chunk was uploaded.
    pub fn date_time(&self) -> DateTime<Utc> {
        self.date_time
    }
}
