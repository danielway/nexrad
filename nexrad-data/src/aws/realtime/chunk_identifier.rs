use crate::aws::realtime::{ChunkType, ElevationChunkMapper, VolumeIndex};
use chrono::{DateTime, Utc};

/// Identifies a volume chunk within the real-time NEXRAD data bucket. These chunks are uploaded
/// every few seconds and contain a portion of the radar data for a specific volume.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkIdentifier {
    site: String,
    volume: VolumeIndex,
    name: String,
    date_time: Option<DateTime<Utc>>,
}

impl ChunkIdentifier {
    /// Creates a new chunk identifier.
    pub fn new(
        site: String,
        volume: VolumeIndex,
        name: String,
        date_time: Option<DateTime<Utc>>,
    ) -> Self {
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

    /// The chunk's name prefix.
    pub fn name_prefix(&self) -> &str {
        &self.name[..15]
    }

    /// The sequence number of this chunk within the volume.
    pub fn sequence(&self) -> Option<usize> {
        self.name.split('-').nth(2).and_then(|s| s.parse().ok())
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
    pub fn date_time(&self) -> Option<DateTime<Utc>> {
        self.date_time
    }

    /// Identifies the next chunk's expected location.
    pub fn next_chunk(&self, elevation_chunk_mapper: &ElevationChunkMapper) -> Option<NextChunk> {
        if self.chunk_type() == Some(ChunkType::Start) {
            return Some(NextChunk::Sequence(ChunkIdentifier::new(
                self.site().to_string(),
                self.volume,
                format!("{}-{:03}-{}", self.name_prefix(), 2, "S"),
                None,
            )));
        }

        if elevation_chunk_mapper.is_final_sequence(self.sequence()?) {
            return Some(NextChunk::Volume(self.volume.next()));
        }

        Some(NextChunk::Sequence(ChunkIdentifier::new(
            self.site().to_string(),
            self.volume,
            format!("{}-{:03}-{}", self.name_prefix(), self.sequence()? + 1, "I"),
            None,
        )))
    }
}

/// Identifies where to find the next expected chunk.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NextChunk {
    /// The next chunk is expected to be located in the same volume at this sequence. The
    /// [ChunkIdentifier::with_sequence] method can be used to create the next chunk's identifier
    /// and it can be downloaded using the [crate::aws::realtime::download_chunk()] function. You
    /// may need to poll by checking if that function returns
    /// [crate::result::aws::AWSError::S3ObjectNotFoundError].
    Sequence(ChunkIdentifier),

    /// The chunk is expected to be located in the next volume. The next volume's chunks can be
    /// listed using the [crate::aws::realtime::list_chunks_in_volume()] function.
    Volume(VolumeIndex),
}
