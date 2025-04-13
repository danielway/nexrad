use crate::{
    aws::realtime::{ChunkType, VolumeIndex},
    result::{aws::AWSError, Error, Result},
};
use chrono::{DateTime, NaiveDateTime, Utc};

#[cfg(feature = "nexrad-decode")]
use crate::aws::realtime::ElevationChunkMapper;

/// Identifies a volume chunk within the real-time NEXRAD data bucket. These chunks are uploaded
/// every few seconds and contain a portion of the radar data for a specific volume.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkIdentifier {
    // These three fields are the same for all chunks in a volume
    site: String,
    volume: VolumeIndex,
    date_time_prefix: NaiveDateTime,

    // These fields identify a specific chunk within the volume
    sequence: usize,
    chunk_type: ChunkType,

    // This is derived from the other fields
    name: String,

    // If this chunk was downloaded, this is the upload time
    upload_date_time: Option<DateTime<Utc>>,
}

impl ChunkIdentifier {
    /// Creates a new chunk identifier.
    pub fn new(
        site: String,
        volume: VolumeIndex,
        date_time_prefix: NaiveDateTime,
        sequence: usize,
        chunk_type: ChunkType,
        upload_date_time: Option<DateTime<Utc>>,
    ) -> Self {
        let name = format!(
            "{}-{:03}-{}",
            date_time_prefix.format("%Y%m%d-%H%M%S"),
            sequence,
            chunk_type.abbreviation()
        );

        Self {
            site,
            volume,
            date_time_prefix,
            sequence,
            chunk_type,
            name,
            upload_date_time,
        }
    }

    /// Creates a new chunk identifier by parsing a chunk name.
    pub fn from_name(
        site: String,
        volume: VolumeIndex,
        name: String,
        upload_date_time: Option<DateTime<Utc>>,
    ) -> Result<Self> {
        let date_time_prefix = NaiveDateTime::parse_from_str(&name[..15], "%Y%m%d-%H%M%S")
            .map_err(|_| Error::AWS(AWSError::UnrecognizedChunkDateTime(name[..15].to_string())))?;

        let sequence_str = &name[16..19];
        let sequence = sequence_str.parse::<usize>().map_err(|_| {
            Error::AWS(AWSError::UnrecognizedChunkSequence(
                sequence_str.to_string(),
            ))
        })?;

        let chunk_type = ChunkType::from_abbreviation(
            name.chars()
                .last()
                .ok_or(Error::AWS(AWSError::UnrecognizedChunkType(None)))?,
        )?;

        Ok(Self {
            site,
            volume,
            date_time_prefix,
            sequence,
            chunk_type,
            name,
            upload_date_time,
        })
    }

    /// The chunk's radar site identifier.
    pub fn site(&self) -> &str {
        &self.site
    }

    /// The chunk's rotating volume index.
    pub fn volume(&self) -> &VolumeIndex {
        &self.volume
    }

    /// The chunk's date and time prefix, consistent across all chunks in a volume.
    pub fn date_time_prefix(&self) -> &NaiveDateTime {
        &self.date_time_prefix
    }

    /// The sequence number of this chunk within the volume.
    pub fn sequence(&self) -> usize {
        self.sequence
    }

    /// The chunk's type.
    pub fn chunk_type(&self) -> ChunkType {
        self.chunk_type
    }

    /// The chunk's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The date and time this chunk was uploaded.
    pub fn upload_date_time(&self) -> Option<DateTime<Utc>> {
        self.upload_date_time
    }

    /// Identifies the next chunk's expected location.
    #[cfg(feature = "nexrad-decode")]
    pub fn next_chunk(&self, elevation_chunk_mapper: &ElevationChunkMapper) -> Option<NextChunk> {
        let final_sequence = elevation_chunk_mapper.final_sequence();
        if self.sequence == final_sequence {
            return Some(NextChunk::Volume(self.volume.next()));
        }

        Some(NextChunk::Sequence(ChunkIdentifier::new(
            self.site().to_string(),
            self.volume,
            self.date_time_prefix,
            self.sequence + 1,
            if self.sequence + 1 == final_sequence {
                ChunkType::End
            } else {
                ChunkType::Intermediate
            },
            None,
        )))
    }
}

/// Identifies where to find the next expected chunk.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NextChunk {
    /// The next chunk is expected to be located in the same volume at this sequence. Once the next
    /// chunk's identifier is determined, it can be downloaded using the
    /// [crate::aws::realtime::download_chunk()] function. You may need to poll by checking if that
    /// function returns [crate::result::aws::AWSError::S3ObjectNotFoundError].
    Sequence(ChunkIdentifier),

    /// The chunk is expected to be located in the next volume. The next volume's chunks can be
    /// listed using the [crate::aws::realtime::list_chunks_in_volume()] function.
    Volume(VolumeIndex),
}
