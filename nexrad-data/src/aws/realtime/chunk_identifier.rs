use crate::aws::realtime::{ChunkType, VolumeIndex};
use chrono::{DateTime, Utc};

/// Identifies a volume chunk within the real-time NEXRAD data bucket. These chunks are uploaded
/// every few seconds and contain a portion of the radar data for a specific volume.
#[derive(Debug, Clone)]
pub struct ChunkIdentifier {
    site: String,
    volume: VolumeIndex,
    name: String,
    date_time: DateTime<Utc>,
}

impl ChunkIdentifier {
    /// Creates a new chunk identifier.
    pub fn new(site: String, volume: VolumeIndex, name: String, date_time: DateTime<Utc>) -> Self {
        Self {
            site,
            volume,
            name,
            date_time,
        }
    }

    /// The chunk's radar site identifier.
    pub fn site(&self) -> &str {
        &self.site
    }

    /// The chunk's rotating volume index.
    pub fn volume(&self) -> &VolumeIndex {
        &self.volume
    }

    /// The chunk's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The position of this chunk within the volume.
    pub fn chunk_type(&self) -> Option<ChunkType> {
        match self.name.chars().last() {
            Some('S') => Some(ChunkType::Start),
            Some('I') => Some(ChunkType::Intermediate),
            Some('E') => Some(ChunkType::End),
            _ => None,
        }
    }

    /// The date and time this chunk was uploaded.
    pub fn date_time(&self) -> DateTime<Utc> {
        self.date_time
    }
}
