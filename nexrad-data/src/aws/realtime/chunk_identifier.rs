use crate::aws::realtime::{ChunkType, VolumeIndex};
use chrono::{DateTime, Utc};

/// Identifies a volume chunk within the real-time NEXRAD data bucket. These chunks are uploaded
/// every few seconds and contain a portion of the radar data for a specific volume.
#[derive(Debug, Clone)]
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

    /// Creates a new chunk identifier with the given sequence number. The chunk type will be
    /// inferred from the sequence and date/time will be omitted since it is unknown.
    pub fn with_sequence(&self, sequence: usize) -> Self {
        let name = format!(
            "{}-{:03}-{}",
            self.name_prefix(),
            sequence,
            match sequence {
                1 => "S",
                55 => "E",
                _ => "I",
            }
        );

        Self {
            site: self.site.clone(),
            volume: self.volume.clone(),
            name,
            date_time: None,
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
    pub fn next_chunk(&self) -> Option<NextChunk> {
        let sequence = self.sequence()?;
        let mut chunk_type;

        if sequence < 55 {
            let next_sequence = sequence + 1;

            chunk_type = ChunkType::Intermediate;
            if next_sequence == 55 {
                chunk_type = ChunkType::End;
            }

            let name = format!(
                "{}-{:03}-{}",
                self.name_prefix(),
                next_sequence,
                match chunk_type {
                    ChunkType::Start => "S",
                    ChunkType::Intermediate => "I",
                    ChunkType::End => "E",
                }
            );

            let next_chunk = ChunkIdentifier::new(self.site().to_string(), self.volume, name, None);
            return Some(NextChunk::Sequence(next_chunk));
        }

        let mut volume = self.volume.as_number() + 1;
        if volume > 999 {
            volume = 1;
        }

        Some(NextChunk::Volume(VolumeIndex::new(volume)))
    }
}

/// Identifies where to find the next expected chunk.
pub enum NextChunk {
    /// The next chunk is expected to be located in the same volume at this sequence. The
    /// [ChunkIdentifier::with_sequence] method can be used to create the next chunk's identifier
    /// and it can be downloaded using the [crate::aws::realtime::download_chunk] function. You
    /// may need to poll by checking if that function returns
    /// [crate::result::aws::AWSError::S3ObjectNotFoundError].
    Sequence(ChunkIdentifier),

    /// The chunk is expected to be located in the next volume. The next volume's chunks can be
    /// listed using the [crate::aws::realtime::list_chunks_in_volume] function.
    Volume(VolumeIndex),
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::TimeZone;

    #[test]
    fn test_chunk_identifier() {
        let site = "KTLX";
        let volume = 50;
        let name = "20240813-123330-014-I";
        let date_time = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();

        let chunk = ChunkIdentifier::new(
            site.to_string(),
            VolumeIndex::new(volume),
            name.to_string(),
            Some(date_time),
        );

        assert_eq!(chunk.site(), site);
        assert_eq!(chunk.volume().as_number(), 50);
        assert_eq!(chunk.name(), name);
        assert_eq!(chunk.name_prefix(), "20240813-123330");
        assert_eq!(chunk.chunk_type(), Some(ChunkType::Intermediate));
        assert_eq!(chunk.sequence(), Some(14));
        assert_eq!(chunk.date_time(), Some(date_time));
    }

    #[test]
    fn test_next_chunk_start() {
        let site = "KTLX";
        let volume = 50;
        let name = "20240813-123330-001-S";
        let date_time = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();

        let chunk = ChunkIdentifier::new(
            site.to_string(),
            VolumeIndex::new(volume),
            name.to_string(),
            Some(date_time),
        );

        let next_chunk = chunk.next_chunk().expect("Expected next chunk");
        match next_chunk {
            NextChunk::Sequence(next_chunk) => {
                assert_eq!(next_chunk.site(), site);
                assert_eq!(next_chunk.volume().as_number(), 50);
                assert_eq!(next_chunk.name(), "20240813-123330-002-I");
                assert_eq!(next_chunk.name_prefix(), "20240813-123330");
                assert_eq!(next_chunk.sequence(), Some(2));
                assert_eq!(next_chunk.chunk_type(), Some(ChunkType::Intermediate));
                assert_eq!(next_chunk.date_time(), None);
            }
            _ => panic!("Expected sequence"),
        }
    }

    #[test]
    fn test_next_chunk_intermediate() {
        let site = "KTLX";
        let volume = 999;
        let name = "20240813-123330-014-I";
        let date_time = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();

        let chunk = ChunkIdentifier::new(
            site.to_string(),
            VolumeIndex::new(volume),
            name.to_string(),
            Some(date_time),
        );

        let next_chunk = chunk.next_chunk().expect("Expected next chunk");
        match next_chunk {
            NextChunk::Sequence(next_chunk) => {
                assert_eq!(next_chunk.site(), site);
                assert_eq!(next_chunk.volume().as_number(), 999);
                assert_eq!(next_chunk.name(), "20240813-123330-015-I");
                assert_eq!(next_chunk.name_prefix(), "20240813-123330");
                assert_eq!(next_chunk.sequence(), Some(15));
                assert_eq!(next_chunk.chunk_type(), Some(ChunkType::Intermediate));
                assert_eq!(next_chunk.date_time(), None);
            }
            _ => panic!("Expected sequence"),
        }
    }

    #[test]
    fn test_next_chunk_end() {
        let site = "KTLX";
        let volume = 50;
        let name = "20240813-123330-055-E";
        let date_time = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();

        let chunk = ChunkIdentifier::new(
            site.to_string(),
            VolumeIndex::new(volume),
            name.to_string(),
            Some(date_time),
        );

        let next_chunk = chunk.next_chunk().expect("Expected next chunk");
        match next_chunk {
            NextChunk::Volume(next_volume) => {
                assert_eq!(next_volume.as_number(), 51);
            }
            _ => panic!("Expected volume"),
        }
    }

    #[test]
    fn test_next_chunk_last_volume() {
        let site = "KTLX";
        let volume = 999;
        let name = "20240813-123330-055-E";
        let date_time = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();

        let chunk = ChunkIdentifier::new(
            site.to_string(),
            VolumeIndex::new(volume),
            name.to_string(),
            Some(date_time),
        );

        let next_chunk = chunk.next_chunk().expect("Expected next chunk");
        match next_chunk {
            NextChunk::Volume(next_volume) => {
                assert_eq!(next_volume.as_number(), 1);
            }
            _ => panic!("Expected volume"),
        }
    }

    #[test]
    fn test_chunk_from_sequence() {
        let site = "KTLX";
        let volume = 50;
        let name = "20240813-123330-014-I";
        let date_time = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();

        let chunk = ChunkIdentifier::new(
            site.to_string(),
            VolumeIndex::new(volume),
            name.to_string(),
            Some(date_time),
        );

        let next_chunk = chunk.with_sequence(15);
        assert_eq!(next_chunk.site(), site);
        assert_eq!(next_chunk.volume().as_number(), 50);
        assert_eq!(next_chunk.name(), "20240813-123330-015-I");
        assert_eq!(next_chunk.name_prefix(), "20240813-123330");
        assert_eq!(next_chunk.sequence(), Some(15));
        assert_eq!(next_chunk.chunk_type(), Some(ChunkType::Intermediate));
        assert_eq!(next_chunk.date_time(), None);

        assert_eq!(chunk.with_sequence(1).chunk_type(), Some(ChunkType::Start));
        assert_eq!(chunk.with_sequence(55).chunk_type(), Some(ChunkType::End));
    }
}
